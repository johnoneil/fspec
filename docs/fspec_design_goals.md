# fspec — Design Goals

## 1. Declarative first

fspec rules describe *what should exist*, not *how to check it*.

There is no execution model exposed to the user.
The same specification should be interpretable by different tools.

---

## 2. Readable by non-authors

An `.fspec` file should be understandable months or years later by
someone who did not write it.

Rules are line-based, ordered, and explicit.

---

## 3. Order matters, but predictably

Rules are evaluated top-to-bottom.
If multiple rules match, the last rule wins.

This mirrors familiar patterns from tools like `.gitignore`, while
remaining deterministic.

---

## 4. Anchoring is explicit

Patterns are either:

- anchored at the `.fspec` root, or
- unanchored and match anywhere

There is no implicit anchoring.
This avoids subtle mismatches and makes intent clear.

---

## 5. Structure over cleverness

fspec avoids:

- complex regex-driven semantics
- hidden backtracking behavior
- implicit directory creation rules

Directories implied by allowed files are considered structurally allowed,
but nothing more.

---

## 6. Consistency enforcement

Repeated placeholders must match the same value within a rule.

This encodes consistency constraints that humans routinely violate but
rarely document.

---

## 7. Staged capability growth

fspec is developed in explicit conformance levels.

Each level adds capability without invalidating earlier specifications.
This allows real-world use before the system is “complete”.

---

## 8. Tooling-friendly core

The core rule engine is designed to be reusable:

- CLI tools
- CI checks
- editors
- higher-level automation

The `.fspec` file is the stable interface.
