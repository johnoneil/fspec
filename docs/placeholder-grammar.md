## Placeholder / Component Mini-Grammar

This grammar applies to a **single path component** (between `/` separators). A component is parsed into a sequence of **parts**.

---

### 1. Component structure

```
component        := part*

part             := literal_run
                  | star
                  | placeholder

star             := "*"
```

---

### 2. Literal text (outside placeholders)

Outside placeholders, `{`, `}`, and `*` are special.

To write literal braces, use doubling.

```
literal_run      := literal_char+

literal_char     := any character except '{' '}' '*' '/'
                  | "{{"          // literal '{'
                  | "}}"          // literal '}'
```

*(A literal `*` outside placeholders must be written using a string literal placeholder — see section 4.)*

---

### 3. Placeholder syntax

```
placeholder      := "{" placeholder_body "}"
```

---

### 4. Placeholder body forms

```
placeholder_body :=
      named_capture
    | limiter_only
    | oneof_shorthand
    | string_literal
```

#### 4.1 Named capture

```
named_capture    := IDENT [ ":" limiter_expr ]
```

Examples:

* `{year}`
* `{year:int(4)}`
* `{show_name}`

Interpretation:

* If IDENT is *not* a reserved limiter name → named placeholder
* Default limiter is `Any`

---

#### 4.2 Limiter-only (anonymous)

```
limiter_only     := ":" limiter_expr
                  | limiter_expr
```

Examples:

* `{int(4)}`
* `{PascalCase}`
* `{:snake_case}`

Only valid if `limiter_expr` uses a **reserved limiter identifier**.

---

#### 4.3 One-of shorthand (restricted)

```
oneof_shorthand  := oneof_token ( "|" oneof_token )+

oneof_token      := [A-Za-z0-9_.-]+
```

Example:

* `{mp4|mkv|avi}`

For complex strings (spaces, pipes, braces), use `oneof("...")` instead.

---

#### 4.4 Literal string placeholder (escape hatch)

```
string_literal   := STRING

STRING           := '"' STRING_CHAR* '"'
STRING_CHAR      := '""'              // doubled quote → literal "
                  | any char except '"'
```

Example:

* `{"***this_is_a_file***"}`
* `{"*.rs"}`
* `{"{weird|name}*"}`

Interpretation:

* Produces a **literal component part**
* No placeholders, globs, or limiters are processed inside
* No capture occurs

---

### 5. Limiter expressions

```
limiter_expr     := limiter_ident [ "(" limiter_args? ")" ]

limiter_ident    := IDENT           // must be reserved

limiter_args     := limiter_arg ( "," limiter_arg )*

limiter_arg      := INT
                  | range
                  | plus_int
                  | IDENT
                  | STRING

range            := INT ".." INT     // e.g. 1..4
plus_int         := INT "+"          // e.g. 3+
```

---

### 6. Identifiers and tokens

```
IDENT            := [A-Za-z_][A-Za-z0-9_]*
INT              := [0-9]+
```

---

### 7. Interpretation rules (normative)

1. `{IDENT}`:

   * If IDENT is a reserved limiter → anonymous limiter-only placeholder
   * Otherwise → named capture with limiter `Any`

2. `{IDENT:lim(...)}` → named capture with explicit limiter

3. `{lim(...)}` or `{:lim(...)}` → anonymous limiter-only placeholder

4. `{a|b|c}` → anonymous `OneOf` limiter (restricted tokens only)

5. `{"..."}` → literal text (escape hatch), no glob or placeholder semantics

6. `*` always matches any string **within a single component**

7. Placeholders and `*` never match `/`

---

### 8. Design intent (why this works)

* Most common cases (`*.rs`, `{year:int(4)}`, `{mp4|mkv}`) stay concise
* Escaping is rare and localized
* Complex or “ugly” literals have a single, obvious escape hatch
* Grammar stays deterministic and easy to hand-parse
* Future features (references, regex limiters, raw strings) can be added without breaking syntax

---

