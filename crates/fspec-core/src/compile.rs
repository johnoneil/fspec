// Compilation module: converts ComponentAst to CompiledComponent with pre-compiled regexes

use crate::error::Error;
use crate::spec::CompiledComponent;
use fspec_placeholder::ast::{Choice, ComponentAst, Part, PlaceholderNode};
use regex::Regex;

/// Compile a ComponentAst into a CompiledComponent with a pre-compiled regex.
///
/// This extracts the regex pattern from the AST and compiles it once,
/// storing both the compiled regex and the placeholder indices for efficient matching.
pub fn compile_component(ast: &ComponentAst) -> Result<CompiledComponent, Error> {
    let mut pat = String::from("^");
    let mut placeholder_indices: Vec<(String, usize)> = Vec::new(); // name -> capture group index
    let mut capture_group = 1;

    for part in &ast.parts {
        match part {
            Part::Literal(lit) => pat.push_str(&regex::escape(&lit.value)),
            Part::Star(_) => pat.push_str(".*"),
            Part::Placeholder(ph) => match &ph.node {
                PlaceholderNode::OneOf(oneof) => {
                    // Named one-of: extract the matched choice
                    if let Some(named) = &oneof.name {
                        let mut alts: Vec<String> = Vec::new();
                        for choice in &oneof.choices {
                            let s = match choice {
                                Choice::Ident { value, .. } => value,
                                Choice::Str { value, .. } => value,
                            };
                            alts.push(regex::escape(s));
                        }
                        pat.push('('); // capture group for named one-of
                        pat.push_str(&alts.join("|"));
                        pat.push(')');
                        placeholder_indices.push((named.name.clone(), capture_group));
                        capture_group += 1;
                    } else {
                        // Unnamed one-of: no capture
                        let mut alts: Vec<String> = Vec::new();
                        for choice in &oneof.choices {
                            let s = match choice {
                                Choice::Ident { value, .. } => value,
                                Choice::Str { value, .. } => value,
                            };
                            alts.push(regex::escape(s));
                        }
                        pat.push_str("(?:");
                        pat.push_str(&alts.join("|"));
                        pat.push(')');
                    }
                }
                PlaceholderNode::Capture(cap) => {
                    // Capture with name: extract the matched value
                    let mut cap_re = String::from(".+");

                    if let Some(lim) = &cap.limiter {
                        cap_re = lim.to_regex_fragment();
                    }

                    pat.push('('); // capture group for named capture
                    pat.push_str(&cap_re);
                    pat.push(')');
                    placeholder_indices.push((cap.name.clone(), capture_group));
                    capture_group += 1;
                }
            },
        }
    }

    pat.push('$');

    // Compile the regex pattern
    let regex = Regex::new(&pat).map_err(|e| Error::Semantic {
        msg: format!("invalid regex pattern for component: {}", e),
    })?;

    Ok(CompiledComponent {
        ast: ast.clone(),
        regex,
        placeholder_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fspec_placeholder::parse_component;

    #[test]
    fn test_oneof_case_sensitivity() {
        // Test that one-of choices match both cases when explicitly provided
        let ast = parse_component("{snake|SNAKE}").unwrap();
        let compiled = compile_component(&ast).unwrap();

        // The regex should be ^(?:snake|SNAKE)$
        assert!(compiled.regex.is_match("snake"));
        assert!(compiled.regex.is_match("SNAKE"));
        assert!(!compiled.regex.is_match("Snake")); // Mixed case shouldn't match
    }

    #[test]
    fn test_oneof_with_literal_prefix() {
        // Test pattern like "test.{snake|SNAKE}"
        let ast = parse_component("test.{snake|SNAKE}").unwrap();
        let compiled = compile_component(&ast).unwrap();

        assert!(compiled.regex.is_match("test.snake"));
        assert!(compiled.regex.is_match("test.SNAKE"));
        assert!(!compiled.regex.is_match("test.Snake"));
    }

    #[test]
    fn test_full_pattern_like_golden_test() {
        // Test the actual pattern from golden_limiters.rs
        // Pattern: {name:snake_case}_{name}_{year:int(4)}.{snake|SNAKE}
        let ast = parse_component("{name:snake_case}_{name}_{year:int(4)}.{snake|SNAKE}").unwrap();
        let compiled = compile_component(&ast).unwrap();

        let test1 = "snaked_name_snaked_name_1999.snake";
        let test2 = "snaked_name_snaked_name_1999.SNAKE";

        assert!(compiled.regex.is_match(test1), "Should match .snake");
        assert!(compiled.regex.is_match(test2), "Should match .SNAKE");
    }
}
