use crate::ast::{FSPattern, Limiter, LimiterKind, Node, Quant, SegPart, Segment};
use crate::error::ParseError;

use pest::Parser;
use pest::iterators::Pair;

#[derive(pest_derive::Parser)]
#[grammar = "pattern.pest"]
pub struct FSPatternParser;

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

fn parse_limiter_call(pair: Pair<Rule>) -> Result<Limiter, ParseError> {
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

pub fn parse_pattern(input: &str) -> Result<FSPattern, ParseError> {
    let mut pairs = FSPatternParser::parse(Rule::pattern, input)
        .map_err(|_e| ParseError::new("TODO: translate pest error.".into()))?;

    let pattern_pair = pairs
        .next()
        .ok_or_else(|| ParseError::new("expected pattern".into()))?;

    let mut nodes = Vec::new();
    for p in pattern_pair.into_inner() {
        match p.as_rule() {
            Rule::slash => nodes.push(Node::Slash),
            Rule::segment => nodes.push(Node::Segment(parse_segment(p)?)),
            Rule::soi | Rule::eoi => {}
            other => {
                return Err(ParseError::new(format!(
                    "unexpected rule in pattern: {:?}",
                    other
                )));
            }
        }
    }

    Ok(FSPattern { nodes })
}

fn parse_segment(pair: Pair<Rule>) -> Result<Segment, ParseError> {
    if pair.as_rule() != Rule::segment {
        return Err(ParseError::new(format!(
            "parse_segment expected segment, got {:?}",
            pair.as_rule()
        )));
    }

    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::new("empty segment".into()))?;

    // grammar: segment = { path_segment }
    parse_path_segment(inner)
}

fn parse_path_segment(pair: Pair<Rule>) -> Result<Segment, ParseError> {
    if pair.as_rule() != Rule::path_segment {
        return Err(ParseError::new(format!(
            "parse_path_segment expected path_segment, got {:?}",
            pair.as_rule()
        )));
    }

    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::new("empty path_segment".into()))?;

    match inner.as_rule() {
        Rule::globstar_segment => Ok(Segment::GlobStar),
        Rule::dot_segment => Ok(Segment::Dot),
        Rule::dotdot_segment => Ok(Segment::DotDot),
        Rule::star_segment => Ok(Segment::Star),
        Rule::parts_segment => Ok(Segment::Parts(parse_parts_segment(inner)?)),
        other => Err(ParseError::new(format!(
            "unexpected rule in path_segment: {:?}",
            other
        ))),
    }
}

fn parse_parts_segment(pair: Pair<Rule>) -> Result<Vec<SegPart>, ParseError> {
    if pair.as_rule() != Rule::parts_segment {
        return Err(ParseError::new(format!(
            "parse_parts_segment expected parts_segment, got {:?}",
            pair.as_rule()
        )));
    }

    let mut parts = Vec::new();

    for p in pair.into_inner() {
        eprintln!("parts_segment child: {:?} => {:?}", p.as_rule(), p.as_str());
        match p.as_rule() {
            Rule::literal_piece => {
                parts.push(SegPart::Literal(p.as_str().to_string()));
            }

            Rule::placeholder => {
                parts.push(parse_placeholder_as_part(p)?);
            }

            other => {
                return Err(ParseError::new(format!(
                    "unexpected rule in parts_segment: {:?}",
                    other
                )));
            }
        }
    }

    Ok(parts)
}

fn parse_placeholder_as_part(pair: Pair<Rule>) -> Result<SegPart, ParseError> {
    match pair.as_rule() {
        Rule::named_placeholder => parse_named_placeholder_part(pair),
        Rule::anonymous_placeholder => parse_anonymous_placeholder_part(pair),

        Rule::placeholder => {
            // wrapper: should contain exactly one inner pair
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| ParseError::new("empty placeholder".into()))?;
            parse_placeholder_as_part(inner)
        }

        other => Err(ParseError::new(format!(
            "expected placeholder, got {:?}",
            other
        ))),
    }
}

fn parse_named_placeholder_part(pair: Pair<Rule>) -> Result<SegPart, ParseError> {
    if pair.as_rule() != Rule::named_placeholder {
        return Err(ParseError::new(format!(
            "parse_named_placeholder_part expected named_placeholder, got {:?}",
            pair.as_rule()
        )));
    }

    let mut inner = pair.into_inner();

    // First inner item should be the identifier (name).
    let name_pair = inner
        .next()
        .ok_or_else(|| ParseError::new("named placeholder missing identifier".into()))?;

    if name_pair.as_rule() != Rule::ident {
        return Err(ParseError::new(format!(
            "named placeholder expected identifier, got {:?}",
            name_pair.as_rule()
        )));
    }

    let name = name_pair.as_str().to_string();

    // Optional second item: limiter_call (":" is a literal in grammar so it won't show up here)
    let limiter = match inner.next() {
        None => None,
        Some(p) => {
            if p.as_rule() != Rule::limiter_call {
                return Err(ParseError::new(format!(
                    "named placeholder expected limiter_call, got {:?}",
                    p.as_rule()
                )));
            }
            Some(parse_limiter_call(p)?)
        }
    };

    // Defensive: nothing else should remain
    if let Some(extra) = inner.next() {
        return Err(ParseError::new(format!(
            "unexpected extra content in named placeholder: {:?}",
            extra.as_rule()
        )));
    }

    Ok(SegPart::NamedPlaceholder { name, limiter })
}

fn parse_anonymous_placeholder_part(pair: Pair<Rule>) -> Result<SegPart, ParseError> {
    if pair.as_rule() != Rule::anonymous_placeholder {
        return Err(ParseError::new(format!(
            "parse_anonymous_placeholder_part expected anonymous_placeholder, got {:?}",
            pair.as_rule()
        )));
    }

    let mut inner = pair.into_inner();

    // Should contain exactly one meaningful inner pair: limiter_call
    let lc = inner
        .next()
        .ok_or_else(|| ParseError::new("anonymous placeholder missing limiter".into()))?;

    if lc.as_rule() != Rule::limiter_call {
        return Err(ParseError::new(format!(
            "anonymous placeholder expected limiter_call, got {:?}",
            lc.as_rule()
        )));
    }

    let limiter = parse_limiter_call(lc)?;

    if let Some(extra) = inner.next() {
        return Err(ParseError::new(format!(
            "unexpected extra content in anonymous placeholder: {:?}",
            extra.as_rule()
        )));
    }

    Ok(SegPart::AnonymousPlaceholder { limiter })
}

fn parse_quant(pair: Pair<Rule>) -> Result<Quant, ParseError> {
    // pair is Rule::quant, inner is one of: range | at_least | exactly | any
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ParseError::new("empty quant".into()))?;

    parse_quant_leaf(inner)
}
