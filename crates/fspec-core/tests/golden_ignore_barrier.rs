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
    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("bin/allowed.txt"));
    assert!(report.is_allowed("bin/"));

    assert!(report.is_ignored("bin/ignored.txt"));
}

#[test]
fn ignored_subtree_barrier_and_rooted_reallow_with_dot_slash() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
ignore ./bin/
allow ./bin/allowed.txt
"#,
    );

    write_file(&root.join("bin/allowed.txt"), "dummy_file");
    write_file(&root.join("bin/ignored.txt"), "dummy_file");

    // Run check
    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("bin/allowed.txt"));
    assert!(report.is_allowed("bin/"));

    assert!(report.is_ignored("bin/ignored.txt"));
}

#[test]
fn optional_allow_keyword_find_output_compatibility() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Simulating find output - paths without 'allow' keyword
    write_file(
        &root.join(".fspec"),
        r#"
# This simulates find output - just paths, no 'allow' keyword
./src/main.rs
./src/lib.rs
# But 'ignore' still requires explicit keyword
ignore ./target/
./src/utils.rs
"#,
    );

    write_file(&root.join("src/main.rs"), "fn main() {}");
    write_file(&root.join("src/lib.rs"), "pub fn lib() {}");
    write_file(&root.join("src/utils.rs"), "pub fn utils() {}");
    write_file(&root.join("target/debug/app"), "binary");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    // Paths without 'allow' keyword should default to allowed
    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("src/lib.rs"));
    assert!(report.is_allowed("src/utils.rs"));
    assert!(report.is_allowed("src/"));

    // Explicit ignore should still work
    assert!(report.is_ignored("target/"));
    assert!(report.is_ignored("target/debug"));
}
