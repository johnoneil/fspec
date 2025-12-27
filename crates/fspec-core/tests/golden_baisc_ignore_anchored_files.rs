use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[ignore]
#[test]
fn golden_basic_ignore_anchored_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# anchored file
ignore /ignored.file
# anchored file in subdir
ignore /ignored/subdir/file.txt
# anchored ignored subdir
ignore /another/ignored/subdir/
"#,
    );

    write_file(&root.join("ignored.file"), "dummy_file");
    write_file(&root.join("ignored/subdir/file.txt"), "dummy_file");
    write_file(&root.join("another/ignored/subdir/file.txt"), "dummy_file");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_ignored("ignored.file"));
    assert!(report.is_ignored("ignored/subdir/file.txt"));
    assert!(report.is_ignored("another/ignored/subdir"));

    // descendants of ignores are ignored
    assert!(report.is_ignored("another/ignored/subdir/file.txt"));

    // ancestors of ignores are not ignored
    assert!(!report.is_ignored("ignored/subdir/"));
    assert!(!report.is_unaccounted("another/ignored"));
}
