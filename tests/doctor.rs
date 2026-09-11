//! `plotplot bundle build` and `plotplot doctor`, driven as the binary a planter runs.
//!
//! The fixture is a repository planted by hand from the same renderers `init` will use, so
//! what `doctor` reads is what the stem writes and not a shape invented here. The beds are
//! the contracts' own manifest fixtures, the lock is the contracts' own `garden.lock`, and
//! the bundles are written by `bundle build` itself.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use sha2::{Digest, Sha256};

use plotplot::bed::{Bed, GitHook};
use plotplot::harness::Harness;
use plotplot::plant::{garden_block, gitconfig, githooks};
use plotplot::{layout, lock, manifest};

/// The two beds the fixture plants: weeder brings hooks, git hooks and a skill at the
/// artifact root; tilth brings a skill one directory down and an MCP face with no launch
/// line, which is the channel `bundle build` must name on stderr.
const BEDS: [&str; 2] = ["tilth", "weeder"];

/// The season the stem's lock fixture (tests/fixtures/garden.lock) pins.
const SEASON: &str = "2026.09";

fn contracts(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join(relative)
}

fn plotplot(cwd: &Path, home: &Path) -> Command {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(cwd);
    command.env("HOME", home);
    command
}

fn git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git runs on the build machine");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the parent directory");
    }
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("the file was written")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("the file can be made executable");
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .permissions()
        .mode()
        & 0o111
        != 0
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Every file under `directory`, relative path and digest, sorted: what "identical bytes"
/// means for a whole tree.
fn fingerprint(directory: &Path) -> Vec<(PathBuf, String)> {
    fn walk(directory: &Path, relative: &Path, found: &mut Vec<(PathBuf, String)>) {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => panic!("{}: {error}", directory.display()),
        };
        for entry in entries {
            let entry = entry.expect("a directory entry");
            let child = relative.join(entry.file_name());
            if entry.path().is_dir() {
                walk(&entry.path(), &child, found);
            } else {
                let bytes = std::fs::read(entry.path()).expect("a file the walk just listed");
                found.push((child, digest(&bytes)));
            }
        }
    }

    let mut found = Vec::new();
    walk(directory, Path::new(""), &mut found);
    found.sort();
    found
}

/// A repository carrying what `bundle build` reads: the lock, the cached bed manifests with
/// their skill files, and the judges the lock pins. No bundles, no hooks, no garden block.
fn seeded() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("a temporary root");
    let path = root.path();

    git(path, &["init", "--quiet"]);
    git(
        path,
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/garden.git",
        ],
    );

    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/garden.lock"),
        layout::garden_lock(path),
    )
    .expect("the stem's lock fixture");

    for bed in BEDS {
        let manifest_path = layout::bed_manifest(path, bed);
        std::fs::create_dir_all(layout::bed_dir(path, bed)).expect("the bed's directory");
        std::fs::copy(
            contracts(&format!("fixtures/manifest/{bed}.garden.json")),
            &manifest_path,
        )
        .unwrap_or_else(|error| panic!("{bed}: {error}"));

        let parsed = manifest::parse_manifest(&read(&manifest_path))
            .unwrap_or_else(|error| panic!("{bed}: {error}"));
        let skill = manifest::to_bed(&parsed)
            .unwrap_or_else(|error| panic!("{bed}: {error}"))
            .skill
            .unwrap_or_else(|| panic!("{bed} declares a skill in its fixture manifest"));
        write(
            &layout::bed_artifact(path, bed).join(skill),
            &format!("---\nname: {bed}\n---\n\nwhat {bed} does.\n"),
        );
    }

    seed_judges(path);
    root
}

/// The judges the lock pins, as `plotplot lock` will fetch them: the binary under
/// `.plotplot/bin/`, and beside it the digest of the archive it came out of.
fn seed_judges(root: &Path) {
    let lock = lock::parse_lock(&read(&layout::garden_lock(root))).expect("the fixture lock");
    for (name, judge) in &lock.judges {
        let binary = layout::judge_binary(root, name);
        write(&binary, &format!("#!/bin/sh\n# {name} {}\n", judge.version));
        make_executable(&binary);
        if let Some((_, artifact)) = lock::artifact_for(judge, lock::platform()) {
            write(
                &binary.with_extension("sha256"),
                &format!("{}\n", artifact.sha256),
            );
        }
    }
}

/// A repository planted end to end: everything `seeded` writes, plus the three bundles from
/// `bundle build`, the five git hooks, the garden block and the git configuration.
///
/// The home directory comes back with it because `doctor` reads Codex's trust from there,
/// and a temporary one is what keeps the build machine's own Codex out of the test.
fn planted() -> (tempfile::TempDir, tempfile::TempDir) {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");
    let path = root.path();

    plotplot(path, home.path())
        .args(["bundle", "build"])
        .assert()
        .success();

    let beds = manifest::load_beds(path).expect("the cached beds");
    for hook in GitHook::ALL {
        let file = layout::git_hook(path, hook);
        write(&file, &githooks::render(hook, &beds));
        make_executable(&file);
    }

    let block = garden_block::render(SEASON, &beds);
    let agents = layout::agents_md(path);
    write(
        &agents,
        &garden_block::replace_in("# the fixture garden\n", &block).expect("a plantable AGENTS.md"),
    );

    gitconfig::apply(path, &gitconfig::desired(path)).expect("git takes the planted entries");

    (root, home)
}

/// A `<home>/.codex/config.toml` that trusts `root`, in Codex's own shape (§5).
fn trust_in_codex(home: &Path, root: &Path) {
    write(
        &home.join(".codex/config.toml"),
        &format!(
            "[projects.\"{}\"]\ntrust_level = \"trusted\"\n",
            root.display()
        ),
    );
}

/// One row of `doctor`'s table: the check, its verdict, and its detail.
fn columns(line: &str) -> (String, String, String) {
    let mut fields = Vec::new();
    let mut rest = line;
    for _ in 0..2 {
        let cut = rest
            .find("  ")
            .unwrap_or_else(|| panic!("three columns in {line:?}"));
        fields.push(rest[..cut].to_owned());
        rest = rest[cut..].trim_start();
    }
    (fields[0].clone(), fields[1].clone(), rest.to_owned())
}

/// Every row of a `doctor` run, and its exit code.
fn doctor(root: &Path, home: &Path) -> (i32, Vec<(String, String, String)>) {
    let output = plotplot(root, home)
        .arg("doctor")
        .output()
        .expect("doctor ran");
    let stdout = String::from_utf8(output.stdout).expect("doctor writes utf-8");
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "doctor said something on stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = stdout.lines().map(columns).collect();
    (output.status.code().expect("doctor exited"), rows)
}

/// One check's row, and a failure naming the whole table when the check is not there at all.
fn row<'a>(rows: &'a [(String, String, String)], check: &str) -> &'a (String, String, String) {
    rows.iter()
        .find(|(name, _, _)| name == check)
        .unwrap_or_else(|| panic!("{check} is not in the table: {rows:#?}"))
}

/// What one check answered: `ok` or `fail`.
fn verdict(rows: &[(String, String, String)], check: &str) -> String {
    row(rows, check).1.clone()
}

/// What one check's answer rests on.
fn detail(rows: &[(String, String, String)], check: &str) -> String {
    row(rows, check).2.clone()
}

/// Assert that exactly `failing` failed, and that every one of the nine checks is present.
fn only_failure(rows: &[(String, String, String)], failing: &str) {
    let names: Vec<&str> = rows.iter().map(|(name, _, _)| name.as_str()).collect();
    assert_eq!(names, plotplot::doctor::CHECKS, "the table lost a check");
    for (name, verdict, detail) in rows {
        let expected = if name == failing { "fail" } else { "ok" };
        assert_eq!(verdict, expected, "{name}: {detail}");
    }
}

// ------------------------------------------------------------------ bundle build

#[test]
fn bundle_build_writes_three_bundles_and_a_second_run_changes_no_bytes() {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");

    plotplot(root.path(), home.path())
        .args(["bundle", "build"])
        .assert()
        .success();

    for harness in Harness::ALL {
        let bundle = layout::bundle_dir(root.path(), harness);
        assert!(bundle.is_dir(), "{harness} has no bundle directory");
        let binary = bundle.join("bin/plotplot");
        assert!(binary.is_file(), "{harness} has no bin/plotplot");
        assert!(
            is_executable(&binary),
            "{harness}'s bin/plotplot is not executable"
        );
    }

    let before = fingerprint(&layout::bundles_dir(root.path()));
    assert!(!before.is_empty(), "the bundles directory is empty");

    plotplot(root.path(), home.path())
        .args(["bundle", "build"])
        .assert()
        .success();

    assert_eq!(
        fingerprint(&layout::bundles_dir(root.path())),
        before,
        "the second run changed bytes"
    );
}

#[test]
fn bundle_build_names_the_channel_it_cannot_plant_on_stderr() {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");

    let output = plotplot(root.path(), home.path())
        .args(["bundle", "build"])
        .output()
        .expect("bundle build ran");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("tilth"),
        "the unplantable channel is not named: {stderr}"
    );
}

#[test]
fn bundle_build_with_one_harness_writes_only_that_bundle() {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");

    plotplot(root.path(), home.path())
        .args(["bundle", "build", "gemini"])
        .assert()
        .success();

    assert!(layout::bundle_dir(root.path(), Harness::Gemini).is_dir());
    assert!(!layout::bundle_dir(root.path(), Harness::Claude).exists());
    assert!(!layout::bundle_dir(root.path(), Harness::Codex).exists());
}

#[test]
fn bundle_build_without_a_lock_names_garden_lock_and_fails() {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");
    std::fs::remove_file(layout::garden_lock(root.path())).expect("the lock is removed");

    let output = plotplot(root.path(), home.path())
        .args(["bundle", "build"])
        .output()
        .expect("bundle build ran");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("garden.lock"), "{stderr}");
    assert!(stderr.contains("season"), "{stderr}");
    assert!(
        !layout::bundles_dir(root.path()).exists(),
        "a bundle was written without a season"
    );
}

// ------------------------------------------------------------------------ doctor

#[test]
fn doctor_on_a_planted_repository_passes_every_check() {
    let (root, home) = planted();
    let (code, rows) = doctor(root.path(), home.path());

    let names: Vec<&str> = rows.iter().map(|(name, _, _)| name.as_str()).collect();
    assert_eq!(names, plotplot::doctor::CHECKS);
    for (name, verdict, detail) in &rows {
        assert_eq!(verdict, "ok", "{name}: {detail}");
    }
    assert_eq!(code, 0);
    assert_eq!(detail(&rows, "codex trust"), "awaiting project trust");
}

#[test]
fn doctor_reports_codex_trust_when_the_home_config_carries_it() {
    let (root, home) = planted();
    trust_in_codex(home.path(), root.path());

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 0);
    assert!(
        detail(&rows, "codex trust").contains("trusted"),
        "{:?}",
        detail(&rows, "codex trust")
    );
    assert_ne!(detail(&rows, "codex trust"), "awaiting project trust");
}

#[test]
fn doctor_fails_on_a_codex_config_it_cannot_read() {
    let (root, home) = planted();
    write(&home.path().join(".codex/config.toml"), "[projects\n");

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "codex trust");
}

#[test]
fn a_changed_byte_in_a_hook_file_fails_the_bundles_check() {
    let (root, home) = planted();
    let hooks = layout::bundle_dir(root.path(), Harness::Claude).join("hooks/hooks.json");
    let edited = read(&hooks).replacen("\"timeout\": 10", "\"timeout\": 11", 1);
    write(&hooks, &edited);

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "bundles");
    assert!(detail(&rows, "bundles").contains("hooks.json"));
}

#[test]
fn a_hook_entry_calling_something_else_fails_the_hook_entries_check() {
    let (root, home) = planted();
    let hooks = layout::bundle_dir(root.path(), Harness::Codex).join("hooks.json");
    // The command is a JSON string, so the quotes around the program are escaped in the file.
    let before = read(&hooks);
    let edited = before.replacen(
        "/bin/plotplot\\\" hook codex",
        "/bin/weeder\\\" hook codex",
        1,
    );
    assert_ne!(edited, before, "the hook command did not read as expected");
    write(&hooks, &edited);

    let (_, rows) = doctor(root.path(), home.path());
    assert_eq!(verdict(&rows, "hook entries"), "fail", "{rows:#?}");
}

/// A bundle that registers nothing would make "every entry calls the dispatcher" true by
/// having no entries, so the check has to refuse an empty hook file rather than pass it.
#[test]
fn a_bundle_registering_no_hooks_fails_the_hook_entries_check() {
    let (root, home) = planted();
    let hooks = layout::bundle_dir(root.path(), Harness::Gemini).join("hooks/hooks.json");
    write(&hooks, "{\n  \"hooks\": {}\n}\n");

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    assert_eq!(verdict(&rows, "hook entries"), "fail", "{rows:#?}");
    assert!(detail(&rows, "hook entries").contains("gemini"));
}

#[test]
fn a_deleted_git_hook_fails_the_git_hooks_check() {
    let (root, home) = planted();
    std::fs::remove_file(layout::git_hook(root.path(), GitHook::PrePush))
        .expect("pre-push is removed");

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "git hooks");
    assert!(detail(&rows, "git hooks").contains("pre-push"));
}

#[test]
fn a_git_hook_without_its_executable_bit_fails_the_git_hooks_check() {
    use std::os::unix::fs::PermissionsExt;

    let (root, home) = planted();
    let hook = layout::git_hook(root.path(), GitHook::PreCommit);
    let mut permissions = std::fs::metadata(&hook).expect("pre-commit").permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&hook, permissions).expect("the bit comes off");

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "git hooks");
    assert!(detail(&rows, "git hooks").contains("pre-commit"));
}

#[test]
fn an_edited_garden_block_fails_the_garden_block_check() {
    let (root, home) = planted();
    let agents = layout::agents_md(root.path());
    let edited = read(&agents).replace("Season: 2026.09.", "Season: 2027.01.");
    write(&agents, &edited);

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "garden block");
}

#[test]
fn an_unset_hooks_path_fails_its_check() {
    let (root, home) = planted();
    git(root.path(), &["config", "--unset", "core.hooksPath"]);

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "core.hooksPath");
}

#[test]
fn a_receipts_refspec_in_the_config_fails_the_receipts_check() {
    let (root, home) = planted();
    git(
        root.path(),
        &[
            "config",
            "--add",
            "remote.origin.fetch",
            "+refs/notes/plotplot/receipts:refs/notes/plotplot/receipts",
        ],
    );

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "receipts ref");
    assert!(detail(&rows, "receipts ref").contains("fails every fetch"));
}

#[test]
fn no_receipt_yet_is_reported_and_is_not_a_fault() {
    let (root, home) = planted();
    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 0);
    assert!(detail(&rows, "receipts ref").contains("no receipt"));
}

#[test]
fn a_judge_whose_recorded_digest_differs_fails_the_judges_check() {
    let (root, home) = planted();
    let recorded = layout::judge_binary(root.path(), "tilth").with_extension("sha256");
    assert!(recorded.is_file(), "the fixture records tilth's digest");
    write(&recorded, &format!("{}\n", "0".repeat(64)));

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "judges");
    assert!(detail(&rows, "judges").contains("tilth"));
}

#[test]
fn a_missing_judge_fails_the_judges_check() {
    let (root, home) = planted();
    std::fs::remove_file(layout::judge_binary(root.path(), "tend2")).expect("tend2 is removed");

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "judges");
    assert!(detail(&rows, "judges").contains("tend2"));
}

#[test]
fn another_binary_in_a_bundle_fails_the_stem_binary_check() {
    let (root, home) = planted();
    let binary = layout::bundle_dir(root.path(), Harness::Gemini).join("bin/plotplot");
    write(&binary, "#!/bin/sh\nexit 0\n");
    make_executable(&binary);

    let (code, rows) = doctor(root.path(), home.path());
    assert_eq!(code, 3);
    only_failure(&rows, "stem binary");
    assert!(detail(&rows, "stem binary").contains("gemini"));
}

#[test]
fn doctor_on_an_unplanted_directory_says_so_and_exits_three() {
    let root = tempfile::tempdir().expect("a temporary root");
    let home = tempfile::tempdir().expect("a temporary home");

    let output = plotplot(root.path(), home.path())
        .arg("doctor")
        .output()
        .expect("doctor ran");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("not planted"), "{stdout}");
    assert!(stdout.contains(layout::GARDEN_LOCK), "{stdout}");
}

#[test]
fn doctor_without_the_cached_beds_says_the_repository_is_not_planted() {
    let root = seeded();
    let home = tempfile::tempdir().expect("a temporary home");
    std::fs::remove_dir_all(layout::beds_dir(root.path())).expect("the beds are removed");

    let output = plotplot(root.path(), home.path())
        .arg("doctor")
        .output()
        .expect("doctor ran");
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.contains("not planted"), "{stdout}");
    assert!(stdout.contains(".plotplot/beds"), "{stdout}");
}

/// The renderers `init` will use are the ones `doctor` checks against, so a fixture built
/// from them and a `doctor` that passes prove each other and nothing else. This asserts the
/// fixture really is what the stem would write, rather than something shaped to pass.
#[test]
fn the_fixture_is_what_the_renderers_produce() {
    let (root, _home) = planted();
    let beds = manifest::load_beds(root.path()).expect("the cached beds");
    assert_eq!(
        beds.iter()
            .map(|bed: &Bed| bed.name.as_str())
            .collect::<Vec<_>>(),
        BEDS
    );
    for hook in GitHook::ALL {
        assert_eq!(
            read(&layout::git_hook(root.path(), hook)),
            githooks::render(hook, &beds)
        );
    }
    assert!(read(&layout::agents_md(root.path())).contains(&garden_block::render(SEASON, &beds)));
}

// ------------------------------------------------------------------- doctor --live

/// A search path holding `git`, which `doctor` needs to read the repository's configuration,
/// and nothing else. What it deliberately does not hold is `umbel`: a probe that started a
/// session would fail to find the driver and say something other than "not planted here".
fn only_git() -> tempfile::TempDir {
    let bin = tempfile::tempdir().expect("a temporary bin directory");
    let git = std::process::Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .expect("a shell that can look for git");
    let git = String::from_utf8_lossy(&git.stdout).trim().to_owned();
    assert!(!git.is_empty(), "git is on the build machine's path");
    std::os::unix::fs::symlink(&git, bin.path().join("git")).expect("a link to git");
    bin
}

/// `doctor --live`'s two tables and its exit code, with a `PATH` that holds no `umbel`.
///
/// Nothing here starts a session: the fixture is planted but nothing is installed at project
/// scope, so live mode has no harness to drive and says so. That the driver is never reached
/// is asserted by emptying `PATH`; a probe that spawned anything would fail to find `umbel`
/// and say something else than "not planted here".
fn doctor_live(root: &Path, home: &Path, args: &[&str]) -> (i32, String, String) {
    let bin = only_git();
    let output = plotplot(root, home)
        .arg("doctor")
        .arg("--live")
        .args(args)
        .env("PATH", bin.path())
        .output()
        .expect("doctor --live ran");
    (
        output.status.code().expect("doctor exited"),
        String::from_utf8(output.stdout).expect("doctor writes utf-8"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn live_mode_prints_the_static_table_first_and_a_line_for_every_harness_after_it() {
    let (root, home) = planted();
    let (code, stdout, stderr) = doctor_live(root.path(), home.path(), &[]);

    let (statics, live) = stdout
        .split_once("\n\n")
        .unwrap_or_else(|| panic!("two tables, one blank line between them: {stdout}"));
    let names: Vec<String> = statics.lines().map(|line| columns(line).0).collect();
    assert_eq!(names, plotplot::doctor::CHECKS, "{stdout}");

    let rows: Vec<(String, String, String)> = live.lines().map(columns).collect();
    assert_eq!(
        rows.iter()
            .map(|(name, _, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["claude session", "gemini session", "codex session"],
        "{stdout}"
    );
    for (name, verdict, detail) in &rows {
        assert_eq!(verdict, "unavailable", "{name}: {detail}");
        assert!(detail.contains("not planted here"), "{name}: {detail}");
    }

    // A vendor that is not planted is not a broken stem.
    assert_eq!(code, 0, "{stdout}{stderr}");
}

#[test]
fn live_mode_drives_only_the_harnesses_asked_for() {
    let (root, home) = planted();
    let (code, stdout, _) = doctor_live(root.path(), home.path(), &["--harness", "codex,claude"]);

    let live = stdout.split_once("\n\n").expect("two tables").1;
    let names: Vec<String> = live.lines().map(|line| columns(line).0).collect();
    assert_eq!(names, ["claude session", "codex session"], "{stdout}");
    assert_eq!(code, 0);
}

#[test]
fn a_project_file_registering_somebody_elses_hook_is_not_planted() {
    let (root, home) = planted();
    write(
        &root.path().join(".codex/hooks.json"),
        r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "/usr/bin/true"}]}]}}"#,
    );

    let (code, stdout, _) = doctor_live(root.path(), home.path(), &["--harness", "codex"]);
    let live = stdout.split_once("\n\n").expect("two tables").1;
    let (name, verdict, detail) = columns(live.lines().next().expect("one live line"));
    assert_eq!(
        (name.as_str(), verdict.as_str()),
        ("codex session", "unavailable")
    );
    assert!(detail.contains("plotplot hook codex"), "{detail}");
    assert_eq!(code, 0);
}

#[test]
fn live_mode_on_an_unplanted_repository_says_so_once_and_drives_nothing() {
    let root = tempfile::tempdir().expect("a temporary root");
    let home = tempfile::tempdir().expect("a temporary home");

    let (code, stdout, _) = doctor_live(root.path(), home.path(), &[]);
    assert_eq!(code, 3);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.starts_with("not planted: "), "{stdout}");
}

// --------------------------------------------------------------- doctor --platform

#[test]
fn the_platform_table_sits_between_the_static_and_the_live_tables() {
    let (root, home) = planted();
    // Only git on the search path: no gh to ask the platform with, and no umbel to drive.
    let (code, stdout, stderr) = doctor_live(root.path(), home.path(), &["--platform"]);

    let tables: Vec<&str> = stdout.split("\n\n").collect();
    assert_eq!(tables.len(), 3, "static, platform, live: {stdout}");
    let names =
        |table: &str| -> Vec<String> { table.lines().map(|line| columns(line).0).collect() };
    assert_eq!(names(tables[0]), plotplot::doctor::CHECKS, "{stdout}");

    let platform: Vec<(String, String, String)> = tables[1].lines().map(columns).collect();
    assert_eq!(
        platform.len(),
        1,
        "one line when the mode is skipped: {stdout}"
    );
    assert_eq!(
        (platform[0].0.as_str(), platform[0].1.as_str()),
        ("platform", "unavailable"),
        "{stdout}"
    );
    assert!(platform[0].2.contains("gh"), "{stdout}");

    assert_eq!(
        names(tables[2]),
        ["claude session", "gemini session", "codex session"],
        "{stdout}"
    );
    // The platform was not read, so it was not proved: 3, never a guess.
    assert_eq!(code, 3, "{stdout}{stderr}");
}

#[test]
fn the_platform_mode_alone_prints_two_tables() {
    let (root, home) = planted();
    let bin = only_git();
    let output = plotplot(root.path(), home.path())
        .args(["doctor", "--platform"])
        .env("PATH", bin.path())
        .output()
        .expect("doctor --platform ran");
    let stdout = String::from_utf8(output.stdout).expect("doctor writes utf-8");
    let (statics, platform) = stdout
        .split_once("\n\n")
        .unwrap_or_else(|| panic!("two tables: {stdout}"));
    assert_eq!(
        statics.lines().count(),
        plotplot::doctor::CHECKS.len(),
        "{stdout}"
    );
    assert_eq!(
        platform, "platform  unavailable  gh is not on PATH\n",
        "{stdout}"
    );
    assert_eq!(output.status.code(), Some(3));
}

/// A gh standing in for GitHub on one fixture repository, `example-owner/example-repo`, whose
/// default branch is `main`. It answers the three read-only calls `doctor --platform` makes, in
/// GitHub's shapes, the workflow only when asked for the file's raw bytes, and any other call
/// with GitHub's 404. Every call's arguments go to `gh.calls` beside it. Shell builtins only,
/// since the search path it sits on holds nothing but git.
const READING_GH: &str = r#"#!/bin/sh
here="${0%/*}"
printf '%s\n' "$*" >> "$here/gh.calls"
case "$*" in
  "api repos/example-owner/example-repo")
    printf '%s\n' '{"id":1,"name":"example-repo","full_name":"example-owner/example-repo","default_branch":"main"}' ;;
  "api repos/example-owner/example-repo/rules/branches/main")
    printf '%s\n' '[{"type":"deletion","ruleset_id":41},{"type":"non_fast_forward","ruleset_id":41},{"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"garden"}],"strict_required_status_checks_policy":false},"ruleset_id":41}]' ;;
  "api -H Accept: application/vnd.github.raw+json repos/example-owner/example-repo/contents/.github/workflows/plotplot-check.yml?ref=main")
    while IFS= read -r line; do printf '%s\n' "$line"; done < "$here/workflow.yml" ;;
  *)
    printf '%s\n' '{"message":"Not Found","status":"404"}'
    printf '%s\n' 'gh: Not Found (HTTP 404)' >&2
    exit 1 ;;
esac
"#;

/// A planted repository whose origin is `example-owner/example-repo` on github.com, and a
/// search path holding git and [`READING_GH`], with `workflow` as what `main` holds.
fn planted_on_github(workflow: &str) -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let (root, home) = planted();
    git(
        root.path(),
        &[
            "config",
            "remote.origin.url",
            "https://github.com/example-owner/example-repo.git",
        ],
    );
    let bin = only_git();
    write(&bin.path().join("workflow.yml"), workflow);
    let gh = bin.path().join("gh");
    write(&gh, READING_GH);
    make_executable(&gh);
    (root, home, bin)
}

/// `doctor --platform`'s exit code and its platform table, one row per line.
fn doctor_platform(root: &Path, home: &Path, bin: &Path) -> (i32, Vec<(String, String, String)>) {
    let output = plotplot(root, home)
        .args(["doctor", "--platform"])
        .env("PATH", bin)
        .output()
        .expect("doctor --platform ran");
    let stdout = String::from_utf8(output.stdout).expect("doctor writes utf-8");
    let (_, platform) = stdout
        .split_once("\n\n")
        .unwrap_or_else(|| panic!("two tables: {stdout}"));
    (
        output.status.code().expect("doctor exited"),
        platform.lines().map(columns).collect(),
    )
}

#[test]
fn the_platform_table_through_gh_is_four_lines_ending_in_the_gate_job() {
    let (root, home, bin) = planted_on_github(&plotplot::init::workflow(false));
    let (code, rows) = doctor_platform(root.path(), home.path(), bin.path());

    let lines: Vec<(&str, &str)> = rows
        .iter()
        .map(|(check, verdict, _)| (check.as_str(), verdict.as_str()))
        .collect();
    assert_eq!(
        lines,
        [
            ("required check", "ok"),
            ("force push", "ok"),
            ("deletion", "ok"),
            ("gate job", "ok"),
        ],
        "{rows:?}"
    );
    assert_eq!(rows[3].2, "main's plotplot-check.yml reports garden");
    // Every static check and every platform line is ok.
    assert_eq!(code, 0, "{rows:?}");

    let calls = read(&bin.path().join("gh.calls"));
    assert_eq!(calls.lines().count(), 3, "{calls}");
    assert!(!calls.contains("-X"), "doctor --platform wrote: {calls}");
}

#[test]
fn a_default_branch_whose_workflow_reports_another_job_fails_the_gate_job_line_alone() {
    let old = plotplot::init::workflow(false)
        .replace("\n  garden:\n", "\n  check:\n")
        .replace("    name: garden\n", "    name: plotplot check --strict\n");
    let (root, home, bin) = planted_on_github(&old);
    let (code, rows) = doctor_platform(root.path(), home.path(), bin.path());

    assert_eq!(rows.len(), 4, "{rows:?}");
    assert!(
        rows[..3].iter().all(|(_, verdict, _)| verdict == "ok"),
        "{rows:?}"
    );
    assert_eq!(
        (rows[3].0.as_str(), rows[3].1.as_str()),
        ("gate job", "fail")
    );
    assert_eq!(
        rows[3].2,
        "main's .github/workflows/plotplot-check.yml names no job \"garden\" \
         (jobs: check, reported as \"plotplot check --strict\")"
    );
    assert_eq!(code, 3);
}
