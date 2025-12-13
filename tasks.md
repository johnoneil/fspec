1. look at handling of '{' and '}' and introduce escaped versions like '{{' and '}}'.
2. improve error messages to be more meaningful and update unit tests to check them.
3. should we allow patterns with \ on windows? how to handle that?
4. what about filesystem root. I think it's well handled.
5. what about single asterisk? any meaning to filesystems?
6. What about single dots (current directory?)
7. implement globstar ** as only existing when it's the only path segment (/** or /**/), otherwise it's part of a literal.
8. automatically split off the extension into its own object as it's a kind of "special" thing?
9. allow this case? { id:int( 3 + ) }--> yes we will allow it for ergonomics.