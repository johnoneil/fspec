// fspec-placeholder/src/tokenizer.rs

#![allow(dead_code)]

use std::fmt;

/// Token kinds produced by the component tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Structural single-char tokens
    Star,   // '*'
    LBrace, // '{'
    RBrace, // '}'
    Colon,  // ':'
    Pipe,   // '|'
    LParen, // '('
    RParen, // ')'
    Comma,  // ','

    // Value tokens
    /// An unquoted literal run outside braces. May contain spaces.
    LiteralRun(String),

    /// A double-quoted string (outside or inside braces).
    /// The value is unescaped: `""` in the source becomes `"` in the output.
    QuotedString(String),

    /// An identifier inside braces (or if you ever want it elsewhere).
    Ident(String),

    /// A number (digits) inside braces.
    Number(String),
}

/// Token with span information (byte offsets in the original input).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub start: usize, // inclusive
    pub end: usize,   // exclusive
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeErrorKind {
    UnterminatedQuotedString,
    UnmatchedRBrace,
    UnexpectedChar(char),
    InvalidIdentStart(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeError {
    pub kind: TokenizeErrorKind,
    pub at: usize, // byte offset
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenizeErrorKind::*;
        match self.kind {
            UnterminatedQuotedString => write!(f, "unterminated quoted string at byte {}", self.at),
            UnmatchedRBrace => write!(f, "unmatched '}}' at byte {}", self.at),
            UnexpectedChar(c) => write!(f, "unexpected character {:?} at byte {}", c, self.at),
            InvalidIdentStart(c) => {
                write!(f, "invalid identifier start {:?} at byte {}", c, self.at)
            }
        }
    }
}

impl std::error::Error for TokenizeError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    OutsideBraces,
    InsideBraces,
}

/// Tokenizer for a single fspec *component* (string between `/` separators).
///
/// Key behaviors (per grammar):
/// - Outside braces:
///   - `*` => Star
///   - `{` => LBrace (switch to inside-braces mode)
///   - `"` => QuotedString (supports `""` escape)
///   - otherwise: LiteralRun up to next special char among `* { } "`
/// - Inside braces:
///   - whitespace is skipped (tolerant formatting)
///   - recognizes: `}` `:` `|` `(` `)` `,`
///   - `"` => QuotedString
///   - IDENT => Ident
///   - NUMBER => Number
///
/// Notes:
/// - A `}` outside braces is considered an error (unmatched).
pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize, // byte offset
    mode: Mode,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            mode: Mode::OutsideBraces,
        }
    }

    pub fn tokenize_all(mut self) -> Result<Vec<SpannedToken>, TokenizeError> {
        let mut out = Vec::new();
        while let Some(tok) = self.next_token()? {
            out.push(tok);
        }
        Ok(out)
    }

    /// Returns next spanned token, or Ok(None) at end-of-input.
    pub fn next_token(&mut self) -> Result<Option<SpannedToken>, TokenizeError> {
        if self.pos >= self.input.len() {
            return Ok(None);
        }

        match self.mode {
            Mode::OutsideBraces => self.next_outside(),
            Mode::InsideBraces => self.next_inside(),
        }
    }

    fn next_outside(&mut self) -> Result<Option<SpannedToken>, TokenizeError> {
        let b = self.peek_byte().unwrap();
        let start = self.pos;

        match b {
            b'*' => {
                self.pos += 1;
                Ok(Some(SpannedToken {
                    token: Token::Star,
                    start,
                    end: self.pos,
                }))
            }
            b'{' => {
                self.pos += 1;
                self.mode = Mode::InsideBraces;
                Ok(Some(SpannedToken {
                    token: Token::LBrace,
                    start,
                    end: self.pos,
                }))
            }
            b'}' => Err(TokenizeError {
                kind: TokenizeErrorKind::UnmatchedRBrace,
                at: self.pos,
            }),
            b'"' => {
                let (s, end) = self.read_quoted_string()?;
                Ok(Some(SpannedToken {
                    token: Token::QuotedString(s),
                    start,
                    end,
                }))
            }
            _ => {
                // LiteralRun: read until next special: * { } "
                let end = self.scan_until_any(&[b'*', b'{', b'}', b'"']);
                let lit = &self.input[start..end];
                self.pos = end;
                Ok(Some(SpannedToken {
                    token: Token::LiteralRun(lit.to_string()),
                    start,
                    end,
                }))
            }
        }
    }

    fn next_inside(&mut self) -> Result<Option<SpannedToken>, TokenizeError> {
        // Skip whitespace (tolerant inside braces)
        self.skip_ws();
        if self.pos >= self.input.len() {
            return Ok(None);
        }

        let start = self.pos;
        let b = self.peek_byte().unwrap();

        let single = match b {
            b'}' => Some(Token::RBrace),
            b':' => Some(Token::Colon),
            b'|' => Some(Token::Pipe),
            b'(' => Some(Token::LParen),
            b')' => Some(Token::RParen),
            b',' => Some(Token::Comma),
            _ => None,
        };

        if let Some(tok) = single {
            self.pos += 1;
            if matches!(tok, Token::RBrace) {
                self.mode = Mode::OutsideBraces;
            }
            return Ok(Some(SpannedToken {
                token: tok,
                start,
                end: self.pos,
            }));
        }

        if b == b'"' {
            let (s, end) = self.read_quoted_string()?;
            return Ok(Some(SpannedToken {
                token: Token::QuotedString(s),
                start,
                end,
            }));
        }

        // Number?
        if is_ascii_digit(b) {
            let end = self.scan_while(is_ascii_digit);
            let num = self.input[start..end].to_string();
            self.pos = end;
            return Ok(Some(SpannedToken {
                token: Token::Number(num),
                start,
                end,
            }));
        }

        // Ident?
        if is_ident_start(b) {
            let end = self.scan_while(is_ident_continue);
            let ident = self.input[start..end].to_string();
            self.pos = end;
            return Ok(Some(SpannedToken {
                token: Token::Ident(ident),
                start,
                end,
            }));
        }

        // Any other char inside braces is unexpected (since WS already skipped)
        let ch = self.peek_char().unwrap_or('\0');
        if !ch.is_ascii() {
            // Still report it; caller can decide how to handle non-ASCII in future.
        }
        Err(TokenizeError {
            kind: TokenizeErrorKind::UnexpectedChar(ch),
            at: self.pos,
        })
    }

    fn read_quoted_string(&mut self) -> Result<(String, usize), TokenizeError> {
        // At opening quote
        let start = self.pos;
        debug_assert_eq!(self.peek_byte(), Some(b'"'));
        self.pos += 1;

        let mut out = String::new();
        while self.pos < self.input.len() {
            let b = self.peek_byte().unwrap();
            if b == b'"' {
                // Could be end quote OR escaped quote `""`
                if self.peek_byte_at(self.pos + 1) == Some(b'"') {
                    // Escaped quote -> consume both, emit one
                    out.push('"');
                    self.pos += 2;
                    continue;
                } else {
                    // End quote
                    self.pos += 1;
                    return Ok((out, self.pos));
                }
            }

            // Copy one UTF-8 char
            let ch = self.peek_char().unwrap();
            out.push(ch);
            self.pos += ch.len_utf8();
        }

        Err(TokenizeError {
            kind: TokenizeErrorKind::UnterminatedQuotedString,
            at: start,
        })
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() {
            let b = self.peek_byte().unwrap();
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn scan_until_any(&self, bytes: &[u8]) -> usize {
        let mut i = self.pos;
        while i < self.input.len() {
            let b = self.input.as_bytes()[i];
            if bytes.contains(&b) {
                break;
            }
            // advance by UTF-8 char length
            let ch = self.input[i..].chars().next().unwrap();
            i += ch.len_utf8();
        }
        i
    }

    fn scan_while<F>(&self, mut pred: F) -> usize
    where
        F: FnMut(u8) -> bool,
    {
        let mut i = self.pos;
        while i < self.input.len() {
            let b = self.input.as_bytes()[i];
            if !pred(b) {
                break;
            }
            i += 1;
        }
        i
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn peek_byte_at(&self, pos: usize) -> Option<u8> {
        self.input.as_bytes().get(pos).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
}

// Helpers: ASCII-only for now (good fit for initial conformance).
fn is_ascii_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

fn is_ident_start(b: u8) -> bool {
    (b'a' <= b && b <= b'z') || (b'A' <= b && b <= b'Z') || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    is_ident_start(b) || is_ascii_digit(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(input: &str) -> Vec<Token> {
        Tokenizer::new(input)
            .tokenize_all()
            .unwrap()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn outside_literals_and_star() {
        assert_eq!(
            toks(r#"ab*c"#),
            vec![
                Token::LiteralRun("ab".into()),
                Token::Star,
                Token::LiteralRun("c".into()),
            ]
        );
    }

    #[test]
    fn outside_quoted_literal() {
        assert_eq!(
            toks(r#""***filename_literal***".o"#),
            vec![
                Token::QuotedString("***filename_literal***".into()),
                Token::LiteralRun(".o".into()),
            ]
        );
    }

    #[test]
    fn inside_braces_skips_ws_and_tokens() {
        assert_eq!(
            toks(r#"{ year : int( 4 ) }"#),
            vec![
                Token::LBrace,
                Token::Ident("year".into()),
                Token::Colon,
                Token::Ident("int".into()),
                Token::LParen,
                Token::Number("4".into()),
                Token::RParen,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn oneof_with_quoted_choices() {
        assert_eq!(
            toks(r#"{ "mp*4" | "m/v" | """in quotes""" }"#),
            vec![
                Token::LBrace,
                Token::QuotedString("mp*4".into()),
                Token::Pipe,
                Token::QuotedString("m/v".into()),
                Token::Pipe,
                Token::QuotedString(r#""in quotes""#.into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn unmatched_rbrace_is_error() {
        let err = Tokenizer::new("}").tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnmatchedRBrace);
    }

    #[test]
    fn unterminated_quote_is_error() {
        let err = Tokenizer::new(r#""abc"#).tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnterminatedQuotedString);
    }

    // ===== Additional comprehensive tests =====

    #[test]
    fn empty_input() {
        assert_eq!(toks(""), vec![]);
    }

    #[test]
    fn only_literal_run() {
        assert_eq!(toks("hello"), vec![Token::LiteralRun("hello".into())]);
    }

    #[test]
    fn literal_run_with_spaces() {
        assert_eq!(
            toks("hello world"),
            vec![Token::LiteralRun("hello world".into())]
        );
    }

    #[test]
    fn multiple_stars() {
        assert_eq!(
            toks("*.*"),
            vec![Token::Star, Token::LiteralRun(".".into()), Token::Star]
        );
    }

    #[test]
    fn only_star() {
        assert_eq!(toks("*"), vec![Token::Star]);
    }

    #[test]
    fn multiple_quoted_strings() {
        assert_eq!(
            toks(r#""a" "b""#),
            vec![
                Token::QuotedString("a".into()),
                Token::LiteralRun(" ".into()),
                Token::QuotedString("b".into()),
            ]
        );
    }

    #[test]
    fn quoted_string_with_escaped_quotes() {
        assert_eq!(
            toks(r#""say ""hello""""#),
            vec![Token::QuotedString(r#"say "hello""#.into())]
        );
    }

    #[test]
    fn quoted_string_escaped_quote_at_start() {
        assert_eq!(
            toks(r#""""hello""#),
            vec![Token::QuotedString(r#""hello"#.into())]
        );
    }

    #[test]
    fn quoted_string_escaped_quote_at_end() {
        assert_eq!(
            toks(r#""hello""""#),
            vec![Token::QuotedString(r#"hello""#.into())]
        );
    }

    #[test]
    fn quoted_string_only_escaped_quotes() {
        assert_eq!(toks(r#""""""#), vec![Token::QuotedString("\"".into())]);
    }

    #[test]
    fn quoted_string_empty() {
        assert_eq!(toks(r#""""#), vec![Token::QuotedString("".into())]);
    }

    #[test]
    fn literal_run_with_special_chars_except_reserved() {
        assert_eq!(
            toks("file-name_123.test"),
            vec![Token::LiteralRun("file-name_123.test".into())]
        );
    }

    #[test]
    fn literal_run_stops_at_star() {
        assert_eq!(
            toks("prefix*suffix"),
            vec![
                Token::LiteralRun("prefix".into()),
                Token::Star,
                Token::LiteralRun("suffix".into()),
            ]
        );
    }

    #[test]
    fn literal_run_stops_at_brace() {
        assert_eq!(
            toks("prefix{suffix}"),
            vec![
                Token::LiteralRun("prefix".into()),
                Token::LBrace,
                Token::Ident("suffix".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn literal_run_stops_at_quote() {
        assert_eq!(
            toks(r#"prefix"quoted"suffix"#),
            vec![
                Token::LiteralRun("prefix".into()),
                Token::QuotedString("quoted".into()),
                Token::LiteralRun("suffix".into()),
            ]
        );
    }

    #[test]
    fn inside_braces_just_ident() {
        assert_eq!(
            toks("{x}"),
            vec![Token::LBrace, Token::Ident("x".into()), Token::RBrace,]
        );
    }

    #[test]
    fn inside_braces_just_number() {
        assert_eq!(
            toks("{123}"),
            vec![Token::LBrace, Token::Number("123".into()), Token::RBrace,]
        );
    }

    #[test]
    fn inside_braces_just_quoted_string() {
        assert_eq!(
            toks(r#"{ "hello" }"#),
            vec![
                Token::LBrace,
                Token::QuotedString("hello".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_whitespace_tolerance_around_colon() {
        assert_eq!(
            toks("{x:y}"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::RBrace,
            ]
        );
        assert_eq!(
            toks("{ x : y }"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_whitespace_tolerance_around_pipe() {
        assert_eq!(
            toks("{a|b}"),
            vec![
                Token::LBrace,
                Token::Ident("a".into()),
                Token::Pipe,
                Token::Ident("b".into()),
                Token::RBrace,
            ]
        );
        assert_eq!(
            toks("{ a | b }"),
            vec![
                Token::LBrace,
                Token::Ident("a".into()),
                Token::Pipe,
                Token::Ident("b".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_whitespace_tolerance_around_parens() {
        assert_eq!(
            toks("{x:y()}"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::LParen,
                Token::RParen,
                Token::RBrace,
            ]
        );
        assert_eq!(
            toks("{ x : y( ) }"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::LParen,
                Token::RParen,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_whitespace_tolerance_around_comma() {
        assert_eq!(
            toks("{x:y(a,b)}"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::LParen,
                Token::Ident("a".into()),
                Token::Comma,
                Token::Ident("b".into()),
                Token::RParen,
                Token::RBrace,
            ]
        );
        assert_eq!(
            toks("{ x : y( a , b ) }"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("y".into()),
                Token::LParen,
                Token::Ident("a".into()),
                Token::Comma,
                Token::Ident("b".into()),
                Token::RParen,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_multiple_numbers() {
        assert_eq!(
            toks("{123 456}"),
            vec![
                Token::LBrace,
                Token::Number("123".into()),
                Token::Number("456".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_ident_with_underscore() {
        assert_eq!(
            toks("{my_var}"),
            vec![Token::LBrace, Token::Ident("my_var".into()), Token::RBrace,]
        );
    }

    #[test]
    fn inside_braces_ident_with_digits() {
        assert_eq!(
            toks("{var123}"),
            vec![Token::LBrace, Token::Ident("var123".into()), Token::RBrace,]
        );
    }

    #[test]
    fn inside_braces_quoted_string_with_special_chars() {
        assert_eq!(
            toks(r#"{ "a*b{c}d" }"#),
            vec![
                Token::LBrace,
                Token::QuotedString("a*b{c}d".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn inside_braces_quoted_string_with_escaped_quotes() {
        assert_eq!(
            toks(r#"{ """quoted""" }"#),
            vec![
                Token::LBrace,
                Token::QuotedString(r#""quoted""#.into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn nested_braces_rejected_by_tokenizer() {
        // Nested braces are not allowed - the tokenizer should reject a { inside braces
        let err = Tokenizer::new("{{x}}").tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnexpectedChar('{'));
        assert_eq!(err.at, 1);
    }

    #[test]
    fn complex_mixed_component() {
        assert_eq!(
            toks(r#"file"*"*{ext|"txt"}"#),
            vec![
                Token::LiteralRun("file".into()),
                Token::QuotedString("*".into()),
                Token::Star,
                Token::LBrace,
                Token::Ident("ext".into()),
                Token::Pipe,
                Token::QuotedString("txt".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn unterminated_quote_inside_braces() {
        let err = Tokenizer::new(r#"{ "abc"#).tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnterminatedQuotedString);
    }

    #[test]
    fn unterminated_quote_after_escaped_quote() {
        let err = Tokenizer::new(r#""abc"""#).tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnterminatedQuotedString);
    }

    #[test]
    fn unmatched_rbrace_after_valid_brace() {
        let err = Tokenizer::new("{x}}").tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnmatchedRBrace);
    }

    #[test]
    fn unmatched_rbrace_in_middle() {
        let err = Tokenizer::new("a}b").tokenize_all().unwrap_err();
        assert_eq!(err.kind, TokenizeErrorKind::UnmatchedRBrace);
    }

    #[test]
    fn whitespace_only_inside_braces() {
        // Whitespace is skipped, so this should result in just braces
        assert_eq!(toks("{   }"), vec![Token::LBrace, Token::RBrace]);
    }

    #[test]
    fn tab_and_newline_whitespace() {
        assert_eq!(
            toks("{\t\n\rx\t\n\r}"),
            vec![Token::LBrace, Token::Ident("x".into()), Token::RBrace,]
        );
    }

    #[test]
    fn number_followed_by_ident() {
        assert_eq!(
            toks("{123abc}"),
            vec![
                Token::LBrace,
                Token::Number("123".into()),
                Token::Ident("abc".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn ident_starting_with_underscore() {
        assert_eq!(
            toks("{_private}"),
            vec![
                Token::LBrace,
                Token::Ident("_private".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn quoted_string_inside_braces_with_spaces() {
        // Spaces inside quoted strings are literal
        assert_eq!(
            toks(r#"{ "hello world" }"#),
            vec![
                Token::LBrace,
                Token::QuotedString("hello world".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn limiter_with_all_arg_types() {
        assert_eq!(
            toks("{x:lim(123, abc, \"str\")}"),
            vec![
                Token::LBrace,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("lim".into()),
                Token::LParen,
                Token::Number("123".into()),
                Token::Comma,
                Token::Ident("abc".into()),
                Token::Comma,
                Token::QuotedString("str".into()),
                Token::RParen,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn empty_quoted_string_inside_braces() {
        assert_eq!(
            toks(r#"{ "" }"#),
            vec![Token::LBrace, Token::QuotedString("".into()), Token::RBrace,]
        );
    }

    #[test]
    fn multiple_consecutive_literal_runs() {
        // This shouldn't happen in practice, but tests the tokenizer behavior
        assert_eq!(
            toks("a*b*c"),
            vec![
                Token::LiteralRun("a".into()),
                Token::Star,
                Token::LiteralRun("b".into()),
                Token::Star,
                Token::LiteralRun("c".into()),
            ]
        );
    }
}
