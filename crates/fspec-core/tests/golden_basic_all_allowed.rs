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
fn golden_basic_all_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /Cargo.toml
allow /src/**/*.rs
"#,
    );

    write_file(&root.join("Cargo.toml"), "[package]\nname = \"demo\"\n");
    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("src/utils/helpers.rs"), "pub fn help() {}\n");
    write_file(
        &root.join("src/this_is_snake_case.rs"),
        "pub fn my_function() {}\n",
    );

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("Cargo.toml"));
    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("src/utils/helpers.rs"));
    assert!(report.is_allowed("src/this_is_snake_case.rs"));

    // Nothing else should be flagged.
    assert!(
        report.unaccounted_paths().is_empty(),
        "unexpected unaccounted paths"
    );
}
