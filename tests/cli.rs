use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_output_stdout() {
    Command::cargo_bin("ygg")
        .unwrap()
        .args(&["--show", "rs", "--only", "src"])
        .assert()
        .success()
        .stdout(contains("📄 Files"));
}

#[test]
fn test_markdown_output_to_file() {
    let tmpdir = tempfile::tempdir().unwrap();
    let outfile = tmpdir.path().join("output.md");

    Command::cargo_bin("ygg")
        .unwrap()
        .args(&["--show", "rs", "--only", "src", "--contents", "--out"])
        .arg(&outfile)
        .assert()
        .success();

    let contents = fs::read_to_string(&outfile).unwrap();
    assert!(contents.contains("# ✨ Project Snapshot"));
    assert!(contents.contains("## 📄 Files"));
    assert!(contents.contains("## 📑 File Contents"));
}

#[test]
fn test_against_fixture() {
    let fixture_path = Path::new("tests/fixtures/expected_show.txt");
    let tmpdir = tempfile::tempdir().unwrap();
    let outfile = tmpdir.path().join("out.txt"); // CLI formatter mode

    // Run CLI
    Command::cargo_bin("ygg")
        .unwrap()
        .args(&["--show", "rs", "--only", "src", "--no-lines", "--out"])
        .arg(&outfile)
        .assert()
        .success();

    let got = fs::read_to_string(&outfile).unwrap();

    // If fixture is missing, create it
    if !fixture_path.exists() {
        fs::create_dir_all(fixture_path.parent().unwrap()).unwrap();
        fs::write(fixture_path, &got).unwrap();
        panic!("Fixture was missing. Created at {:?}. Re-run tests.", fixture_path);
    }

    let expected = fs::read_to_string(fixture_path).unwrap();

    assert!(
        got.contains(&expected),
        "Expected fixture content not found in output.\n--- Expected snippet ---\n{}\n--- Got ---\n{}",
        expected,
        got
    );
}
