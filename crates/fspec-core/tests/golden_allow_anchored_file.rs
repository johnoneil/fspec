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
fn golden_basic_all_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /allowed.txt
"#,
    );

    write_file(&root.join("allowed.txt"), "dummy_file");
    write_file(&root.join("unaccounted.txt"), "dummy_file");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("/allowed.txt"));
    assert!(report.is_unaccounted("/unaccounted.rs"));
}
