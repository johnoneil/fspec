**fspec — A Declarative Way to Tame Filesystems**

Modern teams accumulate enormous numbers of files — from source code to art assets, logs, CAD files, renders, and everything in between.
Over time, naming conventions drift, contributors improvise, directory structures evolve organically, and eventually nobody knows:

* *Where things are supposed to go*
* *How files are supposed to be named*
* *What depends on what*
* *Which assets are stale, unused, misplaced, or redundant*

**fspec** is a declarative tool that solves this problem.
It lets teams **describe the filesystem they *intend* to have**, and then compare the real world against that description.

Think of it as **a style guide for your directory tree — one the computer can enforce.**

---

## Why fspec exists

Directory structure and naming conventions are prime candidates for pure tribal knowledge. And even if documented, the relationships between files may not be. People are required to carry such information around in their heads.

Auditing large file systems therefore either require a lot of manual work or custom scripts. But by specifying an fspec for your filesystem, no matter how large, auditing becomes possible.

Most tools only lint source code.
**fspec lints the filesystem itself.**

---

# Example fspec

```toml
# fspec.toml — example video archive

# unambiguously pin down your conventions
# These identifiers may be shared across fspecs.
[identifiers]
year = "{int(4)}" # 1946
title = "{snake_case}" # its_a_wonderful_life
show = "{PascalCase}" # CubbyBear
season = "s{int(2+)}" # s01, s02, s999
episode_number = "e{int(2+)}" # e01, e02, e999
ext = "{mp4,mov,mkv,avi}"

# unambiguously pin down your directory and filename structure
[file.movie]
# e.g. "movies/1926/its_a_wonderful_life.1926.mp4"
pattern = "movies/{year}/{title}_{year}.{ext}"

# and episodes by season
[file.episode]
# e.g. shows/CubbyBear/s01/CubbyBear.s01e03.mkv
pattern = "shows/{show}/{season}/{show}.{season}{episode_number}.{ext}"

# and associated files like subtitles
[file.episode_subtitle]
# the same pattern as episode, but with a different extension
pattern = { use = "episode", ext = "{srt,ass}" }
# create a dependency, which tells us that if there's a subtitle file,
# there should be an existing, similarly named episode.
depends_on = [ "episode by show,season,episode_number" ]

```

Readable. Declarative. Self-documenting.

---

# What fspec does

## **1. Defines how your filesystem *should* look**

Provide an`.fspec.toml` in a directory you want to tame, which describes:

* expected directory layout
* filename patterns
* reusable identifiers for frequently used elements (version, dates, case standards like camelCase or snake_case )
* allowed variants
* optional dependencies between files
* whether something is required or optional

The description becomes the shared source of truth for your team.

---

## **2. Checks the real filesystem against the intended one**

```
$ fspec check --suggest
✓ directory structure matches spec
✗ filename "renders/approved/shot OP 010.mp4" does not match spec
  did you mean "shot_OP_010.mp4"?
```

fspec reports:

* misplaced files
* missing files
* pattern violations
* unknown files
* structural drift

With `--suggest`, it offers fixes.

---

## **3. Allows you to audit the filesystem as an evidence of process**

Your `.fspec.toml` may also include dependencies between files, which allow auditing:

* Whether files are *missing* according to spec.
* Whether files are *stale* according to spec, that is, when derived files are older than their sources.
* Whether files are *blocked* according to spec, that is, when derived files exist when their sources are missing.



```toml
# an example of c source files which depend on a software design document.
# If the design is newer than the produced source, your source needs to be looked at, that is, it's "stale".

[file.software_design]
pattern = "designs/**/{name:PascalCase}.{rev:DDMonYY}.pdf"

[file.c_source]
pattern = "source/**/{name:camelCase}.c"
# c source depends on the latest software design doc.
depends_on = ["software_design latest_by rev"]
```

```
$ fspec stale
✗ source/matrixCalc.c is stale (designs/SoftwareDesign.12Dec25.pdf updated)
```

---

## **4. Helps clean, reorganize, or migrate messy folders**

Because fspec knows the *intended* structure:

* it finds files that don’t belong anywhere
* it identifies dead directories
* it shows mismatches between legacy naming and current standards
* it guides controlled migrations without guesswork

This is extremely useful for:

* long-lived projects
* Art and design teams with many files, many contributors, naming things by hand
* enterprise and process heavy directories with compliance requirements
* people who dislike writing regexes, file manipulation scripts and awk

---

## **5. Generates filesystem-related code, tooling and documentation**

Because the description is formal and consistent, fspec can generate:

* directory templates
* ingest/rename scripts
* file validators
* migration tools
* watchers
* regexes for downstream tools
* skeleton repos or project trees
* documentation of naming rules
* dependency graphs

fspec becomes a **single declarative source** from which your filesystem-related automation can be derived.

Teams no longer write ad hoc scripts full of brittle regexes.

---

## **6. Provides a shared vocabulary for contributors**

The filesystem often *is* the workflow, and fspec just makes this explicit, providing:

* a readable map of how things work
* a live specification of where assets belong
* a safety net to prevent accidental misplacement
* a way to learn team conventions without guesswork

---

# CLI Overview

```
fspec check       # Validate structure + naming
fspec stale       # Report stale derived assets
fspec suggest     # Suggest valid names for mismatched files
fspec explain     # Show how patterns expand / which regexes are generated
fspec graph       # Show dependency graph (optional)
```

Common flags:

```
--suggest   # add suggestions during check
--ai        # allow LLM-assisted fuzzy matching
--root      # break inheritance from parent directories
--json      # output machine-readable results
```

---

# Philosophy

* **The filesystem is part of the workflow. Treat it that way.**
* **Conventions are valuable only if enforced consistently.**
* **Humans are bad at remembering naming rules; fspec isn’t.**
* **Declarative rules beat ad hoc scripts.**
* **Process integrity begins with structure.**

fspec makes structure explicit, and structured workflows enforce themselves.

fspec brings the benefits of linting, static analysis, schema validation, and dependency checking
**into the directory tree itself.**

---

# Roadmap

TODO:

---

