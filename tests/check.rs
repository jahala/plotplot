//! `plotplot check`, driven as the binary a planter runs.
//!
//! The fixture is a planted repository whose beds are manifests in the contracts' own shape
//! and whose gates are scripts under `.plotplot/bin/` that print the contracts' own SARIF
//! fixture. A gate script records the arguments it was called with, so what the stem passed
//! a gate is read from the gate itself rather than asserted about the stem's intentions.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serde_json::{Value, json};

use plotplot::sarif;

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
