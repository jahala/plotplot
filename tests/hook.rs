//! The dispatcher and the two hook-time faces, driven as the binary a vendor calls.
//!
//! Every test here builds a temporary repository root, writes the beds it needs as executable
//! `sh` scripts under `.plotplot/bin/` with a matching `garden.json` under `.plotplot/beds/`,
//! and runs the real binary against a real vendor payload from `tests/fixtures/payloads/`.
//! A bed script touches a marker file when it runs, so a test can prove a bed was called and,
//! more importantly, prove one was not.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

use plotplot::layout;

const CLAUDE_SESSION: &str = "6f3c1b2a-8d47-4f0e-9b31-2a5c7e91d044";
const CODEX_SESSION: &str = "e21a4f78-9c05-4b3d-a6f1-70d82c5e9b4a";
const GEMINI_SESSION: &str = "3a71f5c8-6e2d-4b90-8137-5cf9d0a4e6b1";

/// The dispatcher's own overhead with no bed registered, from spawn to exit.
const BUDGET: Duration = Duration::from_millis(50);

fn plotplot(root: &Path) -> Command {
    let mut command = Command::cargo_bin("plotplot").expect("the plotplot binary is built");
    command.current_dir(root);
    command
}

fn payload(harness: &str, event: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/payloads")
        .join(harness)
        .join(format!("{event}.json"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// One fixture payload with one field changed.
///
/// The corpus under `tests/fixtures/payloads/` keeps one payload per harness event, and its
/// own tests assert that shape, so a variant one dispatcher test needs is made here from the
/// real payload rather than added beside it as a second `PreToolUse`.
fn variant(harness: &str, event: &str, edit: impl FnOnce(&mut Value)) -> String {
    let mut payload: Value =
        serde_json::from_str(&payload(harness, event)).expect("the fixture is JSON");
    edit(&mut payload);
    serde_json::to_string_pretty(&payload).expect("a serializable payload")
}

/// The Claude Bash call the stem's own deny list refuses: a commit that skips the gate.
fn no_verify() -> String {
    variant("claude", "PreToolUse", |payload| {
        payload["tool_input"]["command"] =
            Value::String("git commit --no-verify -m \"wire the dispatcher\"".to_owned());
        payload["tool_input"]["description"] = Value::String("Commit the dispatcher".to_owned());
    })
}

/// The same call, reading a file instead of running a shell command.
fn reading() -> String {
    variant("claude", "PreToolUse", |payload| {
        payload["tool_name"] = Value::String("Read".to_owned());
        payload["tool_input"] = serde_json::json!({"file_path": "/work/repo/src/hook.rs"});
    })
}

/// Where a bed script leaves its mark, so a test can see whether it ran.
fn marker(root: &Path, bed: &str) -> PathBuf {
    root.join("markers").join(bed)
}

/// A bed the stem can find: an executable at `.plotplot/bin/<name>` and the manifest at
/// `.plotplot/beds/<name>/garden.json` that registers it, built from the contracts' own
/// weeder fixture with the name, the CLI and the hook entries changed.
fn plant(root: &Path, name: &str, hooks: &[(&str, &[&str])], body: &str) {
    plant_manifest(root, name, hooks);
    plant_binary(root, name, body);
}

fn plant_manifest(root: &Path, name: &str, hooks: &[(&str, &[&str])]) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts/fixtures/manifest/weeder.garden.json");
    let text =
        fs::read_to_string(&fixture).unwrap_or_else(|e| panic!("{}: {e}", fixture.display()));
    let mut manifest: Value = serde_json::from_str(&text).expect("the weeder fixture is JSON");

    manifest["name"] = Value::String(name.to_owned());
    manifest["faces"]["cli"] = Value::String(name.to_owned());
    manifest["faces"]["hooks"] = Value::Object(
        hooks
            .iter()
            .map(|(harness, entries)| {
                let entries = entries
                    .iter()
                    .map(|entry| Value::String((*entry).to_owned()))
                    .collect();
                ((*harness).to_owned(), Value::Array(entries))
            })
            .collect(),
    );

    let path = layout::bed_manifest(root, name);
    let directory = path.parent().expect("a bed directory");
    fs::create_dir_all(directory).unwrap_or_else(|e| panic!("{}: {e}", directory.display()));
    fs::write(
        &path,
        serde_json::to_string_pretty(&manifest).expect("a serializable manifest"),
    )
    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

/// The bed's program: it drains the payload, marks that it ran, then does what `body` says.
fn plant_binary(root: &Path, name: &str, body: &str) {
    let markers = root.join("markers");
    fs::create_dir_all(&markers).unwrap_or_else(|e| panic!("{}: {e}", markers.display()));
    let bin = layout::bin_dir(root);
    fs::create_dir_all(&bin).unwrap_or_else(|e| panic!("{}: {e}", bin.display()));

    let script = format!(
        "#!/bin/sh\ncat > /dev/null\n: > '{}'\n{body}\n",
        marker(root, name).display()
    );
    let path = layout::judge_binary(root, name);
    fs::write(&path, script).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    make_executable(&path);
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

/// The Claude answer a bed writes on stdout when it refuses a tool call (§5).
fn claude_deny(reason: &str) -> String {
    format!(
        "{{\"hookSpecificOutput\":{{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\"permissionDecisionReason\":\"{reason}\"}}}}"
    )
}

/// A bed that refuses: the vendor's answer on stdout, the reason on stderr, exit 2.
fn denial(reason: &str) -> String {
    format!(
        "printf '%s' '{}'\nprintf '%s\\n' '{reason}' >&2\nexit 2",
        claude_deny(reason)
    )
}

/// Every line of every friction journal under the root, whatever month the clock is in.
fn journal_lines(root: &Path) -> Vec<Value> {
    let directory = layout::friction_dir(root);
    let Ok(entries) = fs::read_dir(&directory) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for line in text.lines() {
            lines.push(serde_json::from_str(line).expect("every journal line is one JSON object"));
        }
    }
    lines
}

fn journal_text(root: &Path) -> String {
    let directory = layout::friction_dir(root);
    let Ok(entries) = fs::read_dir(&directory) else {
        return String::new();
    };
    let mut text = String::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            text.push_str(
                &fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
            );
        }
    }
    text
}

// ---------------------------------------------------------------------------------------
// the empty repository: allow, in silence, inside the budget
// ---------------------------------------------------------------------------------------

#[test]
fn an_unplanted_repository_allows_in_silence_and_writes_no_journal() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0)
        .stdout("")
        .stderr("");

    assert!(
        journal_lines(root).is_empty(),
        "a plain PreToolUse earns no friction record, so no journal line: {:#?}",
        journal_lines(root)
    );
}

#[test]
fn the_dispatcher_answers_an_unplanted_repository_inside_its_budget() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let binary = assert_cmd::cargo::cargo_bin("plotplot");
    let payload = payload("claude", "PreToolUse");

    let mut runs = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let mut child = std::process::Command::new(&binary)
            .args(["hook", "claude", "PreToolUse"])
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary runs");
        child
            .stdin
            .take()
            .expect("the child's stdin")
            .write_all(payload.as_bytes())
            .expect("the payload is written");
        let output = child.wait_with_output().expect("the child ends");
        runs.push(start.elapsed());
        assert_eq!(output.status.code(), Some(0));
    }

    runs.sort();
    let median = (runs[4] + runs[5]) / 2;
    assert!(
        median < BUDGET,
        "the median of ten runs was {median:?}, over the {BUDGET:?} budget: {runs:?}"
    );
}

// ---------------------------------------------------------------------------------------
// the deny list, before any bed
// ---------------------------------------------------------------------------------------

#[test]
fn a_skipped_gate_is_denied_before_any_bed_is_called() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(root, "eager", &[("claude", &["PreToolUse"])], "exit 0");

    let assert = plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(no_verify())
        .assert()
        .code(2);
    let output = assert.get_output();

    let answer: Value =
        serde_json::from_slice(&output.stdout).expect("the deny answer is the vendor's JSON");
    assert_eq!(answer["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(answer["hookSpecificOutput"]["permissionDecision"], "deny");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--no-verify"), "{stderr}");

    assert!(
        !marker(root, "eager").exists(),
        "a denied tool call never reaches a bed"
    );
}

// ---------------------------------------------------------------------------------------
// the contract with beds
// ---------------------------------------------------------------------------------------

#[test]
fn a_bed_is_called_as_hook_harness_with_the_payload_on_its_stdin() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let seen = root.join("seen.txt");
    plant(
        root,
        "listener",
        &[("claude", &["PreToolUse"])],
        &format!("printf '%s\\n' \"$*\" > '{}'\nexit 0", seen.display()),
    );

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0)
        .stdout("");

    let arguments = fs::read_to_string(&seen).expect("the bed recorded its arguments");
    assert_eq!(arguments.trim(), "hook claude");
}

#[test]
fn a_bed_that_reads_its_payload_sees_the_bytes_the_vendor_sent() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let copy = root.join("payload.json");
    plant_manifest(root, "reader", &[("claude", &["PreToolUse"])]);
    let bin = layout::bin_dir(root);
    fs::create_dir_all(&bin).expect("the bin directory");
    let path = layout::judge_binary(root, "reader");
    fs::write(
        &path,
        format!("#!/bin/sh\ncat > '{}'\nexit 0\n", copy.display()),
    )
    .expect("the bed script");
    make_executable(&path);

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0);

    assert_eq!(
        fs::read_to_string(&copy).expect("the bed's copy of the payload"),
        payload("claude", "PreToolUse")
    );
}

#[test]
fn a_denying_bed_carries_its_answer_out_and_the_second_denial_follows_on_stderr() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(
        root,
        "alpha",
        &[("claude", &["PreToolUse"])],
        &denial("alpha refused"),
    );
    plant(
        root,
        "beta",
        &[("claude", &["PreToolUse"])],
        &denial("beta refused"),
    );

    let assert = plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(2);
    let output = assert.get_output();

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        claude_deny("alpha refused"),
        "the first denial's stdout is the answer the vendor reads"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let alpha = stderr.find("alpha refused").expect("alpha's reason");
    let beta = stderr.find("beta refused").expect("beta's reason");
    assert!(alpha < beta, "the first denial's reason leads: {stderr}");
}

#[test]
fn a_bed_that_cannot_judge_makes_the_dispatcher_cannot_judge() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(
        root,
        "unsure",
        &[("claude", &["PreToolUse"])],
        "printf 'unsure could not reach its index\\n' >&2\nexit 3",
    );

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(3)
        .stdout("")
        .stderr(predicates::str::contains(
            "unsure could not reach its index",
        ));
}

#[test]
fn a_matcher_calls_the_bed_only_for_the_tool_it_names() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(
        root,
        "basher",
        &[("claude", &["PreToolUse:Bash"])],
        "exit 0",
    );

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0);
    assert!(
        marker(root, "basher").exists(),
        "a Bash call reaches a bed on PreToolUse:Bash"
    );

    fs::remove_file(marker(root, "basher")).expect("the marker is cleared");
    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(reading())
        .assert()
        .code(0);
    assert!(
        !marker(root, "basher").exists(),
        "a Read call does not reach a bed on PreToolUse:Bash"
    );
}

#[test]
fn a_bed_declared_on_stop_is_silent_on_the_before_tool_event() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(root, "stopper", &[("claude", &["Stop"])], "exit 2");

    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0)
        .stdout("")
        .stderr("");
    assert!(
        !marker(root, "stopper").exists(),
        "a bed on Stop is not called on PreToolUse"
    );

    plotplot(root)
        .args(["hook", "claude", "Stop"])
        .write_stdin(payload("claude", "Stop"))
        .assert()
        .code(2);
    assert!(
        marker(root, "stopper").exists(),
        "a bed on Stop is called on Stop"
    );
}

#[test]
fn a_bed_whose_binary_is_missing_cannot_judge_and_names_the_path() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant_manifest(root, "absent", &[("claude", &["PreToolUse"])]);

    let expected = layout::judge_binary(root, "absent");
    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(3)
        .stdout("")
        .stderr(predicates::str::contains(
            expected.display().to_string().as_str(),
        ));
}

#[test]
fn a_bed_past_the_timeout_is_killed_and_reported_as_unable_to_judge() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    // Stop carries the 4 s timeout; the script outlives it by far.
    plant(root, "slow", &[("claude", &["Stop"])], "sleep 60");

    let start = Instant::now();
    let assert = plotplot(root)
        .args(["hook", "claude", "Stop"])
        .write_stdin(payload("claude", "Stop"))
        .assert()
        .code(3);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "the dispatcher took {elapsed:?}, past the 4 s timeout plus a second"
    );
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("slow"),
        "the reason names the bed: {stderr}"
    );
}

#[test]
fn two_beds_run_at_the_same_time() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    plant(root, "first", &[("claude", &["PreToolUse"])], "sleep 0.3");
    plant(root, "second", &[("claude", &["PreToolUse"])], "sleep 0.3");

    // macOS scans an executable the first time it is run, which costs a few hundred
    // milliseconds per freshly written file and is the operating system's, not the
    // dispatcher's. One dispatch pays it; the one that is timed measures the fan-out.
    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0);

    let start = Instant::now();
    plotplot(root)
        .args(["hook", "claude", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(0);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "two beds sleeping 300 ms each took {elapsed:?}, so they ran one after the other"
    );
    assert!(marker(root, "first").exists());
    assert!(marker(root, "second").exists());
}

// ---------------------------------------------------------------------------------------
// the two faces the stem owns
// ---------------------------------------------------------------------------------------

#[test]
fn the_session_end_event_drafts_the_receipt_then_ends_the_session() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();

    for event in ["PreToolUse", "PostToolUse"] {
        plotplot(root)
            .args(["hook", "claude", event])
            .write_stdin(payload("claude", event))
            .assert()
            .code(0);
    }
    let state = layout::friction_state(root, CLAUDE_SESSION);
    assert!(state.exists(), "the session's state is kept while it runs");

    plotplot(root)
        .args(["hook", "claude", "SessionEnd"])
        .write_stdin(payload("claude", "SessionEnd"))
        .assert()
        .code(0)
        .stdout("")
        .stderr("");

    let draft = layout::receipt_draft(root, CLAUDE_SESSION);
    let text = fs::read_to_string(&draft).unwrap_or_else(|e| panic!("{}: {e}", draft.display()));
    let draft: Value = serde_json::from_str(&text).expect("the draft is JSON");
    assert_eq!(draft["sessions"][0], CLAUDE_SESSION);
    assert_eq!(
        draft["cost"]["toolCalls"]["Read"], 1,
        "the draft is written before the emitter prunes the state it counts from: {text}"
    );

    let lines = journal_lines(root);
    assert_eq!(lines.len(), 1, "{lines:#?}");
    assert_eq!(lines[0]["plotplot.kind"], "session.ended");
    assert_eq!(lines[0]["gen_ai.conversation.id"], CLAUDE_SESSION);
    assert!(!state.exists(), "the state is pruned at the session's end");
}

#[test]
fn friction_emit_writes_the_journal_and_leaks_nothing() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();

    // Twice: the first write of a file is silent, the second is churn and earns its record.
    for _ in 0..2 {
        plotplot(root)
            .args(["friction", "emit", "--harness", "gemini"])
            .write_stdin(payload("gemini", "leaky-AfterTool"))
            .assert()
            .code(0)
            .stdout("")
            .stderr("");
    }

    let lines = journal_lines(root);
    assert!(!lines.is_empty(), "the second write earns a record");
    for line in &lines {
        assert_eq!(line["event.name"], "plotplot.friction");
        assert_eq!(line["plotplot.harness"], "gemini-cli");
        assert_eq!(line["gen_ai.conversation.id"], GEMINI_SESSION);
    }

    let text = journal_text(root);
    for leak in [
        "Rewrite the onboarding email",
        "onboarding-v2",
        "running 24 tests",
        "/Users/",
        "someone",
    ] {
        assert!(!text.contains(leak), "the journal leaked {leak}: {text}");
    }
}

#[test]
fn receipt_draft_writes_the_draft_for_the_harness_it_is_given() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();

    plotplot(root)
        .args(["receipt", "draft", "--harness", "codex"])
        .write_stdin(payload("codex", "Stop"))
        .assert()
        .code(0)
        .stdout("")
        .stderr("");

    let draft = layout::receipt_draft(root, CODEX_SESSION);
    let text = fs::read_to_string(&draft).unwrap_or_else(|e| panic!("{}: {e}", draft.display()));
    let draft: Value = serde_json::from_str(&text).expect("the draft is JSON");
    assert_eq!(draft["harness"]["name"], "codex-cli");
    assert_eq!(draft["sessions"][0], CODEX_SESSION);
    assert_eq!(draft["models"][0], "gpt-5-codex");
}

// ---------------------------------------------------------------------------------------
// arguments the stem refuses
// ---------------------------------------------------------------------------------------

#[test]
fn an_unknown_harness_is_a_usage_error_naming_the_three() {
    let temp = tempfile::tempdir().expect("a temporary root");
    plotplot(temp.path())
        .args(["hook", "cursor", "PreToolUse"])
        .write_stdin(payload("claude", "PreToolUse"))
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicates::str::contains("claude, gemini and codex"));
}

#[test]
fn an_event_the_harness_does_not_fire_is_a_usage_error_naming_the_events() {
    let temp = tempfile::tempdir().expect("a temporary root");
    plotplot(temp.path())
        .args(["hook", "gemini", "PreToolUse"])
        .write_stdin(payload("gemini", "BeforeTool"))
        .assert()
        .code(2)
        .stdout("")
        .stderr(
            predicates::str::contains("PreToolUse")
                .and(predicates::str::contains("BeforeTool"))
                .and(predicates::str::contains("SessionEnd")),
        );
}

#[test]
fn an_unknown_harness_on_the_two_faces_is_a_usage_error_too() {
    let temp = tempfile::tempdir().expect("a temporary root");
    for face in [["friction", "emit"], ["receipt", "draft"]] {
        plotplot(temp.path())
            .args(face)
            .args(["--harness", "cursor"])
            .write_stdin(payload("claude", "PreToolUse"))
            .assert()
            .code(2)
            .stderr(predicates::str::contains("claude, gemini and codex"));
    }
}
