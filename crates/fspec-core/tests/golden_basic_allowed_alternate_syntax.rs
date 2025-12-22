use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[ignore = "enable when alternate syntaxes are enabled."]
#[test]
fn golden_basic_all_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow /aaa.file
allow    /bbb.file
allow ddd.file
allow    eee.file
+ /ggg.file
+    /hhh.file
+/iii.file
+ jjj.file
+    kkk.file
"#,
    );

    write_file(&root.join("aaa.file"), "dummy_file\n");
    write_file(&root.join("bbb.file"), "dummy_file\n");
    write_file(&root.join("ddd.file"), "dummy_file\n");
    write_file(&root.join("a/b/c/ddd.file"), "dummy_file\n");
    write_file(&root.join("eee.file"), "dummy_file\n");
    write_file(&root.join("e/f/g/eee.file"), "dummy_file\n");
    write_file(&root.join("ggg.file"), "dummy_file\n");
    write_file(&root.join("hhh.file"), "dummy_file\n");
    write_file(&root.join("iii.file"), "dummy_file\n");
    write_file(&root.join("jjj.file"), "dummy_file\n");
    write_file(&root.join("k/l/m/jjj.file"), "dummy_file\n");
    write_file(&root.join("kkk.file"), "dummy_file\n");
    write_file(&root.join("n/o/p/kkk.file"), "dummy_file\n");

    let report = check_tree(root, Severity::Error).unwrap();

    assert!(report.is_allowed("aaa.file"));
    assert!(report.is_allowed("bbb.file"));
    assert!(report.is_allowed("ddd.file"));
    assert!(report.is_allowed("a/b/c/ddd.file"));
    assert!(report.is_allowed("eee.file"));
    assert!(report.is_allowed("e/f/g/eee.file"));
    assert!(report.is_allowed("ggg.file"));
    assert!(report.is_allowed("hhh.file"));
    assert!(report.is_allowed("iii.file"));
    assert!(report.is_allowed("jjj.file"));
    assert!(report.is_allowed("k/l/m/jjj.file"));
    assert!(report.is_allowed("kkk.file"));
    assert!(report.is_allowed("n/o/p/kkk.file"));

    // Nothing else should be flagged.
    assert!(
        report.unaccounted_paths().is_empty(),
        "unexpected unaccounted paths"
    );
}
