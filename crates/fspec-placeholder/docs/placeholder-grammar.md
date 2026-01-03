## Placeholder / Component Mini-Grammar

EBNF-style notation

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

placeholder_body := oneof
                  | capture_or_ref

capture_or_ref   := IDENT (WS* ":" WS* limiter_spec)?
                 // (if you already had `{year}` meaning “reference year” vs capture,
                 // keep that semantic rule at a higher layer; grammar-wise it’s IDENT.)
```

### 3.1 One-of placeholder

```ebnf
oneof            := choice (WS* "|" WS* choice)+

choice           := IDENT
                  | quoted_string

quoted_string    := DQUOTE qchar* DQUOTE    // same rules as quoted_literal
```

Examples:

```fspec
allow file.{mp4|mkv}
allow file.{"mp*4"|"m/v"|"""in quotes"""}
```

* `IDENT` choices are “simple tokens”
* quoted choices allow any weirdness, with `""` as the quote escape.

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
