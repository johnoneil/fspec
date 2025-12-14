
# `fspec-pattern` crate


## up next

* BIG QUESTION: allow a kind of regex as a limiter? ultimately all limiters will boil down to a regex, but do we allow it directly? getting one inside the grammar could be tough without putting it in quotes or something.

# next

* implement globstar ** as only existing when it's the only path segment (/** or /**/), otherwise it's part of a literal. What about single asterisk? any meaning to filesystems?

`/**` -> globstar.

`/**/` -> globstar

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