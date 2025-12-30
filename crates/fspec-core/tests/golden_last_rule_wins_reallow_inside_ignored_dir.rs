use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn golden_last_rule_wins_reallow_inside_ignored_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# ignore the whole subtree (trailing slash is explicitly a dir).
ignore /a/
# but then re-allow one specific nested file (should win)
allow /a/b/c/file.txt
"#,
    );

    write_file(&root.join("a/b/c/file.txt"), "dummy-file");
    write_file(&root.join("a/other.txt"), "nope");

    let report = check_tree(root, Severity::Error).unwrap();

    // The re-allowed file must be allowed even though /a is ignored.
    assert!(report.is_allowed("a/b/c/file.txt"));

    // And its parents should be promoted to allowed.
    assert!(report.is_allowed("a"));
    assert!(report.is_allowed("a/b"));
    assert!(report.is_allowed("a/b/c"));

    // Another file under /a that isn't explicitly allowed should remain ignored.
    assert!(report.is_ignored("a/other.txt"));
}
