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
fn golden_basic_ignore_anchored_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# anchored directory
ignore /ignored/
# anchored subdir
ignore /ignore/this/too/
"#,
    );

    write_file(&root.join("ignored/file.txt"), "dummy_file");
    write_file(&root.join("ignore/this/too/file.txt"), "dummy_file");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_ignored("ignored"));
    assert!(report.is_ignored("ignored/file.txt"));
    assert!(report.is_ignored("ignore/this/too"));
    assert!(report.is_ignored("ignore/this/too/file.txt"));

    // ancestors of ignores are *NOT* ignored. Descendants are.
    assert!(!report.is_ignored("ignore"));
    assert!(!report.is_ignored("ignore/this"));
}
