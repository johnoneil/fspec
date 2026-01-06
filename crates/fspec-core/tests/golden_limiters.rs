use std::fs;
use std::path::Path;

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
fn golden_allow_anchored_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".fspec"),
        r#"
allow ./{name:snake_case}_{name}_{year:int(4)}.{snake|SNAKE}
allow ./{name:pascal_case}_{name}_{:int(4)}.{pascal|PASCAL}
allow ./{name:upper_case}_{ name }_{ : int(4)}.{upper|UPPER}
allow ./{anonymous}.txt
allow ./{lower_case}.lower
allow ./{kebab_case}.kebab
allow ./{letters}.letters
allow ./{numbers}.numbers
allow ./{alnum}.alnum
"#,
    );

    write_file(
        &root.join("snaked_name_snaked_name_1999.snake"),
        "dummy_file",
    );
    write_file(
        &root.join("snaked_name_snaked_name_1999.SNAKE"),
        "dummy_file",
    );
    write_file(
        &root.join("PascalName_PascalName_1999.pascal"),
        "dummy_file",
    );
    write_file(
        &root.join("PascalName_PascalName_1999.PASCAL"),
        "dummy_file",
    );
    write_file(&root.join("UPPERNAME_UPPERNAME_1999.upper"), "dummy_file");
    write_file(&root.join("UPPERNAME_UPPERNAME_1999.UPPER"), "dummy_file");
    write_file(&root.join("lowername.lower"), "dummy_file");
    write_file(&root.join("kebab-name.kebab"), "dummy_file");
    write_file(
        &root.join("abcdefghijklmnopqrstuvwxyz.letters"),
        "dummy_file",
    );
    write_file(&root.join("1234567890.numbers"), "dummy_file");
    write_file(&root.join("abc123.alnum"), "dummy_file");

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("snaked_name_snaked_name_1999.snake"));
    assert!(report.is_allowed("snaked_name_snaked_name_1999.SNAKE"));
    assert!(report.is_allowed("PascalName_PascalName_1999.pascal"));
    assert!(report.is_allowed("PascalName_PascalName_1999.PASCAL"));
    assert!(report.is_allowed("UPPERNAME_UPPERNAME_1999.upper"));
    assert!(report.is_allowed("UPPERNAME_UPPERNAME_1999.UPPER"));
    assert!(report.is_allowed("lowername.lower"));
    assert!(report.is_allowed("kebab-name.kebab"));
    assert!(report.is_allowed("abcdefghijklmnopqrstuvwxyz.letters"));
    assert!(report.is_allowed("1234567890.numbers"));
    assert!(report.is_allowed("abc123.alnum"));
}
