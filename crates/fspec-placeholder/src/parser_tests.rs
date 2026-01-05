#[cfg(test)]
mod tests {
    use crate::ast::{Choice, LimiterArg, Part, PlaceholderNode};
    use crate::parser::{ParseErrorKind, parse_component};

    #[test]
    fn parse_simple_parts() {
        let ast = parse_component(r#"ab"*"*{x}"#).unwrap();
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

    // ===== Additional comprehensive tests =====

    #[test]
    fn parse_empty_component() {
        let ast = parse_component("").unwrap();
        assert_eq!(ast.parts.len(), 0);
    }

    #[test]
    fn parse_only_literal() {
        let ast = parse_component("hello").unwrap();
        assert_eq!(ast.parts.len(), 1);
        match &ast.parts[0] {
            Part::Literal(l) => assert_eq!(l.value, "hello"),
            _ => panic!("expected literal"),
        }
    }

    #[test]
    fn parse_only_star() {
        let ast = parse_component("*").unwrap();
        assert_eq!(ast.parts.len(), 1);
        match &ast.parts[0] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
    }

    #[test]
    fn parse_only_placeholder() {
        let ast = parse_component("{x}").unwrap();
        assert_eq!(ast.parts.len(), 1);
        match &ast.parts[0] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::Capture(c) => assert_eq!(c.name, "x"),
                _ => panic!("expected capture"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_multiple_placeholders() {
        let ast = parse_component("{x}*{y}").unwrap();
        assert_eq!(ast.parts.len(), 3);
        match &ast.parts[0] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::Capture(c) => assert_eq!(c.name, "x"),
                _ => panic!("expected capture"),
            },
            _ => panic!("expected placeholder"),
        }
        match &ast.parts[1] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
        match &ast.parts[2] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::Capture(c) => assert_eq!(c.name, "y"),
                _ => panic!("expected capture"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_oneof_with_only_ident_choices() {
        let ast = parse_component("{a|b|c}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Ident { value, .. } => assert_eq!(value, "a"),
            _ => panic!("expected ident choice"),
        }
        match &one.choices[1] {
            Choice::Ident { value, .. } => assert_eq!(value, "b"),
            _ => panic!("expected ident choice"),
        }
        match &one.choices[2] {
            Choice::Ident { value, .. } => assert_eq!(value, "c"),
            _ => panic!("expected ident choice"),
        }
    }

    #[test]
    fn parse_oneof_with_only_quoted_choices() {
        let ast = parse_component(r#"{ "a" | "b" | "c" }"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Str { value, .. } => assert_eq!(value, "a"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[1] {
            Choice::Str { value, .. } => assert_eq!(value, "b"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[2] {
            Choice::Str { value, .. } => assert_eq!(value, "c"),
            _ => panic!("expected str choice"),
        }
    }

    #[test]
    fn parse_oneof_with_mixed_choices() {
        let ast = parse_component(r#"{a|"b"|c}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Ident { value, .. } => assert_eq!(value, "a"),
            _ => panic!("expected ident choice"),
        }
        match &one.choices[1] {
            Choice::Str { value, .. } => assert_eq!(value, "b"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[2] {
            Choice::Ident { value, .. } => assert_eq!(value, "c"),
            _ => panic!("expected ident choice"),
        }
    }

    #[test]
    fn parse_oneof_with_quoted_choices_containing_special_chars() {
        let ast = parse_component(r#"{ "mp*4" | "m/v" | """in quotes""" }"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Str { value, .. } => assert_eq!(value, "mp*4"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[1] {
            Choice::Str { value, .. } => assert_eq!(value, "m/v"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[2] {
            Choice::Str { value, .. } => assert_eq!(value, r#""in quotes""#),
            _ => panic!("expected str choice"),
        }
    }

    #[test]
    fn parse_capture_without_limiter() {
        let ast = parse_component("{year}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        assert_eq!(c.name, "year");
        assert!(c.limiter.is_none());
    }

    #[test]
    fn parse_capture_with_limiter_no_args() {
        let ast = parse_component("{year:int}").unwrap();
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
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_capture_with_limiter_empty_args() {
        let ast = parse_component("{year:int()}").unwrap();
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
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_capture_with_limiter_single_number_arg() {
        let ast = parse_component("{year:int(4)}").unwrap();
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
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Number { value, .. } => assert_eq!(value, "4"),
            _ => panic!("expected number arg"),
        }
    }

    #[test]
    fn parse_capture_with_limiter_single_ident_arg() {
        let ast = parse_component("{x:re(abc)}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Ident { value, .. } => assert_eq!(value, "abc"),
            _ => panic!("expected ident arg"),
        }
    }

    #[test]
    fn parse_capture_with_limiter_single_str_arg() {
        let ast = parse_component(r#"{x:re("str")}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Str { value, .. } => assert_eq!(value, "str"),
            _ => panic!("expected str arg"),
        }
    }

    #[test]
    fn parse_capture_with_limiter_multiple_args() {
        let ast = parse_component(r#"{x:re(123, abc, "str")}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 3);
        match &lim.args[0] {
            LimiterArg::Number { value, .. } => assert_eq!(value, "123"),
            _ => panic!("expected number arg"),
        }
        match &lim.args[1] {
            LimiterArg::Ident { value, .. } => assert_eq!(value, "abc"),
            _ => panic!("expected ident arg"),
        }
        match &lim.args[2] {
            LimiterArg::Str { value, .. } => assert_eq!(value, "str"),
            _ => panic!("expected str arg"),
        }
    }

    #[test]
    fn parse_whitespace_tolerance_in_placeholders() {
        // These should all parse the same way
        let inputs = vec![
            "{year:int(4)}",
            "{ year : int( 4 ) }",
            "{year :int(4)}",
            "{ year:int(4) }",
        ];
        for input in inputs {
            let ast = parse_component(input).unwrap();
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
            assert_eq!(lim.args.len(), 1);
        }
    }

    #[test]
    fn parse_whitespace_tolerance_in_oneof() {
        let inputs = vec!["{a|b|c}", "{ a | b | c }", "{a |b| c}"];
        for input in inputs {
            let ast = parse_component(input).unwrap();
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
    }

    #[test]
    fn parse_empty_placeholder_rejected() {
        let err = parse_component("{}").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyPlaceholder));
    }

    #[test]
    fn parse_single_ident_is_capture_not_oneof() {
        // A single ident without pipe is a capture, not a oneof
        // One-of requires at least one pipe
        let ast = parse_component("{a}").unwrap();
        match &ast.parts[0] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::Capture(c) => {
                    assert_eq!(c.name, "a");
                }
                _ => panic!("single ident should be capture, not oneof"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_oneof_must_have_pipe() {
        // A oneof must have at least one pipe. If we have {a|b} it's valid.
        // But if we try to parse something that looks like oneof but has no pipe, it's capture
        // Actually, the parser logic: if first token after ident/quoted is pipe, it's oneof
        // So {a} is capture, {a|b} is oneof
        // Let me test that {a|b} requires the pipe
        let ast = parse_component("{a|b}").unwrap();
        match &ast.parts[0] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::OneOf(_) => {} // Correct
                _ => panic!("expected oneof"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_literal_with_spaces() {
        let ast = parse_component("hello world").unwrap();
        assert_eq!(ast.parts.len(), 1);
        match &ast.parts[0] {
            Part::Literal(l) => assert_eq!(l.value, "hello world"),
            _ => panic!("expected literal"),
        }
    }

    #[test]
    fn parse_complex_mixed_component() {
        let ast = parse_component(r#"file"*"*{ext|"txt"}"#).unwrap();
        assert_eq!(ast.parts.len(), 4);
        match &ast.parts[0] {
            Part::Literal(l) => assert_eq!(l.value, "file"),
            _ => panic!("expected literal"),
        }
        match &ast.parts[1] {
            Part::Literal(l) => assert_eq!(l.value, "*"),
            _ => panic!("expected literal"),
        }
        match &ast.parts[2] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
        match &ast.parts[3] {
            Part::Placeholder(p) => match &p.node {
                PlaceholderNode::OneOf(o) => {
                    assert_eq!(o.choices.len(), 2);
                }
                _ => panic!("expected oneof"),
            },
            _ => panic!("expected placeholder"),
        }
    }

    #[test]
    fn parse_quoted_literal_outside_braces() {
        let ast = parse_component(r#""hello".txt"#).unwrap();
        assert_eq!(ast.parts.len(), 2);
        match &ast.parts[0] {
            Part::Literal(l) => assert_eq!(l.value, "hello"),
            _ => panic!("expected literal"),
        }
        match &ast.parts[1] {
            Part::Literal(l) => assert_eq!(l.value, ".txt"),
            _ => panic!("expected literal"),
        }
    }

    #[test]
    fn parse_multiple_stars() {
        let ast = parse_component("*.*").unwrap();
        assert_eq!(ast.parts.len(), 3);
        match &ast.parts[0] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
        match &ast.parts[1] {
            Part::Literal(l) => assert_eq!(l.value, "."),
            _ => panic!("expected literal"),
        }
        match &ast.parts[2] {
            Part::Star(_) => {}
            _ => panic!("expected star"),
        }
    }

    #[test]
    fn parse_capture_with_underscore_name() {
        let ast = parse_component("{_private}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        assert_eq!(c.name, "_private");
    }

    #[test]
    fn parse_capture_with_digit_in_name() {
        let ast = parse_component("{var123}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        assert_eq!(c.name, "var123");
    }

    #[test]
    fn parse_limiter_with_whitespace_around_comma() {
        let ast = parse_component("{x:re( a , b )}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 2);
    }

    #[test]
    fn parse_oneof_starting_with_quoted_string() {
        let ast = parse_component(r#"{ "a" | b }"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 2);
        match &one.choices[0] {
            Choice::Str { value, .. } => assert_eq!(value, "a"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[1] {
            Choice::Ident { value, .. } => assert_eq!(value, "b"),
            _ => panic!("expected ident choice"),
        }
    }

    #[test]
    fn parse_error_unexpected_eof_in_placeholder() {
        let err = parse_component("{x").unwrap_err();
        assert!(matches!(
            err.kind,
            ParseErrorKind::TokenizeFailed | ParseErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn parse_error_unexpected_eof_after_colon() {
        let err = parse_component("{x:").unwrap_err();
        assert!(matches!(
            err.kind,
            ParseErrorKind::UnexpectedEof | ParseErrorKind::ExpectedToken(_)
        ));
    }

    #[test]
    fn parse_error_unexpected_eof_after_limiter_name() {
        let err = parse_component("{x:int").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnexpectedEof));
    }

    #[test]
    fn parse_error_unexpected_eof_in_limiter_args() {
        let err = parse_component("{x:int(4}").unwrap_err();
        assert!(matches!(
            err.kind,
            ParseErrorKind::ExpectedToken(_)
                | ParseErrorKind::UnexpectedToken
                | ParseErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn parse_error_unexpected_eof_after_pipe() {
        let err = parse_component("{a|").unwrap_err();
        assert!(matches!(
            err.kind,
            ParseErrorKind::UnexpectedEof | ParseErrorKind::EmptyOneOfArm
        ));
    }

    #[test]
    fn parse_error_expected_comma_or_rparen_in_limiter() {
        let err = parse_component("{x:int(4 5)}").unwrap_err();
        // This should error because after 4, we expect comma or rparen, not another number
        // Actually, the tokenizer will produce: Ident, Colon, Ident, LParen, Number, Number, RParen
        // The parser should catch that after Number, we need Comma or RParen
        assert!(matches!(
            err.kind,
            ParseErrorKind::ExpectedToken(_) | ParseErrorKind::UnexpectedToken
        ));
    }

    #[test]
    fn parse_error_empty_oneof_arm_at_start() {
        // This is tricky - can we have {|a}? The grammar says oneof needs choice | choice+
        // So the first term must be a choice, not empty
        // But the parser logic checks if first token is Pipe, which it isn't in {|a}
        // Let me check what happens
        let err = parse_component("{|a}").unwrap_err();
        // The first token after LBrace would be Pipe, which is unexpected
        assert!(matches!(err.kind, ParseErrorKind::UnexpectedToken));
    }

    #[test]
    fn parse_limiter_with_quoted_string_arg_containing_special_chars() {
        let ast = parse_component(r#"{x:re("a*b{c}d")}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Str { value, .. } => assert_eq!(value, "a*b{c}d"),
            _ => panic!("expected str arg"),
        }
    }

    #[test]
    fn parse_limiter_with_quoted_string_arg_containing_escaped_quotes() {
        let ast = parse_component(r#"{x:re("""quoted""")}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Str { value, .. } => assert_eq!(value, r#""quoted""#),
            _ => panic!("expected str arg"),
        }
    }

    #[test]
    fn parse_oneof_with_many_choices() {
        let ast = parse_component("{a|b|c|d|e}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert_eq!(one.choices.len(), 5);
    }

    // ===== Tests for named one-of placeholders =====

    #[test]
    fn parse_named_oneof_basic() {
        let ast = parse_component("{ext:mp4|mkv}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_some());
        let name = one.name.as_ref().unwrap();
        assert_eq!(name.name, "ext");
        assert_eq!(one.choices.len(), 2);
        match &one.choices[0] {
            Choice::Ident { value, .. } => assert_eq!(value, "mp4"),
            _ => panic!("expected ident choice"),
        }
        match &one.choices[1] {
            Choice::Ident { value, .. } => assert_eq!(value, "mkv"),
            _ => panic!("expected ident choice"),
        }
    }

    #[test]
    fn parse_named_oneof_with_quoted_choices() {
        let ast = parse_component(r#"{type:"video"|"audio"|"text"}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_some());
        let name = one.name.as_ref().unwrap();
        assert_eq!(name.name, "type");
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Str { value, .. } => assert_eq!(value, "video"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[1] {
            Choice::Str { value, .. } => assert_eq!(value, "audio"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[2] {
            Choice::Str { value, .. } => assert_eq!(value, "text"),
            _ => panic!("expected str choice"),
        }
    }

    #[test]
    fn parse_named_oneof_with_mixed_choices() {
        let ast = parse_component(r#"{format:mp4|"mkv"|avi}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_some());
        assert_eq!(one.name.as_ref().unwrap().name, "format");
        assert_eq!(one.choices.len(), 3);
        match &one.choices[0] {
            Choice::Ident { value, .. } => assert_eq!(value, "mp4"),
            _ => panic!("expected ident choice"),
        }
        match &one.choices[1] {
            Choice::Str { value, .. } => assert_eq!(value, "mkv"),
            _ => panic!("expected str choice"),
        }
        match &one.choices[2] {
            Choice::Ident { value, .. } => assert_eq!(value, "avi"),
            _ => panic!("expected ident choice"),
        }
    }

    #[test]
    fn parse_named_oneof_with_whitespace() {
        let ast = parse_component("{ ext : mp4 | mkv }").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_some());
        assert_eq!(one.name.as_ref().unwrap().name, "ext");
        assert_eq!(one.choices.len(), 2);
    }

    #[test]
    fn parse_named_oneof_many_choices() {
        let ast = parse_component("{ext:mp4|mkv|avi|mov|wmv}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_some());
        assert_eq!(one.name.as_ref().unwrap().name, "ext");
        assert_eq!(one.choices.len(), 5);
    }

    #[test]
    fn parse_named_oneof_requires_pipe() {
        // {ext:mp4} should NOT be parsed as named one-of (no pipe)
        // It should be parsed as a capture with limiter, which will fail validation
        let ast = parse_component("{ext:mp4}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        // Should be a Capture, not OneOf
        match &p.node {
            PlaceholderNode::Capture(c) => {
                assert_eq!(c.name, "ext");
                assert!(c.limiter.is_some());
                assert_eq!(c.limiter.as_ref().unwrap().name, "mp4");
            }
            PlaceholderNode::OneOf(_) => panic!("should not be parsed as oneof (no pipe)"),
        }
    }

    #[test]
    fn parse_unnamed_oneof_has_no_name() {
        // Unnamed one-of should have name: None
        let ast = parse_component("{mp4|mkv}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let one = match &p.node {
            PlaceholderNode::OneOf(o) => o,
            _ => panic!("expected oneof"),
        };
        assert!(one.name.is_none());
        assert_eq!(one.choices.len(), 2);
    }

    // ===== Tests for valid Level-1 limiter names =====

    #[test]
    fn parse_snake_case_limiter() {
        let ast = parse_component("{name:snake_case}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        assert_eq!(c.name, "name");
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "snake_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_snake_case_limiter_with_parens() {
        let ast = parse_component("{name:snake_case()}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "snake_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_kebab_case_limiter() {
        let ast = parse_component("{name:kebab_case}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "kebab_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_pascal_case_limiter() {
        let ast = parse_component("{name:pascal_case}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "pascal_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_upper_case_limiter() {
        let ast = parse_component("{name:upper_case}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "upper_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_lower_case_limiter() {
        let ast = parse_component("{name:lower_case}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "lower_case");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_letters_limiter() {
        let ast = parse_component("{tok:letters}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "letters");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_numbers_limiter() {
        let ast = parse_component("{tok:numbers}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "numbers");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_alnum_limiter() {
        let ast = parse_component("{tok:alnum}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "alnum");
        assert_eq!(lim.args.len(), 0);
    }

    #[test]
    fn parse_int_limiter_with_numeric_arg() {
        let ast = parse_component("{year:int(4)}").unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "int");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Number { value, .. } => assert_eq!(value, "4"),
            _ => panic!("expected number arg"),
        }
    }

    #[test]
    fn parse_re_limiter_with_string_arg() {
        let ast = parse_component(r#"{slug:re("[a-z0-9_-]+")}"#).unwrap();
        let p = match &ast.parts[0] {
            Part::Placeholder(p) => p,
            _ => panic!("expected placeholder"),
        };
        let c = match &p.node {
            PlaceholderNode::Capture(c) => c,
            _ => panic!("expected capture"),
        };
        let lim = c.limiter.as_ref().unwrap();
        assert_eq!(lim.name, "re");
        assert_eq!(lim.args.len(), 1);
        match &lim.args[0] {
            LimiterArg::Str { value, .. } => assert_eq!(value, "[a-z0-9_-]+"),
            _ => panic!("expected str arg"),
        }
    }
}
