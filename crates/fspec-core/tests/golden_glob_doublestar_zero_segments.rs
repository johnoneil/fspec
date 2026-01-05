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
fn golden_glob_doublestar_zero_segments() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /src/**/file.txt
"#,
    );

    // ** should match zero segments -> allows src/main.rs
    write_file(&root.join("src/file.txt"), "dummy_file");
    // ...and deeper nesting too
    write_file(&root.join("src/a/b/c/file.txt"), "dummy_file");

    let report = check_tree(root, &MatchSettings::default()).unwrap();
    assert!(report.is_allowed("src/file.txt"));
    assert!(report.is_allowed("src/a/b/c/file.txt"));
}
