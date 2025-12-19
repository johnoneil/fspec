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

---

## Core Ideas

* `.fspec` is an **allow-list**: anything not explicitly allowed is reported.
* Rules are **declarative**, readable, and order-dependent.
* Filesystem structure is validated *without* scripts, regex soup, or tribal knowledge.
* The format is intentionally small and constrained.

---

## Example: Media / Video Archive

`fspec` was intentionally designed to bring order to large hand-named file collections, though this is not its only use case.

```fspec
# movies
+ /movies/{year:int(4)}/{snake_case}_{year}.{ext:mp4|mkv}
+ /movies/unsorted/**/*.{ext:mp4|mkv}

# series
+ /series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}.{ext:mp4|mkv}
+ /series/unsorted/**/*.{ext:mp4|mkv}

# artwork
+ /movies/**/{snake_case}_{year}_thumbnail.png
+ /series/{year:int(4)}/{name:PascalCase}/{name}_thumbnail.png
+ /series/{year:int(4)}/{name:PascalCase}/season_{season:int(2+)}/{name}.s{season}e{episode:int(2+)}_thumbnail.png
```

This allows structured organization *and* transitional “unsorted” areas and is especially good at detecting deviations which creep in via hand naming of files.

---

## Example: Rust Workspace

`fspec` can also be used to enforce file structure and naming in code repositories, though tht is not its focus.

```fspec
# ignore build artifacts everywhere
- target/

# root workspace files
+ /Cargo.toml
+ /Cargo.lock

# Rust source must be snake_case
{snake_case}.rs

# crates layout
+ /crates/{crate:kebab-case}/Cargo.toml
+ /crates/{crate:kebab-case}/src/**/
+ /crates/{crate:kebab-case}/src/**/{snake_case}.rs
```

Anything not matching these rules will be reported.

---

## The `.fspec` File Format (v1)

The `.fspec` file format was intentionally pattered after `.gitignore` though with much more functionality. But while `.gitignore` is merely an ignore list, `.fspec` is an allow/ignore list.

An `.fspec` file is a **line-based specification**.

### Comments

```fspec
# this is a comment
```

### Rule types

Each non-comment line is a rule:

| Prefix   | Meaning  |
| -------- | -------- |
| `+`      | `allow`  |
| `allow`  | `allow`  |
| `-`      | `ignore` |
| `ignore` | `ignore` |

Examples:

```fspec
+ {snake_case}.rs
+ /Cargo.toml
- target/
ignore .git/
```

---

## Rule Evaluation Semantics

These rules are fundamental and should be considered *part of the spec*:

### 1. Default policy

Anything not matched by an `allow` rule is reported (warning or error).

### 2. Order matters

Rules are evaluated **top to bottom**.
If multiple rules match a path, **the last matching rule wins**.

### 3. Rooted vs unanchored patterns

* Patterns starting with `/` are **rooted** at the directory containing the `.fspec`.
* Patterns without `/` are **unanchored** and may match anywhere.

```fspec
/bin        # only matches ./bin
bin         # matches bin at any depth
```

### 4. Ignore rules

* `ignore` applies to **files or directories**
* A trailing `/` means *directory-only*

```fspec
- bin        # ignore file or directory named "bin"
- bin/       # ignore directory "bin" and everything under it
```

### 5. Ignored-subtree barrier

Ignored directories form a barrier:

* **Unanchored `allow` rules do NOT apply inside ignored directories**
* **Rooted `allow` rules MAY re-allow specific paths inside ignored directories**

```fspec
- /bin/
{snake_case}.rs      # does NOT re-allow /bin/foo.rs
+ /bin/tool.rs       # DOES re-allow this file (with a warning)
```

Re-allowing files inside ignored directories is permitted but should emit a warning.

### 6. Directories implied by allowed files

If a file or directory is allowed, the directories required to reach it are considered **structurally allowed**.
You do not need to separately allow every directory component.

---

## Pattern Language

Patterns are path-like strings with literals, globs, and placeholders.

### Globs

| Pattern | Meaning                                |
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
* `int(n+)` (at least n digits)

Examples:

```fspec
{snake_case}.rs
{year:int(4)}
season_{season:int(2+)}
```

### Repeated placeholders

If the same placeholder tag appears more than once in a pattern, all occurrences must match the **same value**.

```fspec
movies/{year:int(4)}/{snake_case}_{year}.mp4
```

---

## Conformance Levels / Roadmap

fspec is intentionally staged. Not all features need to exist at once.

### Level 0 — Matching

* parse `.fspec`
* walk filesystem
* classify paths as allowed / ignored / unaccounted
* report violations

### Level 1 — Extraction

* placeholder capture
* repeated placeholder equality
* ambiguity detection and warnings

### Level 2 — Diagnostics

* explain which rule matched
* warn on re-allowed ignored paths
* warn on ambiguous matches

### Level 3 — Suggestions

* generate candidate allowed paths
* edit-distance / structural matching
* “did you mean …” rename proposals

### Level 4 — Extensions (future)

* hierarchical `.fspec` inheritance
* dependency / freshness rules
* documentation and tooling generation

---

## Philosophy

* **The filesystem is part of the system.**
* **Conventions only matter if enforced.**
* **Explicit structure beats folklore.**
* **Small, declarative specs scale better than scripts.**

fspec is about making filesystem structure *auditable, explainable, and intentional*.

---
