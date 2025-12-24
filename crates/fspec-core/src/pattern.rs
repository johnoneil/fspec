use crate::error::Error;
use crate::spec::{FSEntry, FSPattern, Segment};

pub(crate) fn parse_pattern_str(raw: &str, line: usize) -> Result<FSPattern, Error> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(Error::Parse {
            line,
            col: 1,
            msg: "empty pattern".into(),
        });
    }

    // Anchored vs unanchored.
    let (pat_kind, s) = if let Some(rest) = s.strip_prefix('/') {
        ("anchored", rest)
    } else {
        ("unanchored", s)
    };

    // Trailing slash means the last component is a directory component.
    let (dir_trailing, s) = if let Some(rest) = s.strip_suffix('/') {
        (true, rest)
    } else {
        (false, s)
    };

    // After stripping leading/trailing '/', pattern can't be empty.
    if s.is_empty() {
        return Err(Error::Parse {
            line,
            col: 1,
            msg: "pattern cannot be just '/'".into(),
        });
    }

    // Split on '/', disallow empty segments (e.g., "a//b").
    let parts: Vec<&str> = s.split('/').collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(Error::Parse {
            line,
            col: 1,
            msg: "empty path segment (did you write '//'?)".into(),
        });
    }

    let mut comps: Vec<FSEntry> = Vec::with_capacity(parts.len());

    for (i, part) in parts.iter().enumerate() {
        let seg = match *part {
            "*" => Segment::Star,
            "**" => Segment::DoubleStar,
            lit => Segment::Lit(lit.to_string()),
        };

        let is_last = i + 1 == parts.len();
        if is_last {
            if dir_trailing {
                comps.push(FSEntry::Dir(seg));
            } else {
                comps.push(FSEntry::File(seg));
            }
        } else {
            comps.push(FSEntry::Dir(seg));
        }
    }

    Ok(match pat_kind {
        "anchored" => FSPattern::Anchored(comps),
        _ => FSPattern::Unanchored(comps),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{FSEntry::*, FSPattern::*, Segment::*};

    #[test]
    fn unanchored_dir_then_entry() {
        let p = parse_pattern_str("assets/*/*.png", 1).unwrap();
        assert_eq!(
            p,
            Unanchored(vec![
                Dir(Lit("assets".into())),
                Dir(Star),
                File(Lit("*.png".into()))
            ])
        );
    }

    #[test]
    fn trailing_slash_makes_last_component_dir() {
        let p = parse_pattern_str("assets/*/", 1).unwrap();
        assert_eq!(p, Unanchored(vec![Dir(Lit("assets".into())), Dir(Star)]));
    }

    #[test]
    fn anchored_pattern() {
        let p = parse_pattern_str("/assets/**/x", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                Dir(Lit("assets".into())),
                Dir(DoubleStar),
                File(Lit("x".into()))
            ])
        );
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
                Dir(Lit("assets".into())),
                Dir(Lit("this dir has spaces ".into())),
                File(Lit("x".into()))
            ])
        );
    }

    #[test]
    fn spaces_in_file_literal() {
        let p = parse_pattern_str("/assets/approved/My mom named this file.png", 1).unwrap();
        assert_eq!(
            p,
            Anchored(vec![
                Dir(Lit("assets".into())),
                Dir(Lit("approved".into())),
                File(Lit("My mom named this file.png".into()))
            ])
        );
    }
}
