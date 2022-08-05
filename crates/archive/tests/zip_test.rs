use moon_archive::{unzip, zip};
use moon_utils::test::create_sandbox;
use std::fs;
use std::path::Path;

fn file_contents_match(a: &Path, b: &Path) -> bool {
    fs::read_to_string(a).unwrap() == fs::read_to_string(b).unwrap()
}

#[test]
fn zips_file() {
    let fixture = create_sandbox("archives");

    // Pack
    let input = fixture.path().join("file.txt");
    let archive = fixture.path().join("out.zip");

    zip(&input, &archive, None).unwrap();

    assert!(archive.exists());
    assert_ne!(archive.metadata().unwrap().len(), 0);

    // Unpack
    let output = fixture.path().join("out");

    unzip(&archive, &output, None).unwrap();

    moon_utils::test::debug_sandbox_files(fixture.path());

    assert!(output.exists());
    assert!(output.join("file.txt").exists());

    // Compare
    assert!(file_contents_match(&input, &output.join("file.txt")));
}

#[test]
fn zips_file_with_prefix() {
    let fixture = create_sandbox("archives");

    // Pack
    let input = fixture.path().join("file.txt");
    let archive = fixture.path().join("out.zip");

    zip(&input, &archive, Some("some/prefix")).unwrap();

    assert!(archive.exists());
    assert_ne!(archive.metadata().unwrap().len(), 0);

    // Unpack
    let output = fixture.path().join("out");

    unzip(&archive, &output, None).unwrap();

    assert!(output.exists());
    assert!(output.join("some/prefix/file.txt").exists());

    // Compare
    assert!(file_contents_match(
        &input,
        &output.join("some/prefix/file.txt")
    ));
}

#[test]
fn zips_file_with_prefix_thats_removed() {
    let fixture = create_sandbox("archives");

    // Pack
    let input = fixture.path().join("file.txt");
    let archive = fixture.path().join("out.zip");

    zip(&input, &archive, Some("some/prefix")).unwrap();

    assert!(archive.exists());
    assert_ne!(archive.metadata().unwrap().len(), 0);

    // Unpack
    let output = fixture.path().join("out");

    unzip(&archive, &output, Some("some/prefix")).unwrap();

    assert!(output.exists());
    assert!(output.join("file.txt").exists());

    // Compare
    assert!(file_contents_match(&input, &output.join("file.txt")));
}
