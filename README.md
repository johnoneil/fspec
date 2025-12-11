# **fspec — A Declarative Way to Tame and Audit Filesystems**

fspec is a tool that helps define file system structure and naming conventions such that:

* Current filesystem structure may be continually checked to prevent drift from the agreed-upon spec.
* The introduction of new file types and directories must be reviewed via updates to the spec.
* Documentation for teams or migration/maintenance code may be generated from the spec.
* Complex audits of highly process-oriented filesystems may be conducted.
* Explicit conventions are agreed upon, reducing reliance on friction-inducing tribal knowledge.
* Optional rules and dependencies allow missing, stale, and other process-related file states to be inferred.

It lets teams **describe the filesystem they *intend* to have**, and then compare the real world against that description. In other words, **fspec lints the filesystem itself.**

Think of it as **a style guide for your directory tree — one the computer can enforce.**

---

# Example fspec

```toml
# fspec.toml example — a video archive

# Establish conventions which may be shared across fspecs in a tree.
[identifiers]
year = "{int(4)}"               # 1946
title = "{snake_case}"          # its_a_wonderful_life
show = "{PascalCase}"           # CubbyBear
season = "s{int(2+)}"           # s01, s02, s999
episode_number = "e{int(2+)}"   # e01, e02, e999
ext = "{mp4,mov,mkv,avi}"

# movie files: e.g. "movies/1926/its_a_wonderful_life.1926.mp4"
[file.movie]
pattern = "movies/{year}/{title}_{year}.{ext}"

# episodes by season: e.g. "shows/CubbyBear/s01/CubbyBear.1936.s01e03.mkv"
[file.episode]
pattern = "shows/{show}/{season}/{show}.{year}.{season}{episode_number}.{ext}"

# For any subtitle files, there must be a similarly named episode file
[file.episode_subtitle]
pattern = { use = "episode", ext = "{srt,ass}" }
rule = [ "episode exists_by show,season,episode_number" ]
```

Readable. Declarative. Self-documenting.

---

# Typical Use Cases

## **1. Check the current filesystem against the intended one**

```
$ fspec check --suggest
✓ directory structure matches spec
✗ filename "renders/approved/shot OP 010.mp4" does not match spec
  did you mean "shot_OP_010.mp4"?
```

fspec reports:

* pattern violations
* misplaced files
* missing files
* unknown / extraneous files
* structural drift

Deviations from the spec can be categorized as errors or warnings.

With `--suggest`, fspec offers fixes.

---

## **2. Audit the filesystem as evidence of process**

Your `.fspec.toml` may also include rules and dependencies, enabling fspec to infer:

* whether files are *missing*
* whether files are *stale* (derived files older than their sources)
* whether files are *blocked* (derived files exist without required sources)

```toml
# a set of software design docs, named by revision.
[file.software_design]
pattern = "designs/approved/**/{name:PascalCase}.{rev:DDMonYY}.pdf"

# c source is stale if it's older than the latest design doc.
# c source requires an associated unit test.
[file.c_source]
pattern = "source/**/{name:camelCase}.c"
depends_on = ["software_design latest_by rev"]
rule = ["c_unit_test exists_by name"]

# warn if there are orphaned unit tests
[file.c_unit_test]
pattern = "tests/**/{name:camelCase}_unit_test.c"
depends_on = ["c_source by name warn"]
```

```
$ fspec stale
✗ source/matrixCalc.c is stale (designs/approved/SoftwareDesign.12Dec25.pdf updated)
```

---

## **3. Clean, reorganize, or migrate messy folders**

Because fspec knows the *intended* structure:

* it finds files that don’t belong anywhere
* it identifies dead directories
* it shows mismatches between legacy naming and current standards
* it guides controlled migrations without guesswork

---

## **4. Generate filesystem-related documentation, tooling, and code**

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

fspec becomes a **single declarative source** from which the filesystem automation ecosystem can be derived.

---

# CLI Overview

```
fspec check       # Validate structure + naming
fspec stale       # Report stale derived assets
fspec suggest     # Suggest valid names for mismatched files
fspec explain     # Show how patterns expand / which regexes are generated
fspec graph       # Show dependency graph
```

Common flags:

```
--suggest   # add suggestions during check
--ai        # allow LLM-assisted fuzzy matching
--root      # prevent the current directory .fspec from inheriting parents
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

# Roadmap

### **1. Initial Implementation**

* [ ] Define `.fspec.toml` format
* [ ] Implement identifier grammar (`int()`, `snake_case`, unions, etc.)
* [ ] Pattern compiler (expand patterns → regex)
* [ ] Directory walking + match engine
* [ ] Basic CLI (`check`, `suggest`, etc.)
* [ ] Basic reporting (pattern violations, unknown files)

---

### **2. Dependency & Rule Engine**

* [ ] Implement dependency resolution
* [ ] Support `depends_on` grammar
* [ ] Detect stale files
* [ ] Detect blocked files
* [ ] Detect missing files
* [ ] Implement rule grammar (`exists_by`, etc.)
* [ ] Rule violation reporting

---

### **3. Matching Engine & AI Integration**

* [ ] Edit-distance filename matcher
* [ ] Structural matcher (e.g., extract season/episode, numeric groups)
* [ ] “Did you mean” suggestion engine
* [ ] Optional LLM-assisted matching (`--ai`)
* [ ] Configurable match strategies / thresholds

---

### **4. Documentation & Code Generation**

* [ ] Generate human-readable Markdown documentation from fspec
* [ ] Generate regexes from compiled patterns
* [ ] Generate ingest/rename scripts (Python)
* [ ] Generate directory scaffolding templates
* [ ] Dependency graph output (`fspec graph`)
* [ ] JSON output mode for tooling integration

---
