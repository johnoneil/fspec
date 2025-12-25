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
fn golden_allowed_nested_dirs_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# allow a specific file anchored at root
allow /file.txt
# allow a, b, c, d, *and* a/b/c/file.txt
allow /a/b/c/file.txt
# allow any file.txt *and* its parent directories.
allow file.txt
"#,
    );

    write_file(&root.join("file.txt"), "dummy-file");
    write_file(&root.join("a/b/c/file.txt"), "dummy-file");
    write_file(&root.join("e/f/g/file.txt"), "pub fn help() {}\n");
    // an unaccounted for file
    write_file(&root.join("e/f/g/other.txt"), "nope");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("file.txt"));
    assert!(report.is_allowed("a/"));
    assert!(report.is_allowed("a/b"));
    assert!(report.is_allowed("a/b/c"));
    assert!(report.is_allowed("a/b/c/file.txt"));
    assert!(report.is_allowed("e/"));
    assert!(report.is_allowed("e/f/"));
    assert!(report.is_allowed("e/f/g/"));
    assert!(report.is_allowed("e/f/g/file.txt"));

    // for now the root .fspec file is allowed
    assert!(report.is_allowed(".fspec"));
    assert!(report.is_unaccounted(".fspec") == false);

    assert!(report.is_unaccounted("e/f/g/other.txt"));
}
