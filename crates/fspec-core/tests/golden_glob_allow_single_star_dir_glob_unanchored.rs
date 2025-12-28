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
fn golden_basic_allow_single_star_dir_glob() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow subdir/*/main.rs
"#,
    );

    write_file(&root.join("src/main.rs"), "fn main() {}\n"); // should NOT match: no intermediate segment
    write_file(&root.join("name/subdir/src/main.rs"), "pub fn help() {}\n"); // should match: one intermediate segment
    write_file(
        &root.join("another/name/subdir/src/utils/deeper/main.rs"),
        "pub fn more() {}\n",
    ); // should NOT match: too deep

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("name/subdir/src/main.rs"));
    assert!(report.is_allowed("name/subdir/src"));
    assert!(report.is_allowed("name/subdir"));
    assert!(report.is_allowed("name"));

    assert!(report.is_unaccounted("src/main.rs"));
    assert!(report.is_unaccounted("another/name/subdir/src/utils/deeper/main.rs"));
}
