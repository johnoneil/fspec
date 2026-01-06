# fspec — Declarative Filesystem Specifications

**fspec** is a tool for describing the filesystem you *intend* to have, and continuously checking the real filesystem against that description.

An `.fspec` file defines which files and directories are **allowed**, which are **ignored**, and—by omission—which paths are unexpected and should be reported.

In short:
**fspec lints the filesystem itself.**

This is especially useful for:

* large media / asset archives
* research datasets
* build artifacts and outputs
* long-lived project folders
* repositories where naming and placement conventions matter
* providing a deterministic filesystem definition to teams and LLMs for auditing.

---

## Core Ideas

* `.fspec` is an **allow-list**: anything not explicitly allowed is reported.
* Rules are **declarative**, readable, and order-dependent.
* Filesystem structure is validated *without* scripts, regex soup, or tribal knowledge.
* The format is intentionally small and constrained.
* Additions to the `.fspec` are easy, but should be discussed by shareholders. This allows filesystem conventions to be subject to a carefully controlled review.
* LLMs will plainly be heavily involved in maintenance of large filesystems. `.fspec` allows a way to explicitly define filesystem structure and file naming conventions such that LLMs (and human teams) can make informed decisions.

---

## Example: Media / Video Archive

`fspec` was intentionally designed to bring order to large hand-named file collections, though this is not its only use case.

```fspec
# movies
allow /movies/{year:int(4)}/{snake_case}_{year}.{mp4|mkv}
allow /movies/unsorted/**/*.{mp4|mkv}

# series
allow /series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}.{mp4|mkv}
allow /series/unsorted/**/*.{mp4|mkv}

# artwork
allow /movies/**/{snake_case}_{year}_thumbnail.png
allow /series/{year:int(4)}/{name:PascalCase}/{name}_thumbnail.png
allow /series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}_thumbnail.png
```

This allows structured organization and transitional “unsorted” areas and is especially good at detecting deviations which creep in via hand naming of files.

---

## Example: Rust Workspace

`fspec` can also be used to enforce file structure and naming in code repositories, though that is not its focus.

```fspec
# ignore build artifacts at root only.
# if different artifacts are polluting crates, it will be reported.
ignore ./target/

# root workspace files
# we allow a Cargo.lock at workspace root and nowhere else.
allow ./Cargo.toml
allow ./Cargo.lock

# allow markdown anywhere
allow *.md

# crates.
# kebab-case crate naming
allow ./crates/{crate:kebab_case}/Cargo.toml
# snake_case naming of source files under certain trees.
allow ./crates/{crate:kebab_case}/{src|tests|examples}/**/{snake_case}.rs
```

Anything not matching these rules will be reported.

---

## The `.fspec` File Format (v1)

This section gives a brief outline of the `.fspec` file format. [See the mini-grammar description for more detail](./crates/fspec-core/docs/fspec-file-grammar.md).

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

---

## Rule Evaluation Semantics

These rules are fundamental and should be considered *part of the spec*:

### 1. Default policy

Anything not ignored or matched by an `allow` rule is reported (warning or error).

### 2. Anchored vs unanchored patterns

* FSPatterns starting with `./` or `/` (equivalent) are **anchored** at the directory containing the `.fspec`.

```fspec
# these three examples are equivalent patterns, anchored at directory containing the .fspec
./src/main.rs
allow ./src/main.rs
allow /src/main.rs
```

* FSPatterns without a leading `./` or `/` are **unanchored** and may match anywhere.

```fspec
# only matches ./bin
./bin
 # only matches ./bin
/bin
 # matches bin at any depth
bin
```

### 3. Ignore rules

* ignore suppresses reporting of violations for a file or directory and its descendants unless later rules re-include them.
* A trailing `/` means *directory-only*

```fspec
# ignore a *directory* named "bin" at fspec root (anchored).
ignore ./bin/
# ignore a file named "bin" at fspec root (anchored)
ignore ./bin
# ignore a directory named "bin" anywhere.
ignore bin/
# ignore a *file* named "bin" anywhere.
ignore bin
```
### 4. Order matters

Rules are evaluated **top to bottom**.

If multiple rules match a path, **the last matching rule wins**. As a consequence, later allow rules may re-include files or directories that were previously matched by ignore rules, including via anchored or unanchored patterns.

Such re-inclusions are permitted by default, and may optionally emit warnings or be disallowed under stricter user settings.

### 5. Directories implied by allowed files

If a file or directory is allowed, the directories required to reach it are considered **structurally allowed**.
You do not need to separately allow every directory component.

### 6. Directory/file name collisions

Files which pass directory naming specs are emitted as warnings by default.

Directories which pass file naming specs are emitted as warnings by default.

---

## `fspec` Pattern Language

`fspec` patterns are path-like strings with literals, globs, and placeholders.

This section gives a general overview of the pattern syntax. [See the mini-grammar description for more detail](./crates/fspec-placeholder/docs/placeholder-grammar.md).

### Globs

| FSPattern | Meaning                                |
| ------- | -------------------------------------- |
| `*`     | any characters within one path segment |
| `**`    | zero or more path segments (recursive) |

Examples:

```fspec
src/*
src/**
```

### Placeholders

Placeholders match exactly **one path segment** and may enforce constraints.

Syntax:

```
{tag}
{tag:limiter}
{tag:limiter(args)}
```

Common built-in limiters. These ensure path or file segments match certain patterns:

* `snake_case`
* `PascalCase`
* `kebab-case`
* `int(n)` (exact width)

Examples:

```fspec
{snake_case}.rs
{year:int(4)}
```

### Repeated placeholders

If the same placeholder tag appears more than once in a single pattern, all occurrences must match the **same value**.

```fspec
# Year must match across the `year` placeholders for the rule to apply.
# This helps enforce the sort of consistency in naming that humans are bad at.
movies/{year:int(4)}/{snake_case}_{year}.mp4
```

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
- [ ] Introduce a command line tool wrapper crate.
- [ ] Make the basic rule engine usable in real world cases.
- [ ] Command line tool output switches and JSON report output.

## Level 2 — Diagnostics and Expansion

- [ ] explain which rule matched
- [ ] ambiguity detection and warnings
- [ ] warn on re-allowed ignored paths
- [ ] warn on ambiguous matches
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

* **The filesystem is part of the system.**
* **Conventions only matter if enforced.**
* **Explicit structure beats folklore.**
* **Small, declarative specs scale better than scripts.**

fspec is about making filesystem structure *auditable, explainable, and intentional*.

---
