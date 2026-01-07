# fspec — Philosophy

## The filesystem is part of the system

Modern projects rely on filesystem structure as an implicit API:
file placement, directory names, naming conventions, and invariants
are assumed by humans, scripts, build systems, and increasingly by LLMs.

Yet these assumptions are rarely formalized.

**fspec is based on the premise that filesystem structure itself is a contract.**

If that contract exists, it should be:

- explicit
- human-readable
- machine-checkable
- versioned
- reviewable

---

## Explicit structure beats folklore

Most filesystem conventions live in one of three places:

- tribal knowledge
- ad-hoc scripts
- informal documentation that drifts over time

These approaches fail quietly. Deviations accumulate slowly and are
often only discovered when downstream tooling breaks.

fspec replaces folklore with a small, declarative specification that
describes *what is allowed to exist*.

Anything not explicitly allowed is surfaced.

---

## Allow-lists, not heuristics

fspec intentionally uses an **allow-list** model:

> If it is not allowed, it is unexpected.

This flips the default from permissive to intentional.
Ignored paths are explicit, scoped, and reviewable.

This makes filesystem evolution visible and auditable.

---

## Humans, teams, and LLMs

Large filesystems are increasingly maintained with the assistance of
automation and LLMs.

Without an explicit specification, these tools must infer intent from
partial context and naming patterns.

fspec provides a shared, deterministic definition of filesystem
structure so that:

- humans can reason about layout
- teams can review structural changes
- tools and LLMs can act with constraints instead of guesses

---

## Small language, constrained power

fspec is intentionally limited:

- no scripting
- no arbitrary conditionals
- no filesystem mutation

This constraint is a feature.

A small, declarative language is easier to audit, reason about, and
maintain over long-lived projects.

---

## What fspec is not

fspec is not:

- a build system
- a replacement for `.gitignore`
- a general-purpose policy engine

It is a **structural lint** for the filesystem itself.

> fspec lints the filesystem.
