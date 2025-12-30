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
fn golden_alternate_syntax() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
ignore /aaa.file
ignore    /bbb.file
ignore ddd.file
"#,
    );

    write_file(&root.join("aaa.file"), "dummy_file\n");
    write_file(&root.join("bbb.file"), "dummy_file\n");
    write_file(&root.join("ddd.file"), "dummy_file\n");
    write_file(&root.join("a/b/c/ddd.file"), "dummy_file\n");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_ignored("aaa.file"));
    assert!(report.is_ignored("bbb.file"));
    assert!(report.is_ignored("ddd.file"));
    assert!(report.is_ignored("a/b/c/ddd.file"));
}
