use crate::error::Error;
use crate::spec::{MatchSettings, Rule, RuleKind};

use crate::pattern::parse_pattern_str;

pub(crate) fn parse_fspec(src: &str, settings: &MatchSettings) -> Result<Vec<Rule>, Error> {
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
        // If no keyword is found, default to 'allow' (for find output compatibility).
        let (kind, raw_pattern) = if let Some((k, rest_owned)) = split_kw_owned(trimmed) {
            // Found a keyword (allow or ignore)
            let rest = rest_owned.trim_start();
            if rest.is_empty() {
                return Err(Error::Parse {
                    line: line_no,
                    col: 1,
                    msg: "expected a pattern after keyword".into(),
                });
            }
            (k, rest.trim_end().to_string())
        } else {
            // No keyword found - treat entire line as pattern, default to 'allow'
            (RuleKind::Allow, trimmed.trim_end().to_string())
        };
        let pattern = parse_pattern_str(&raw_pattern, line_no, settings)?;

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

    #[test]
    fn parses_basic_rules_smoke() {
        let src = r#"
            # comment
            allow movies/Foo Bar (2001).mkv
            ignore **/*.tmp
        "#;

        let rules = parse_fspec(src, &MatchSettings::default()).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].line, 3);
        assert_eq!(rules[0].kind, RuleKind::Allow);
        assert_eq!(rules[1].kind, RuleKind::Ignore);
    }

    #[test]
    fn allow_keyword_is_optional() {
        let src = r#"
            # comment
            /src/main.rs
            allow /src/lib.rs
            ignore /target/
            /src/utils.rs
        "#;

        let rules = parse_fspec(src, &MatchSettings::default()).unwrap();
        assert_eq!(rules.len(), 4);
        // Line without keyword defaults to Allow
        assert_eq!(rules[0].kind, RuleKind::Allow);
        assert_eq!(rules[0].line, 3);
        // Explicit allow
        assert_eq!(rules[1].kind, RuleKind::Allow);
        assert_eq!(rules[1].line, 4);
        // Explicit ignore (required)
        assert_eq!(rules[2].kind, RuleKind::Ignore);
        assert_eq!(rules[2].line, 5);
        // Another line without keyword defaults to Allow
        assert_eq!(rules[3].kind, RuleKind::Allow);
        assert_eq!(rules[3].line, 6);
    }

    #[test]
    fn find_output_compatibility() {
        // Simulating find output - just paths, no keywords
        let src = r#"
            ./src/main.rs
            ./src/lib.rs
            ./target/
        "#;

        let rules = parse_fspec(src, &MatchSettings::default()).unwrap();
        assert_eq!(rules.len(), 3);
        // All should default to Allow
        assert!(rules.iter().all(|r| r.kind == RuleKind::Allow));
    }
}
