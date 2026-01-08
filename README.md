# fspec — Declarative Filesystem Specifications

![CI](https://github.com/johnoneil/fspec/actions/workflows/ci.yml/badge.svg)

fspec is an experimental, declarative filesystem specification and validation tool, suitable for CI pipelines, auditing, and long-lived project structure enforcement. It provides a way to formally describe expected filesystem structure (directories, files, naming conventions, style, and invariants) in a way that is:

* human-readable
* machine-verifiable
* suitable for CI, tooling, and long-lived projects

The core idea is that filesystem layout itself can be treated as a contract, rather than an informal convention.

The imagined process is that an `.fspec` file can be introduced into filesystems/repositories, which defines which files and directories are **allowed**, which are **ignored**, and—by omission—which paths are **unexpected** or **non-compliant** and should be reported.

In effect, `fspec` is a tool and specification for linting filesystems.

The goal is to help manage:

* large media / asset archives
* research datasets
* build artifacts and outputs
* long-lived project folders
* repositories where naming and placement conventions matter
* providing a deterministic filesystem definition to teams and LLMs for auditing.

For further information [see the design goals document here.](./docs/fspec_design_goals.md)

I did work on this tool during winter break from grad school 2025/26 as an experiment to become more familiar with modern LLM/AI code assist workflows. This project will probably remain dormant while I complete schoolwork.

---

## Core Ideas

* `.fspec` is an **allow-list**: anything not explicitly allowed is reported.
* Rules are **declarative**, readable, and order-dependent.
* Filesystem structure is validated *without* scripts, regex soup, or tribal knowledge.
* The format is intentionally small and constrained.
* Additions to the `.fspec` are easy, but should be discussed by shareholders. This allows filesystem conventions to be subject to a carefully controlled review.
* LLMs will plainly be heavily involved in maintenance of large filesystems. `.fspec` allows a way to explicitly define filesystem structure and file naming conventions such that LLMs (and human teams) can make informed decisions.

---

## Use cases

- CI validation of project structure
- Auditing large data or media archives
- Enforcing naming and placement conventions
- Providing explicit filesystem contracts to teams and tools

---

## Example: Media / Video Archive

`fspec` was intentionally designed to bring order to large hand-named file collections, though this is not its only use case.

```fspec
# Allow a tree of video files that conform to the pattern: 
./movies/{year:int(4)}/{snake_case}_{year}.{mp4|mkv}
# the "allow" keyword may also be used explicitly.
allow ./movies/unsorted/**/*.{mp4|mkv}
# don't try to verify/audit anything old.
ignore ./movies/old

# More complex paths where path elements must match filename elements can be allowlisted.
./series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}.{mp4|mkv}
./series/unsorted/**/*.{mp4|mkv}

# Further verification that associated artwork also conform to expectations.
./movies/**/{snake_case}_{year}_thumbnail.png
./series/{year:int(4)}/{name:PascalCase}/{name}_thumbnail.png
./series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}_thumbnail.png
```

The idea is to represent your desired filesystem as a series of readable patterns, then the tool will report how well the actual filesystem conforms to the spec.

Although explicit paths (as would be reported by `find`) can be included in the `.fspec`, it's hoped that more abstract patterns would arise. More a series of rules than an ad-hoc file listing.

---

## Example: Rust Workspace

`fspec` can also be used to enforce file structure and naming in code repositories, though that is not its focus.

```fspec
# build artifacts at root only.
ignore ./target/

# we currently have to explicitly ignore hidden files
ignore ./.*

# root workspace files
allow ./Cargo.toml
allow ./Cargo.lock

# docs
allow ./README.md
allow docs/**/*.md

# crates.
allow ./crates/{crate:kebab_case}/Cargo.toml
allow ./crates/{crate:kebab_case}/{src|tests|examples}/**/{snake_case}.rs
allow ./crates/{crate:kebab_case}/README.md
```

Anything not matching these rules will be reported.

---

## The `.fspec` File Format (v1)

This section gives a brief outline of the `.fspec` file format. [See the mini-grammar description for more detail](./crates/fspec-core/README.md).

An `.fspec` file is a **line-based specification**.

### Comments

```fspec
# this is a comment
```

### Rule types

Each non-comment line is a rule:

| Prefix   | Meaning  |
| -------- | -------- |
| `` (none) | Defaults to `allow`, below |
| `allow`  | Allows a given directory or file and its ancestors to exist without error.  |
| `ignore` | Ignores a file or directory, removing it and all its possible descendants from checks.  |

Examples:

```fspec
# allow the file named `Cargo.toml` file at fspec root.
./Cargo.toml
# allow a rust file anywhere as long as its filename is snake_case
allow {snake_case}.rs
# Ignore the .git directory at fspec root.
ignore /.git/
# ignore any directory named `target` anywhere and all its descendants.
ignore target/

```

For more detail about the general `.fspec` file format [see the design documents here.](./crates/fspec-core/README.md)

For more detail about the general `.fspec` placeholder format, [see the design documents here.](./crates/fspec-placeholder/README.md)

---

## Conformance Levels / Roadmap

fspec is intentionally staged. Not all features need to exist at once.

### Level 0 — Matching

- [x] parse `.fspec` to rules AST.
- [x] walk filesystem.
- [x] report violations.
- [x] Unaccounted file
- [x] Unaccounted dir
- [x] Anchored allow file rule
- [x] Anchored allow dir rule
- [x] Anchored ignore file rule
- [x] Anchored ignore dir rule
- [x] Trailing `/` directory-only allow
- [x] Unanchored allow file rule
- [x] Unanchored allow dir rule
- [x] Unanchored ignore file rule
- [x] Unanchored ignore dir rule
- [x] `*` single-segment glob
- [x] `**` recursive glob
- [x] `**` matches zero segments
- [x] anchored re-allow inside ignored subtree (plus warning)
- [x] Last match wins
- [x] optionally omit `allow` command.
- [x] also support use of `./` to mean anchored allow/ignore.


## Level 1 — Extraction

- [x] placeholder capture and parsing.
- [x] Initial group of limiters parsed and provided to clients as part of an AST.
- [ ] ~~Improve allowed comments in the grammar and impl. Allow `#` anywhere.~~
- [x] repeated placeholder equality (`year` = `year`)
- [x] Improve union limiter to allow names like `ext` in `{ext:mp4|mkv}`
- [x] Implement `./file` as either a file or directory to match `fined` and `.gitignore` behavior. Ensure behavior is switchable (between "file only" and "file or directory" to allow later strictness switches.)
- [x] Precompile matching rules after rule parsing to improve matching performance.
- [x] Improve the parsing grammar design to require/allow the `:` sigil for limiters.
- [x] Introduce a command line tool wrapper crate.
- [x] Make the basic rule engine usable in real world cases.
- [x] Command line tool output switches and JSON report output.

## Level 2 — Diagnostics and Expansion

- [ ] Named path aliases (reduce repetition; improve readability for hierarchical specs) [see proposal.](./docs/proposals/named-path-aliases.md)
- [ ] Improve command line switches
- [ ] Improve logging and verbosity
- [ ] explain which rule matched
- [ ] ambiguity detection and warnings
- [ ] warn on re-allowed ignored paths
- [ ] warn on ambiguous matches
- [ ] Ignore hidden files by settings/switches.
- [ ] Determine symlink behavior and allow control via settings/switches.
- [ ] expand limiter list to a more ergonomic set including GUID, Date, URL, etc.

## Level 3 — Suggestions

- [ ] generate candidate allowed paths
- [ ] edit-distance / structural matching
- [ ] “did you mean …” rename proposals

## Level 4 — Extensions (future)

- [ ] hierarchical `.fspec` inheritance
- [ ] dependency / freshness rules
- [ ] documentation and tooling generation
---

## Philosophy

* Make filesystem conventions carried around as tribal knowledge explicit.
* Allow agreed upon filesystem conventions to be easily enforced.
* Small, declarative specs scale better than endless custom bash scripts.

fspec is about making filesystem structure *auditable, explainable, and intentional*.

[see philosophy doc here.](./docs/fspec_philosophy.md)

## License

Licensed under the Apache License, Version 2.0. See `LICENSE` and `NOTICE`.


---
