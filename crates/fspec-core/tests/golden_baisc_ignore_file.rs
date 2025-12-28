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
fn golden_basic_ignore_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# rooted file
ignore /ignored.file
# unanchored file anywhere
ignore other.ignored.file
"#,
    );

    write_file(&root.join("ignored.file"), "this file is ignored\n");
    write_file(
        &root.join("a/b/c/d/other.ignored.file"),
        "this is ignored too\n",
    );
    write_file(&root.join("other.ignored.file"), "this is ignored too\n");
    write_file(&root.join("a/b/c/d/not.ignored.file"), "not ignored\n");
    write_file(&root.join("not.ignored.file"), "not ignored\n");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_ignored("ignored.file"));
    assert!(report.is_ignored("a/b/c/d/other.ignored.file"));
    assert!(report.is_ignored("other.ignored.file"));

    assert!(report.is_unaccounted("a/b/c/d/not.ignored.file"));
    assert!(report.is_unaccounted("not.ignored.file"));
}
