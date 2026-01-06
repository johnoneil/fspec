use std::fs;
use std::path::Path;

use fspec_core::Severity;
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
fn golden_strict_leaf_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow ./allowed/
allow ./allowed_dir
allow unanchored_allowed/
allow unanchored_allowed_dir
"#,
    );

    create_dir(&root.join("allowed"));
    create_dir(&root.join("allowed_dir"));
    create_dir(&root.join("a/b/c/unanchored_allowed"));
    create_dir(&root.join("d/e/f/unanchored_allowed_dir"));

    // set `allow_file_or_dir` leaf to false,
    // this means that ./allowed_dir *can't* be matched with a dir.
    // that is, the setting says "no trailing `/` means file."
    let settings = MatchSettings {
        allow_file_or_dir_leaf: false,
        default_severity: Severity::Warning,
    };

    let report = check_tree(root, &settings).unwrap();

    assert!(report.is_allowed("/allowed/"));
    assert!(report.is_unaccounted("./allowed_dir"));
    assert!(report.is_allowed("a/b/c/unanchored_allowed"));
    assert!(report.is_unaccounted("d/e/f/unanchored_allowed_dir"));
}
