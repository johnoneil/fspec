use crate::error::Error;
use crate::spec::{DirType, FSEntry, FSPattern, FileType};
//use fspec_placeholder::ast::*;
use fspec_placeholder::ast::*;
use fspec_placeholder::parse_component;

pub(crate) fn parse_pattern_str(raw: &str, line: usize) -> Result<FSPattern, Error> {
    let s0 = raw.trim();
    if s0.is_empty() {
        return Err(parse_err(line, 1, "empty pattern"));
    }

    // Anchored vs unanchored.
    // Support both '/' and './' as anchored prefixes
    let (anchored, mut s, base_col) = if let Some(rest) = s0.strip_prefix("./") {
        (true, rest, 3) // we consumed './'
    } else if let Some(rest) = s0.strip_prefix('/') {
        (true, rest, 2) // we consumed '/'
    } else {
        (false, s0, 1)
    };

    // Directory vs file is determined by trailing slash.
    let ends_with_slash = s.ends_with('/');
    if ends_with_slash {
        // Strip exactly one trailing slash; anything else will be handled by empty-segment checks.
        s = &s[..s.len() - 1];
    }

    if s.is_empty() {
        return Err(parse_err(
            line,
            base_col,
            "pattern must not be just '/' (no path components)",
        ));
    }

    // Split into segments. We disallow empty segments like `a//b`.
    let parts: Vec<&str> = s.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            // Column: approximate start of this empty segment.
            // (Good enough; you can refine later if you want exact columns.)
            let col = base_col + parts[..i].iter().map(|p| p.len() + 1).sum::<usize>();
            return Err(parse_err(line, col, "empty path segment (// not allowed)"));
        }
    }

    let last_idx = parts.len() - 1;
    let mut entries = Vec::with_capacity(parts.len());

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == last_idx;

        if !is_last {
            entries.push(FSEntry::Dir(parse_dir(part)?));
            continue;
        }

        // Final component depends on trailing slash.
        if ends_with_slash {
            entries.push(FSEntry::Dir(parse_dir(part)?));
        } else {
            entries.push(FSEntry::File(parse_file(part)?));
        }
    }

    Ok(if anchored {
        FSPattern::Anchored(entries)
    } else {
        FSPattern::Unanchored(entries)
    })
}

fn parse_dir(s: &str) -> Result<DirType, Error> {
    match s {
        "*" => Ok(DirType::Star),
        "**" => Ok(DirType::DoubleStar),
        _ => {
            let component = parse_component(s)?;
            Ok(DirType::Component(component))
        }
    }
}

fn parse_file(s: &str) -> Result<FileType, Error> {
    match s {
        "*" => Ok(FileType::Star),
        _ => {
            let component = parse_component(s)?;
            Ok(FileType::Component(component))
        }
    }
}

fn parse_err(line: usize, col: usize, msg: impl Into<String>) -> Error {
    Error::Parse {
        line,
        col,
        msg: msg.into(),
    }
}

#[cfg(test)]
mod tests {
    use fspec_placeholder::ComponentAst;

    use super::*;
    use crate::spec::{DirType, FSEntry::*, FSPattern::*, FileType};

    use fspec_placeholder::parse_component;

    fn c(s: &str) -> ComponentAst {
        parse_component(s).unwrap()
    }

    #[test]
    fn unanchored_dir_then_entry() {
        let p = parse_pattern_str("assets/*/*.png", 1).unwrap();
        assert_eq!(
            p,
            Unanchored(vec![
                //Dir(DirType::Component(c("assets"))),
                Dir(DirType::Component(c("assets"))),
                Dir(DirType::Star),
                //File(FileType::Component(c("*.png"))),
                File(FileType::Component(c("*.png"))),
            ])
        );
    }

    #[test]
    fn trailing_slash_makes_last_component_dir() {
        let p = parse_pattern_str("assets/*/", 1).unwrap();
        assert_eq!(
            p,
            Unanchored(vec![
                //Dir(DirType::Lit("assets".into())),
                Dir(DirType::Component(c("assets"))),
                Dir(DirType::Star)
            ])
        );
    }

    #[test]
    fn anchored_pattern() {
        let p = parse_pattern_str("/assets/**/x", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                //Dir(DirType::Lit("assets".into())),
                Dir(DirType::Component(c("assets"))),
                Dir(DirType::DoubleStar),
                //File(FileType::Lit("x".into()))
                File(FileType::Component(c("x"))),
            ])
        );
    }

    #[test]
    fn anchored_pattern_with_dot_slash() {
        let p = parse_pattern_str("./assets/**/x", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                //Dir(DirType::Lit("assets".into())),
                Dir(DirType::Component(c("assets"))),
                Dir(DirType::DoubleStar),
                //File(FileType::Lit("x".into()))
                File(FileType::Component(c("x"))),
            ])
        );
    }

    #[test]
    fn anchored_dir_with_dot_slash() {
        let p = parse_pattern_str("./bin/", 1).unwrap();
        assert_eq!(p, Anchored(vec![Dir(DirType::Component(c("bin")))]));
    }

    #[test]
    fn rejects_double_slash() {
        assert!(parse_pattern_str("a//b", 1).is_err());
    }

    #[test]
    fn spaces_in_dir_literal() {
        let p = parse_pattern_str("/assets/this dir has spaces /x", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                //Dir(DirType::Lit("assets".into())),
                Dir(DirType::Component(c("assets"))),
                //Dir(DirType::Lit("this dir has spaces ".into())),
                Dir(DirType::Component(c("this dir has spaces "))),
                //File(FileType::Lit("x".into()))
                File(FileType::Component(c("x"))),
            ])
        );
    }

    #[test]
    fn spaces_in_file_literal() {
        let p = parse_pattern_str("/assets/approved/My mom named this file.png", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                Dir(DirType::Component(c("assets"))),
                Dir(DirType::Component(c("approved"))),
                File(FileType::Component(c("My mom named this file.png"))),
            ])
        );
    }
}
