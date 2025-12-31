# Code Review: fspec-core

## Overview
This review covers the `fspec-core` crate and workspace structure. The codebase is well-structured and functional, but there are several areas for improvement before moving to Level 1 functionality.

---

## 🔴 Critical Issues

### 1. Debug Code in Production
**Location:** `src/lib.rs:37-43`, `src/walk.rs:152,233,257,265,365,382`

Multiple `eprintln!` statements are left in the code with "DEBUG: remove later" comments. These should be removed or made conditional.

**Impact:** Clutters output, may confuse users, and adds unnecessary overhead.

**Recommendation:** Remove all debug `eprintln!` statements or wrap them in a `cfg(debug_assertions)` block.

---

## ⚠️ Compiler Warnings

### 2. Unused Parameter
**Location:** `src/lib.rs:20`
```rust
pub fn check_tree(root: &Path, default_severity: Severity) -> Result<Report, Error>
```
The `default_severity` parameter is unused. This suggests it's planned for future use.

**Recommendation:** Prefix with `_` if intentionally unused, or implement the severity logic if ready.

### 3. Unreachable Pattern
**Location:** `src/walk.rs:343`
The `_ => {}` pattern in the match statement is unreachable because all combinations of `RuleKind` and `EntryKind` are already covered.

**Recommendation:** Remove the unreachable pattern.

### 4. Unused Fields
**Location:** `src/walk.rs:285,287`
Fields `rule_idx` in `Verdict::Allow` and `Verdict::IgnoredByInheritance` are never read, though they're likely needed for future diagnostics.

**Recommendation:** Prefix with `_` if intentionally unused, or use them in diagnostics if ready.

### 5. Unused Imports
**Location:** `src/parse.rs:70`, `src/pattern.rs:98`
Unused imports in test modules.

**Recommendation:** Remove unused imports.

---

## 🐌 Performance Issues

### 6. O(n) Lookups in `WalkOutput`
**Location:** `src/walk.rs:29,35,53,68,72,80,84,90,98`

The `WalkOutput` struct uses `Vec<PathBuf>` for tracking paths and uses `.contains()` for lookups, which is O(n). For large filesystems, this will be slow.

**Current approach:**
```rust
if !self.allowed_files.contains(&path.to_path_buf()) {
    self.allowed_files.push(path.to_path_buf());
}
```

**Recommendation:** Use `HashSet<PathBuf>` or `BTreeSet<PathBuf>` for O(1) or O(log n) lookups. This is a significant performance improvement for large filesystems.

**Trade-off:** `HashSet` requires `PathBuf` to implement `Hash` (it does), but `BTreeSet` provides sorted iteration which might be useful for deterministic output.

### 7. Inefficient String Operations in `canon_key`
**Location:** `src/report.rs:13-32`

The `canon_key` function performs multiple string allocations in loops:
```rust
while t.starts_with("./") {
    t = t[2..].to_string();  // Allocation on each iteration
}
```

**Recommendation:** Use string slicing with `&str` where possible, or use `strip_prefix` which is more efficient:
```rust
while let Some(rest) = t.strip_prefix("./") {
    t = rest;
}
```

Or better yet, use a single pass with string manipulation to avoid multiple allocations.

---

## 🗂️ Code Organization

### 8. Unused Module
**Location:** `src/engine.rs`

The `engine.rs` file exists but is empty and not imported in `lib.rs`. This suggests it's either:
- Planned for future use
- Leftover from refactoring
- Should be removed

**Recommendation:** Either remove it or add a comment explaining its future purpose.

### 9. Missing Error Display Implementation
**Location:** `src/error.rs`

The `Error` enum doesn't implement `std::error::Error` or `Display`, making it harder to use with error handling libraries and less user-friendly.

**Recommendation:** Implement `std::error::Error` and `Display` for better error handling:
```rust
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { path, source } => {
                write!(f: "IO error at {}: {}", path.display(), source)
            }
            Error::Parse { line, col, msg } => {
                write!(f, "Parse error at line {}, column {}: {}", line, col, msg)
            }
            Error::Semantic { msg } => {
                write!(f, "Semantic error: {}", msg)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

---

## 🔍 Code Quality

### 10. Path Comparison Inefficiency
**Location:** `src/walk.rs:32-33,38-39,57-58`

The code compares `PathBuf` with `&Path` using `retain`, which requires conversion:
```rust
self.unaccounted_files.retain(|p| p != path);
```

**Recommendation:** This is fine, but be aware that `PathBuf` implements `PartialEq<Path>`, so the comparison works. However, for consistency and clarity, consider using `PathBuf` throughout or documenting the choice.

### 11. Heuristic for File vs Directory
**Location:** `src/walk.rs:28`

The code uses `path.extension().is_some()` to determine if a path is a file. This is a heuristic that may not always be correct (e.g., files without extensions, or directories with dots in their names).

**Current code:**
```rust
if path.extension().is_some() {
    // treat as file
} else {
    // treat as directory
}
```

**Recommendation:** Since you're already walking the filesystem and have access to `file_type()` in `walk_dir`, consider passing the actual file type through to `allow_with_ancestors` instead of inferring it. However, this might require refactoring. For now, this heuristic is probably acceptable for Level 0.

### 12. Commented-Out Test Code
**Location:** `src/parse.rs:128-146`

There's a large block of commented-out test code. This should either be:
- Removed if no longer needed
- Uncommented and fixed if it's a work-in-progress
- Moved to a separate file if it's a reference

**Recommendation:** Clean up commented code.

---

## 📝 Documentation

### 13. Missing Public API Documentation
**Location:** Throughout public API

Public functions and types lack documentation comments. This is important for a library crate.

**Recommendation:** Add `///` documentation comments for:
- `check_tree` - main entry point
- `Error` enum variants
- `Report` methods
- `WalkCtx` and `WalkOutput` (if they're meant to be public)
- `Rule`, `FSPattern`, etc.

### 14. Module-Level Documentation
**Location:** `src/lib.rs`

The crate lacks a module-level doc comment explaining what `fspec-core` does.

**Recommendation:** Add a `#![doc = "..."]` or module-level `//!` comment.

---

## 🎯 Suggestions for Improvement

### 15. Consider Using `thiserror` or `anyhow`
For better error handling ergonomics, consider using `thiserror` for structured errors or `anyhow` for application-level error handling. This would make the `Error` enum more ergonomic and provide better error context.

### 16. Consider Adding `serde` Support
If you plan to serialize/deserialize `Report` or other structures (e.g., for JSON output), consider adding `serde` derives.

### 17. Path Normalization
The `canon_key` function normalizes paths, but this normalization happens in the report layer. Consider whether path normalization should happen earlier in the pipeline for consistency.

### 18. Test Organization
The golden tests are well-organized. Consider adding unit tests for edge cases in:
- Pattern matching (especially `**` edge cases)
- Path normalization
- Error message formatting

### 19. Workspace Structure
The workspace structure is clean. Consider:
- Adding a `CHANGELOG.md` for tracking changes
- Adding a `CONTRIBUTING.md` if you plan to accept contributions
- Documenting the relationship between `fspec-core` and `fspec-pattern` (even if `fspec-pattern` will be rewritten)

---

## ✅ Positive Observations

1. **Good test coverage** - The golden tests cover many scenarios
2. **Clear module structure** - Separation of concerns is good
3. **Thoughtful error types** - The `Error` enum is well-structured
4. **Good use of types** - `RuleKind`, `FSPattern`, etc. provide good type safety
5. **Deterministic traversal** - Sorting entries for deterministic output is a good practice
6. **Clean parsing logic** - The parser handles edge cases well (CRLF, comments, etc.)

---

## 📊 Summary

**Priority Fixes:**
1. Remove debug `eprintln!` statements
2. Fix compiler warnings (unused parameter, unreachable pattern, unused imports)
3. Replace `Vec` with `HashSet`/`BTreeSet` in `WalkOutput` for performance
4. Optimize `canon_key` function

**Nice to Have:**
5. Implement `Display` and `Error` for `Error` enum
6. Add public API documentation
7. Remove or document empty `engine.rs`
8. Clean up commented code

**Future Considerations:**
- Consider `thiserror` for error handling
- Add `serde` support if needed
- Improve file/directory detection in `allow_with_ancestors`

---

## 🧪 Testing

All existing tests pass. The suggested changes should not break existing functionality, but it's recommended to:
1. Run the full test suite after each change
2. Consider adding performance benchmarks for large filesystems
3. Test edge cases (very deep directory trees, many files, etc.)

