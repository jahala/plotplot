//! The three vendors as the stem sees them.
//!
//! One enum for the harness, one lenient reader for the payload it puts on stdin, one
//! renderer for the answer it expects on stdout. Every event name and every wire shape here
//! is a vendor fact verified on 2026-09-08 and recorded in `docs/prompts/stem-build-2026-09.md`
//! §5; when a vendor changes one, that table changes first and this module follows.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde_json::Value;

use crate::error::{Error, Result};

/// The harnesses the stem plants into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Harness {
    Claude,
    Gemini,
    Codex,
}

const CLAUDE_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionDenied",
    "PermissionRequest",
    "Notification",
    "UserPromptSubmit",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PostCompact",
    "Setup",
    "Elicitation",
    "ElicitationResult",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
    "InstructionsLoaded",
    "FileChanged",
    "CwdChanged",
    "TeammateIdle",
    "TaskCompleted",
];

const GEMINI_EVENTS: &[&str] = &[
    "BeforeTool",
    "AfterTool",
    "BeforeAgent",
    "AfterAgent",
    "BeforeModel",
    "AfterModel",
    "BeforeToolSelection",
    "Notification",
    "SessionStart",
    "SessionEnd",
    "PreCompress",
];

const CODEX_EVENTS: &[&str] = &[
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "SessionStart",
    "UserPromptSubmit",
    "SubagentStart",
    "SubagentStop",
    "Stop",
];

impl Harness {
    /// Every harness the stem knows, in the order the bundles are generated.
    pub const ALL: [Harness; 3] = [Harness::Claude, Harness::Gemini, Harness::Codex];

    /// The lowercase name a manifest, a CLI argument and a bundle directory all use.
    pub fn name(self) -> &'static str {
        match self {
            Harness::Claude => "claude",
            Harness::Gemini => "gemini",
            Harness::Codex => "codex",
        }
    }

    /// Every hook event this vendor fires (§5). A bed registering anything else is refused.
    pub fn events(self) -> &'static [&'static str] {
        match self {
            Harness::Claude => CLAUDE_EVENTS,
            Harness::Gemini => GEMINI_EVENTS,
            Harness::Codex => CODEX_EVENTS,
        }
    }

    /// Whether this vendor fires `event`, matched exactly as the vendor spells it.
    pub fn has_event(self, event: &str) -> bool {
        self.events().contains(&event)
    }

    /// The event that ends a session here. Codex has no `SessionEnd`, so its `Stop` carries
    /// the session's end as well as the turn's.
    pub fn session_end_event(self) -> &'static str {
        match self {
            Harness::Claude | Harness::Gemini => "SessionEnd",
            Harness::Codex => "Stop",
        }
    }

    /// The event fired before a tool runs, where a refusal can still prevent it.
    pub fn before_tool_event(self) -> &'static str {
        match self {
            Harness::Claude | Harness::Codex => "PreToolUse",
            Harness::Gemini => "BeforeTool",
        }
    }

    /// The variable each vendor expands to the installed bundle's own directory.
    pub fn bundle_root_variable(self) -> &'static str {
        match self {
            Harness::Claude | Harness::Codex => "${CLAUDE_PLUGIN_ROOT}",
            Harness::Gemini => "${extensionPath}",
        }
    }
}

impl FromStr for Harness {
    type Err = Error;

    fn from_str(s: &str) -> Result<Harness> {
        match s {
            "claude" => Ok(Harness::Claude),
            "gemini" => Ok(Harness::Gemini),
            "codex" => Ok(Harness::Codex),
            other => Err(Error::Harness {
                problem: format!(
                    "unknown harness \"{other}\"; the stem knows claude, gemini and codex"
                ),
            }),
        }
    }
}

impl fmt::Display for Harness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The fields of a hook payload the stem reads.
///
/// Parsed leniently: a missing field is `None`, a field of the wrong type is `None`, an
/// unknown field is ignored, and `raw` keeps the whole document so a bed can read what the
/// stem does not.
#[derive(Clone, Debug, PartialEq)]
pub struct Payload {
    pub harness: Harness,
    /// `hook_event_name`, the one field a payload must carry.
    pub event: String,
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub tool_name: Option<String>,
    pub tool_input: Option<Value>,
    /// Claude `tool_use_id`; Gemini and Codex when present.
    pub tool_call_id: Option<String>,
    pub tool_response: Option<Value>,
    /// Claude `PostToolUseFailure` `error`; Gemini `AfterTool` `error`.
    pub tool_error: Option<String>,
    pub model: Option<String>,
    pub transcript_path: Option<PathBuf>,
    pub turn_id: Option<String>,
    pub agent_id: Option<String>,
    /// Claude `SessionEnd` and Gemini `AfterAgent` / `SessionEnd` `reason`.
    pub reason: Option<String>,
    /// `PreCompact` and `PreCompress` `trigger`.
    pub trigger: Option<String>,
    /// `SessionStart` `source`.
    pub source: Option<String>,
    pub raw: Value,
}

/// Read a vendor's hook payload.
///
/// # Errors
///
/// [`Error::Json`] when the bytes are not JSON, and [`Error::Harness`] when they are JSON but
/// not an object, or an object with no `hook_event_name` string.
pub fn parse_payload(harness: Harness, json: &str) -> Result<Payload> {
    let raw: Value =
        serde_json::from_str(json).map_err(|source| Error::Json { path: None, source })?;
    let object = raw.as_object().ok_or_else(|| Error::Harness {
        problem: format!("the {harness} payload is not a JSON object"),
    })?;
    let string = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_owned);
    let event = string("hook_event_name").ok_or_else(|| Error::Harness {
        problem: format!("the {harness} payload has no hook_event_name"),
    })?;

    Ok(Payload {
        harness,
        event,
        session_id: string("session_id"),
        cwd: string("cwd").map(PathBuf::from),
        tool_name: string("tool_name"),
        tool_input: object.get("tool_input").cloned(),
        tool_call_id: string("tool_use_id"),
        tool_response: object.get("tool_response").cloned(),
        tool_error: string("error"),
        model: string("model"),
        transcript_path: string("transcript_path").map(PathBuf::from),
        turn_id: string("turn_id"),
        agent_id: string("agent_id"),
        reason: string("reason"),
        trigger: string("trigger"),
        source: string("source"),
        raw,
    })
}

/// What a hook can say back to the harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Nothing to say; the harness carries on.
    Allow,
    /// A tool call refused, on the before-tool event.
    Deny { reason: String },
    /// A stop refused, on `Stop` or `AfterAgent`.
    Block { reason: String },
    /// The hook could not judge. The harness is told nothing; the reason goes to stderr and
    /// the process exits 3 so the caller knows the answer is missing rather than permissive.
    Cannot { reason: String },
}

/// The vendor's own answer shape, ready for stdout.
///
/// Rendered by hand rather than through a serializer so the byte order is the contract and
/// not an implementation detail of the derive.
pub fn render_answer(harness: Harness, event: &str, answer: &Answer) -> String {
    match answer {
        Answer::Allow | Answer::Cannot { .. } => "{}".to_owned(),
        Answer::Deny { reason } => match harness {
            Harness::Claude | Harness::Codex => format!(
                r#"{{"hookSpecificOutput":{{"hookEventName":{},"permissionDecision":"deny","permissionDecisionReason":{}}}}}"#,
                json_string(event),
                json_string(reason)
            ),
            Harness::Gemini => format!(r#"{{"decision":"deny","reason":{}}}"#, json_string(reason)),
        },
        Answer::Block { reason } => {
            format!(r#"{{"decision":"block","reason":{}}}"#, json_string(reason))
        }
    }
}

/// The exit code that carries the answer where a vendor reads the code rather than stdout.
pub fn exit_code(answer: &Answer) -> i32 {
    match answer {
        Answer::Allow => 0,
        Answer::Deny { .. } | Answer::Block { .. } => 2,
        Answer::Cannot { .. } => 3,
    }
}

/// A JSON string literal, quotes and escapes included. `Value`'s `Display` cannot fail, so
/// this stays total where `serde_json::to_string` would return a `Result`.
fn json_string(text: &str) -> String {
    Value::String(text.to_owned()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn payloads() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/payloads")
    }

    const CLAUDE_EVENTS: [&str; 26] = [
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "PermissionDenied",
        "PermissionRequest",
        "Notification",
        "UserPromptSubmit",
        "SessionStart",
        "SessionEnd",
        "Stop",
        "StopFailure",
        "SubagentStart",
        "SubagentStop",
        "PreCompact",
        "PostCompact",
        "Setup",
        "Elicitation",
        "ElicitationResult",
        "ConfigChange",
        "WorktreeCreate",
        "WorktreeRemove",
        "InstructionsLoaded",
        "FileChanged",
        "CwdChanged",
        "TeammateIdle",
        "TaskCompleted",
    ];

    const GEMINI_EVENTS: [&str; 11] = [
        "BeforeTool",
        "AfterTool",
        "BeforeAgent",
        "AfterAgent",
        "BeforeModel",
        "AfterModel",
        "BeforeToolSelection",
        "Notification",
        "SessionStart",
        "SessionEnd",
        "PreCompress",
    ];

    const CODEX_EVENTS: [&str; 10] = [
        "PreToolUse",
        "PermissionRequest",
        "PostToolUse",
        "PreCompact",
        "PostCompact",
        "SessionStart",
        "UserPromptSubmit",
        "SubagentStart",
        "SubagentStop",
        "Stop",
    ];

    #[test]
    fn from_str_accepts_exactly_the_three_names() {
        for (name, expected) in [
            ("claude", Harness::Claude),
            ("gemini", Harness::Gemini),
            ("codex", Harness::Codex),
        ] {
            match name.parse::<Harness>() {
                Ok(harness) => assert_eq!(harness, expected),
                Err(error) => panic!("{name} was refused: {error}"),
            }
        }
        for refused in [
            "",
            "Claude",
            "CLAUDE",
            "claude-code",
            "cursor",
            "gemini-cli",
            " codex",
        ] {
            assert!(
                refused.parse::<Harness>().is_err(),
                "{refused} was accepted as a harness"
            );
        }
    }

    #[test]
    fn name_and_display_agree_and_round_trip() {
        for harness in Harness::ALL {
            assert_eq!(harness.to_string(), harness.name());
            match harness.name().parse::<Harness>() {
                Ok(round_tripped) => assert_eq!(round_tripped, harness),
                Err(error) => panic!("{harness} did not round trip: {error}"),
            }
        }
        assert_eq!(Harness::ALL.len(), 3);
    }

    #[test]
    fn events_are_exactly_the_vendor_lists() {
        assert_eq!(Harness::Claude.events(), &CLAUDE_EVENTS[..]);
        assert_eq!(Harness::Claude.events().len(), 26);
        assert_eq!(Harness::Gemini.events(), &GEMINI_EVENTS[..]);
        assert_eq!(Harness::Gemini.events().len(), 11);
        assert_eq!(Harness::Codex.events(), &CODEX_EVENTS[..]);
        assert_eq!(Harness::Codex.events().len(), 10);
    }

    #[test]
    fn has_event_answers_for_the_vendor_that_owns_the_name() {
        assert!(Harness::Claude.has_event("PreToolUse"));
        assert!(Harness::Claude.has_event("PostToolUseFailure"));
        assert!(!Harness::Claude.has_event("BeforeTool"));
        assert!(!Harness::Claude.has_event("pretooluse"));
        assert!(Harness::Gemini.has_event("BeforeTool"));
        assert!(Harness::Gemini.has_event("PreCompress"));
        assert!(!Harness::Gemini.has_event("PreToolUse"));
        assert!(!Harness::Gemini.has_event("Stop"));
        assert!(Harness::Codex.has_event("PreToolUse"));
        assert!(Harness::Codex.has_event("Stop"));
        assert!(!Harness::Codex.has_event("SessionEnd"));
        assert!(!Harness::Codex.has_event("PostToolUseFailure"));
    }

    #[test]
    fn the_session_end_and_before_tool_events_are_the_vendors_own() {
        assert_eq!(Harness::Claude.session_end_event(), "SessionEnd");
        assert_eq!(Harness::Gemini.session_end_event(), "SessionEnd");
        assert_eq!(Harness::Codex.session_end_event(), "Stop");
        assert_eq!(Harness::Claude.before_tool_event(), "PreToolUse");
        assert_eq!(Harness::Gemini.before_tool_event(), "BeforeTool");
        assert_eq!(Harness::Codex.before_tool_event(), "PreToolUse");
        for harness in Harness::ALL {
            assert!(harness.has_event(harness.session_end_event()));
            assert!(harness.has_event(harness.before_tool_event()));
        }
    }

    #[test]
    fn the_bundle_root_variable_is_the_vendors_own() {
        assert_eq!(
            Harness::Claude.bundle_root_variable(),
            "${CLAUDE_PLUGIN_ROOT}"
        );
        assert_eq!(Harness::Gemini.bundle_root_variable(), "${extensionPath}");
        assert_eq!(
            Harness::Codex.bundle_root_variable(),
            "${CLAUDE_PLUGIN_ROOT}"
        );
    }

    #[test]
    fn every_fixture_payload_parses_into_the_fields_it_carries() {
        let mut seen = 0;
        for harness in Harness::ALL {
            let dir = payloads().join(harness.name());
            let entries =
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
            for entry in entries {
                let path = entry.expect("a readable directory entry").path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                seen += 1;
                let json = std::fs::read_to_string(&path).expect("a readable fixture");
                let payload = parse_payload(harness, &json)
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .expect("a fixture file name");
                // A fixture is named by the event it carries; a variant of that event
                // prefixes the name with what makes it a variant.
                let expected_event = match stem.rsplit_once('-') {
                    Some((_, event)) => event,
                    None => stem,
                };

                assert_eq!(payload.harness, harness);
                assert_eq!(payload.event, expected_event, "{}", path.display());
                assert!(
                    harness.has_event(&payload.event),
                    "{} names an event {harness} does not have",
                    path.display()
                );

                let raw: serde_json::Value =
                    serde_json::from_str(&json).expect("valid fixture JSON");
                let text = |key: &str| {
                    raw.get(key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                };
                assert_eq!(payload.session_id, text("session_id"));
                assert_eq!(payload.cwd, text("cwd").map(PathBuf::from));
                assert_eq!(
                    payload.transcript_path,
                    text("transcript_path").map(PathBuf::from)
                );
                assert_eq!(payload.tool_name, text("tool_name"));
                assert_eq!(payload.tool_input, raw.get("tool_input").cloned());
                assert_eq!(payload.tool_response, raw.get("tool_response").cloned());
                assert_eq!(payload.tool_call_id, text("tool_use_id"));
                assert_eq!(payload.tool_error, text("error"));
                assert_eq!(payload.model, text("model"));
                assert_eq!(payload.turn_id, text("turn_id"));
                assert_eq!(payload.agent_id, text("agent_id"));
                assert_eq!(payload.reason, text("reason"));
                assert_eq!(payload.trigger, text("trigger"));
                assert_eq!(payload.source, text("source"));
                assert_eq!(payload.raw, raw);

                assert!(payload.session_id.is_some(), "{}", path.display());
                // A fixture's name says which working directory it carries: the leaky ones
                // a path outside the repository, the `no-cwd-` variant none at all.
                let expected_cwd = if stem.starts_with("no-cwd-") {
                    None
                } else if stem.starts_with("leaky-") {
                    Some(Path::new("/Users/someone/code/billing"))
                } else {
                    Some(Path::new("/work/repo"))
                };
                assert_eq!(payload.cwd.as_deref(), expected_cwd, "{}", path.display());
            }
        }
        assert_eq!(
            seen, 26,
            "one fixture per harness event, one leaky each, and the two a write cannot be \
             judged from"
        );
    }

    #[test]
    fn the_tool_events_carry_a_tool_name_and_input() {
        let json = std::fs::read_to_string(payloads().join("claude/PreToolUse.json"))
            .expect("the Claude PreToolUse fixture");
        let payload = parse_payload(Harness::Claude, &json).expect("a parsed payload");
        assert_eq!(payload.tool_name.as_deref(), Some("Bash"));
        assert_eq!(
            payload
                .tool_input
                .as_ref()
                .and_then(|i| i.get("command"))
                .and_then(serde_json::Value::as_str),
            Some("cargo test --all-targets")
        );

        let json = std::fs::read_to_string(payloads().join("codex/PreToolUse.json"))
            .expect("the Codex PreToolUse fixture");
        let payload = parse_payload(Harness::Codex, &json).expect("a parsed payload");
        assert_eq!(payload.tool_name.as_deref(), Some("shell"));
        assert_eq!(payload.turn_id.as_deref(), Some("turn_0007"));

        let json = std::fs::read_to_string(payloads().join("gemini/BeforeTool.json"))
            .expect("the Gemini BeforeTool fixture");
        let payload = parse_payload(Harness::Gemini, &json).expect("a parsed payload");
        assert_eq!(payload.tool_name.as_deref(), Some("run_shell_command"));
    }

    #[test]
    fn a_failure_payload_carries_the_error() {
        let json = std::fs::read_to_string(payloads().join("claude/PostToolUseFailure.json"))
            .expect("the Claude PostToolUseFailure fixture");
        let payload = parse_payload(Harness::Claude, &json).expect("a parsed payload");
        assert_eq!(
            payload.tool_error.as_deref(),
            Some("String to replace not found in file.")
        );
    }

    #[test]
    fn a_payload_that_is_not_an_object_is_a_harness_error() {
        for json in ["[]", "\"PreToolUse\"", "12", "null", "true"] {
            match parse_payload(Harness::Claude, json) {
                Err(Error::Harness { .. }) => {}
                other => panic!("{json} gave {other:?}"),
            }
        }
    }

    #[test]
    fn a_payload_without_hook_event_name_is_a_harness_error() {
        for json in [
            r#"{"session_id":"s","cwd":"/work/repo"}"#,
            r#"{"hook_event_name":null}"#,
            r#"{"hook_event_name":7}"#,
        ] {
            match parse_payload(Harness::Claude, json) {
                Err(Error::Harness { .. }) => {}
                other => panic!("{json} gave {other:?}"),
            }
        }
    }

    #[test]
    fn unparseable_json_is_a_json_error() {
        match parse_payload(Harness::Claude, "{") {
            Err(Error::Json { path: None, .. }) => {}
            other => panic!("gave {other:?}"),
        }
    }

    #[test]
    fn a_field_of_the_wrong_type_is_read_as_absent() {
        let json = r#"{"hook_event_name":"PreToolUse","session_id":42,"cwd":[],"tool_name":null}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert_eq!(payload.session_id, None);
        assert_eq!(payload.cwd, None);
        assert_eq!(payload.tool_name, None);
        assert!(payload.raw.get("session_id").is_some());
    }

    #[test]
    fn render_answer_is_byte_exact_per_vendor() {
        let deny = Answer::Deny {
            reason: "plotplot: --no-verify is refused at the boundary; run the gate instead."
                .to_owned(),
        };
        let block = Answer::Block {
            reason: "plotplot: the loop's checks are not stamped; run `tend2 verify`.".to_owned(),
        };

        assert_eq!(
            render_answer(Harness::Claude, "PreToolUse", &deny),
            r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"plotplot: --no-verify is refused at the boundary; run the gate instead."}}"#
        );
        assert_eq!(
            render_answer(Harness::Claude, "Stop", &block),
            r#"{"decision":"block","reason":"plotplot: the loop's checks are not stamped; run `tend2 verify`."}"#
        );
        assert_eq!(
            render_answer(Harness::Gemini, "BeforeTool", &deny),
            r#"{"decision":"deny","reason":"plotplot: --no-verify is refused at the boundary; run the gate instead."}"#
        );
        assert_eq!(
            render_answer(Harness::Gemini, "AfterAgent", &block),
            r#"{"decision":"block","reason":"plotplot: the loop's checks are not stamped; run `tend2 verify`."}"#
        );
        assert_eq!(
            render_answer(Harness::Codex, "PreToolUse", &deny),
            r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"plotplot: --no-verify is refused at the boundary; run the gate instead."}}"#
        );
        assert_eq!(
            render_answer(Harness::Codex, "Stop", &block),
            r#"{"decision":"block","reason":"plotplot: the loop's checks are not stamped; run `tend2 verify`."}"#
        );
    }

    #[test]
    fn allow_renders_the_empty_object_everywhere() {
        for harness in Harness::ALL {
            for event in [harness.before_tool_event(), harness.session_end_event()] {
                assert_eq!(render_answer(harness, event, &Answer::Allow), "{}");
            }
        }
    }

    #[test]
    fn cannot_judge_renders_no_decision_and_leaves_the_reason_to_stderr() {
        let cannot = Answer::Cannot {
            reason: "weeder is not in .plotplot/bin".to_owned(),
        };
        for harness in Harness::ALL {
            assert_eq!(
                render_answer(harness, harness.before_tool_event(), &cannot),
                "{}"
            );
        }
    }

    #[test]
    fn a_reason_with_json_punctuation_is_escaped() {
        let deny = Answer::Deny {
            reason: "\"garden.lock\" is written by the stem\n\tnot by hand".to_owned(),
        };
        let rendered = render_answer(Harness::Gemini, "BeforeTool", &deny);
        assert_eq!(
            rendered,
            r#"{"decision":"deny","reason":"\"garden.lock\" is written by the stem\n\tnot by hand"}"#
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&rendered).expect("the rendered answer is JSON");
        assert_eq!(
            parsed.get("reason").and_then(serde_json::Value::as_str),
            Some("\"garden.lock\" is written by the stem\n\tnot by hand")
        );
    }

    #[test]
    fn exit_codes_are_allow_zero_deny_two_block_two_cannot_three() {
        assert_eq!(exit_code(&Answer::Allow), 0);
        assert_eq!(
            exit_code(&Answer::Deny {
                reason: String::new()
            }),
            2
        );
        assert_eq!(
            exit_code(&Answer::Block {
                reason: String::new()
            }),
            2
        );
        assert_eq!(
            exit_code(&Answer::Cannot {
                reason: String::new()
            }),
            3
        );
    }
}
