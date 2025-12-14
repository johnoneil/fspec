use crate::ast::{Limiter, LimiterKind, Node, Pattern, Quant};
use crate::error::ParseError;

use pest::Parser;
use pest::iterators::Pair;

#[derive(pest_derive::Parser)]
#[grammar = "pattern.pest"]
pub struct PatternParser;

fn parse_limiter_kind(s: &str) -> Result<LimiterKind, ParseError> {
    use LimiterKind::*;

    let kind = match s {
        "int" => Int,

        "semver" => Semver,

        "snake_case" => SnakeCase,

        "kebab_case" | "kebab-case" => KebabCase,

        "camel_case" | "camelCase" => CamelCase,

        "pascal_case" | "PascalCase" => PascalCase,

        "flat_case" | "flatcase" => FlatCase,

        "upper_case" | "UPPER_CASE" => UpperCase,

        _ => {
            return Err(ParseError::new(format!("unknown limiter kind: {s}")));
        }
    };

    Ok(kind)
}

fn parse_quant_leaf(pair: Pair<Rule>) -> Result<Quant, ParseError> {
    Ok(match pair.as_rule() {
        Rule::any => Quant::Any,

        Rule::exactly => {
            let n: usize =
                pair.as_str().trim().parse().map_err(|_| {
                    ParseError::new(format!("invalid exact quant: {}", pair.as_str()))
                })?;
            Quant::Exactly(n)
        }

        Rule::at_least => {
            let mut it = pair.into_inner();
            let n_str = it
                .next()
                .ok_or_else(|| ParseError::new("at_least missing number".into()))?
                .as_str();

            let n: usize = n_str
                .parse()
                .map_err(|_| ParseError::new(format!("invalid at_least number: {n_str}")))?;

            Quant::AtLeast(n)
        }

        Rule::range => {
            // Assumes grammar makes range inner pairs: number, number
            let mut it = pair.into_inner();
            let min: usize = it
                .next()
                .ok_or_else(|| ParseError::new("range missing min".into()))?
                .as_str()
                .trim()
                .parse()
                .map_err(|_| ParseError::new("range min not a number".into()))?;

            let max: usize = it
                .next()
                .ok_or_else(|| ParseError::new("range missing max".into()))?
                .as_str()
                .trim()
                .parse()
                .map_err(|_| ParseError::new("range max not a number".into()))?;

            Quant::Range { min, max }
        }

        _ => {
            return Err(ParseError::new(format!(
                "expected quant, got {:?}",
                pair.as_rule()
            )));
        }
    })
}

fn parse_limiter(pair: Pair<Rule>) -> Result<Limiter, ParseError> {
    let mut inner = pair.into_inner();
    let kind_str = inner.next().unwrap().as_str();
    let kind = parse_limiter_kind(kind_str)?;

    // default if args absent
    let mut quant = Quant::Any;

    if let Some(args) = inner.next() {
        // args is Rule::limiter_args, contains (quant | set_items)
        let payload = args.into_inner().next().unwrap();
        match payload.as_rule() {
            Rule::quant => quant = parse_quant(payload)?,

            // NEW: handle quant leaf rules directly
            Rule::exactly | Rule::at_least | Rule::range | Rule::any => {
                quant = parse_quant_leaf(payload)?;
            }

            Rule::set_items => {
                return Err(ParseError::new(
                    "set-style limiter args are not supported by this AST".to_string(),
                ));
            }

            _ => {}
        }
    }

    Ok(Limiter { kind, quant })
}

pub fn parse_pattern(input: &str) -> Result<Pattern, ParseError> {
    let mut pairs =
        PatternParser::parse(Rule::pattern, input).map_err(|e| ParseError::new(e.to_string()))?;

    let pattern_pair = pairs.next().unwrap();

    let mut nodes = Vec::new();
    for pair in pattern_pair.into_inner() {
        match pair.as_rule() {
            Rule::slash => nodes.push(Node::Slash),
            Rule::globstar => nodes.push(Node::GlobStar),
            Rule::placeholder => nodes.push(parse_placeholder(pair)?),
            Rule::literal => nodes.push(Node::Literal(unescape_literal(pair.as_str()))),
            _ => {}
        }
    }

    // Merge consecutive literals so your AST stays nice.
    let mut merged = Vec::new();
    for n in nodes {
        match (merged.last_mut(), n) {
            (Some(Node::Literal(a)), Node::Literal(b)) => a.push_str(&b),
            (_, n) => merged.push(n),
        }
    }

    Ok(Pattern { nodes: merged })
}

fn parse_placeholder(pair: Pair<Rule>) -> Result<Node, ParseError> {
    match pair.as_rule() {
        Rule::named_placeholder => parse_named_placeholder(pair),
        Rule::anonymous_placeholder => parse_anonymous_placeholder(pair),
        Rule::placeholder => {
            // wrapper: it should contain exactly one inner pair,
            // either named_placeholder or anonymous_placeholder
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::new("empty placeholder".into()))?;
            parse_placeholder(inner)
        }
        _ => Err(ParseError::new("expected placeholder".into())),
    }
}

fn parse_named_placeholder(pair: Pair<Rule>) -> Result<Node, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let limiter = if let Some(next) = inner.next() {
        // next is the limiter (because ":" is a literal token in the grammar)
        Some(parse_limiter(next)?)
    } else {
        None
    };

    Ok(Node::NamedPlaceholder { name, limiter })
}

fn parse_anonymous_placeholder(pair: Pair<Rule>) -> Result<Node, ParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::anonymous_placeholder);

    let mut inner = pair.into_inner();

    let limiter_call = inner
        .next()
        .ok_or_else(|| ParseError::new("anonymous placeholder missing limiter".into()))?;

    // Optionally sanity-check there isn't extra stuff:
    // if inner.next().is_some() { return Err(ParseError::new("unexpected tokens in anonymous placeholder")); }

    let limiter = parse_limiter(limiter_call)?;

    Ok(Node::AnonymousPlaceholder { limiter })
}

fn parse_quant(pair: Pair<Rule>) -> Result<Quant, ParseError> {
    // pair is Rule::quant, inner is one of: range | at_least | exactly | any
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::new("empty quant".into()))?;

    parse_quant_leaf(inner)
}

fn unescape_literal(s: &str) -> String {
    // turns "\{" into "{" etc. Keep it simple initially.
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(next) = it.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}
