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

#[test]
fn golden_allow_anchored_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /allowed/
"#,
    );

    create_dir(&root.join("allowed"));
    create_dir(&root.join("unaccounted"));

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("/allowed/"));
    assert!(report.is_unaccounted("/unaccounted/"));
}
