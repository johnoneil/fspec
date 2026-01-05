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
fn golden_repeated_tag_equality() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // TODO: support named "option list" like {ext:mp4|mkv}
    write_file(
        &root.join(".fspec"),
        r#"
allow /movies/{year:int(4)}/{tag:snake_case}_{year}.{ext:mp4|mkv}
"#,
    );

    // matches: year directory == year suffix
    write_file(
        &root.join("movies/1946/its_a_wonderful_life_1946.mp4"),
        "video",
    );

    // does NOT match: directory year != filename year
    write_file(
        &root.join("movies/1946/its_a_wonderful_life_1947.mp4"),
        "video",
    );

    let report = check_tree(root, &MatchSettings::default()).unwrap();

    assert!(report.is_allowed("movies/1946/its_a_wonderful_life_1946.mp4"));
    assert!(report.is_unaccounted("movies/1946/its_a_wonderful_life_1947.mp4"));
}
