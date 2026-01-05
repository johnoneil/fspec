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
fn golden_trailing_slash_directory_only() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /data/
"#,
    );

    fs::create_dir_all(root.join("data")).unwrap();
    write_file(&root.join("data/file.txt"), "hello\n");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("data")); // the directory is allowed
    assert!(
        report.is_unaccounted("data/file.txt"),
        "contents should not be allowed by a directory-only rule"
    );
}
