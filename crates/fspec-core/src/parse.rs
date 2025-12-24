use crate::error::Error;
use crate::pattern::parse_pattern_str;
use crate::spec::{Rule, RuleKind};

pub(crate) fn parse_fspec(src: &str) -> Result<Vec<Rule>, Error> {
    let mut rules = Vec::new();

    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;

        // Handle Windows CRLF files.
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

        // We only trim for control flow; the pattern itself will be handled below.
        let trimmed = line.trim_start();

        // Comments only at start of line (after optional leading whitespace).
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse keyword and the rest of the line.
        let (kind, rest_owned) = split_kw_owned(trimmed).ok_or_else(|| Error::Parse {
            line: line_no,
            col: 1,
            msg: "expected 'allow' or 'ignore'".into(),
        })?;

        // Everything after the keyword is the pattern (spaces allowed).
        let rest = rest_owned.trim_start();
        if rest.is_empty() {
            return Err(Error::Parse {
                line: line_no,
                col: 1,
                msg: "expected a pattern after keyword".into(),
            });
        }

        let raw_pattern = rest.trim_end();
        let pattern = parse_pattern_str(raw_pattern, line_no)?;

        rules.push(Rule {
            line: line_no,
            kind,
            pattern,
        });
    }

    Ok(rules)
}

fn split_kw_owned(s: &str) -> Option<(RuleKind, String)> {
    fn kw(s: &str, word: &str, kind: RuleKind) -> Option<(RuleKind, String)> {
        let rest = s.strip_prefix(word)?;
        // Require a boundary so "allowance" doesn't match "allow".
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            Some((kind, rest.to_string()))
        } else {
            None
        }
    }

    kw(s, "allow", RuleKind::Allow).or_else(|| kw(s, "ignore", RuleKind::Ignore))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::spec::{FSPattern, Segment};

    #[test]
    fn parses_basic_rules_smoke() {
        let src = r#"
            # comment
            allow movies/Foo Bar (2001).mkv
            ignore **/*.tmp
        "#;

        let rules = parse_fspec(src).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].line, 3);
        assert_eq!(rules[0].kind, RuleKind::Allow);
        assert_eq!(rules[1].kind, RuleKind::Ignore);
    }

    // #[test]
    // fn parses_basic_rules() {
    //     let src = r#"
    //     # comment
    //     allow movies/Foo Bar (2001).mkv
    //     ignore **/*.tmp
    // "#;

    //     let rules = parse_fspec(src).unwrap();
    //     assert_eq!(rules.len(), 2);
    //     assert_eq!(rules[0].line, 3);

    //     assert_eq!(
    //         rules[0].pattern,
    //         FSPattern::Unanchored(vec![Component::FSEntry(Segment::Lit(
    //             "movies/Foo Bar (2001).mkv".to_string()
    //         ))])
    //     );
    // }
}
