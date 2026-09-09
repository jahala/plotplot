//! `plotplot version`, driven as the binary a hook would call.

use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn plotplot(cwd: &Path) -> Command {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(cwd);
    command
}

fn fixture_lock() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/garden.lock")
}

#[test]
fn version_in_an_unplanted_repository_prints_the_binarys_version_alone() {
    let root = tempfile::tempdir().expect("a temp root");
    plotplot(root.path())
        .arg("version")
        .assert()
        .success()
        .stdout("plotplot 0.1.0\n")
        .stderr("");
}

#[test]
fn version_in_a_planted_repository_prints_the_season_and_every_judge() {
    let root = tempfile::tempdir().expect("a temp root");
    std::fs::copy(fixture_lock(), root.path().join("garden.lock")).expect("the fixture lock");

    plotplot(root.path())
        .arg("version")
        .assert()
        .success()
        .stdout(concat!(
            "plotplot 0.1.0\n",
            "season 2026.09\n",
            "tend2 1.0.0 (npm @plotplot/tend2)\n",
            "tilth 0.10.1\n",
            "weeder 0.1.0\n",
        ))
        .stderr("");
}

#[test]
fn the_version_flag_prints_the_first_line() {
    let root = tempfile::tempdir().expect("a temp root");
    std::fs::copy(fixture_lock(), root.path().join("garden.lock")).expect("the fixture lock");

    plotplot(root.path())
        .arg("--version")
        .assert()
        .success()
        .stdout("plotplot 0.1.0\n");
}

#[test]
fn a_lock_the_stem_cannot_read_fails_with_the_path_first() {
    let root = tempfile::tempdir().expect("a temp root");
    std::fs::write(
        root.path().join("garden.lock"),
        "season = \"2026.09\"\n\n[judges.weeder]\nversion = \"0.1.0\"\n",
    )
    .expect("a lock the contracts refuse");

    let assert = plotplot(root.path()).arg("version").assert().code(1);
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stderr.lines().next().unwrap_or_default();
    let path = first_line.split(": ").next().unwrap_or_default();

    assert!(
        path.ends_with("garden.lock"),
        "the path comes first: {stderr}"
    );
    assert!(
        stderr.contains("lock.schema.json"),
        "the contracts are named as the refuser: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "nothing is claimed on stdout"
    );
}

#[test]
fn a_broken_lock_still_reports_before_any_judge_line() {
    let root = tempfile::tempdir().expect("a temp root");
    std::fs::write(root.path().join("garden.lock"), "season = \n").expect("a broken lock");

    plotplot(root.path()).arg("version").assert().code(1);
}

#[test]
fn no_face_at_all_is_a_usage_error() {
    let root = tempfile::tempdir().expect("a temp root");
    plotplot(root.path()).assert().failure();
}
