// tests/parse.rs
use fspec_pattern::{parse_pattern, Node};

#[test]
fn parses_simple_path() {
    let p = parse_pattern("movies/{year}/**").unwrap();

    assert_eq!(
        p.nodes,
        vec![
            Node::Literal("movies".into()),
            Node::Slash,
            Node::Placeholder { name: "year".into() },
            Node::Slash,
            Node::GlobStar,
        ]
    );
}

#[test]
fn parses_file_name() {
    let p = parse_pattern("title.mp4").unwrap();

    assert_eq!(
        p.nodes,
        vec![
            Node::Literal("title.mp4".into()),
        ]
    );
}
