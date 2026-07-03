use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn shows_help_with_brand_when_help_requested() {
    // Given
    let mut command = Command::cargo_bin("rocv").expect("binary should build");

    // When / Then
    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RipCanvas"))
        .stdout(predicate::str::contains(
            "A fast Rust viewer for Obsidian Canvas",
        ));
}

#[test]
fn rejects_missing_canvas_path_when_path_does_not_exist() {
    // Given
    let mut command = Command::cargo_bin("rocv").expect("binary should build");

    // When / Then
    command
        .arg("tests/fixtures/does-not-exist.canvas")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Canvas file not found"));
}

#[test]
fn rejects_non_canvas_path_when_extension_is_wrong() {
    // Given
    let mut command = Command::cargo_bin("rocv").expect("binary should build");

    // When / Then
    command
        .arg("Cargo.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Expected a .canvas file"));
}

#[test]
fn exports_canvas_to_png_when_export_path_is_provided() {
    // Given
    let temp = tempdir().expect("temp dir should be created");
    let output = temp.path().join("basic.png");
    let mut command = Command::cargo_bin("rocv").expect("binary should build");

    // When / Then
    command
        .arg("--export")
        .arg(&output)
        .arg("tests/fixtures/basic.canvas")
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    assert!(output.exists());
    assert!(std::fs::metadata(output).expect("png should exist").len() > 0);
}

#[test]
fn rejects_export_without_canvas_path() {
    // Given
    let mut command = Command::cargo_bin("rocv").expect("binary should build");

    // When / Then
    command
        .arg("--export")
        .arg("out.png")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Export requires a .canvas file path",
        ));
}
