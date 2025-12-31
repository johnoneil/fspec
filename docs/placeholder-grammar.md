
## Placeholder / Component Mini-Grammar

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

### 2.2 Quoted literals (new primary escape mechanism)

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

## 4. Limiters (with whitespace tolerance)

```ebnf
limiter_spec     := IDENT (WS* "(" WS* limiter_args? WS* ")" )?

limiter_args     := limiter_arg (WS* "," WS* limiter_arg)*

limiter_arg      := NUMBER
                  | IDENT
                  | quoted_string
```

Whitespace rule inside `{ ... }`:

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

---
