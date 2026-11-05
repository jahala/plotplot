//! `plotplot check`, driven as the binary a planter runs.
//!
//! The fixture is a planted repository whose beds are manifests in the contracts' own shape
//! and whose gates are scripts under `.plotplot/bin/` that print the contracts' own SARIF
//! fixture. A gate script records the arguments it was called with, so what the stem passed
//! a gate is read from the gate itself rather than asserted about the stem's intentions.
//! Each gate carries the digest record `lock verify` would have left beside it, so a lock
//! can pin it, and a range can be judged only by what the base's lock pins.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serde_json::{Value, json};

use plotplot::{layout, lock, sarif};

fn contracts(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join(relative)
}

/// The contracts' own weeder log, renamed to `tool`, and with every block-level result
/// dropped to a warning when the gate is meant to pass. Two results either way, so a count
/// of results and a count of blocks cannot be confused for each other.
fn sarif_log(tool: &str, blocking: bool) -> Value {
    let text = std::fs::read_to_string(contracts("fixtures/sarif/weeder-check.sarif.json"))
        .expect("the contracts' SARIF fixture");
    let mut log: Value = serde_json::from_str(&text).expect("the fixture is JSON");
    let run = log
        .get_mut("runs")
        .and_then(|runs| runs.get_mut(0))
        .expect("the fixture has one run");
    run["tool"]["driver"]["name"] = json!(tool);
    if !blocking {
        let results = run
            .get_mut("results")
            .and_then(Value::as_array_mut)
            .expect("the fixture's results");
        for result in results {
            if result.get("level").and_then(Value::as_str) == Some("error") {
                result["level"] = json!("warning");
            }
        }
    }
    log
}

/// A manifest in the contracts' shape, declaring a gate the stem can run.
fn manifest(name: &str, binary: &str, check: &str, strict: bool) -> Value {
    let mut manifest = json!({
        "name": name,
        "kind": ["gate"],
        "version": "0.1.0",
        "install": { "cargo": name },
        "faces": { "cli": binary },
        "check": check,
        "metric": "scripts/metric.sh",
        "context": { "upfront_tokens": 0 },
        "coverage": { "languages": [], "kinds": [] }
    });
    if strict {
        manifest["check_strict"] = json!(true);
    }
    manifest
}

/// The digest record a gate carries until a test says otherwise: well-formed, and the gate's
/// own, so two gates never share one.
fn digest_for(binary: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(binary.as_bytes()))
}

/// A lock pinning each named judge at its digest, for the platform this binary was built
/// for, from a host that never resolves: nothing in these tests fetches.
fn lock_pinning(pins: &[(&str, &str)]) -> String {
    let mut toml = "season = \"2026.09\"\n".to_owned();
    for (name, sha256) in pins {
        toml.push_str(&format!(
            "\n[judges.{name}]\nversion = \"0.1.0\"\n\n\
             [judges.{name}.platforms.\"{platform}\"]\n\
             url = \"https://plotplot-fit.invalid/{name}.tar.gz\"\n\
             sha256 = \"{sha256}\"\n",
            platform = lock::platform()
        ));
    }
    toml
}

/// A planted repository with beds and gates, made one call at a time.
struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        Fixture {
            root: tempfile::tempdir().expect("a temporary root"),
        }
    }

    fn path(&self) -> &Path {
        self.root.path()
    }

    /// One bed's `garden.json` under `.plotplot/beds/<name>/`.
    fn bed(&self, name: &str, binary: &str, check: &str, strict: bool) -> &Fixture {
        let path = self
            .path()
            .join(".plotplot/beds")
            .join(name)
            .join("garden.json");
        write(
            &path,
            &serde_json::to_string_pretty(&manifest(name, binary, check, strict))
                .expect("a manifest serialises"),
        );
        self
    }

    /// One gate under `.plotplot/bin/<binary>`: it records its arguments, prints `log` when
    /// there is one, and exits with `code`.
    fn gate(&self, binary: &str, log: Option<&Value>, code: i32) -> &Fixture {
        let argv = self.argv_path(binary);
        let body = match log {
            Some(log) => {
                let sarif = self.path().join(format!(".plotplot/{binary}.sarif.json"));
                write(
                    &sarif,
                    &serde_json::to_string_pretty(log).expect("a log serialises"),
                );
                format!("cat {}\n", sarif.display())
            }
            None => format!("echo \"{binary} could not judge this tree\" >&2\n"),
        };
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\n{body}exit {code}\n",
            argv.display()
        );
        let path = self.path().join(".plotplot/bin").join(binary);
        write(&path, &script);
        make_executable(&path);
        self.record(binary, &digest_for(binary))
    }

    /// The digest record beside one gate, as `lock verify` writes it after placing a judge.
    fn record(&self, binary: &str, digest: &str) -> &Fixture {
        write(
            &layout::judge_digest(self.path(), binary),
            &format!("{digest}\n"),
        );
        self
    }

    /// `garden.lock`, pinning each named gate at its own digest.
    fn pin(&self, names: &[&str]) -> &Fixture {
        let digests: Vec<String> = names.iter().map(|name| digest_for(name)).collect();
        let pins: Vec<(&str, &str)> = names
            .iter()
            .zip(&digests)
            .map(|(name, digest)| (*name, digest.as_str()))
            .collect();
        write(&layout::garden_lock(self.path()), &lock_pinning(&pins));
        self
    }

    fn argv_path(&self, binary: &str) -> PathBuf {
        self.path().join(format!(".plotplot/{binary}.argv"))
    }

    /// The arguments one gate was called with, one to a line, as the gate itself recorded
    /// them.
    fn argv(&self, binary: &str) -> Vec<String> {
        let text = std::fs::read_to_string(self.argv_path(binary))
            .unwrap_or_else(|error| panic!("{binary} recorded no arguments: {error}"));
        text.lines().map(str::to_owned).collect()
    }

    /// Commit everything in the root, making it a repository first if it is not one yet, and
    /// return the commit, which is the ref a range is named by.
    fn commit(&self, message: &str) -> String {
        if !self.path().join(".git").exists() {
            git(self.path(), &["init", "--quiet"]);
            // What `init` ignores: the stem's own tree is never part of what is judged.
            write(&self.path().join(".gitignore"), ".plotplot/\n");
        }
        git(self.path(), &["add", "--all"]);
        git(
            self.path(),
            &["commit", "--quiet", "--allow-empty", "--message", message],
        );
        git(self.path(), &["rev-parse", "HEAD"]).trim().to_owned()
    }

    fn check(&self, args: &[&str]) -> std::process::Output {
        let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
        command.current_dir(self.path());
        command.arg("check");
        command.args(args);
        command.output().expect("the stem runs")
    }
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the parent directory");
    }
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

/// git in `repo`, with an identity of its own and no hook or signing of the machine's, and
/// what it printed.
fn git(repo: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "-c",
            "user.name=plotplot test",
            "-c",
            "user.email=test@plotplot.invalid",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "commit.gpgSign=false",
        ])
        .args(args)
        .output()
        .expect("git runs on the build machine");
    assert!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("the file was written")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("the mode is set");
}

fn stdout_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn code_of(output: &std::process::Output) -> i32 {
    output.status.code().expect("the stem exited on its own")
}

/// The names of the tools the merged log's runs carry, in order.
fn tool_names(log: &Value) -> Vec<String> {
    log.get("runs")
        .and_then(Value::as_array)
        .expect("the merged log has runs")
        .iter()
        .map(|run| {
            run.pointer("/tool/driver/name")
                .and_then(Value::as_str)
                .expect("every run names its tool")
                .to_owned()
        })
        .collect()
}

#[test]
fn two_gates_become_one_log_of_two_runs_that_the_sarif_schema_accepts() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .bed("petals", "petals", "petals check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .gate("petals", Some(&sarif_log("petals", false)), 0);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    let merged = stdout_of(&output);
    let log = sarif::validate(&merged).expect("the merged log validates against the schema");
    assert_eq!(log["version"], json!("2.1.0"));
    assert_eq!(log["$schema"], json!(sarif::SCHEMA_URL));
    assert_eq!(log["runs"].as_array().expect("runs").len(), 2, "{merged}");
}

/// petals' own log as `petals gate` writes it: an error, a warning and a note from one real change.
fn petals_log() -> Value {
    let text = std::fs::read_to_string(contracts("fixtures/sarif/petals-check.sarif.json"))
        .expect("the contracts' petals SARIF fixture");
    serde_json::from_str(&text).expect("the petals fixture is JSON")
}

#[test]
fn petals_real_log_and_weeders_become_one_log_that_blocks_on_petals_error() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .bed("petals", "petals", "petals gate --format sarif", true)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .gate("petals", Some(&petals_log()), 1);

    let output = fixture.check(&[]);
    // One error-level result in petals' log blocks the merged run.
    assert_eq!(code_of(&output), 2, "{}", stderr_of(&output));

    let merged = stdout_of(&output);
    let log = sarif::validate(&merged).expect("the merged log validates against the schema");
    assert_eq!(tool_names(&log), ["petals", "weeder"]);
    let ids: Vec<&str> = log["runs"][0]["results"]
        .as_array()
        .expect("petals' results")
        .iter()
        .map(|result| result["ruleId"].as_str().expect("a rule id"))
        .collect();
    assert_eq!(
        ids,
        ["content/placeholder", "color/off-palette", "layout/spacing"]
    );
}

#[test]
fn the_merged_logs_tool_names_are_each_gates_own() {
    let fixture = Fixture::new();
    fixture
        .bed("petals", "petals", "petals check --format sarif", false)
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("petals", Some(&sarif_log("petals", false)), 0)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    let log: Value = serde_json::from_str(&stdout_of(&output)).expect("the merged log is JSON");
    // load_beds sorts by name, so petals runs before weeder and the runs keep that order.
    assert_eq!(tool_names(&log), ["petals", "weeder"]);
}

#[test]
fn one_block_level_result_exits_two() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", true)), 2);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 2, "{}", stderr_of(&output));

    let log = sarif::validate(&stdout_of(&output)).expect("a valid log even when it blocks");
    assert!(sarif::block_count(&log) > 0, "{log}");
}

#[test]
fn a_gate_that_exits_without_sarif_exits_three_and_names_the_bed() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", None, 1);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains("weeder"), "{stderr}");
    // And the log it printed is still a log, carrying the runs it did get.
    let log = sarif::validate(&stdout_of(&output)).expect("a valid log with no runs");
    assert_eq!(log["runs"].as_array().expect("runs").len(), 0);
}

#[test]
fn a_missing_binary_exits_three() {
    let fixture = Fixture::new();
    fixture.bed("weeder", "weeder", "weeder check --format sarif", false);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains("weeder"), "{stderr}");
    assert!(stderr.contains(".plotplot/bin/weeder"), "{stderr}");
}

#[test]
fn a_block_and_a_gate_that_could_not_run_exit_three() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .bed("petals", "petals", "petals check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", true)), 2)
        .gate("petals", None, 1);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let log = sarif::validate(&stdout_of(&output)).expect("the blocking run still reaches the log");
    assert_eq!(tool_names(&log), ["weeder"]);
    assert!(sarif::block_count(&log) > 0, "{log}");
}

#[test]
fn strict_reaches_only_the_bed_that_declared_it() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", true)
        .bed("petals", "petals", "petals check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .gate("petals", Some(&sarif_log("petals", false)), 0);

    let output = fixture.check(&["--strict"]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    assert_eq!(
        fixture.argv("weeder"),
        ["check", "--format", "sarif", "--strict"]
    );
    assert_eq!(fixture.argv("petals"), ["check", "--format", "sarif"]);
}

#[test]
fn without_strict_the_flag_reaches_nobody() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", true)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0);

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));
    assert_eq!(fixture.argv("weeder"), ["check", "--format", "sarif"]);
}

#[test]
fn a_range_reaches_every_gate_as_base_whether_or_not_it_declared_strict() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", true)
        .bed("petals", "petals", "petals gate --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .gate("petals", Some(&sarif_log("petals", false)), 0)
        .pin(&["petals", "weeder"]);
    let base = fixture.commit("the base of the range");

    let output = fixture.check(&["--strict", "--base", &base]);

    assert_eq!(
        fixture.argv("weeder"),
        [
            "check",
            "--format",
            "sarif",
            "--strict",
            "--base",
            base.as_str()
        ]
    );
    assert_eq!(
        fixture.argv("petals"),
        ["gate", "--format", "sarif", "--base", base.as_str()]
    );
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));
}

/// The runs in a merged log that the stem wrote itself, rather than a gate.
fn the_stems_own_runs(log: &Value) -> Vec<&Value> {
    log["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| run.pointer("/tool/driver/name") == Some(&json!("plotplot")))
        .collect()
}

#[test]
fn a_range_that_changes_no_file_says_so_in_the_log() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .pin(&["weeder"]);
    let base = fixture.commit("everything, and nothing after it");

    let output = fixture.check(&["--base", &base]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    let log = sarif::validate(&stdout_of(&output)).expect("the merged log validates");
    assert_eq!(tool_names(&log), ["weeder", "plotplot"]);
    let own = the_stems_own_runs(&log);
    let results = own[0]["results"].as_array().expect("the stem's results");
    assert_eq!(results.len(), 1, "{log}");
    assert_eq!(results[0]["level"], json!("note"));
    assert_eq!(results[0]["ruleId"], json!("plotplot/empty-range"));
    let text = results[0]["message"]["text"].as_str().expect("a message");
    assert!(text.contains(&base), "{text}");
    // A note is never a block: an empty range is a fact about the run, not a refusal.
    assert_eq!(sarif::block_count(&log), 0);
    // And whoever reads the job's log, not the SARIF, is told the same.
    assert!(
        stderr_of(&output).contains("changes no tracked file"),
        "{}",
        stderr_of(&output)
    );
}

#[test]
fn a_range_that_changes_a_file_carries_no_note_of_the_stems() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .pin(&["weeder"]);
    let base = fixture.commit("the base of the range");
    write(&fixture.path().join("src/lib.rs"), "pub fn changed() {}\n");
    fixture.commit("one change in the range");

    let output = fixture.check(&["--base", &base]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    let log = sarif::validate(&stdout_of(&output)).expect("the merged log validates");
    assert_eq!(tool_names(&log), ["weeder"]);
}

#[test]
fn without_a_base_there_is_no_range_and_no_note() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0);
    fixture.commit("a clean tree");

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));
    assert_eq!(fixture.argv("weeder"), ["check", "--format", "sarif"]);
    let log = sarif::validate(&stdout_of(&output)).expect("the merged log validates");
    assert_eq!(tool_names(&log), ["weeder"]);
}

#[test]
fn a_base_git_cannot_read_exits_three_naming_it_and_runs_no_gate() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", true)), 2);
    fixture.commit("the only commit");

    let output = fixture.check(&["--base", "origin/no-such-branch"]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains("origin/no-such-branch"), "{stderr}");
    assert!(
        !fixture.argv_path("weeder").exists(),
        "a gate was called over a range that could not be read"
    );
}

/// A repository whose base pins weeder at its own digest and whose change moves the pin to
/// another judge, the way a pull request can edit the lock. It answers the base commit.
fn a_change_that_moves_the_pin(fixture: &Fixture) -> String {
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", true)), 2)
        .pin(&["weeder"]);
    let base = fixture.commit("the base pins weeder");
    write(
        &layout::garden_lock(fixture.path()),
        &lock_pinning(&[("weeder", &digest_for("another judge"))]),
    );
    write(
        &fixture.path().join("tests/kept.rs"),
        "#[test]\nfn kept() {}\n",
    );
    fixture.commit("the change pins another judge");
    base
}

#[test]
fn a_judge_the_change_pinned_does_not_judge_the_range() {
    let fixture = Fixture::new();
    let base = a_change_that_moves_the_pin(&fixture);
    // What a plain `lock verify` leaves after reading the change's own lock.
    fixture.record("weeder", &digest_for("another judge"));

    let output = fixture.check(&["--strict", "--base", &base]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains("weeder: "), "{stderr}");
    assert!(stderr.contains("the base's garden.lock"), "{stderr}");
    assert!(stderr.contains("lock verify --base"), "{stderr}");
    assert!(
        !fixture.argv_path("weeder").exists(),
        "a judge the base does not pin was called"
    );
}

#[test]
fn the_judge_the_base_pins_judges_the_range_whatever_the_change_pins() {
    let fixture = Fixture::new();
    let base = a_change_that_moves_the_pin(&fixture);
    // What `lock verify --base` leaves: the base's judge, with the base's digest.

    let output = fixture.check(&["--strict", "--base", &base]);
    assert_eq!(code_of(&output), 2, "{}", stderr_of(&output));
    assert_eq!(
        fixture.argv("weeder"),
        ["check", "--format", "sarif", "--base", base.as_str()]
    );
}

#[test]
fn a_bed_the_base_does_not_pin_is_no_judge_over_a_range() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .pin(&["weeder"]);
    let base = fixture.commit("the base pins weeder alone");
    fixture
        .bed("petals", "petals", "petals gate --format sarif", false)
        .gate("petals", Some(&sarif_log("petals", false)), 0)
        .pin(&["petals", "weeder"]);
    fixture.commit("the change adds petals");

    let output = fixture.check(&["--base", &base]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains("petals: "), "{stderr}");
    assert!(!fixture.argv_path("petals").exists());
    // weeder is the base's, and still judges.
    assert_eq!(
        fixture.argv("weeder"),
        ["check", "--format", "sarif", "--base", base.as_str()]
    );
}

#[test]
fn a_tree_that_carries_the_stems_own_files_is_refused_over_a_range() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0)
        .pin(&["weeder"]);
    let base = fixture.commit("the base");
    // A change that commits a judge and its record past the ignore, so the record matches
    // the base's lock and the bytes are whatever the change wants.
    git(
        fixture.path(),
        &[
            "add",
            "--force",
            ".plotplot/bin/weeder",
            ".plotplot/bin/weeder.sha256",
        ],
    );
    git(
        fixture.path(),
        &["commit", "--quiet", "--message", "a judge of its own"],
    );

    let output = fixture.check(&["--base", &base]);
    assert_eq!(code_of(&output), 3, "{}", stdout_of(&output));

    let stderr = stderr_of(&output);
    assert!(stderr.contains(".plotplot/bin/weeder"), "{stderr}");
    assert!(!fixture.argv_path("weeder").exists());
}

#[test]
fn an_empty_base_is_refused_before_any_gate_runs() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", false)), 0);
    fixture.commit("the only commit");

    let output = fixture.check(&["--base="]);
    assert_ne!(code_of(&output), 0, "{}", stdout_of(&output));
    assert!(
        stderr_of(&output).contains("names no range"),
        "{}",
        stderr_of(&output)
    );
    assert!(!fixture.argv_path("weeder").exists());
}

#[test]
fn the_table_format_prints_one_row_per_gate() {
    let fixture = Fixture::new();
    fixture
        .bed("weeder", "weeder", "weeder check --format sarif", false)
        .bed("petals", "petals", "petals check --format sarif", false)
        .bed("tilth", "tilth", "tilth check --format sarif", false)
        .gate("weeder", Some(&sarif_log("weeder", true)), 2)
        .gate("petals", Some(&sarif_log("petals", false)), 0)
        .gate("tilth", None, 1);

    let output = fixture.check(&["--format", "table"]);
    assert_eq!(code_of(&output), 3, "{}", stderr_of(&output));

    let table = stdout_of(&output);
    let row = |bed: &str| {
        table
            .lines()
            .find(|line| line.split_whitespace().next() == Some(bed))
            .unwrap_or_else(|| panic!("no row for {bed} in\n{table}"))
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };

    assert_eq!(row("petals")[..3], ["petals", "2", "0"]);
    assert_eq!(row("weeder")[..3], ["weeder", "2", "2"]);
    let tilth = row("tilth");
    assert_eq!(tilth[0], "tilth");
    assert!(tilth.join(" ").contains("could not judge"), "{table}");
    // A table is what stdout carries; the merged log is not also printed.
    assert!(!table.contains("\"version\""), "{table}");
}

#[test]
fn a_planted_repository_with_no_gates_prints_a_log_with_no_runs_and_exits_zero() {
    let fixture = Fixture::new();
    // A bed that is planted and declares no gate of its own.
    let mut manifest = manifest("pollen", "pollen", "pollen check", false);
    manifest["check"] = Value::Null;
    manifest["kind"] = json!(["channel"]);
    write(
        &fixture.path().join(".plotplot/beds/pollen/garden.json"),
        &serde_json::to_string_pretty(&manifest).expect("a manifest serialises"),
    );

    let output = fixture.check(&[]);
    assert_eq!(code_of(&output), 0, "{}", stderr_of(&output));

    let log = sarif::validate(&stdout_of(&output)).expect("a log with no runs is still a log");
    assert_eq!(log["runs"].as_array().expect("runs").len(), 0);
}

#[test]
fn validate_refuses_a_log_with_no_runs_key() {
    let error = sarif::validate("{\"version\": \"2.1.0\"}")
        .expect_err("the schema requires runs, so a log without it is refused");
    let message = error.to_string();
    assert!(message.contains("runs"), "{message}");
}
