use std::fs;
use std::path::Path;

use fspec_core::{Severity, check_tree};

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[ignore = "enable when alternate syntaxes implemented."]
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
ignore    eee.file
- /ggg.file
-    /hhh.file
-/iii.file
- jjj.file
-    kkk.file
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

    assert!(report.is_ignored("aaa.file"));
    assert!(report.is_ignored("bbb.file"));
    assert!(report.is_ignored("ccc.file"));
    assert!(report.is_ignored("ddd.file"));
    assert!(report.is_ignored("a/b/c/ddd.file"));
    assert!(report.is_ignored("eee.file"));
    assert!(report.is_ignored("e/f/g/eee.file"));
    assert!(report.is_ignored("fff.file"));
    assert!(report.is_ignored("h/i/j/fff.file"));
    assert!(report.is_ignored("ggg.file"));
    assert!(report.is_ignored("hhh.file"));
    assert!(report.is_ignored("iii.file"));
    assert!(report.is_ignored("jjj.file"));
    assert!(report.is_ignored("k/l/m/jjj.file"));
    assert!(report.is_ignored("kkk.file"));
    assert!(report.is_ignored("n/o/p/kkk.file"));
    assert!(report.is_ignored("lll.file"));
    assert!(report.is_ignored("q/r/s/lll.file"));

    // Nothing else should be flagged.
    assert!(
        report.unaccounted_paths().is_empty(),
        "unexpected unaccounted paths"
    );
}
