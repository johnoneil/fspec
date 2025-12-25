use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
    assert!(path.is_dir());
}

#[ignore]
#[test]
fn golden_basic_ignore_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# rooted dir
ignore /ignored/
# unanchored dir anywhere
ignore ignored/
ignore also_ignored/
ignore also.ignored/
"#,
    );

    create_dir(&root.join("ignored"));
    create_dir(&root.join("also_ignored"));
    create_dir(&root.join("also.ignored"));
    create_dir(&root.join("a/b/c/d/ignored"));
    create_dir(&root.join("e/f/g/h/also_ignored"));
    create_dir(&root.join("i/j/k/l/also.ignored"));

    // some not ignored
    create_dir(&root.join("not_ignored"));
    create_dir(&root.join("m/n/o/p/not.ignored"));

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_ignored("ignored"));
    assert!(report.is_ignored("also_ignored"));
    assert!(report.is_ignored("also.ignored"));
    assert!(report.is_ignored("a/b/c/d/ignored"));
    assert!(report.is_ignored("e/f/g/h/also_ignored"));
    assert!(report.is_ignored("i/j/k/l/also.ignored"));

    assert!(report.is_unaccounted("not_ignored"));
    assert!(report.is_unaccounted("m/n/o/p/not.ignored"));
}
