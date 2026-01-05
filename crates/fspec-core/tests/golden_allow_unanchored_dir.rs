use std::fs;
use std::path::Path;

use fspec_core::{MatchSettings, check_tree};

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

#[test]
fn golden_allow_unanchored_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow allowed/
"#,
    );

    create_dir(&root.join("all/ancestors/allowed/not_this"));
    create_dir(&root.join("allowed"));

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("allowed"));
    assert!(report.is_allowed("all/ancestors/allowed"));
    assert!(report.is_allowed("all/ancestors"));
    assert!(report.is_allowed("all"));

    assert!(report.is_unaccounted("all/ancestors/allowed/not_this"));
}
