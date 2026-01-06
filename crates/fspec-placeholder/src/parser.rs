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
            span: Some(span(err.at, err.at.saturating_add(1))),
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
                let sp = span(t.start, t.end);
                let value = s.clone();
                c.bump();
                parts.push(Part::Literal(LiteralPart { value, span: sp }));
            }
            Token::QuotedString(s) => {
                // Outside braces, QuotedString is just literal text.
                let sp = span(t.start, t.end);
                let value = s.clone();
                c.bump();
                parts.push(Part::Literal(LiteralPart { value, span: sp }));
            }
            Token::Star => {
                let sp = span(t.start, t.end);
                c.bump();
                parts.push(Part::Star(sp));
            }
            Token::LBrace => {
                let ph = parse_placeholder(&mut c)?;
                parts.push(Part::Placeholder(ph));
            }
            other => {
                // These should never appear outside braces if tokenizer is correct.
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    t.start,
                    Some(span(t.start, t.end)),
                    format!("unexpected token outside braces: {:?}", other),
                ));
            }
        }
    }

    Ok(ComponentAst { parts })
}

fn parse_placeholder(c: &mut Cursor<'_>) -> Result<PlaceholderPart, ParseError> {
    let lbrace = c.expect_token("'{'", |t| matches!(t, Token::LBrace))?;
    let lspan = span(lbrace.start, lbrace.end);

    let first = c.peek().ok_or_else(|| {
        ParseError::new(
            ParseErrorKind::UnexpectedEof,
            lbrace.end,
            Some(lspan),
            "unexpected end of input after '{'",
        )
    })?;

    // "{}" (or "{   }") should not happen often because tokenizer skips WS inside braces,
    // but we keep this check for nicer errors.
    if matches!(first.token, Token::RBrace) {
        let r = c.bump().unwrap();
        let full = span_join(lspan, span(r.start, r.end));
        return Err(ParseError::new(
            ParseErrorKind::EmptyPlaceholder,
            first.start,
            Some(full),
            "empty placeholder '{}' is not allowed",
        ));
    }

    // Check for anonymous placeholder: {:limiter} or {:limiter(args)} or {:choice|choice}
    // Anonymous placeholders start with ':' directly after '{'
    if matches!(first.token, Token::Colon) {
        c.bump(); // consume ':'

        // Look ahead to determine if this is an anonymous one-of or anonymous capture with limiter
        let after_colon = c.peek().ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                first.end,
                Some(span(first.start, first.end)),
                "expected limiter or one-of choice after ':'",
            )
        })?;

        let after_that = c.toks.get(c.i + 1);

        // Check if this looks like an anonymous one-of: {:choice|choice}
        // We need: IDENT/QuotedString followed by '|'
        let looks_like_anonymous_oneof =
            matches!(after_colon.token, Token::Ident(_) | Token::QuotedString(_))
                && after_that
                    .map(|t| matches!(t.token, Token::Pipe))
                    .unwrap_or(false);

        if looks_like_anonymous_oneof {
            // Parse as anonymous one-of: {:choice|choice}
            let first_choice_tok = c.peek().ok_or_else(|| {
                ParseError::new(
                    ParseErrorKind::UnexpectedEof,
                    after_colon.start,
                    Some(span(after_colon.start, after_colon.end)),
                    "expected choice after ':' in anonymous one-of",
                )
            })?;

            let (first_choice_kind, first_choice_value, first_choice_span) = match &first_choice_tok
                .token
            {
                Token::Ident(s) => {
                    let sp = span(first_choice_tok.start, first_choice_tok.end);
                    c.bump();
                    ("ident", s.clone(), sp)
                }
                Token::QuotedString(s) => {
                    let sp = span(first_choice_tok.start, first_choice_tok.end);
                    c.bump();
                    ("str", s.clone(), sp)
                }
                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken,
                        first_choice_tok.start,
                        Some(span(first_choice_tok.start, first_choice_tok.end)),
                        "expected identifier or quoted string as first choice in anonymous one-of",
                    ));
                }
            };

            let mut choices: Vec<Choice> = Vec::new();
            choices.push(match first_choice_kind {
                "ident" => Choice::Ident {
                    value: first_choice_value,
                    span: first_choice_span,
                },
                _ => Choice::Str {
                    value: first_choice_value,
                    span: first_choice_span,
                },
            });

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
                        Some(span(t.start, t.end)),
                        "expected choice after '|' in anonymous one-of",
                    )
                })?;

                // Reject empty arm: "{:a|}"
                if matches!(choice_tok.token, Token::RBrace) {
                    return Err(ParseError::new(
                        ParseErrorKind::EmptyOneOfArm,
                        choice_tok.start,
                        Some(span(choice_tok.start, choice_tok.end)),
                        "empty one-of arm is not allowed",
                    ));
                }

                let ch = match &choice_tok.token {
                    Token::Ident(s) => {
                        let sp = span(choice_tok.start, choice_tok.end);
                        c.bump();
                        Choice::Ident {
                            value: s.clone(),
                            span: sp,
                        }
                    }
                    Token::QuotedString(s) => {
                        let sp = span(choice_tok.start, choice_tok.end);
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
                            Some(span(choice_tok.start, choice_tok.end)),
                            "expected identifier or quoted string as one-of choice",
                        ));
                    }
                };

                choices.push(ch);
            }

            if !saw_pipe {
                return Err(ParseError::new(
                    ParseErrorKind::EmptyOneOf,
                    first_choice_span.start,
                    Some(first_choice_span),
                    "one-of must contain at least one '|'",
                ));
            }

            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let full = span_join(lspan, span(rbrace.start, rbrace.end));

            return Ok(PlaceholderPart {
                node: PlaceholderNode::OneOf(OneOfNode {
                    name: None, // anonymous one-of has no name
                    choices,
                    span: full,
                }),
                span: full,
            });
        } else {
            // Parse as anonymous capture with limiter: {:limiter} or {:limiter(args)}
            let limiter = Some(parse_limiter_spec(c)?);
            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let full = span_join(lspan, span(rbrace.start, rbrace.end));

            return Ok(PlaceholderPart {
                node: PlaceholderNode::Capture(CaptureNode {
                    name: String::new(),                     // empty name for anonymous placeholder
                    name_span: span(lbrace.end, lbrace.end), // empty span at start of placeholder
                    limiter,
                    span: full,
                }),
                span: full,
            });
        }
    }

    // First term: Ident or QuotedString
    let (term_kind, term_value, term_span) = match &first.token {
        Token::Ident(s) => {
            let sp = span(first.start, first.end);
            c.bump();
            ("ident", s.clone(), sp)
        }
        Token::QuotedString(s) => {
            let sp = span(first.start, first.end);
            c.bump();
            ("str", s.clone(), sp)
        }
        _ => {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                first.start,
                Some(span(first.start, first.end)),
                "expected identifier, quoted string, or ':' (for anonymous placeholder) inside '{...}'",
            ));
        }
    };

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
            // OneOf: choice ('|' choice)+
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
                        Some(span(t.start, t.end)),
                        "expected choice after '|'",
                    )
                })?;

                // Reject empty arm: "{a|}"
                if matches!(choice_tok.token, Token::RBrace) {
                    return Err(ParseError::new(
                        ParseErrorKind::EmptyOneOfArm,
                        choice_tok.start,
                        Some(span(choice_tok.start, choice_tok.end)),
                        "empty one-of arm is not allowed",
                    ));
                }

                let ch = match &choice_tok.token {
                    Token::Ident(s) => {
                        let sp = span(choice_tok.start, choice_tok.end);
                        c.bump();
                        Choice::Ident {
                            value: s.clone(),
                            span: sp,
                        }
                    }
                    Token::QuotedString(s) => {
                        let sp = span(choice_tok.start, choice_tok.end);
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
                            Some(span(choice_tok.start, choice_tok.end)),
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

            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let full = span_join(lspan, span(rbrace.start, rbrace.end));

            PlaceholderNode::OneOf(OneOfNode {
                name: None,
                choices,
                span: full,
            })
        }

        Token::Colon | Token::RBrace => {
            // Capture: name must be IDENT (not quoted)
            if term_kind != "ident" {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    term_span.start,
                    Some(term_span),
                    "capture name must be an identifier (quoted string not allowed here)",
                ));
            }

            // Check if this looks like a named one-of: {name:choice|choice}
            // We need to look ahead: after ':' should be IDENT/Quoted, then '|'
            if matches!(next.token, Token::Colon) {
                let after_colon_idx = c.i + 1;
                let after_colon = c.toks.get(after_colon_idx);
                let after_that = c.toks.get(after_colon_idx + 1);

                if let (Some(after_colon_tok), Some(after_that_tok)) = (after_colon, after_that) {
                    let looks_like_named_oneof = matches!(
                        after_colon_tok.token,
                        Token::Ident(_) | Token::QuotedString(_)
                    ) && matches!(after_that_tok.token, Token::Pipe);

                    if looks_like_named_oneof {
                        // Parse as named one-of: {name:choice|choice}
                        c.bump(); // consume ':'

                        let first_choice_tok = c.peek().ok_or_else(|| {
                            ParseError::new(
                                ParseErrorKind::UnexpectedEof,
                                after_colon_tok.start,
                                Some(span(after_colon_tok.start, after_colon_tok.end)),
                                "expected choice after ':' in named one-of",
                            )
                        })?;

                        let (first_choice_kind, first_choice_value, first_choice_span) =
                            match &first_choice_tok.token {
                                Token::Ident(s) => {
                                    let sp = span(first_choice_tok.start, first_choice_tok.end);
                                    c.bump();
                                    ("ident", s.clone(), sp)
                                }
                                Token::QuotedString(s) => {
                                    let sp = span(first_choice_tok.start, first_choice_tok.end);
                                    c.bump();
                                    ("str", s.clone(), sp)
                                }
                                _ => {
                                    return Err(ParseError::new(
                                        ParseErrorKind::UnexpectedToken,
                                        first_choice_tok.start,
                                        Some(span(first_choice_tok.start, first_choice_tok.end)),
                                        "expected identifier or quoted string as first choice in named one-of",
                                    ));
                                }
                            };

                        let mut choices: Vec<Choice> = Vec::new();
                        choices.push(match first_choice_kind {
                            "ident" => Choice::Ident {
                                value: first_choice_value,
                                span: first_choice_span,
                            },
                            _ => Choice::Str {
                                value: first_choice_value,
                                span: first_choice_span,
                            },
                        });

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
                                    Some(span(t.start, t.end)),
                                    "expected choice after '|' in named one-of",
                                )
                            })?;

                            // Reject empty arm: "{name:a|}"
                            if matches!(choice_tok.token, Token::RBrace) {
                                return Err(ParseError::new(
                                    ParseErrorKind::EmptyOneOfArm,
                                    choice_tok.start,
                                    Some(span(choice_tok.start, choice_tok.end)),
                                    "empty one-of arm is not allowed",
                                ));
                            }

                            let ch = match &choice_tok.token {
                                Token::Ident(s) => {
                                    let sp = span(choice_tok.start, choice_tok.end);
                                    c.bump();
                                    Choice::Ident {
                                        value: s.clone(),
                                        span: sp,
                                    }
                                }
                                Token::QuotedString(s) => {
                                    let sp = span(choice_tok.start, choice_tok.end);
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
                                        Some(span(choice_tok.start, choice_tok.end)),
                                        "expected identifier or quoted string as one-of choice",
                                    ));
                                }
                            };

                            choices.push(ch);
                        }

                        if !saw_pipe {
                            return Err(ParseError::new(
                                ParseErrorKind::EmptyOneOf,
                                first_choice_span.start,
                                Some(first_choice_span),
                                "one-of must contain at least one '|'",
                            ));
                        }

                        let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
                        let full = span_join(lspan, span(rbrace.start, rbrace.end));

                        return Ok(PlaceholderPart {
                            node: PlaceholderNode::OneOf(OneOfNode {
                                name: Some(NamedOneOf {
                                    name: term_value,
                                    name_span: term_span,
                                }),
                                choices,
                                span: full,
                            }),
                            span: full,
                        });
                    }
                }
            }

            // Not a named one-of, parse as capture with optional limiter
            let mut limiter: Option<LimiterSpec> = None;
            if matches!(next.token, Token::Colon) {
                c.bump(); // ':'
                limiter = Some(parse_limiter_spec(c)?);
            }

            let rbrace = c.expect_token("'}'", |t| matches!(t, Token::RBrace))?;
            let full = span_join(lspan, span(rbrace.start, rbrace.end));

            PlaceholderNode::Capture(CaptureNode {
                name: term_value,
                name_span: term_span,
                limiter,
                span: full,
            })
        }

        _ => {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                next.start,
                Some(span(next.start, next.end)),
                "expected '|' (one-of), ':' (limiter), or '}' (end of placeholder)",
            ));
        }
    };

    let ph_span = match &node {
        PlaceholderNode::OneOf(n) => n.span,
        PlaceholderNode::Capture(n) => n.span,
    };

    Ok(PlaceholderPart {
        node,
        span: ph_span,
    })
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
            let sp = span(name_tok.start, name_tok.end);
            c.bump();
            (s.clone(), sp)
        }
        _ => {
            return Err(ParseError::new(
                ParseErrorKind::ExpectedToken("IDENT"),
                name_tok.start,
                Some(span(name_tok.start, name_tok.end)),
                "expected limiter identifier after ':'",
            ));
        }
    };

    let mut args: Vec<LimiterArg> = Vec::new();
    let mut full_span = name_span;

    // Optional "( ... )"
    if c.peek()
        .map(|t| matches!(t.token, Token::LParen))
        .unwrap_or(false)
    {
        let lp = c.bump().unwrap();
        full_span = span_join(full_span, span(lp.start, lp.end));

        // Allow empty args: lim()
        if c.peek()
            .map(|t| matches!(t.token, Token::RParen))
            .unwrap_or(false)
        {
            let rp = c.bump().unwrap();
            full_span = span_join(full_span, span(rp.start, rp.end));
            return Ok(LimiterSpec {
                name,
                name_span,
                args,
                span: full_span,
            });
        }

        loop {
            let a = parse_limiter_arg(c)?;
            full_span = span_join(full_span, limiter_arg_span(&a));
            args.push(a);

            let next = c.peek().ok_or_else(|| {
                ParseError::new(
                    ParseErrorKind::UnexpectedEof,
                    full_span.end,
                    Some(full_span),
                    "expected ')' or ',' after limiter argument",
                )
            })?;

            if matches!(next.token, Token::Comma) {
                c.bump();
                continue;
            }
            if matches!(next.token, Token::RParen) {
                let rp = c.bump().unwrap();
                full_span = span_join(full_span, span(rp.start, rp.end));
                break;
            }

            return Err(ParseError::new(
                ParseErrorKind::ExpectedToken("',' or ')'"),
                next.start,
                Some(span(next.start, next.end)),
                "expected ',' or ')' after limiter argument",
            ));
        }
    }

    Ok(LimiterSpec {
        name,
        name_span,
        args,
        span: full_span,
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

    let sp = span(t.start, t.end);

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

fn limiter_arg_span(a: &LimiterArg) -> Span {
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
                Some(span(t.start, t.end)),
                format!("expected {}, got {:?}", expected, t.token),
            ))
        }
    }
}

// ---- span helpers (avoid depending on Span::new / Span::join) ----

#[inline]
fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

#[inline]
fn span_join(a: Span, b: Span) -> Span {
    Span {
        start: a.start.min(b.start),
        end: a.end.max(b.end),
    }
}
