use std::fs;
use std::path::Path;

use fspec_core::{MatchSettings, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn golden_basic_allow_single_star_file_glob() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow src/*
"#,
    );

    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("src/a.rs"), "another_file");
    write_file(&root.join("src/b.rs"), "another_file");
    write_file(&root.join("nested/src/a.rs"), "pub fn help() {}\n"); // should match: one intermediate segment
    write_file(&root.join("nested/src/subdir/b.rs"), "pub fn more() {}\n"); // should NOT match: too deep

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("src/a.rs"));
    assert!(report.is_allowed("src/b.rs"));
    assert!(report.is_allowed("src"));
    assert!(report.is_allowed("nested/src/a.rs"));
    assert!(report.is_allowed("nested/src"));
    assert!(report.is_allowed("nested"));

    assert!(report.is_unaccounted("nested/src/subdir/b.rs"));
}
