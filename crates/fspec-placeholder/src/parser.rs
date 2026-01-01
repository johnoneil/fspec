// fspec-placeholder/src/parser.rs

#![allow(dead_code)]

use crate::ast::*;
use crate::tokenizer::{SpannedToken, Token, TokenizeError, Tokenizer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    TokenizeFailed,
    UnexpectedEof,
    UnexpectedToken,
    ExpectedToken(&'static str),
    EmptyPlaceholder,
    EmptyOneOf,
    EmptyOneOfArm,
    TrailingTokensInsidePlaceholder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub at: usize,          // byte offset
    pub span: Option<Span>, // token span if available
    pub message: String,
    pub source_tokenize: Option<String>,
}

impl ParseError {
    fn new(
        kind: ParseErrorKind,
        at: usize,
        span: Option<Span>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            at,
            span,
            message: message.into(),
            source_tokenize: None,
        }
    }

    fn from_tokenize(err: TokenizeError) -> Self {
        Self {
            kind: ParseErrorKind::TokenizeFailed,
            at: err.at,
            span: Some(Span::new(err.at, err.at + 1)),
            message: format!("{}", err),
            source_tokenize: Some(format!("{:?}", err.kind)),
        }
    }
}

/// Parse a single component string (between `/`) into a ComponentAst.
pub fn parse_component(input: &str) -> Result<ComponentAst, ParseError> {
    let tokens = Tokenizer::new(input)
        .tokenize_all()
        .map_err(ParseError::from_tokenize)?;

    let mut c = Cursor::new(&tokens);

    let mut parts: Vec<Part> = Vec::new();

    while !c.is_eof() {
        let t = c.peek().ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                input.len(),
                None,
                "unexpected end of input",
            )
        })?;

        match &t.token {
            Token::LiteralRun(s) => {
                let span = Span::new(t.start, t.end);
                let value = s.clone();
                c.bump();
                parts.push(Part::Literal(LiteralPart { value, span }));
            }
            Token::QuotedString(s) => {
                // Outside braces, QuotedString behaves as a literal run.
                let span = Span::new(t.start, t.end);
                let value = s.clone();
                c.bump();
                parts.push(Part::Literal(LiteralPart { value, span }));
            }
            Token::Star => {
                let span = Span::new(t.start, t.end);
                c.bump();
                parts.push(Part::Star(span));
            }
            Token::LBrace => {
                let ph = parse_placeholder(&mut c)?;
                parts.push(Part::Placeholder(ph));
            }

            // These should never appear outside braces if tokenizer is correct.
            other => {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    t.start,
                    Some(Span::new(t.start, t.end)),
                    format!("unexpected token outside braces: {:?}", other),
                ));
            }
        }
    }

    Ok(ComponentAst { parts })
}

fn parse_placeholder(c: &mut Cursor<'_>) -> Result<PlaceholderPart, ParseError> {
    let lbrace = c.expect_token("'{", |t| matches!(t, Token::LBrace))?;
    let lspan = Span::new(lbrace.start, lbrace.end);

    // Placeholder body must begin with Ident or QuotedString
    let first = c.peek().ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::UnexpectedEof,
            lbrace.end,
            Some(lspan),
            "unexpected end of input after '{'",
        )
    })?;

    // Empty placeholder "{}" should not be possible with current tokenizer because
    // inside-brace whitespace is skipped and next token would be RBrace.
    if matches!(first.token, Token::RBrace) {
        let r = c.bump().unwrap();
        let full_span = Span::join(lspan, Span::new(r.start, r.end));
        return Err(ParseError::new(
            ParseErrorKind::EmptyPlaceholder,
            first.start,
            Some(full_span),
            "empty placeholder '{}' is not allowed",
        ));
    }

    // Parse initial term for either one-of or capture.
    let (term_kind, term_value, term_span) = match &first.token {
        Token::Ident(s) => {
            let sp = Span::new(first.start, first.end);
            c.bump();
            ("ident", s.clone(), sp)
        }
        Token::QuotedString(s) => {
            let sp = Span::new(first.start, first.end);
            c.bump();
            ("str", s.clone(), sp)
        }
        _ => {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                first.start,
                Some(Span::new(first.start, first.end)),
                "expected identifier or quoted string inside '{...}'",
            ));
        }
    };

    // Decide whether this is one-of (if next token is Pipe) or capture (Colon or RBrace)
    let next = c.peek().ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::UnexpectedEof,
            term_span.end,
            Some(term_span),
            "unexpected end of input inside placeholder",
        )
    })?;

    let node = match &next.token {
        Token::Pipe => {
            // OneOf
            let mut choices: Vec<Choice> = Vec::new();
            choices.push(match term_kind {
                "ident" => Choice::Ident {
                    value: term_value,
                    span: term_span,
                },
                _ => Choice::Str {
                    value: term_value,
                    span: term_span,
                },
            });

            // Parse ( '|' choice )+
            let mut saw_pipe = false;
            while let Some(t) = c.peek() {
                if !matches!(t.token, Token::Pipe) {
                    break;
                }
                saw_pipe = true;
                c.bump(); // consume '|'

                let choice_tok = c.peek().ok_or_else(|| {
                    ParseError::new(
                        ParseErrorKind::UnexpectedEof,
                        t.end,
                        Some(Span::new(t.start, t.end)),
                        "expected choice after '|'",
                    )
                })?;

                // Reject empty arms like "{a|}"
                if matches!(choice_tok.token, Token::RBrace) {
                    return Err(ParseError::new(
                        ParseErrorKind::EmptyOneOfArm,
                        choice_tok.start,
                        Some(Span::new(choice_tok.start, choice_tok.end)),
                        "empty one-of arm is not allowed",
                    ));
                }

                let ch = match &choice_tok.token {
                    Token::Ident(s) => {
                        let sp = Span::new(choice_tok.start, choice_tok.end);
                        c.bump();
                        Choice::Ident {
                            value: s.clone(),
                            span: sp,
                        }
                    }
                    Token::QuotedString(s) => {
                        let sp = Span::new(choice_tok.start, choice_tok.end);
                        c.bump();
                        Choice::Str {
                            value: s.clone(),
                            span: sp,
                        }
                    }
                    _ => {
                        return Err(ParseError::new(
                            ParseErrorKind::UnexpectedToken,
                            choice_tok.start,
                            Some(Span::new(choice_tok.start, choice_tok.end)),
                            "expected identifier or quoted string as one-of choice",
                        ));
                    }
                };
                choices.push(ch);
            }

            if !saw_pipe {
                return Err(ParseError::new(
                    ParseErrorKind::EmptyOneOf,
                    term_span.start,
                    Some(term_span),
                    "one-of must contain at least one '|'",
                ));
            }

            // Expect closing brace
            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let rspan = Span::new(rbrace.start, rbrace.end);

            let full = Span::join(lspan, rspan);
            let oneof = OneOfNode {
                choices,
                span: full,
            };
            PlaceholderNode::OneOf(oneof)
        }

        Token::Colon | Token::RBrace => {
            // Capture: first term must be IDENT, not quoted.
            if term_kind != "ident" {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    term_span.start,
                    Some(term_span),
                    "capture name must be an identifier (quoted string not allowed here)",
                ));
            }

            let mut limiter: Option<LimiterSpec> = None;

            if matches!(next.token, Token::Colon) {
                c.bump(); // ':'
                limiter = Some(parse_limiter_spec(c)?);
            }

            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let full_span = Span::join(lspan, Span::new(rbrace.start, rbrace.end));

            let cap = CaptureNode {
                name: term_value,
                name_span: term_span,
                limiter,
                span: full_span,
            };
            PlaceholderNode::Capture(cap)
        }

        _ => {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                next.start,
                Some(Span::new(next.start, next.end)),
                "expected '|' (one-of), ':' (limiter), or '}' (end of placeholder)",
            ));
        }
    };

    // The parse branches already consumed the rbrace. Ensure we are not mid-placeholder.
    // (This is mostly a sanity check; if you later allow more forms it may matter.)
    let span = match &node {
        PlaceholderNode::OneOf(n) => n.span,
        PlaceholderNode::Capture(n) => n.span,
    };

    Ok(PlaceholderPart { node, span })
}

fn parse_limiter_spec(c: &mut Cursor<'_>) -> Result<LimiterSpec, ParseError> {
    let name_tok = c.peek().ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::UnexpectedEof,
            0,
            None,
            "expected limiter name after ':'",
        )
    })?;

    let (name, name_span) = match &name_tok.token {
        Token::Ident(s) => {
            let sp = Span::new(name_tok.start, name_tok.end);
            c.bump();
            (s.clone(), sp)
        }
        _ => {
            return Err(ParseError::new(
                ParseErrorKind::ExpectedToken("IDENT"),
                name_tok.start,
                Some(Span::new(name_tok.start, name_tok.end)),
                "expected limiter identifier after ':'",
            ));
        }
    };

    // Optional "( ... )"
    let mut args: Vec<LimiterArg> = Vec::new();
    let mut span = name_span;

    if let Some(t) = c.peek() {
        if matches!(t.token, Token::LParen) {
            let lp = c.bump().unwrap();
            span = Span::join(span, Span::new(lp.start, lp.end));

            // Allow empty args: lim()
            if let Some(t2) = c.peek() {
                if matches!(t2.token, Token::RParen) {
                    let rp = c.bump().unwrap();
                    span = Span::join(span, Span::new(rp.start, rp.end));
                    return Ok(LimiterSpec {
                        name,
                        name_span,
                        args,
                        span,
                    });
                }
            }

            // Parse args: arg (',' arg)*
            loop {
                let a = parse_limiter_arg(c)?;
                span = Span::join(span, a_span(&a));
                args.push(a);

                let next = c.peek().ok_or_else(|| {
                    ParseError::new(
                        ParseErrorKind::UnexpectedEof,
                        span.end,
                        Some(span),
                        "expected ')' or ',' after limiter argument",
                    )
                })?;

                if matches!(next.token, Token::Comma) {
                    c.bump();
                    continue;
                }
                if matches!(next.token, Token::RParen) {
                    let rp = c.bump().unwrap();
                    span = Span::join(span, Span::new(rp.start, rp.end));
                    break;
                }

                return Err(ParseError::new(
                    ParseErrorKind::ExpectedToken("',' or ')'"),
                    next.start,
                    Some(Span::new(next.start, next.end)),
                    "expected ',' or ')' after limiter argument",
                ));
            }
        }
    }

    Ok(LimiterSpec {
        name,
        name_span,
        args,
        span,
    })
}

fn parse_limiter_arg(c: &mut Cursor<'_>) -> Result<LimiterArg, ParseError> {
    let t = c.peek().ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::UnexpectedEof,
            0,
            None,
            "expected limiter argument",
        )
    })?;

    let sp = Span::new(t.start, t.end);

    let arg = match &t.token {
        Token::Number(s) => {
            c.bump();
            LimiterArg::Number {
                value: s.clone(),
                span: sp,
            }
        }
        Token::Ident(s) => {
            c.bump();
            LimiterArg::Ident {
                value: s.clone(),
                span: sp,
            }
        }
        Token::QuotedString(s) => {
            c.bump();
            LimiterArg::Str {
                value: s.clone(),
                span: sp,
            }
        }
        _ => {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                t.start,
                Some(sp),
                "expected number, identifier, or quoted string as limiter argument",
            ));
        }
    };

    Ok(arg)
}

fn a_span(a: &LimiterArg) -> Span {
    match a {
        LimiterArg::Number { span, .. } => *span,
        LimiterArg::Ident { span, .. } => *span,
        LimiterArg::Str { span, .. } => *span,
    }
}

/// Simple cursor over spanned tokens.
struct Cursor<'a> {
    toks: &'a [SpannedToken],
    i: usize,
}

impl<'a> Cursor<'a> {
    fn new(toks: &'a [SpannedToken]) -> Self {
        Self { toks, i: 0 }
    }

    fn is_eof(&self) -> bool {
        self.i >= self.toks.len()
    }

    fn peek(&self) -> Option<&'a SpannedToken> {
        self.toks.get(self.i)
    }

    fn bump(&mut self) -> Option<&'a SpannedToken> {
        let t = self.toks.get(self.i);
        if t.is_some() {
            self.i += 1;
        }
        t
    }

    fn expect_token<F>(
        &mut self,
        expected: &'static str,
        pred: F,
    ) -> Result<&'a SpannedToken, ParseError>
    where
        F: Fn(&Token) -> bool,
    {
        let t = self.peek().ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                0,
                None,
                format!("expected {}", expected),
            )
        })?;

        if pred(&t.token) {
            Ok(self.bump().unwrap())
        } else {
            Err(ParseError::new(
                ParseErrorKind::ExpectedToken(expected),
                t.start,
                Some(Span::new(t.start, t.end)),
                format!("expected {}, got {:?}", expected, t.token),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_parts() {
        let ast = parse_component(r#"ab"*"{x}"#).unwrap();
        assert_eq!(ast.parts.len(), 4);

        match &ast.parts[0] {
            Part::Literal(l) => assert_eq!(l.value, "ab"),
            _ => panic!("expected literal"),
        }
        match &ast.parts[1] {
            Part::Literal(l) => assert_eq!(l.value, "*"),
            _ => panic!("expected quoted literal => literal part"),
        }
        match &ast.parts[2] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
        match &ast.parts[3] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::Capture(c) => assert_eq!(c.name, "x"),
                _ => panic!("expected capture"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_oneof() {
        let ast = parse_component(r#"{mp4|mkv|"""x"""}"#).unwrap();
        assert_eq!(ast.parts.len(), 1);
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 3);
    }

    #[test]
    fn parse_capture_with_limiter_args() {
        let ast = parse_component(r#"{ year : int( 4, "x" ) }"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        assert_eq!(c.name, "year");
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "int");
        assert_eq!(lim.args.len(), 2);
    }

    #[test]
    fn quoted_name_in_capture_is_rejected() {
        let err = parse_component(r#"{ "nope" : int(4) }"#).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnexpectedToken));
    }

    #[test]
    fn empty_oneof_arm_is_rejected() {
        let err = parse_component(r#"{a|}"#).unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyOneOfArm));
    }
}
