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

    // Both allows match src/main.rs:
    //  - first: any snake_case rs anywhere
    //  - second: specifically under /src/**
    // Last match should be the winner (useful if you record "winning rule line").
    write_file(
        &root.join(".fspec"),
        r#"
allow {tag:snake_case}.rs
allow /src/**/{tag:snake_case}.rs
"#,
    );

    write_file(&root.join("src/main.rs"), "fn main() {}\n");

    let report = check_tree(root, Severity::Error).unwrap();
    assert!(report.is_allowed("src/main.rs"));

    // If you implement ambiguity diagnostics, this should exist.
    // If not yet implemented, you can comment this out until Level 2.
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.code == "ambiguous_match" && d.path == "src/main.rs"),
        "expected ambiguous_match diagnostic for src/main.rs"
    );
}
