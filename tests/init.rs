//! `plotplot init`, driven as the binary a planter runs.
//!
//! The fixture is an empty git repository, a lock template naming two judges, and weeder
//! already placed under `.plotplot/bin/` with the digest the template pins beside it, which
//! is exactly what a previous `lock verify` leaves behind. `init` therefore resolves the
//! lock without a network, and every assertion below is about what `init` itself writes.
//!
//! Claude's own install runs the `claude` binary, so it is proved by `scripts/fit/stem.sh
//! init` against a temporary home rather than here; these tests plant Gemini and Codex,
//! whose project-scope installs are file writes the stem makes itself.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use sha2::{Digest, Sha256};

use plotplot::harness::Harness;
use plotplot::{layout, lock};

/// The season the fixture template pins.
const SEASON: &str = "2026.09";

/// The digest the template pins for weeder, and the digest the fixture records beside the
/// judge it places, so `lock verify` answers `verified` and fetches nothing.
const WEEDER_SHA: &str = "11a1cdbb1f2a4d2ff53f3f0d2ae0ff9d1a2c4f81ec4d5ba9b30f5a4c9e17d2b6";

// ------------------------------------------------------------------------ the fixture

fn contracts(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join(relative)
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

fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .permissions()
        .mode()
        & 0o7777
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

fn git_config(root: &Path, key: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "--local", "--get-all", key])
        .output()
        .expect("git runs on the build machine");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A repository with a git identity, a lock template beside it, and weeder placed the way a
/// resolved lock leaves it. Returns the repository root and the template's path.
fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp directory");
    let root = tmp.path().join("repo");
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&root).expect("the repository directory");
    std::fs::create_dir_all(&home).expect("the temporary home");

    git(tmp.path(), &["init", "-q", "repo"]);
    git(&root, &["config", "user.email", "fixture@plotplot.invalid"]);
    git(&root, &["config", "user.name", "plotplot fixture"]);
    git(
        &root,
        &["remote", "add", "origin", "https://example.invalid/o/r.git"],
    );

    // What a resolved lock leaves behind: the manifest read out of the verified artifact,
    // the bed's skill file, the executable, and the digest of the archive it came out of.
    let manifest = contracts("fixtures/manifest/weeder.garden.json");
    write(&layout::bed_manifest(&root, "weeder"), &read(&manifest));
    write(
        &layout::bed_artifact(&root, "weeder").join("SKILL.md"),
        "# weeder\n\nThe judge of the diff.\n",
    );
    let judge = layout::judge_binary(&root, "weeder");
    write(&judge, "#!/bin/sh\ncat >/dev/null\nexit 0\n");
    make_executable(&judge);
    write(
        &layout::judge_digest(&root, "weeder"),
        &format!("{WEEDER_SHA}\n"),
    );

    let template = tmp.path().join("template.lock");
    write(&template, &template_lock(WEEDER_SHA));

    (tmp, root, template)
}

/// A lock template naming two judges: weeder, pinned per platform, and tend2, pinned through
/// npm. The minimal profile keeps weeder and drops tend2, which is what makes the filter
/// visible in the written `garden.lock`.
fn template_lock(weeder_sha: &str) -> String {
    format!(
        "{}\n[judges.tend2]\nversion = \"1.0.0\"\nnpm = \"@plotplot/tend2\"\n",
        weeder_lock(weeder_sha)
    )
}

/// The same lock with weeder alone: what the minimal profile writes, and what a repository
/// that already pins its judges carries.
fn weeder_lock(weeder_sha: &str) -> String {
    format!(
        "season = \"{SEASON}\"\n\
         \n\
         [judges.weeder]\n\
         version = \"0.1.0\"\n\
         \n\
         [judges.weeder.platforms.\"{platform}\"]\n\
         url = \"https://example.invalid/weeder-{platform}.tar.gz\"\n\
         sha256 = \"{weeder_sha}\"\n",
        platform = lock::platform()
    )
}

fn init(root: &Path, home: &Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(root);
    command.env("HOME", home);
    command.arg("init");
    command.args(args);
    command.output().expect("the binary runs")
}

/// Every file under `root`, path to sha256, so two runs can be compared byte for byte.
fn tree_digest(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

fn collect(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // The git directory is git's own bookkeeping and changes on its own account.
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        let kind = entry.metadata().expect("a readable entry");
        if kind.is_dir() {
            collect(root, &path, files);
        } else {
            let bytes = std::fs::read(&path).expect("a readable file");
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let relative = path.strip_prefix(root).expect("a path under the root");
            files.insert(relative.to_path_buf(), hex::encode(hasher.finalize()));
        }
    }
}

/// The fixture planted once with Gemini and Codex, ready for a second run.
fn planted() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini,codex",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(
        output.status.success(),
        "init exited {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (tmp, root, template, home)
}

// ------------------------------------------------------- 1. detection and --harness

#[test]
fn the_harness_flag_limits_what_is_installed_to_the_harnesses_it_names() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    assert!(root.join(".gemini/settings.json").is_file());
    assert!(
        !root.join(".codex").exists(),
        "codex was not asked for and must not be planted"
    );
    assert!(
        !root.join(".claude").exists(),
        "claude was not asked for and must not be planted"
    );
    assert!(layout::bundle_dir(&root, Harness::Gemini).is_dir());
    assert!(
        !layout::bundle_dir(&root, Harness::Codex).exists(),
        "no bundle is generated for a harness nobody asked for"
    );
}

// ------------------------------------------------------------------- 2. the profile

#[test]
fn the_minimal_profile_plants_weeder_and_drops_every_other_judge_in_the_template() {
    let (_tmp, root, _template, _home) = planted();

    let written = lock::read_lock(&root)
        .expect("the written lock parses")
        .expect("init wrote a lock");
    assert_eq!(
        written.judges.keys().collect::<Vec<_>>(),
        vec!["weeder"],
        "the minimal profile keeps weeder alone"
    );
    assert_eq!(written.season, SEASON);
}

#[test]
fn the_beds_flag_overrides_the_profiles_bed_list() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    // `--profile full` alone would keep tend2 too; `--beds` is what cuts it back to weeder.
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--profile",
            "full",
            "--beds",
            "weeder",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let written = lock::read_lock(&root)
        .expect("the written lock parses")
        .expect("init wrote a lock");
    assert_eq!(written.judges.keys().collect::<Vec<_>>(), vec!["weeder"]);
}

#[test]
fn a_bed_the_template_does_not_pin_is_refused_and_nothing_is_planted() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--beds",
            "petals",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("petals: ") || stderr.contains("\npetals: "),
        "{stderr}"
    );
    assert!(!layout::garden_lock(&root).exists());
}

// ---------------------------------------------------------------------- 3. the lock

#[test]
fn a_missing_lock_flag_with_no_garden_lock_is_a_usage_error_naming_the_flag() {
    let (tmp, root, _template) = fixture();
    let home = tmp.path().join("home");
    let output = init(&root, &home, &["--harness", "gemini"]);

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--lock"), "{stderr}");
    assert!(
        !layout::garden_lock(&root).exists(),
        "a usage error writes nothing"
    );
}

#[test]
fn a_lock_that_moved_to_bytes_the_stem_cannot_fetch_aborts_before_any_bundle_is_written() {
    let (tmp, root, _template) = fixture();
    let home = tmp.path().join("home");
    // The lock names other bytes than the record beside the placed judge: the lock moved, so
    // the stem fetches what it now names, and the fixture's remote does not exist.
    let bent = format!("00{}", &WEEDER_SHA[2..]);
    assert_ne!(bent, WEEDER_SHA);
    write(&layout::garden_lock(&root), &weeder_lock(&bent));

    let output = init(&root, &home, &["--harness", "gemini"]);

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("example.invalid"), "{stderr}");
    assert!(
        !layout::bundles_dir(&root).exists(),
        "a refused lock must leave no bundle behind"
    );
    assert!(!root.join(".gemini").exists());
    assert!(!layout::githooks_dir(&root).exists());
}

#[test]
fn an_existing_lock_is_verified_and_never_replaced_by_the_template() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    // A lock the repository already carries, naming weeder alone at a different version.
    let existing = weeder_lock(WEEDER_SHA).replace("0.1.0", "0.2.0");
    write(&layout::garden_lock(&root), &existing);

    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        read(&layout::garden_lock(&root)),
        existing,
        "an existing lock is the repository's, and init does not rewrite it"
    );
}

// ------------------------------------------------------------------- 4. the bundles

#[test]
fn a_bundle_is_generated_for_every_chosen_harness_and_carries_the_stem_binary() {
    let (_tmp, root, _template, _home) = planted();

    for harness in [Harness::Gemini, Harness::Codex] {
        let bundle = layout::bundle_dir(&root, harness);
        assert!(bundle.is_dir(), "{harness}: no bundle");
        assert!(
            bundle.join("bin/plotplot").is_file(),
            "{harness}: no bin/plotplot"
        );
        assert_eq!(mode(&bundle.join("bin/plotplot")) & 0o111, 0o111);
    }
    assert!(!layout::bundle_dir(&root, Harness::Claude).exists());
}

// ------------------------------------------------------------------- 5. the installs

#[test]
fn gemini_gets_the_hooks_and_servers_in_the_projects_own_settings_file() {
    let (_tmp, root, _template, _home) = planted();

    let settings: serde_json::Value =
        serde_json::from_str(&read(&root.join(".gemini/settings.json")))
            .expect("the settings file is JSON");
    let hooks = settings
        .get("hooks")
        .and_then(serde_json::Value::as_object)
        .expect("gemini's settings carry the hooks");
    assert!(hooks.contains_key("BeforeTool"), "{hooks:?}");
    assert!(hooks.contains_key("SessionEnd"), "{hooks:?}");

    let command = hooks["BeforeTool"][0]["hooks"][0]["command"]
        .as_str()
        .expect("a command string");
    assert!(
        command.contains("$GEMINI_PROJECT_DIR"),
        "a project-scope hook resolves through gemini's own project variable: {command}"
    );
    assert!(command.contains("hook gemini BeforeTool"), "{command}");
}

#[test]
fn gemini_settings_keep_every_key_the_planter_already_had() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    write(
        &root.join(".gemini/settings.json"),
        "{\n  \"model\": { \"name\": \"gemini-3-pro\" },\n  \"hooks\": {}\n}\n",
    );

    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let settings: serde_json::Value =
        serde_json::from_str(&read(&root.join(".gemini/settings.json")))
            .expect("the settings file is JSON");
    assert_eq!(
        settings["model"]["name"].as_str(),
        Some("gemini-3-pro"),
        "init touches hooks and mcpServers and nothing else"
    );
    assert!(settings["hooks"].as_object().is_some_and(|h| !h.is_empty()));
}

#[test]
fn codex_gets_the_hook_file_and_the_servers_in_the_projects_own_codex_directory() {
    let (_tmp, root, _template, _home) = planted();

    let hooks: serde_json::Value = serde_json::from_str(&read(&root.join(".codex/hooks.json")))
        .expect("codex's project hook file is JSON");
    let events = hooks["hooks"]
        .as_object()
        .expect("codex's hook file carries its events");
    assert!(events.contains_key("PreToolUse"), "{events:?}");
    assert!(events.contains_key("Stop"), "{events:?}");
    let command = events["PreToolUse"][0]["hooks"][0]["command"]
        .as_str()
        .expect("a command string");
    assert!(
        command.contains("hook codex PreToolUse"),
        "every entry is one dispatcher call: {command}"
    );
    assert!(
        !command.contains("${CLAUDE_PLUGIN_ROOT}"),
        "a project-scope entry has no plugin root to expand: {command}"
    );
}

#[test]
fn codex_is_told_that_its_hooks_wait_on_project_trust() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "codex",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("trust"),
        "init says codex loads project hooks only once the project is trusted: {stderr}"
    );
}

// ---------------------------------------------------- 6. the git hooks and git config

#[test]
fn the_five_git_hooks_are_written_executable_and_git_is_pointed_at_them() {
    let (_tmp, root, _template, _home) = planted();

    for hook in plotplot::bed::GitHook::ALL {
        let path = layout::git_hook(&root, hook);
        assert!(path.is_file(), "{}", path.display());
        assert_eq!(mode(&path), 0o755, "{}", path.display());
    }
    assert_eq!(
        git_config(&root, "core.hooksPath"),
        vec![layout::GITHOOKS_DIR.to_owned()]
    );
    for key in ["remote.origin.fetch", "remote.origin.push"] {
        assert!(
            git_config(&root, key)
                .iter()
                .all(|value| !value.contains("refs/notes/plotplot/receipts")),
            "{key} carries the receipts ref: a fetch refspec for an absent ref fails every fetch, a push refspec sends the ref alone"
        );
    }
}

// ------------------------------------------- 7. the garden block and the own manifest

#[test]
fn the_garden_block_lands_in_agents_md_and_names_the_season_and_the_beds() {
    let (_tmp, root, _template, _home) = planted();

    let agents = read(&layout::agents_md(&root));
    assert!(
        agents.contains(plotplot::plant::garden_block::BEGIN),
        "{agents}"
    );
    assert!(
        agents.contains(plotplot::plant::garden_block::END),
        "{agents}"
    );
    assert!(agents.contains(SEASON), "{agents}");
    assert!(agents.contains("weeder 0.1.0"), "{agents}");
}

#[test]
fn the_planted_repository_gets_its_own_manifest_beside_the_lock() {
    let (_tmp, root, _template, _home) = planted();

    let path = root.join("garden.json");
    assert!(path.is_file());
    let manifest: serde_json::Value =
        serde_json::from_str(&read(&path)).expect("the manifest is JSON");
    assert_eq!(
        manifest["kind"],
        serde_json::json!(["repository"]),
        "the planted repository is not a bed"
    );
    // The fixture's directory is `repo`, its origin remote is `https://example.invalid/o/r.git`,
    // and the remote names the repository.
    assert_eq!(manifest["name"].as_str(), Some("r"));

    plotplot::manifest::validate_repository(&read(&path))
        .expect("the contracts accept the manifest init wrote");
}

#[test]
fn a_manifest_the_repository_already_has_is_left_alone() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let mine = "{\n  \"name\": \"mine\",\n  \"kind\": [\"repository\"]\n}\n";
    write(&root.join("garden.json"), mine);

    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(read(&root.join("garden.json")), mine);
}

// --------------------------------------------------------------- 8. the PR workflow

#[test]
fn the_pull_request_workflow_runs_the_strict_check_on_a_hosted_runner() {
    let (_tmp, root, _template, _home) = planted();

    let workflow = read(&root.join(".github/workflows/plotplot-check.yml"));
    assert!(workflow.contains("pull_request"), "{workflow}");
    assert!(workflow.contains("plotplot check --strict"), "{workflow}");
    assert!(
        !workflow.contains("self-hosted"),
        "a public bed never registers a self-hosted runner: {workflow}"
    );
    assert!(workflow.contains("runs-on: ubuntu-latest"), "{workflow}");
}

// ------------------------------------------------------- 9. what it says, and twice

#[test]
fn the_first_run_names_every_file_and_setting_it_changed() {
    let (tmp, root, template) = fixture();
    let home = tmp.path().join("home");
    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini,codex",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "garden.lock",
        "garden.json",
        "AGENTS.md",
        ".githooks/pre-commit",
        "core.hooksPath",
        ".gemini/settings.json",
        ".codex/hooks.json",
        ".github/workflows/plotplot-check.yml",
    ] {
        assert!(
            stdout.lines().any(|line| line.contains(expected)),
            "init did not report {expected}:\n{stdout}"
        );
    }
    assert!(!stdout.contains("nothing to do"), "{stdout}");
}

#[test]
fn a_second_run_changes_nothing_and_says_so() {
    let (_tmp, root, template, home) = planted();
    let before = tree_digest(&root);

    let output = init(
        &root,
        &home,
        &[
            "--harness",
            "gemini,codex",
            "--lock",
            &template.display().to_string(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "nothing to do"
    );
    assert_eq!(
        tree_digest(&root),
        before,
        "a second run left the tree byte for byte as it was"
    );
}
