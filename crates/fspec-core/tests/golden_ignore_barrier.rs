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
fn ignored_subtree_barrier_and_rooted_reallow() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
ignore /bin/
allow /bin/allowed.txt
"#,
    );

    write_file(&root.join("bin/allowed.txt"), "dummy_file");
    write_file(&root.join("bin/ignored.txt"), "dummy_file");

    // Run check
    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("bin/allowed.txt"));
    assert!(report.is_allowed("bin/"));

    assert!(report.is_ignored("bin/ignored.txt"));
}
