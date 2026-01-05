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
fn golden_allow_anchored_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /allowed.txt
allow /all/ancestors/are/also.allowed.txt
"#,
    );

    write_file(&root.join("allowed.txt"), "dummy_file");
    write_file(
        &root.join("all/ancestors/are/also.allowed.txt"),
        "dummy_file",
    );
    write_file(&root.join("unaccounted.txt"), "dummy_file");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("/allowed.txt"));

    assert!(report.is_allowed("/all"));
    assert!(report.is_allowed("/all/ancestors"));
    assert!(report.is_allowed("/all/ancestors/are"));
    assert!(report.is_allowed("/all/ancestors/are/also.allowed.txt"));

    assert!(report.is_unaccounted("/unaccounted.txt"));
}
