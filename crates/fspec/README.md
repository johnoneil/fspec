# fspec CLI tool design

## Goals

Provide a small command-line wrapper crate (`crates/fspec`) around `fspec-core`:

- discover and load an `.fspec` file
- traverse a target directory tree
- run `fspec-core` rule evaluation
- emit a report in human-readable or JSON form
- configure a small set of matching behaviors (via `MatchSettings`)
- support “show more/less” output knobs (verbosity and report fields)

Non-goals (for now):

- editing `.fspec` files
- “fix”/auto-remediation
- rich diagnostics (“which rule matched”, etc.) — planned for later conformance levels


## Command shape

Primary command:

```

fspec [OPTIONS] [PATH]

```

- With **no args**, `fspec` operates in the current directory:
  - looks for `./.fspec`
  - errors if not present
- With a **PATH positional arg**, `fspec` operates on that directory:
  - looks for `PATH/.fspec` by default
  - can be overridden with `--spec`

Rationale: most invocations want “run fspec here”.


## Inputs and discovery

### Target root (the filesystem to scan)

The target root is the directory that will be traversed.

- default: `.` (current directory)
- override: positional `[PATH]` or `--root <DIR>`

### Spec file location

By default, the spec file is `.fspec` located at the target root.

- default: `<root>/.fspec`
- override: `--spec <FILE>`

If `--spec` is provided:
- the **spec root** is `parent_dir(--spec)`
- anchored rules (`./` or `/`) are anchored at spec root (not the scan root)
- default behavior is still to scan `<root>` unless user sets `--root`

This supports:
- “scan a dir using its `.fspec`” (normal)
- “scan a dir using another spec file elsewhere” (advanced)

### Suggested constraints

- If `--spec` is provided and `--root` is not:
  - default `--root` to `parent_dir(--spec)` (ergonomic)
- If both are provided:
  - allow it, but treat it as advanced and show it clearly in verbose output


## Exit status

`fspec` is intended for CI use.

Recommended exit codes:

- `0`: no findings at or above the configured severity threshold
- `1`: findings exist at or above the configured severity threshold
- `2`: CLI/config error (bad args, spec parse error, IO error, etc.)

(Severity filtering is described below.)


## Core options

### Matching behavior (MatchSettings)

`MatchSettings` currently has:

- `allow_file_or_dir_leaf: bool`
- `default_severity: Severity` (Info / Warning / Error)

CLI should support setting these without creating a large matrix of flags.

#### Leaf interpretation

By default we want the “looser” UNIX-like interpretation:

- pattern `./bin/` => must match a directory
- pattern `./bin`  => may match file *or* directory (default)

Flags:

- `--leaf strict`  => `allow_file_or_dir_leaf = false`
- `--leaf loose`   => `allow_file_or_dir_leaf = true` (default)

(Alternative spelling: `--strict-leaf` / `--loose-leaf`. The `--leaf` enum form scales better.)

#### Default severity

When a path is reported (unaccounted, file/dir collision, etc.) and does not have a more specific severity,
`default_severity` is used.

Flags:

- `--severity info|warning|error` (default: `warning`)

Notes:
- This is not the *format* of output; it’s the “what counts as failing”.
- CI usage typically wants `--severity error` so warnings don’t fail builds.


## Output options

### Output format

Flags:

- `--format human|json` (default: `human`)

- `human` is line-oriented and readable
- `json` is stable, intended for other tools/CI annotation

### Output destination

Flags:

- `--output -` (default) => stdout
- `--output <FILE>` => write report to file

(For JSON, `--output` is especially useful.)

### Report field selection

The report produced by `fspec-core` contains multiple sets/categories (e.g. unaccounted vs allowed vs ignored).
The CLI should allow selecting which categories are emitted.

Flags:

- `--show unaccounted` (default)
- `--show allowed`
- `--show ignored`
- `--show all` (equivalent to enabling everything)
- `--show rules` (emit parsed rules / compiled representation)

Semantics:
- `--show` is additive and may be repeated:
  - `--show unaccounted --show ignored`
- If `--show` is not provided, default is only `unaccounted` (the “lint” use case).

### Verbosity / debug

Flags:

- `-q, --quiet`      => print only essential results (e.g. failing paths), no summaries
- `-v, --verbose`    => include summary counts and basic run context (roots, spec path)
- `-vv`              => include extra debug-like details (timings, traversal stats)
- `--debug`          => include internal debugging output (reserved; may be noisy)

Notes:
- Verbosity affects human output primarily.
- JSON output should remain machine-stable; verbosity should not change JSON schema, only optionally add top-level metadata fields (e.g. `meta`).


## Suggested UX in human output

Default (`fspec` with no args):

- Print a short header:
  - spec path
  - scan root
  - active settings (leaf mode, severity threshold)
- Print “unaccounted” paths grouped by severity:
  - `WARNING unaccounted file: path/to/file`
  - `WARNING unaccounted dir:  path/to/dir/`
- Print a one-line summary at end:
  - counts per category + severity
  - exit code reason (if non-zero)

Quiet (`-q`):

- Print only the paths (one per line) or minimal `SEVERITY path` lines.

Verbose (`-v` / `-vv`):

- Print totals per category even if empty.
- Optionally print timing and traversal counts at `-vv`.


## Examples

Run in current directory (requires `./.fspec`):

```

fspec

```

Run in a specific project directory:

```

fspec ./my_project

```

Use a spec file from elsewhere:

```

fspec --spec ./specs/media.fspec --root /mnt/media

```

Strict leaf interpretation (treat `./bin` as file-only):

```

fspec --leaf strict

```

Fail CI only on errors:

```

fspec --severity error

```

Emit JSON report to a file:

```

fspec --format json --output fspec_report.json

```

Show ignored paths too (diagnostics):

```

fspec --show unaccounted --show ignored

```

Show everything (heavy output):

```

fspec --show all -v

```


## Implementation notes (crate structure)

`crates/fspec` should likely be a small binary crate using `clap` (derive-based) with:

- `main.rs`:
  - parse args
  - resolve root/spec paths
  - build `MatchSettings`
  - call `fspec-core` entrypoint(s)
  - render report
  - choose exit code

- `args.rs`:
  - clap definitions
  - enums for `--format`, `--leaf`, `--severity`, `--show`

- `render.rs`:
  - human renderer
  - JSON renderer (likely `serde_json`)

(Exact module splits are flexible; goal is keeping `main.rs` small and testable.)


## Open questions for later iteration

- Should there be a `check` subcommand (`fspec check`) for future growth (fmt, explain, etc.)?
- Should `--show rules` print source rules, parsed AST, or compiled form?
- Should JSON include raw rules by default or only when requested?
- Should `--root` default to spec parent when `--spec` is provided (recommended above)?
- How should symlinks be handled (follow vs not follow)? (Not in current `MatchSettings`, but will matter.)
