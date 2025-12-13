use crate::ast::{Limiter, LimiterKind, Node, Pattern, Quant};
use crate::error::ParseError;

pub fn parse_pattern(input: &str) -> Result<Pattern, ParseError> {
    let mut p = Parser::new(input);
    let mut nodes = Vec::new();

    while !p.eof() {
        if p.peek_str("**") {
            p.bump_n(2);
            nodes.push(Node::GlobStar);
            continue;
        }

        if p.peek_char('/') {
            p.bump();
            nodes.push(Node::Slash);
            continue;
        }

        if p.peek_char('{') {
            nodes.push(p.parse_placeholder()?);
            continue;
        }

        // literal chunk
        nodes.push(Node::Literal(p.parse_literal()?));
    }

    Ok(Pattern { nodes })
}

struct Parser<'a> {
    s: &'a str,
    i: usize, // byte index
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }
    fn eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn err<T>(&self, msg: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            index: self.i,
            message: msg.into(),
        })
    }

    fn peek_char(&self, c: char) -> bool {
        self.s[self.i..].chars().next() == Some(c)
    }

    fn peek_str(&self, pat: &str) -> bool {
        self.s[self.i..].starts_with(pat)
    }

    fn bump(&mut self) -> Option<char> {
        let mut it = self.s[self.i..].char_indices();
        let (_, ch) = it.next()?;
        let next = it.next().map(|(o, _)| self.i + o).unwrap_or(self.s.len());
        self.i = next;
        Some(ch)
    }

    fn bump_n(&mut self, n: usize) {
        self.i += n;
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.s[self.i..].chars().next() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn parse_literal(&mut self) -> Result<String, ParseError> {
        let start = self.i;
        while !self.eof() {
            if self.peek_char('{') || self.peek_char('/') || self.peek_str("**") {
                break;
            }
            self.bump();
        }
        if self.i == start {
            return self.err("expected literal");
        }
        Ok(self.s[start..self.i].to_string())
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let start = self.i;

        // first char must be [A-Za-z_]
        let first = self.s[self.i..].chars().next().ok_or(ParseError {
            index: self.i,
            message: "expected identifier".into(),
        })?;

        if !(first == '_' || first.is_ascii_alphabetic()) {
            return self.err("expected identifier (must start with letter or _)");
        }
        self.bump();

        while let Some(ch) = self.s[self.i..].chars().next() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                self.bump();
            } else {
                break;
            }
        }

        Ok(self.s[start..self.i].to_string())
    }

    fn parse_placeholder(&mut self) -> Result<Node, ParseError> {
        // "{"
        if !self.peek_char('{') {
            return self.err("expected '{'");
        }
        self.bump();

        let name = self.parse_ident()?;

        // After ident, spaces are not allowed inside placeholders.
        // If we see whitespace here, it means something like "{no spaces}".
        if self.s[self.i..]
            .chars()
            .next()
            .map_or(false, |c| c.is_whitespace())
        {
            return self.err("whitespace not allowed in placeholder");
        }

        // optional ": limiter"
        let limiter = if self.peek_char(':') {
            self.bump();
            Some(self.parse_limiter()?)
        } else {
            None
        };

        // "}"
        if !self.peek_char('}') {
            return self.err("expected '}' to close placeholder");
        }
        self.bump();

        Ok(Node::Placeholder { name, limiter })
    }

    fn parse_limiter(&mut self) -> Result<Limiter, ParseError> {
        let kind = self.parse_limiter_kind()?;
        let quant = self.parse_quant_opt()?;
        Ok(Limiter { kind, quant })
    }

    fn parse_limiter_kind(&mut self) -> Result<LimiterKind, ParseError> {
        // read until one of: '(' or '}' or whitespace
        let start = self.i;
        while !self.eof() {
            let ch = self.s[self.i..].chars().next().unwrap();
            if ch == '(' || ch == '}' || ch.is_whitespace() {
                break;
            }
            self.bump();
        }
        if self.i == start {
            return self.err("expected limiter kind after ':'");
        }
        let raw = &self.s[start..self.i];

        let kind = match raw {
            "int" => LimiterKind::Int,
            "semver" => LimiterKind::Semver,
            "camelCase" => LimiterKind::CamelCase,
            "PascalCase" => LimiterKind::PascalCase,
            "snake_case" => LimiterKind::SnakeCase,
            "kebab-case" => LimiterKind::KebabCase,
            "flatcase" => LimiterKind::FlatCase,
            "UPPER_CASE" => LimiterKind::UpperCase,
            _ => return self.err(format!("unknown limiter kind '{raw}'")),
        };

        Ok(kind)
    }

    fn parse_quant_opt(&mut self) -> Result<Quant, ParseError> {
        self.skip_ws();
        if !self.peek_char('(') {
            return Ok(Quant::Any);
        }
        self.bump(); // '('
        self.skip_ws();

        let n1 = self.parse_uint()?;
        self.skip_ws();

        let quant = if self.peek_char('+') {
            self.bump();
            Quant::AtLeast(n1)
        } else if self.peek_char(',') {
            self.bump();
            self.skip_ws();
            let n2 = self.parse_uint()?;
            Quant::Range { min: n1, max: n2 }
        } else {
            Quant::Exactly(n1)
        };

        self.skip_ws();
        if !self.peek_char(')') {
            return self.err("expected ')' to close limiter arguments");
        }
        self.bump();

        Ok(quant)
    }

    fn parse_uint(&mut self) -> Result<usize, ParseError> {
        let start = self.i;
        while let Some(ch) = self.s[self.i..].chars().next() {
            if ch.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }
        if self.i == start {
            return self.err("expected integer");
        }
        self.s[start..self.i]
            .parse::<usize>()
            .map_err(|_| ParseError {
                index: start,
                message: "invalid integer".into(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::parse_pattern;
    use crate::ast::{Limiter, LimiterKind, Node, Quant};

    #[test]
    fn parses_literal_only() {
        let p = parse_pattern("movies").unwrap();
        assert_eq!(p.nodes, vec![Node::Literal("movies".into())]);
    }

    #[test]
    fn parses_slashes_and_literals() {
        let p = parse_pattern("movies/2024/title.mp4").unwrap();
        assert_eq!(
            p.nodes,
            vec![
                Node::Literal("movies".into()),
                Node::Slash,
                Node::Literal("2024".into()),
                Node::Slash,
                Node::Literal("title.mp4".into()),
            ]
        );
    }

    #[test]
    fn parses_globstar() {
        let p = parse_pattern("root/**/file.txt").unwrap();
        assert_eq!(
            p.nodes,
            vec![
                Node::Literal("root".into()),
                Node::Slash,
                Node::GlobStar,
                Node::Slash,
                Node::Literal("file.txt".into()),
            ]
        );
    }

    #[test]
    fn parses_placeholder_without_limiter() {
        let p = parse_pattern("{year}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "year".into(),
                limiter: None
            }]
        );
    }

    #[test]
    fn parses_placeholder_with_limiter_no_quant_defaults_to_any() {
        let p = parse_pattern("{name:camelCase}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any
                })
            }]
        );
    }

    #[test]
    fn parses_placeholder_with_exact_quant() {
        let p = parse_pattern("{year:int(4)}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "year".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Exactly(4)
                })
            }]
        );
    }

    #[test]
    fn parses_placeholder_with_at_least_quant() {
        let p = parse_pattern("{id:int(3+)}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::AtLeast(3)
                })
            }]
        );
    }

    #[test]
    fn parses_placeholder_with_range_quant_and_whitespace() {
        let p = parse_pattern("{id:int( 2 , 5 )}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Range { min: 2, max: 5 }
                })
            }]
        );
    }

    #[test]
    fn parses_placeholder_with_range_quant_no_whitespace() {
        let p = parse_pattern("{id:int(2,5)}").unwrap();
        assert_eq!(
            p.nodes,
            vec![Node::Placeholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Range { min: 2, max: 5 }
                })
            }]
        );
    }

    #[test]
    fn parses_multiple_placeholders_mixed_with_literals() {
        let p = parse_pattern("movies/{year}/{name:camelCase}_{year}.mp4").unwrap();
        assert_eq!(
            p.nodes,
            vec![
                Node::Literal("movies".into()),
                Node::Slash,
                Node::Placeholder {
                    name: "year".into(),
                    limiter: None
                },
                Node::Slash,
                Node::Placeholder {
                    name: "name".into(),
                    limiter: Some(Limiter {
                        kind: LimiterKind::CamelCase,
                        quant: Quant::Any
                    })
                },
                Node::Literal("_".into()),
                Node::Placeholder {
                    name: "year".into(),
                    limiter: None
                },
                Node::Literal(".mp4".into()),
            ]
        );
    }

    #[test]
    fn error_on_unclosed_placeholder() {
        let err = parse_pattern("movies/{year").unwrap_err();
        assert!(
            err.message.contains("expected '}'"),
            "unexpected error message: {}",
            err.message
        );
    }

    // TODO: turn on when supported.
    #[ignore]
    #[test]
    fn error_on_unopened_placeholder() {
        let err = parse_pattern("movies/year}").unwrap_err();
        println!("error message: {}", err.message);
        // assert!(
        //     err.message.contains("expected ''"),
        //     "unexpected error message: {}",
        //     err.message
        // );
    }

    #[test]
    fn error_on_colon_without_limiter() {
        let err = parse_pattern("{name:}").unwrap_err();
        assert!(
            err.message.contains("expected limiter kind"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn error_on_unknown_limiter_kind() {
        let err = parse_pattern("{x:NotARealLimiter}").unwrap_err();
        assert!(
            err.message.contains("unknown limiter kind"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn error_on_bad_quant_missing_close_paren() {
        let err = parse_pattern("{x:int(3}").unwrap_err();
        assert!(
            err.message.contains("expected ')'"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn error_on_bad_quant_missing_number() {
        let err = parse_pattern("{x:int()}").unwrap_err();
        assert!(
            err.message.contains("expected integer"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn error_on_placeholder_with_space() {
        let err = parse_pattern("{no spaces allowed}").unwrap_err();
        println!("error message: {}", err.message);
    }

    #[test]
    fn placeholder_identifier_allows_underscores_and_digits() {
        let p = parse_pattern("{valid_name_123}").unwrap();
        assert_eq!(p.nodes.len(), 1);
    }
}
