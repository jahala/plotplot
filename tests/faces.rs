//! The stem's own hook-time faces, at the level of the files they write.
//!
//! The pure decisions live beside their code in unit tests; this file drives `friction::emit`
//! and `receipt::draft` against a temporary repository root, because what those two owe the
//! garden is bytes on disk in the places `layout.rs` names.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};

use plotplot::harness::{Harness, Payload, parse_payload};
use plotplot::{friction, layout, receipt};

const CLAUDE_SESSION: &str = "6f3c1b2a-8d47-4f0e-9b31-2a5c7e91d044";
const PINNED: &str = "2026-09-09T12:00:00.000Z";
const PINNED_MONTH: &str = "2026-09";

/// A clock the test pins, so a record's `time` is a value and not a moving target.
struct Pinned(&'static str);

impl friction::Now for Pinned {
    fn rfc3339_utc(&self) -> String {
        self.0.to_owned()
    }
}

fn payloads() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/payloads")
}

fn fixture(harness: Harness, event: &str) -> Payload {
    let path = payloads()
        .join(harness.name())
        .join(format!("{event}.json"));
    let json = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse_payload(harness, &json).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn journal_lines(root: &Path, month: &str) -> Vec<Value> {
    let path = layout::friction_journal(root, month);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines()
        .map(|line| serde_json::from_str(line).expect("every journal line is one JSON object"))
        .collect()
}

fn read_draft(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("the draft is JSON")
}

/// The four payloads a Claude session fires before it ends, in the order it fires them.
fn run_claude_session(root: &Path, clock: &Pinned) {
    for event in [
        "SessionStart",
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
    ] {
        friction::emit(root, &fixture(Harness::Claude, event), clock)
            .unwrap_or_else(|e| panic!("{event}: {e}"));
    }
}

#[test]
fn a_claude_session_appends_only_the_lines_it_earned_and_prunes_its_state() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let clock = Pinned(PINNED);
    let state = layout::friction_state(root, CLAUDE_SESSION);

    assert_eq!(
        friction::emit(root, &fixture(Harness::Claude, "SessionStart"), &clock).expect("emit"),
        0
    );
    assert_eq!(
        friction::emit(root, &fixture(Harness::Claude, "PreToolUse"), &clock).expect("emit"),
        0
    );
    assert_eq!(
        friction::emit(root, &fixture(Harness::Claude, "PostToolUse"), &clock).expect("emit"),
        0
    );
    assert!(state.exists(), "the state file is kept after a tool call");
    assert_eq!(
        friction::emit(
            root,
            &fixture(Harness::Claude, "PostToolUseFailure"),
            &clock
        )
        .expect("emit"),
        1
    );
    assert_eq!(
        friction::emit(root, &fixture(Harness::Claude, "SessionEnd"), &clock).expect("emit"),
        1
    );
    assert!(
        !state.exists(),
        "the state file is pruned at the session end"
    );

    let lines = journal_lines(root, PINNED_MONTH);
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert_eq!(lines[0]["plotplot.kind"], "tool.failed");
    assert_eq!(lines[0]["plotplot.path"], "src/lock.rs");
    assert_eq!(lines[1]["plotplot.kind"], "session.ended");
    for line in &lines {
        assert_eq!(line["time"], PINNED);
        assert_eq!(line["event.name"], "plotplot.friction");
        assert_eq!(line["plotplot.harness"], "claude-code");
        assert_eq!(line["gen_ai.conversation.id"], CLAUDE_SESSION);
    }

    let raw = fs::read_to_string(layout::friction_journal(root, PINNED_MONTH)).expect("journal");
    assert!(!raw.contains("/home/agent"), "{raw}");
    assert!(!raw.contains("/work/repo"), "{raw}");
}

#[test]
fn every_harness_ends_its_session_on_the_event_that_vendor_fires() {
    for (harness, event) in [
        (Harness::Claude, "SessionEnd"),
        (Harness::Gemini, "SessionEnd"),
        (Harness::Codex, "Stop"),
    ] {
        let temp = tempfile::tempdir().expect("a temporary root");
        let root = temp.path();
        assert_eq!(
            friction::emit(root, &fixture(harness, event), &Pinned(PINNED)).expect("emit"),
            1,
            "{harness} {event}"
        );
        let lines = journal_lines(root, PINNED_MONTH);
        assert_eq!(lines[0]["plotplot.kind"], "session.ended", "{harness}");
        assert!(lines[0]["gen_ai.usage.input_tokens"].is_null(), "{harness}");
        assert!(
            lines[0]["gen_ai.usage.output_tokens"].is_null(),
            "{harness}"
        );
    }
}

#[test]
fn a_payload_without_a_session_id_writes_nothing_at_all() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let json = r#"{"hook_event_name":"PostToolUseFailure","cwd":"/work/repo",
        "tool_name":"Edit","tool_input":{"file_path":"src/lib.rs"},"error":"nope"}"#;
    let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");

    assert_eq!(
        friction::emit(root, &payload, &Pinned(PINNED)).expect("emit"),
        0
    );
    assert!(
        !layout::plotplot_dir(root).exists(),
        "a payload with no session wrote a directory"
    );
}

#[test]
fn the_month_of_the_journal_comes_from_the_clock_it_was_given() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    friction::emit(
        root,
        &fixture(Harness::Claude, "SessionEnd"),
        &Pinned("2027-01-31T23:59:59.001Z"),
    )
    .expect("emit");
    assert!(layout::friction_journal(root, "2027-01").exists());
    assert!(!layout::friction_journal(root, PINNED_MONTH).exists());
}

#[test]
fn session_end_emission_completes_under_two_hundred_milliseconds() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let clock = Pinned(PINNED);
    run_claude_session(root, &clock);

    let end = fixture(Harness::Claude, "SessionEnd");
    let start = Instant::now();
    friction::emit(root, &end, &clock).expect("emit");
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(200),
        "session end took {elapsed:?}"
    );
}

#[test]
fn the_draft_carries_what_the_payload_says_and_nulls_what_it_does_not() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let path = receipt::draft(root, &fixture(Harness::Claude, "SessionEnd")).expect("draft");
    assert_eq!(path, layout::receipt_draft(root, CLAUDE_SESSION));

    let draft = read_draft(&path);
    assert_eq!(draft["harness"]["name"], "claude-code");
    assert!(draft["harness"]["version"].is_null());
    assert_eq!(draft["models"], serde_json::json!(["claude-opus-5"]));
    assert_eq!(draft["principal"], "cli");
    assert_eq!(draft["sessions"], serde_json::json!([CLAUDE_SESSION]));
    assert_eq!(draft["cost"]["toolCalls"], serde_json::json!({}));
    assert!(draft["cost"]["wallSeconds"].is_null());
    assert!(draft["cost"]["inputTokens"].is_null());
    assert!(draft["cost"]["outputTokens"].is_null());
    assert!(draft["friction"]["summary"].is_null());
}

#[test]
fn a_second_draft_for_one_session_overwrites_the_first() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let end = fixture(Harness::Claude, "SessionEnd");

    let first = receipt::draft(root, &end).expect("draft");
    let before = fs::read_to_string(&first).expect("the first draft");
    run_claude_session(root, &Pinned(PINNED));
    let second = receipt::draft(root, &end).expect("draft");

    assert_eq!(first, second);
    let after = fs::read_to_string(&second).expect("the second draft");
    assert_ne!(before, after, "the second draft did not replace the first");
    assert_eq!(
        fs::read_dir(layout::receipt_drafts_dir(root))
            .expect("the drafts directory")
            .count(),
        1
    );
}

#[test]
fn a_friction_state_gives_the_draft_its_tool_counts_and_its_summary_digest() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    run_claude_session(root, &Pinned(PINNED));

    let state_bytes =
        fs::read(layout::friction_state(root, CLAUDE_SESSION)).expect("the state file");
    let expected = hex::encode(Sha256::digest(&state_bytes));

    let path = receipt::draft(root, &fixture(Harness::Claude, "SessionEnd")).expect("draft");
    let draft = read_draft(&path);

    let counts: BTreeMap<String, u64> =
        serde_json::from_value(draft["cost"]["toolCalls"].clone()).expect("the tool counts");
    assert_eq!(counts.get("Read"), Some(&1));
    assert_eq!(counts.get("Edit"), Some(&1));
    assert_eq!(counts.len(), 2, "{counts:?}");
    assert_eq!(draft["friction"]["summary"]["sha256"], expected);
}

#[test]
fn a_draft_for_a_payload_with_no_session_is_refused_rather_than_guessed() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let json = r#"{"hook_event_name":"SessionEnd","cwd":"/work/repo","reason":"clear"}"#;
    let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");

    assert!(receipt::draft(root, &payload).is_err());
    assert!(!layout::plotplot_dir(root).exists());
}

#[test]
fn every_harness_drafts_under_its_own_session_id() {
    for (harness, event, name) in [
        (Harness::Claude, "SessionEnd", "claude-code"),
        (Harness::Gemini, "SessionEnd", "gemini-cli"),
        (Harness::Codex, "Stop", "codex-cli"),
    ] {
        let temp = tempfile::tempdir().expect("a temporary root");
        let root = temp.path();
        let payload = fixture(harness, event);
        let session = payload.session_id.clone().expect("a fixture session id");
        let path = receipt::draft(root, &payload).expect("draft");
        assert_eq!(path, layout::receipt_draft(root, &session));
        let draft = read_draft(&path);
        assert_eq!(draft["harness"]["name"], name);
        assert_eq!(draft["sessions"], serde_json::json!([session]));
    }
}

/// Not an assertion: the shape of a real journal, printed with `cargo test -- --nocapture`,
/// so a reader can hold it beside `contracts/friction-profile.md`.
#[test]
fn the_journal_shape_is_readable_beside_the_profile() {
    let temp = tempfile::tempdir().expect("a temporary root");
    let root = temp.path();
    let clock = Pinned(PINNED);
    run_claude_session(root, &clock);
    friction::emit(root, &fixture(Harness::Claude, "PermissionDenied"), &clock).expect("emit");
    friction::emit(root, &fixture(Harness::Claude, "PreCompact"), &clock).expect("emit");
    friction::emit(root, &fixture(Harness::Claude, "SessionEnd"), &clock).expect("emit");
    let text = fs::read_to_string(layout::friction_journal(root, PINNED_MONTH)).expect("journal");
    for line in text.lines() {
        println!("{line}");
    }
    let kinds: Vec<&str> = journal_lines(root, PINNED_MONTH)
        .iter()
        .filter_map(|line| line["plotplot.kind"].as_str().map(str::to_owned))
        .map(|kind| match kind.as_str() {
            "tool.failed" => "tool.failed",
            "tool.denied" => "tool.denied",
            "context.compacted" => "context.compacted",
            "session.ended" => "session.ended",
            other => panic!("unexpected kind {other}"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "tool.failed",
            "tool.denied",
            "context.compacted",
            "session.ended"
        ]
    );
}
