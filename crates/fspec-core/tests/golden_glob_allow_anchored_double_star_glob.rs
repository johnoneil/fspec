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
fn golden_basic_allow_anchored_double_star_glob() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /src/**/file.txt
"#,
    );

    write_file(&root.join("file.txt"), "dummy_file");
    write_file(&root.join("src/file.txt"), "dummy_file");
    write_file(&root.join("src/utils/file.txt"), "dummy_file");
    write_file(&root.join("src/a/b/c/file.txt"), "dummy_file");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("src/file.txt"));
    assert!(report.is_allowed("src/utils/file.txt"));
    assert!(report.is_allowed("src/a/b/c/file.txt"));

    assert!(report.is_unaccounted("file.txt"));
}
