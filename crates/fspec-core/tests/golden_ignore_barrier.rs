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
    // Layout:
    //   root/
    //     .fspec
    //     src/main.rs
    //     bin/tool.rs
    //     bin/junk.rs
    //     bin/notes.txt
    //
    // Spec:
    //   ignore /bin/
    //   allow {snake_case}.rs (unanchored; must NOT pierce ignored subtree)
    //   allow /bin/tool.rs (rooted; may re-allow inside ignored)
    //   allow /src/**/{snake_case}.rs

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // .fspec: explicit prefixes (per your latest decision)
    write_file(
        &root.join(".fspec"),
        r#"
ignore /bin/
allow {tag:snake_case}.rs
allow /bin/tool.rs
allow /src/**/{tag:snake_case}.rs
"#,
    );

    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(&root.join("bin/tool.rs"), "pub fn tool() {}\n");
    write_file(&root.join("bin/junk.rs"), "pub fn junk() {}\n");
    write_file(&root.join("bin/notes.txt"), "hello\n");

    // Run check
    let report = check_tree(root, Severity::Error).unwrap();

    // Expectations:
    // - src/main.rs allowed by rooted src rule
    // - /bin/ ignored, but /bin/tool.rs explicitly re-allowed
    // - /bin/junk.rs should NOT be re-allowed by the unanchored allow (barrier)
    // - /bin/notes.txt ignored due to /bin/ ignore
    //
    // - We also expect a warning about "re-allowed inside ignored subtree" for bin/tool.rs
    //   (depending on how you represent diagnostics)

    assert!(report.is_allowed("src/main.rs"));
    assert!(report.is_allowed("bin/tool.rs"));

    assert!(report.is_ignored("bin")); // directory itself
    assert!(report.is_ignored("bin/notes.txt"));

    // junk.rs should be unaccounted, *not* allowed, because unanchored allow doesn't pierce ignore
    assert!(report.is_unaccounted("bin/junk.rs"));

    // Optional: ensure the "re-allowed under ignore" warning exists
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.code == "reallowed_under_ignore" && d.path == "bin/tool.rs")
    );
}
