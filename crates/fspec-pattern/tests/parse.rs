use fspec_pattern::{Limiter, LimiterKind, Node, Quant, parse_pattern};

#[test]
fn parses_simple_path() {
    let p = parse_pattern("movies/{year}/**").unwrap();

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
            Node::GlobStar,
        ]
    );
}

#[test]
fn parses_file_name() {
    let p = parse_pattern("title.mp4").unwrap();

    assert_eq!(p.nodes, vec![Node::Literal("title.mp4".into()),]);
}

#[test]
fn parses_multiple_placeholders() {
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
                }),
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
