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
fn golden_ambiguous_last_wins_and_warn() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Both allows match src/file.txt
    //  - first: any snake_case rs anywhere
    //  - second: specifically under /src/**
    // Last match should be the winner (useful if you record "winning rule line").
    write_file(
        &root.join(".fspec"),
        r#"
allow file.txt
allow /src/**/file.txt
"#,
    );

    write_file(&root.join("src/file.txt"), "dummy_file");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("src/file.txt"));

    // TODO: Check which rule allowed src/file.txt when tags are available.
}
