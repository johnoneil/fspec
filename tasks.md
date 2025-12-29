
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
- [ ] Ignored-subtree barrier (unanchored allow doesn’t pierce)
- [ ] anchored re-allow inside ignored subtree (plus warning)
- [ ] Last match wins
- [ ] optionally omit `allow` command.
- [ ] also support use of `./` to mean anchored allow/ignore.


## Level 1 — Extraction

- [ ] placeholder capture
- [ ] repeated placeholder equality
- [ ] ambiguity detection and warnings
- [ ] permit alternate syntaxes
- [ ] Union limiter (`ext:mp4|mkv`) or `int(2+)`

## Level 2 — Diagnostics

- [ ] explain which rule matched
- [ ] warn on re-allowed ignored paths
- [ ] warn on ambiguous matches

## Level 3 — Suggestions

- [ ] generate candidate allowed paths
- [ ] edit-distance / structural matching
- [ ] “did you mean …” rename proposals

## Level 4 — Extensions (future)

- [ ] hierarchical `.fspec` inheritance
- [ ] dependency / freshness rules
- [ ] documentation and tooling generation
- [ ] allow "permissive" or less strict mode as default where lack of trailing slash `xxx/` can be interpreted as either a file or directory like gitignore et al.

