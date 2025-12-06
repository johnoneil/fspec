# Proclint

Have you heard any of the following at your workplace?

* "Sorry, I named the file wrong. There are two spaces instead of one."
* "Well, the character design was updated, but we forgot to update the character models."
* "I thought these assets were being rebuilt when the sources changed. Aren't they?"
* "The design review signoff sheet is missing!"
* "Oops, I accidentally skipped a number on the sidecar when checking in the final render."
* "Please don't put spaces in filenames. It makes our lives much more difficult!"
* "We're missing unit tests for vector_math.c."
* "The texture was checked in but its size isn't POT."
* "The jira task says this is complete, but the work isn't where it should be."
* "There was an implicit order definition in the Makefile such that asset A always got built before asset B. I didn't notice when it broke."

`proclint` is a tool that is meant to formalize the many process related issues that arise in art, gaming, engineering, software, and other heavily process oriented development workplaces. It aims to capture "best practice" definitions which even today may be unspecified or carried around by word-of-mouth or obscure documentation. This process related "tribal knowledge" is currently difficult to capture and frequently the source of team friction.

The key is that many processes may be represented by a file structure. That is, for every step in the process, one or more files need to be produced.

This tool helps *define* and *lint* any process which can be represented as a file structure. 

This is applicable to many areas such as:

* Art/Game development: where artwork flows from designs, and which may inform 3D models, and which might be packed in archives. Teams may have difficulty naming and placing everything correctly.
* Engineering: where analyses and code may flow from design documents and knowing the stale status of previous analyses or code is important as designs change.
* Software development: where current linting tools frequently don't address file structure or naming conventions, or such areas may be addressed ad-hoc.

`proclint` finds places where the current files and artifacts don't adhere to a carefully predefined process, and can do so automatically on change as part of CI.

In other words, `proclint`:

* Lets you define directory naming and structure conventions.
* Lets you define file naming and placement conventions.
* Lets you define file dependency structures, so missing or stale downstream artifacts can be detected.
* Reports how well current process artifacts (files) adhere to the defined process. Deviations can be returned as warnings or errors.
* Provides hooks for custom file validators.
* Can provide state information to existing database oriented process management systems (i.e. Jira) which might have an imperfect knowledge of the current project state.
* Is meant to be configuration management system agnostic, so it can therefore work with Git, SVN etc. The process definitions are just files to be checked in.
* No server or network checks are needed.

# Usage

```
proclint check [PATH]   # main thing people do
proclint check --suggest # same, but with “did you mean” style hints
proclint check --json   # machine-readable output for CI tooling
proclint init   # (future) create a default config
proclint explain ITEM   # (future) explain why something is stale / wrong
```

# Examples

Here's some use cases of the `proclint` tool, using the symbology: `✓ = OK`, `✗ = Error`, `? = Warning`.

Configuration can define whether process deviations are reported as warnings or errors to refine CI integration and workflows.

## Naming and Structure Checks

```
$ proclint check
✓ directory structure matches spec
✗ filename "renders/approved/shot OP 010.mp4" does not match spec.
```

## Dependency and Staleness

The tool allows checking whether derived assets flow correctly from their sources.

"Stale" means any file that is newer than a source or dependency. For example a 3D model may be created from a design sheet image. If the design has changed, but not the model, the model is now "stale".

```
$ proclint check
✓ directory structure matches spec
✓ filenames match spec
? file "analyses/unapproved/FMA-01.xlsx" depends on nonexistent parent "designs/unapproved/system-design-01.pdf"
? file "models/approved/character-man-005.3ds" is stale via "designs/approved/characters.jpg"
```

## Blocked Files

A file is "blocked" if required sources do not yet exist, or don't yet exist in the right location.

That is, the work can't be done until something else is done.

```
$ proclint check art/images/house-001.png
✗ file "art/images/house-001.png" is blocked. Missing dependency "designs/approved/architecture.pdf"
```


## Custom File Validation

`proclint` supports customized file validation.

```
$ proclint check
✓ directory structure matches spec
✓ filenames match spec
✗ file "art/textures/approved/tank-010.tga" fails POT check.
```

## Helping Developers with Suggestions

`proclint` can suggest how to conform to the spec.

```
$ proclint check --suggest
✓ directory structure matches spec
✗ filename "art/skins/Bg-1.png" does not match spec. Did you mean "art/skins/bg-001.png" ?
```
