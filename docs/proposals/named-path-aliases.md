# Named Path Aliases Proposal (for .fspec 2.0 definition)

The largest weakness in the `.fspec` file format is the need for repeated path elements. For example:

```
allow ./crates/{crate:kebab_case}/Cargo.toml
allow ./crates/{crate:kebab_case}/{src|tests|examples}/**/{snake_case}.rs
allow ./crates/{crate:kebab_case}/README.md
```

This repeats `./crates/{crate:kebab_case}` which is tedious and error prone.

This proposal suggests we allow path aliases to be constructed to reduce repetition like:

```
allow:crate ./crates/{crate:kebab_case}

allow @crate/Cargo.toml
allow @crate/{src|tests|examples}/**/{snake_case}.rs
allow @crate/README.md

```

This would be done in a way that doesn't cause the `.fspec` to explode in complexity into a programming language.