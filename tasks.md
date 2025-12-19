
# `fspec-pattern` crate

Rethink the parser to be like gitignore

* allow negation at start
* go top to bottom in file (in tool)
* ignore trailing spaces (maybe in tool, remove them before parsing.

gitignore pest:// Whitespace and comments
WHITESPACE = _{ " " | "\t" }
comment    = _{ "#" ~ (!NEWLINE ~ ANY)* }

// Patterns
line     = { (negation? ~ pattern) }
negation = { "!" }
pattern  = { (escape | char)+ ~ folder? }
folder   = { "/" }
escape   = { "\\" ~ ANY }
char     = { !NEWLINE ~ !WHITESPACE ~ !folder ~ ANY }

// File structure
file = { SOI ~ (comment? ~ (line | NEWLINE))* ~ EOI }


~~Segment-special tokens and validation
enforce ** only as a whole segment
add . and .. as whole segments
decide what to do with lone * (I’d do whole-segment-only first)
escaping for braces ({{ / }})
This prevents “I literally need a brace in a filename” edge cases from becoming impossible.~~

Raw regex limiter (re("..."))
Keep it simple: one string argument; compile with Rust regex (or whatever you plan) later.

Depth-controlled multi-segment matching
Revisit after you’ve actually implemented matching semantics and have a reason to control depth.


## up next

* BIG QUESTION: I think placeholders should have a name (tag) or a limiter, or both. we should be able to drop the name if it won't be referred to elsewhere. but no limiter and no tag is meaningless.
* additionally, a globstar as a kind of limiter (or single star glob) should be allowed.
* and a pure regex as a limiter (expressed in some way, in quotes?) should be acceptable.
* and multiple limiters on a placeholder should be... supported??? I don't know. Interactions between limiters are tough to guess.

# next

* implement globstar ** as only existing when it's the only path segment (/** or /**/), otherwise it's part of a literal. What about single asterisk? any meaning to filesystems?

`/**` -> globstar.

`/**/` -> globstar

allow this? `/**(3)/` -> glob exactly 3 directories deep?
alow this? `/**(3+)/` -> glob 3 or more directories deep?
allow this? `/**(3-5)/` -> glob between 3 and 5 directories deep? that is 1, 2 and 6+ don't apply.
Is a globstar a limiter without a tag? allow `/{glob:**(3)}` or `/{**(3)}` ?
allow `/{int(3)}/` ? (placeholders with just limiters, not tags?)

so... allow unnamed placeholders?
allow globs as placeholders?
allow limiters outside of placeholders (implied placeholder?) `/int(3)` or `/camelCase/` ?
allow spaces around placeholders and not have them be literals? `/ {placeholder_surrounded_by_spaces} /` ?


`/** /` -> error badly? allow this for readability like `/ ** /` ? or `dir/ ** /file.txt`

`**hello**` -> allow as literal?

`/*/` -> just what is this? does it have meaning? is it just a literal? all files? as opposed to all nested directories?

* What about single dots (current directory?) double dots?

`./` -> create a "current dir" object for dot? allow `dir/ . /file.txt` (for readability)?
`/./` -> create a current dir object for dot?
`/../` -> create a "up dir" object? allow `dir/ .. /file.txt` (for readability) ?
`/.. / ` -> (space after double dot. fail badly?) (see above)
`/..hello..` -> allow as literal?

* look at handling of '{' and '}' and introduce escaped versions like '{{' and '}}'.

`{{this is a literal not a placeholder}}` -> implement this
`{{ this is an error}` -> unopened placeholder
`{{this could be a big}/{source of error}}` -> do not allow this due to `/` in literal?

## later

* do hidden files like `.fspec.toml` conform to camel case? or other? is there a general expectation for `.toml` files?
* should we allow patterns with \ on windows? how to handle that?
* what about filesystem root. I think it's well handled.
* automatically split off the extension into its own object as it's a kind of "special" thing?
* improve error messages to be more meaningful and update unit tests to check them.