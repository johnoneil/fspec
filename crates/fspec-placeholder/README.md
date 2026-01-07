# `fspec` Placeholder Pattern Language

`fspec` patterns are path-like strings with literals, globs, and placeholders.

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

# EBNF style mini grammar

### 0. Scope

* This grammar parses **one path component** (text between `/` separators).
* `/` is *not matchable* by `*` or placeholders.
* (Practical note: allowing a literal `/` inside a “component” doesn’t make sense without a separate escaping story for separators — so the old `{".../..."} ` idea was a dead-end anyway.)

---

## 1. Component structure

```ebnf
component        := part*

part             := literal_run
                  | quoted_literal
                  | star
                  | placeholder

star             := "*"
```

---

## 2. Literals

### 2.1 Unquoted literal runs

```ebnf
literal_run      := literal_char+

literal_char     := any char except: '*', '{', '}', '"'
```

* Unquoted `*` is always a glob star.
* `{` starts a placeholder.
* `"` starts a quoted literal.

(Everything else — including spaces — is literal outside `{}`.)

### 2.2 Quoted literals (primary escape mechanism)

```ebnf
quoted_literal   := DQUOTE qchar* DQUOTE

qchar            := '""'               // doubled quote → literal "
                  | any char except '"'
```

* Inside quotes, **everything is literal** (including `*`, `{`, `}`, `|`, and spaces).
* To include a literal `"` inside a quoted literal, write `""`.

Example:

```fspec
ignore "***filename_literal***".o
```

This component is: quoted literal `***filename_literal***` + literal run `.o`.

---

## 3. Placeholders

```ebnf
placeholder      := "{" WS* placeholder_body WS* "}"

placeholder_body := named_oneof
                  | oneof
                  | anonymous_oneof
                  | capture_or_ref

anonymous_oneof  := ":" WS* oneof

capture_or_ref   := IDENT (WS* ":" WS* limiter_spec)?
                 // (if you already had `{year}` meaning "reference year" vs capture,
                 // keep that semantic rule at a higher layer; grammar-wise it's IDENT.)
```

### 3.1 One-of placeholder

```ebnf
named_oneof      := IDENT WS* ":" WS* oneof

oneof            := choice (WS* "|" WS* choice)+

choice           := IDENT
                  | quoted_string

quoted_string    := DQUOTE qchar* DQUOTE    // same rules as quoted_literal
```

Notes:

* The leading `:` form (`{:<oneof>}`) is an optional alias for unnamed one-of placeholders. It exists for style consistency with other anonymous limiter placeholders, but is not required because `|` already makes one-of self-identifying.

Examples:

```fspec
# Unnamed one-of
allow file.{mp4|mkv}
# Unnamed one-of (sigiled alias; equivalent to the form above)
allow file.{:mp4|mkv}
allow file.{"mp*4"|"m/v"|"""in quotes"""}
# Named one-of
allow file.{ext:mp4|mkv}
allow file.{type:"video"|"audio"|"text"}
```

* `IDENT` choices are "simple tokens"
* quoted choices allow any weirdness, with `""` as the quote escape.
* Named one-of placeholders allow capturing which choice matched (e.g., `{ext:mp4|mkv}` captures `ext` as `mp4` or `mkv`).
* Note: `{name:single}` (single choice without `|`) is not a one-of; it is parsed as a capture with limiter, and will fail validation if `single` is not a valid limiter name.

---

## 4. Limiters (Level-1 conformance set)

Level-1 goal: parse limiters in a way that is **strict for known names**
but **future-proof** for later expansion.

### 4.1 Syntax

```ebnf
limiter_spec        := limiter_name (WS* "(" WS* limiter_args? WS* ")" )?

// NOTE: At Level 1, `limiter_name` is restricted to the list below.
// Future conformance levels may expand this set.
limiter_name        := "snake_case"
                     | "kebab_case"
                     | "pascal_case"
                     | "upper_case"
                     | "lower_case"
                     | "int"
                     | "re"
                     | "letters"
                     | "numbers"
                     | "alnum"

limiter_args        := limiter_arg (WS* "," WS* limiter_arg)*

limiter_arg         := NUMBER
                     | IDENT
                     | quoted_string
```

### 4.2 Level-1 semantic constraints

After parsing `limiter_spec`, Level-1 validation applies these rules:

* No-arg limiters (must have **zero** args, and may omit parens):
  * `snake_case`, `kebab_case`, `pascal_case`, `upper_case`, `lower_case`
  * `letters`, `numbers`, `alnum`

  Examples:
  ```fspec
  {name:snake_case}
  {name:snake_case()}
  {tok:letters}
  ```

* `int(n)` (must have **exactly one** numeric arg):
  * `n >= 1`
  * matches exactly `n` ASCII digits `[0-9]`

  Example:
  ```fspec
  {year:int(4)}
  ```

* `re("...")` (must have **exactly one** string arg):
  * arg must be a quoted string (so escapes follow `""` rules)
  * regex dialect is implementation-defined (recommend: Rust `regex` crate)

  Example:
  ```fspec
  {slug:re("[a-z0-9_-]+")}
  ```

### 4.3 Whitespace tolerance inside `{ ... }`

* `WS*` is allowed:
  * before/after the placeholder body
  * around `:`
  * around `|`
  * around `(` `)` and `,`
* **Whitespace inside quoted strings is literal.**
* **Whitespace outside `{}` remains exact** (because it’s just part of the path component).

So these are equivalent:

```fspec
{year:int(4)}
{ year : int( 4 ) }
{year :int(4)}
```

---

## 5. Matching constraints (unchanged)

* `*` matches any sequence of chars **except** `/`.
* Placeholders never match `/`.
* Quoted literals match exactly their literal content.
