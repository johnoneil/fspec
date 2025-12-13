// src/parser.rs
use crate::ast::{Limiter, Node, Pattern};
use chumsky::prelude::*;

pub fn pattern_parser() -> impl Parser<char, Pattern, Error = Simple<char>> {
    let ident = text::ident().map(|s: String| s);

    // ----- limiter parsing -----

    // unsigned integer like 3, 12, 999
    let uint = text::int(10).from_str::<usize>().unwrapped();

    // int(3) or int(3+)   (also allow plain "int" as a default)
    let int_limiter = just("int")
        .ignore_then(
            just('(')
                .ignore_then(uint)
                .then(just('+').or_not())
                .then_ignore(just(')'))
                .or_not(),
        )
        .map(|opt| {
            match opt {
                // plain "int"
                None => Limiter::Int {
                    min_digits: 1,
                    allow_more: true,
                },
                // int(n) or int(n+)
                Some((min_digits, plus)) => Limiter::Int {
                    min_digits,
                    allow_more: plus.is_some(),
                },
            }
        });

    let limiter = choice((
        int_limiter,
        just("PascalCase").to(Limiter::PascalCase),
        just("camelCase").to(Limiter::CamelCase),
        just("snake_case").to(Limiter::SnakeCase),
        just("kebab-case").to(Limiter::KebabCase),
        just("flatcase").to(Limiter::FlatCase),
        just("UPPER_CASE").to(Limiter::UpperCase),
    ));

    // ----- placeholder parsing -----

    // {name} or {name:limiter}
    let placeholder = ident
        .then(just(':').ignore_then(limiter).or_not())
        .delimited_by(just('{'), just('}'))
        .map(|(name, limiter)| Node::Placeholder { name, limiter });

    // ... your existing pieces ...
    let globstar = just("**").to(Node::GlobStar);
    let slash = just('/').to(Node::Slash);

    let literal = none_of("{}/*")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Node::Literal);

    let node = choice((globstar, placeholder, slash, literal));

    node.repeated()
        .then_ignore(end())
        .map(|nodes| Pattern { nodes })
}
