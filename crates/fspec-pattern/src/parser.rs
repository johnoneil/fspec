// src/parser.rs
use chumsky::prelude::*;
use crate::ast::{Node, Pattern};

pub fn pattern_parser() -> impl Parser<char, Pattern, Error = Simple<char>> {
    // identifier: year, title, show_name
    let ident =
        text::ident()
            .map(|s: String| s);

    // {name}
    let placeholder =
        ident
            .delimited_by(just('{'), just('}'))
            .map(|name| Node::Placeholder { name });

    // **
    let globstar =
        just("**")
            .to(Node::GlobStar);

    // /
    let slash =
        just('/')
            .to(Node::Slash);

    // literal chunk (stop at / { })
    let literal =
        filter(|c: &char| *c != '/' && *c != '{' && *c != '}')
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(Node::Literal);

    let node =
        choice((
            globstar,
            placeholder,
            slash,
            literal,
        ));

    node
        .repeated()
        .then_ignore(end())
        .map(|nodes| Pattern { nodes })
}
