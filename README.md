**fspec — A Declarative Way to Tame Filesystems**

Modern teams accumulate enormous numbers of files — from animation assets to research datasets, logs, CAD files, renders, source code, and everything in between.
Over time, naming conventions drift, contributors improvise, directory structures evolve organically, and eventually nobody knows:

* *Where things are supposed to go*
* *What files are supposed to be named*
* *What depends on what*
* *Which assets are stale, unused, misplaced, or redundant*

**fspec** is a declarative tool that solves this problem.
It lets teams **describe the filesystem they *intend* to have**, and then compare the real world against that description.

Think of it as **a style guide for your directory tree — one the computer can enforce.**

---

## Why fspec exists

Teams rarely fail because they lack powerful software; they fail because:

* Files migrate into the wrong places
* Contributors name assets differently
* Pipelines break silently
* Stale files linger for months
* Nobody can safely clean up old directories
* New hires learn conventions by tribal knowledge
* External audits (security, safety) require predictable structure
* Scripts downstream assume patterns that aren’t followed upstream

Most tools only lint source code.
**fspec lints the filesystem itself.**
This is where much real-world entropy happens.

---

# What fspec does

## **1. Defines how your filesystem *should* look**

Provide an`.fspec.toml` in a directory you want to tame, which describes:

* expected directory layout
* filename patterns
* reusable identifiers for frequently used elements (version, dates, case standards like camelCase )
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

Your `.fspec.toml` may also include dependencies between files, which allows auditing:

* Whether files are ~missing~ according to spec.
* Whether files are ~stale~ according to spec, that is, derived files which are older than their sources.
* Whether files are ~blocked~ according to spec, that is, derived files which exist when their sources are missing.

Files may depend on pre-production sources, upstream data, layouts, logs, or intermediate artifacts.

```toml
[file.render]
pattern = "renders/{cut}_{version}.mp4"
depends_on = ["file.layout", "file.genga"]
```

```
$ fspec stale
✗ renders/cut_010_v02.mp4 is stale (genga updated)
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
* research labs with thousands of datasets
* enterprise directories with compliance requirements
* shared servers passed through many contributors
* teams inheriting “mystery folders” with no documentation

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

The filesystem often *is* the workflow:

* When a file moves from “layout” to “genga,” that represents progress.
* When a folder changes from “approved” to “final,” that encodes process.
* When a naming scheme includes a revision, that encodes state.

fspec gives new contributors:

* a readable map of how things work
* a live specification of where assets belong
* a safety net to prevent accidental misplacement
* a way to learn team conventions without guesswork

This is not a static wiki.
This is *executable documentation*.

---

# Example fspec

```toml
# fspec.toml — example taming video archives

[identifiers]
year = "{int(4)}" # 1946
title = "{Pearl.Case}" # Its.A.Wonderful.Life
show = title # just an alias
season = "s{int(2+)}" # s01, s02, s999
episode = "e{int(2+)}" # e01, e02, e999
ext = "{mp4,mov,mkv,avi}"
# or possibly:
ext = "{mp4|mov|mkv|avi}"


# movies in directories by year.
[file.movie]
# e.g. "It's.A.Wonderful.Life.1946.mp4"
pattern = "movies/{year}/{title}.{year}.{ext}"

[file.episode]
pattern = "shows/{show}/{season}/{show}.{season}{episode}.{ext}"

```

Readable. Declarative. Self-documenting.

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

fspec brings the benefits of linting, static analysis, schema validation, and dependency checking
**into the directory tree itself.**

---

# Roadmap (short)

* Softer/powerful selector language
* Policy system for stale/blocked/untracked files
* Rewrite mode (`fspec fix`)
* Code generation (in multiple languages)
* LLM plugin interface (custom matching strategies)
* Integration with PART (process-aware repository tooling)

---

If you want, I can also produce:

* A more concise GitHub-style version
* A version aimed specifically at VFX/animation teams
* A version aimed at scientific/engineering datasets
* A more technical reference-spec version

But sleep on it first — structure usually becomes clearer after a night.
