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
fn golden_basic_allow_double_star_glob() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /src/**/{tag:snake_case}.rs
"#,
    );

    write_file(&root.join("WeirdFile.rs"), "fn main() {}\n");
    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("src/utils/helpers.rs"), "pub fn help() {}\n");
    write_file(&root.join("src/a/b/c/deeper.rs"), "pub fn deeper() {}\n");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("src/utils/helpers.rs"));
    assert!(report.is_allowed("src/a/b/c/deeper.rs"));

    // error on deviations
    assert!(
        report.is_unaccounted("WeirdFile.rs"),
        "contents should not be allowed by double star glob rule"
    );
}
