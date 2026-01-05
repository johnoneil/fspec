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
fn golden_unanchored_allow_inside_ignored_tree_and_last_rule_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
# ignore a subtree
ignore /e
# unanchored allow: any file.txt anywhere should be allowed (including inside /e)
allow file.txt
"#,
    );

    write_file(&root.join("e/f/g/file.txt"), "dummy-file");
    write_file(&root.join("e/f/g/other.txt"), "nope");
    write_file(&root.join("x/y/file.txt"), "dummy-file");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    // Unanchored allow should still work inside an ignored subtree, *unless* overridden later.
    assert!(report.is_allowed("e/f/g/file.txt")); // last rule wins
    assert!(report.is_ignored("e/f/g/other.txt")); // optional sanity: it's ignored via /e

    // Outside that subtree, unanchored allow should allow file.txt and promote parents.
    assert!(report.is_allowed("x/y/file.txt"));
    assert!(report.is_allowed("x/y"));
    assert!(report.is_allowed("x"));
}
