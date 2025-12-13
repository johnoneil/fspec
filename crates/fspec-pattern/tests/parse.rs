use fspec_pattern::{Limiter, Node, parse_pattern};

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
                limiter: Some(Limiter::CamelCase)
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
