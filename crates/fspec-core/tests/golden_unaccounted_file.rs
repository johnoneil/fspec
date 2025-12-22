use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[ignore = "enable when placeholder literals implemented."]
#[test]
fn golden_basic_unaccounted_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /src/**/{tag:snake_case}.rs
"#,
    );

    write_file(&root.join("unaccounted.file"), "dummy_file\n");
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

    assert!(report.is_unaccounted("unaccounted.file"));

    let unaccounted = report.unaccounted_paths();
    assert_eq!(unaccounted, vec!["unaccounted.file"]);
}
