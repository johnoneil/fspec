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

#[ignore = "enable when placeholder literals implemented."]
#[test]
fn golden_basic_unaccounted_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /src/**/{tag:snake_case}.rs
"#,
    );

    create_dir(&root.join("target/bin"));

    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("src/utils/helpers.rs"), "pub fn help() {}\n");
    write_file(
        &root.join("src/this_is_snake_case.rs"),
        "pub fn my_function() {}\n",
    );

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("src/utils/helpers.rs"));
    assert!(report.is_allowed("src/this_is_snake_case.rs"));

    assert!(report.is_unaccounted("target"));
    assert!(report.is_unaccounted("target/bin"));

    let unaccounted = report.unaccounted_paths();
    assert_eq!(unaccounted, vec!["target", "target/bin"]);
}
