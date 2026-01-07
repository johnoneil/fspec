## `.fspec` Mini-Grammar (fspec-core)

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

### Scope

This grammar describes **how an `.fspec` file is split into rules** (`allow` / `ignore`) and how each line is interpreted **before** the pattern string is handed off to the pattern parser (`pattern::parse_pattern_str`).

It does **not** describe placeholder/component parsing (that’s `fspec-placeholder`), nor the internal pattern grammar (that lives in the pattern module).

---

# EBNF-style mini grammar

## 1. File structure

An `.fspec` file is parsed as a sequence of lines.

* Lines are processed in order.
* Each parsed rule records its **1-based line number**.
* Windows CRLF is supported (`\r\n`): the trailing `\r` is stripped.

```ebnf
fspec_file  := { line } ;

line        := ws? ( comment_line | rule_line | empty_line ) line_end ;

empty_line  := "" ;
comment_line:= "#" { any_char_except_line_end } ;
```

### Comment rule

Comments are recognized **only** when the first non-whitespace character on the line is `#`.

That is:

* `   # this is a comment` ✅ comment
* `allow foo # not a comment` ❌ (the `#` is part of the pattern, if allowed by pattern parser)
* `foo#bar` ❌ (not a comment; `#` is in the pattern)

---

## 2. Rule lines (keywords + pattern remainder)

A rule line is either:

1. `allow <pattern>`
2. `ignore <pattern>`
3. `<pattern>` (keyword omitted → defaults to `allow`)

Leading whitespace is permitted and ignored for control-flow parsing.

Trailing whitespace after the pattern is ignored (trimmed).

```ebnf
rule_line   := ws? ( keyword ws1 pattern_text | pattern_text ) ;

keyword     := "allow" | "ignore" ;
ws          := { " " | "\t" } ;
ws1         := ( " " | "\t" ) { " " | "\t" } ;
```

### Keyword behavior

* If the line begins with `allow` or `ignore` (after optional leading whitespace), that keyword sets `RuleKind`.
* Otherwise, the line is treated as a pattern-only line and **defaults to `allow`** (for `find` output compatibility).

### Pattern remainder (“pattern_text”)

`pattern_text` is the **rest of the line after the keyword** (or the entire trimmed line if no keyword).

It is passed verbatim (modulo trimming) to the pattern parser.

More precisely:

* If keyword is present:

  * consume keyword
  * consume any leading whitespace after keyword
  * require **at least one non-whitespace character** remaining, otherwise error: “expected a pattern after keyword”
  * trim trailing whitespace
  * send to `pattern::parse_pattern_str(pattern_text, line_no)`

* If keyword is absent:

  * take the whole line with leading whitespace removed (`trim_start`)
  * trim trailing whitespace
  * send to `pattern::parse_pattern_str(...)`

```ebnf
pattern_text := { any_char_except_line_end } ;
```

> Note: Because inline `#` is *not* comment syntax, `pattern_text` may include `#` and anything else up to end-of-line; validity is determined by the pattern parser.

---

## 3. Produced AST (high level)

Each `rule_line` produces:

```text
Rule {
  line: <1-based line number>,
  kind: Allow | Ignore,
  pattern: <result of pattern::parse_pattern_str>,
}
```

---

## 4. Examples

### Comments and blank lines

```fspec
# full-line comment
   # also comment

allow /src/main.rs
```

### Keyword optional (defaults to `allow`)

```fspec
/src/main.rs
allow /src/lib.rs
ignore /target/
/src/utils.rs
```

### Find-output style input (all default to allow)

```fspec
./src/main.rs
./src/lib.rs
./target/
```

---

## 5. Non-goals / delegated grammar

* The syntax of `<pattern>` is **not defined here** (anchoring, `/` vs `./`, `*`, `**`, dir trailing `/`, etc.). That belongs to the pattern module.
* Placeholder / component syntax inside a path component is **not defined here**. That belongs to `fspec-placeholder`.

