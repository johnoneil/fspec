use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[ignore]
#[test]
fn golden_rooted_vs_unanchored() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Rooted allows only root Cargo.toml
    // Unanchored allows Cargo.toml anywhere (second rule is last-match-wins if both match).
    write_file(
        &root.join(".fspec"),
        r#"
allow /Cargo.toml
allow Cargo.toml
"#,
    );

    write_file(&root.join("Cargo.toml"), "[workspace]\n");
    write_file(
        &root.join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\n",
    );

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("Cargo.toml"));
    assert!(report.is_allowed("crates/a/Cargo.toml"));
}
