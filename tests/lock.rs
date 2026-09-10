//! `plotplot lock verify`, driven as the binary a planter runs.
//!
//! The unit tests beside `src/lock.rs` inject the network and prove what happens to bytes.
//! What this file proves is the face: the exit code a caller reads, the line it prints, and
//! that a lock the binary cannot satisfy leaves nothing behind under `.plotplot/bin/`.
//!
//! Nothing here reaches a real host. The one url that is fetched at all is on `.invalid`,
//! the TLD reserved for exactly this, so the fetch fails as a fetch rather than as a test
//! that needed a network.

use std::path::Path;

use assert_cmd::Command;

use plotplot::{layout, lock};

/// A digest that is 64 lowercase hex, so the contracts accept the lock, and that no bytes
/// this test can reach actually have.
const PINNED: &str = "38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55";
/// The digest a stale companion carries: as well-formed as the one above, and not it.
const STALE: &str = "d054263a201ed924d809582b0d13c3eae3e34b65baad941acba44386a913b9e6";
/// A host that does not resolve, ever, by RFC 2606.
const UNREACHABLE: &str = "https://plotplot-fit.invalid/tilth-artifact.tar.gz";

fn plotplot(root: &Path) -> Command {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(root);
    command
}

/// A lock pinning one judge for the platform this binary was built for.
fn lock_for(platform: &str, url: &str, sha256: &str) -> String {
    format!(
        "season = \"2026.09\"\n\n\
         [judges.tilth]\n\
         version = \"0.10.1\"\n\n\
         [judges.tilth.platforms.\"{platform}\"]\n\
         url = \"{url}\"\n\
         sha256 = \"{sha256}\"\n"
    )
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the parent directory");
    }
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

/// A repository carrying nothing but the lock this test wrote.
fn with_lock(toml: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("a temporary root");
    write(&layout::garden_lock(root.path()), toml);
    root
}

/// Every file under `.plotplot/bin/`, sorted; empty when there is no such directory.
fn placed(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(layout::bin_dir(root)) {
        Ok(entries) => entries
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

#[test]
fn a_url_that_gives_up_no_bytes_answers_one_and_names_the_url() {
    let root = with_lock(&lock_for(lock::platform(), UNREACHABLE, PINNED));

    let assert = plotplot(root.path()).args(["lock", "verify"]).assert();
    let output = assert.get_output().clone();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains(UNREACHABLE), "{said}");
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(
        placed(root.path()).is_empty(),
        "a judge that never arrived was placed: {:?}",
        placed(root.path())
    );
}

#[test]
fn a_companion_the_lock_disagrees_with_is_a_lock_that_moved_and_the_bytes_are_fetched() {
    let root = with_lock(&lock_for(lock::platform(), UNREACHABLE, PINNED));
    write(
        &layout::judge_binary(root.path(), "tilth"),
        "an old judge\n",
    );
    write(
        &layout::judge_digest(root.path(), "tilth"),
        &format!("{STALE}\n"),
    );

    let assert = plotplot(root.path()).args(["lock", "verify"]).assert();
    let output = assert.get_output().clone();
    // The record beside the judge names other bytes than the lock, so the lock moved and
    // the bytes it now names are fetched; the url is unreachable, which is an error naming
    // it, not a verdict, and the old judge stays until real bytes replace it.
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(UNREACHABLE), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(layout::judge_binary(root.path(), "tilth")).expect("the judge"),
        "an old judge\n"
    );
    assert_eq!(
        std::fs::read_to_string(layout::judge_digest(root.path(), "tilth"))
            .expect("the companion")
            .trim(),
        STALE
    );
}

#[test]
fn a_platform_the_lock_does_not_cover_answers_three_and_names_the_platform() {
    // A triple the schema accepts and no machine this suite runs on is built for.
    let root = with_lock(&lock_for("sparc64-unknown-linux-gnu", UNREACHABLE, PINNED));

    let assert = plotplot(root.path()).args(["lock", "verify"]).assert();
    let output = assert.get_output().clone();
    assert_eq!(
        output.status.code(),
        Some(3),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(printed.starts_with("tilth 0.10.1 missing: "), "{printed}");
    assert!(printed.contains(lock::platform()), "{printed}");
    assert!(placed(root.path()).is_empty());
}

#[test]
fn a_repository_with_no_lock_answers_one_and_says_which_file_is_absent() {
    let root = tempfile::tempdir().expect("a temporary root");

    let assert = plotplot(root.path()).args(["lock", "verify"]).assert();
    let output = assert.get_output().clone();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains(layout::GARDEN_LOCK), "{said}");
}

#[test]
fn a_lock_the_contracts_refuse_answers_one_and_names_the_schema() {
    let root = with_lock("season = \"2026.09\"\n\n[judges.tilth]\nversion = \"0.10.1\"\n");

    let assert = plotplot(root.path()).args(["lock", "verify"]).assert();
    let output = assert.get_output().clone();
    assert_eq!(output.status.code(), Some(1));
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains(lock::SCHEMA_REFUSED), "{said}");
}
