use fspec_pattern::{Limiter, LimiterKind, Node, Quant, SegPart, Segment, parse_pattern};

#[ignore = "does ** as last member have any meaning? should it be supported?"]
#[test]
fn parses_simple_path() {
    let p = parse_pattern("movies/{year}/**").unwrap();

    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("movies".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: None,
            },])),
            Node::Slash,
            Node::Segment(Segment::GlobStar),
        ]
    );
}

#[test]
fn parses_file_name() {
    let p = parse_pattern("title.mp4").unwrap();

    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![SegPart::Literal(
            "title.mp4".into()
        ),])),]
    );
}

#[test]
fn parses_file_name_unicode() {
    let p = parse_pattern("これは何ですか.mp4").unwrap();

    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![SegPart::Literal(
            "これは何ですか.mp4".into()
        ),])),]
    );
}

#[test]
fn parses_multiple_placeholders() {
    let p = parse_pattern("movies/{year}/{name:camelCase()}_{year}.mp4").unwrap();

    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("movies".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: None,
            }])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![
                SegPart::NamedPlaceholder {
                    name: "name".into(),
                    limiter: Some(Limiter {
                        kind: LimiterKind::CamelCase,
                        quant: Quant::Any,
                    })
                },
                SegPart::Literal("_".into()),
                SegPart::NamedPlaceholder {
                    name: "year".into(),
                    limiter: None,
                },
                SegPart::Literal(".mp4".into()),
            ])),
        ]
    );
}

#[test]
fn parses_literal_only() {
    let p = parse_pattern("movies").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![SegPart::Literal(
            "movies".into()
        ),])),]
    );
}

#[test]
fn parses_literal_unicode_only() {
    let p = parse_pattern("これは何ですか").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![SegPart::Literal(
            "これは何ですか".into()
        ),])),]
    );
}

#[test]
fn parses_slashes_and_literals() {
    let p = parse_pattern("movies/2024/title.mp4").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("movies".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("2024".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("title.mp4".into()),])),
        ]
    );
}

#[test]
fn parses_slashes_and_literals_unicode() {
    let p = parse_pattern("映画/2026/ゴジラ0.mp4").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("映画".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("2026".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(
                vec![SegPart::Literal("ゴジラ0.mp4".into()),]
            )),
        ]
    );
}

#[test]
fn parses_globstar() {
    let p = parse_pattern("root/**/file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::GlobStar),
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_globstar_space_after() {
    let p = parse_pattern("root/** /file.txt").unwrap();
    //debug:
    //println!("{:#?}", parse_pattern("root/** /file.txt"));
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("** ".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_globstar_space_before() {
    let p = parse_pattern("root/ **/file.txt").unwrap();
    //debug:
    //println!("{:#?}", parse_pattern("root/ **/file.txt"));
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal(" **".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_dot() {
    let p = parse_pattern("root/./file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Dot),
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_not_dot() {
    let p = parse_pattern("root/.abcdefg/file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal(".abcdefg".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_doubledot() {
    let p = parse_pattern("root/../file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::DotDot),
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_not_doubledot() {
    let p = parse_pattern("root/..hellothere../file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal(
                "..hellothere..".into()
            ),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_doubledot_with_space() {
    let p = parse_pattern("root/ ../file.txt").unwrap();
    //debug:
    //println!("{:#?}", parse_pattern("root/ ../file.txt"));
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal(" ..".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_star() {
    let p = parse_pattern("root/*/file.txt").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Star),
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_star_with_space() {
    let p = parse_pattern("root/* /file.txt").unwrap();
    // DEBUG:
    //println!("{:#?}", parse_pattern("root/* /file.txt"));
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("root".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("* ".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::Literal("file.txt".into()),])),
        ]
    );
}

#[test]
fn parses_placeholder_without_limiter() {
    let p = parse_pattern("{year}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: None,
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_without_limiter_error_illegal_characters() {
    assert!(parse_pattern("{year-made}").is_err());
}

#[test]
fn parses_placeholder_without_limiter_error_illegal_characters_2() {
    println!("{:#?}", parse_pattern("{映画}"));
    assert!(parse_pattern("{映画}").is_err());
}

#[test]
fn parses_placeholder_with_limiter_no_quant_defaults_to_any() {
    let p = parse_pattern("{ name: camelCase () }").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_limiter_allow_whitespace() {
    let p = parse_pattern("{ name :  camelCase (  )   }").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_exact_quant() {
    let p = parse_pattern("{year:int(4)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Exactly(4),
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_exact_quant_with_whitespace() {
    let p = parse_pattern("{year :int( 4 )}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Exactly(4),
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_at_least_quant() {
    let p = parse_pattern("{id:int(3+)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::AtLeast(3),
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_at_least_quant_tolerable_weird_space() {
    let p = parse_pattern("{id:int(3 +)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::AtLeast(3),
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_at_least_quant_with_whitespace() {
    let p = parse_pattern("{ id:int( 3+ ) }").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::AtLeast(3),
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_range_quant_and_whitespace() {
    let p = parse_pattern("{id:int( 2-5 )}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Range { min: 2, max: 5 },
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_range_quant_no_whitespace() {
    let p = parse_pattern("{id:int(2-5)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Range { min: 2, max: 5 },
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_range_quant_and_tolerable_weird_whitespace() {
    let p = parse_pattern("{id:int(2 -5)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "id".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Range { min: 2, max: 5 },
                })
            },
        ])),]
    );
}

#[test]
fn parses_placeholder_with_range_quant_incorrect_range() {
    assert!(parse_pattern("{id:int(2_5)}").is_err());
}

#[test]
fn parses_placeholder_with_range_quant_error_incomplete_range_1() {
    assert!(parse_pattern("{id:int(2-)}").is_err());
}

#[test]
fn parses_placeholder_with_range_quant_error_incomplete_range_2() {
    assert!(parse_pattern("{id:int(-5)}").is_err());
}

#[test]
fn parses_placeholder_with_range_quant_error_nonsense() {
    assert!(parse_pattern("{id:int(&(Kkjhksjd26)}").is_err());
}

#[test]
fn parses_multiple_placeholders_mixed_with_literals() {
    let p = parse_pattern("movies/{year}/{name:camelCase()}_{year}.mp4").unwrap();
    assert_eq!(
        p.nodes,
        vec![
            Node::Segment(Segment::Parts(vec![SegPart::Literal("movies".into()),])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![SegPart::NamedPlaceholder {
                name: "year".into(),
                limiter: None,
            }])),
            Node::Slash,
            Node::Segment(Segment::Parts(vec![
                SegPart::NamedPlaceholder {
                    name: "name".into(),
                    limiter: Some(Limiter {
                        kind: LimiterKind::CamelCase,
                        quant: Quant::Any,
                    })
                },
                SegPart::Literal("_".into()),
                SegPart::NamedPlaceholder {
                    name: "year".into(),
                    limiter: None,
                },
                SegPart::Literal(".mp4".into()),
            ])),
        ]
    );
}

#[test]
fn error_on_unclosed_placeholder() {
    // TODO: check message
    assert!(parse_pattern("movies/{year").is_err());
}

#[ignore = "need to disallow } in literals"]
#[test]
fn error_on_unopened_placeholder() {
    // TODO: check error message.
    assert!(parse_pattern("movies/year}").is_err());
}

#[test]
fn error_on_colon_without_limiter() {
    // TODO: check message
    assert!(parse_pattern("{name:}").is_err());
}

#[test]
fn error_on_unknown_limiter_kind() {
    assert!(parse_pattern("{x:NotARealLimiter}").is_err());
}

#[test]
fn error_on_bad_quant_missing_close_paren() {
    // TODO: check error message
    assert!(parse_pattern("{x:int(3}").is_err());
}

#[test]
fn error_on_bad_quant_missing_number() {
    // We require limiters to have parentheses for now.
    // TODO: check error message
    assert!(parse_pattern("{x:int}").is_err());
}

#[test]
fn error_on_placeholder_with_space() {
    // TODO: check error message
    assert!(parse_pattern("{no spaces allowed}").is_err());
    //println!("error message: {}", err.message);
}

#[test]
fn placeholder_identifier_allows_underscores_and_digits() {
    let p = parse_pattern("{valid_name_123}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "valid_name_123".into(),
                limiter: None,
            },
        ])),]
    );
}

// Int,
#[test]
fn placeholder_quant_type_int() {
    let p = parse_pattern("{name:int()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// Semver,
#[test]
fn placeholder_quant_type_semver() {
    let p = parse_pattern("{name:semver()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::Semver,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// CamelCase,
#[test]
fn placeholder_quant_type_camelcase() {
    let p = parse_pattern("{name:camelCase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_quant_type_camelcase_2() {
    let p = parse_pattern("{name:camel_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_limiter_camelcase_quant_specific() {
    // exactly 24 chars, not bytes, not codepoints
    let p = parse_pattern("{name:camelCase(24)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Exactly(24),
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_limiter_camelcase_quant_range() {
    // between 10 and 24 characters inclusive, not bytes, not codepoints
    let p = parse_pattern("{name:camelCase(10-24)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Range { min: 10, max: 24 },
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_limiter_camelcase_quant_at_least() {
    // at least 10 characters, not bytes, not codepoints
    let p = parse_pattern("{name:camelCase(10+)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::AtLeast(10),
                })
            },
        ])),]
    );
}

// PascalCase,
#[test]
fn placeholder_quant_type_pascalcase() {
    let p = parse_pattern("{name:PascalCase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::PascalCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// PascalCase,
#[test]
fn placeholder_quant_type_pascalcase_2() {
    let p = parse_pattern("{name:pascal_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::PascalCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// SnakeCase,
#[test]
fn placeholder_quant_type_snakecase() {
    let p = parse_pattern("{name:snake_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::SnakeCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// KebabCase,
#[test]
fn placeholder_quant_type_kebabcase() {
    let p = parse_pattern("{name:kebab-case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::KebabCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// KebabCase,
#[test]
fn placeholder_quant_type_kebabcase_2() {
    let p = parse_pattern("{name:kebab_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::KebabCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// FlatCase,
#[test]
fn placeholder_quant_type_flatcase() {
    let p = parse_pattern("{name:flatcase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::FlatCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_quant_type_flatcase_2() {
    let p = parse_pattern("{name:flat_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::FlatCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// UpperCase,
#[test]
fn placeholder_quant_type_uppercase() {
    let p = parse_pattern("{name:UPPER_CASE()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::UpperCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

#[test]
fn placeholder_quant_type_uppercase_2() {
    let p = parse_pattern("{name:upper_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::NamedPlaceholder {
                name: "name".into(),
                limiter: Some(Limiter {
                    kind: LimiterKind::UpperCase,
                    quant: Quant::Any,
                })
            },
        ])),]
    );
}

// Int,
#[test]
fn anon_placeholder_quant_type_int() {
    let p = parse_pattern("{int()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::Int,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// Semver,
#[test]
fn anon_placeholder_quant_type_semver() {
    let p = parse_pattern("{semver()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::Semver,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// CamelCase,
#[test]
fn anon_placeholder_quant_type_camelcase() {
    let p = parse_pattern("{camelCase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_quant_type_camelcase_2() {
    let p = parse_pattern("{camel_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_limiter_camelcase_quant_specific() {
    // exactly 24 chars, not bytes, not codepoints
    let p = parse_pattern("{camelCase(24)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Exactly(24),
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_limiter_camelcase_quant_range() {
    // between 10 and 24 characters inclusive, not bytes, not codepoints
    let p = parse_pattern("{camelCase(10-24)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::Range { min: 10, max: 24 },
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_limiter_camelcase_quant_at_least() {
    // at least 10 characters, not bytes, not codepoints
    let p = parse_pattern("{camelCase(10+)}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::CamelCase,
                    quant: Quant::AtLeast(10),
                }
            },
        ])),]
    );
}

// PascalCase,
#[test]
fn anon_placeholder_quant_type_pascalcase() {
    let p = parse_pattern("{PascalCase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::PascalCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// PascalCase,
#[test]
fn anon_placeholder_quant_type_pascalcase_2() {
    let p = parse_pattern("{pascal_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::PascalCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// SnakeCase,
#[test]
fn anon_placeholder_quant_type_snakecase() {
    let p = parse_pattern("{snake_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::SnakeCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// KebabCase,
#[test]
fn anon_placeholder_quant_type_kebabcase() {
    let p = parse_pattern("{kebab-case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::KebabCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// KebabCase,
#[test]
fn anon_placeholder_quant_type_kebabcase_2() {
    let p = parse_pattern("{kebab_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::KebabCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// FlatCase,
#[test]
fn anon_placeholder_quant_type_flatcase() {
    let p = parse_pattern("{flatcase()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::FlatCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_quant_type_flatcase_2() {
    let p = parse_pattern("{flat_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::FlatCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

// UpperCase,
#[test]
fn anon_placeholder_quant_type_uppercase() {
    let p = parse_pattern("{UPPER_CASE()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::UpperCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}

#[test]
fn anon_placeholder_quant_type_uppercase_2() {
    let p = parse_pattern("{upper_case()}").unwrap();
    assert_eq!(
        p.nodes,
        vec![Node::Segment(Segment::Parts(vec![
            SegPart::AnonymousPlaceholder {
                limiter: Limiter {
                    kind: LimiterKind::UpperCase,
                    quant: Quant::Any,
                }
            },
        ])),]
    );
}
