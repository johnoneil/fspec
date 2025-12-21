use crate::error::Error;
use crate::spec::{Rule, RuleKind};

pub fn parse_fspec(src: &str) -> Result<Vec<Rule>, Error> {
    let mut rules = Vec::new();

    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;

        // Strip trailing '\r' for Windows CRLF.
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

        // Trim only for control flow; pattern will be trimmed intentionally later.
        let trimmed = line.trim_start();

        // Comments only at start of line (after optional leading whitespace).
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse: <kw> <ws> <pattern>
        let (kw, rest) = split_kw(trimmed).ok_or_else(|| Error::Parse {
            line: line_no,
            col: 1,
            msg: "expected 'allow' or 'ignore'".into(),
        })?;

        let kind = match kw {
            "allow" => RuleKind::Allow,
            "ignore" => RuleKind::Ignore,
            _ => unreachable!(),
        };

        let rest = rest.trim_start();
        if rest.is_empty() {
            return Err(Error::Parse {
                line: line_no,
                col: (trimmed.len() + 1).min(line.len()),
                msg: "expected a pattern after keyword".into(),
            });
        }

        // Since pattern is "rest of line", spaces are allowed.
        // We do trim_end so people can align / pad with whitespace.
        let pattern = rest.trim_end().to_string();

        rules.push(Rule {
            line: line_no,
            kind,
            pattern,
        });
    }

    Ok(rules)
}

fn split_kw(s: &str) -> Option<(&str, &str)> {
    if let Some(rest) = s.strip_prefix("allow") {
        return Some(("allow", rest));
    }
    if let Some(rest) = s.strip_prefix("ignore") {
        return Some(("ignore", rest));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_rules() {
        let src = r#"
        # comment
        allow movies/Foo Bar (2001).mkv
        ignore **/*.tmp    
    "#;

        let rules = parse_fspec(src).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].line, 3);
        assert_eq!(rules[0].pattern, "movies/Foo Bar (2001).mkv");
    }
}
