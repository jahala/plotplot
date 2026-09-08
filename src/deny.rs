//! The hard-limit deny list: the two things no agent may do in a planted repository.
//!
//! One pure decision over one before-tool payload. It refuses a `git commit`, `git push` or
//! `git merge` that carries `--no-verify` or `-n`, and it refuses a write to the four paths
//! the stem owns: `garden.lock`, `.githooks/`, `.plotplot/bin/` and `.plotplot/beds/`.
//! Nothing else is denied here — a bed's own rules are the bed's, and the dispatcher fans out
//! to them separately.
//!
//! Quoting is not interpreted. `git commit -m "no --no-verify here"` is refused, because the
//! token is there and a boundary that can be talked around is not a boundary.

use crate::friction::{
    RepoPath, bare_tool_name, input_paths, redirection_targets, repo_path, shell_command,
    shell_segments, shell_tokens, unquote,
};
use crate::harness::{Answer, Harness, Payload};
use crate::layout::{GARDEN_LOCK, GITHOOKS_DIR};

/// The git subcommands whose verification the gate depends on.
const GUARDED_VERBS: [&str; 3] = ["commit", "push", "merge"];

/// The paths only the stem writes, as prefixes of a repo-relative path.
const GUARDED_PATHS: [&str; 4] = [GARDEN_LOCK, GITHOOKS_DIR, ".plotplot/bin", ".plotplot/beds"];

/// The tools that write a file, by vendor. `tilth_write` is an MCP tool on every harness.
fn is_writing_tool(harness: Harness, tool: &str) -> bool {
    let bare = bare_tool_name(tool);
    if bare == "tilth_write" {
        return true;
    }
    match harness {
        Harness::Claude => matches!(bare, "Write" | "Edit" | "MultiEdit"),
        Harness::Gemini => matches!(bare, "write_file" | "replace"),
        Harness::Codex => bare == "apply_patch",
    }
}

/// What the stem says about this tool call.
///
/// [`Answer::Allow`] for every event but the vendor's before-tool event, because a decision
/// on any other event cannot stop anything and a hook that denies too late is noise.
pub fn decide(payload: &Payload) -> Answer {
    if payload.event != payload.harness.before_tool_event() {
        return Answer::Allow;
    }
    let Some(tool) = payload.tool_name.as_deref() else {
        return Answer::Allow;
    };

    if let Some(command) = shell_command(payload) {
        if let Some(verb) = skipped_verification(&command) {
            return Answer::Deny {
                reason: skip_reason(&verb),
            };
        }
        let tokens = shell_tokens(&command);
        if let Some(guarded) = guarded_target(payload, redirection_targets(&tokens)) {
            return Answer::Deny {
                reason: guarded_reason(&guarded),
            };
        }
        return Answer::Allow;
    }

    if is_writing_tool(payload.harness, tool) {
        let paths = payload
            .tool_input
            .as_ref()
            .map(input_paths)
            .unwrap_or_default();
        if let Some(guarded) = guarded_target(payload, paths) {
            return Answer::Deny {
                reason: guarded_reason(&guarded),
            };
        }
    }
    Answer::Allow
}

/// The git subcommand whose verification this command skips, when it skips one.
///
/// The flag has to belong to the guarded subcommand's own invocation: `git log -n 3` is a
/// different command from `git commit`, so the scan stops where the command does.
fn skipped_verification(command: &str) -> Option<String> {
    let tokens = shell_tokens(command);
    for segment in shell_segments(&tokens) {
        for (index, token) in segment.iter().enumerate() {
            if token != "git" {
                continue;
            }
            let Some(verb) = segment.get(index + 1) else {
                continue;
            };
            if !GUARDED_VERBS.contains(&verb.as_str()) {
                continue;
            }
            let rest = segment[index + 2..]
                .iter()
                .take_while(|token| token.as_str() != "git");
            for argument in rest {
                if skips_verification(argument) {
                    return Some(verb.clone());
                }
            }
        }
    }
    None
}

/// Whether one argument is `--no-verify` or a short-flag cluster carrying `-n`.
fn skips_verification(argument: &str) -> bool {
    let token = unquote(argument);
    if token == "--no-verify" {
        return true;
    }
    match token.strip_prefix('-') {
        Some(short) if !short.is_empty() && short.chars().all(|c| c.is_ascii_alphabetic()) => {
            short.contains('n')
        }
        _ => false,
    }
}

/// The first of these targets that the stem owns, as its repo-relative path.
fn guarded_target(payload: &Payload, targets: Vec<String>) -> Option<String> {
    // Without a `cwd` only a relative target can be judged; an absolute one is then outside
    // every base the stem knows, and is left alone rather than guessed at.
    let base = payload.cwd.clone().unwrap_or_default();
    for target in targets {
        let RepoPath::Inside(relative) = repo_path(&base, &target) else {
            continue;
        };
        if GUARDED_PATHS
            .iter()
            .any(|guarded| relative == *guarded || relative.starts_with(&format!("{guarded}/")))
        {
            return Some(relative);
        }
    }
    None
}

/// One sentence: the rule, then the way through it.
fn skip_reason(verb: &str) -> String {
    format!(
        "plotplot: `git {verb}` carrying --no-verify or -n skips the gate the garden enforces at \
         the boundary; run `plotplot check`, fix what it names, and {verb} without the flag \
         (`--dry-run` if a dry run was what you meant)."
    )
}

/// One sentence: which path is the stem's, and the command that writes it.
fn guarded_reason(relative: &str) -> String {
    let (owner, fix) = if relative == GARDEN_LOCK {
        ("the pinned judges", "`plotplot lock update`")
    } else if relative.starts_with(".plotplot/bin") {
        ("the fetched judges", "`plotplot lock verify`")
    } else if relative.starts_with(".plotplot/beds") {
        (
            "the manifests read from verified artifacts",
            "`plotplot init`",
        )
    } else {
        ("the git hooks the garden installs", "`plotplot init`")
    };
    format!(
        "plotplot: {relative} holds {owner} and is written by the stem, never by hand; \
         run {fix} instead."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::parse_payload;
    use serde_json::json;
    use std::path::{Path, PathBuf};

    fn payloads() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/payloads")
    }

    fn payload(harness: Harness, event: &str, tool: &str, input: serde_json::Value) -> Payload {
        let json = json!({
            "hook_event_name": event,
            "session_id": "6f3c1b2a",
            "cwd": "/work/repo",
            "tool_name": tool,
            "tool_input": input,
        })
        .to_string();
        parse_payload(harness, &json).expect("a parsed payload")
    }

    /// The same shell command as each vendor's own shell tool spells it.
    fn shell(harness: Harness, command: &str) -> Payload {
        let (tool, input) = match harness {
            Harness::Claude => ("Bash", json!({ "command": command })),
            Harness::Gemini => ("run_shell_command", json!({ "command": command })),
            Harness::Codex => ("shell", json!({ "command": ["bash", "-lc", command] })),
        };
        payload(harness, harness.before_tool_event(), tool, input)
    }

    fn reason(answer: &Answer) -> String {
        match answer {
            Answer::Deny { reason } => reason.clone(),
            other => panic!("expected a denial, got {other:?}"),
        }
    }

    #[test]
    fn no_verify_is_refused_on_every_harness_shell_tool() {
        for harness in Harness::ALL {
            let answer = decide(&shell(harness, "git commit --no-verify -m \"wip\""));
            let reason = reason(&answer);
            assert!(reason.contains("--no-verify"), "{harness}: {reason}");
            assert!(reason.contains("git commit"), "{harness}: {reason}");
            assert!(reason.contains("plotplot check"), "{harness}: {reason}");
        }
    }

    #[test]
    fn the_short_flag_is_refused_the_same_way() {
        assert!(matches!(
            decide(&shell(Harness::Claude, "git commit -n -m x")),
            Answer::Deny { .. }
        ));
        assert!(matches!(
            decide(&shell(Harness::Claude, "git commit -an -m x")),
            Answer::Deny { .. }
        ));
        assert!(matches!(
            decide(&shell(Harness::Gemini, "git push -n origin main")),
            Answer::Deny { .. }
        ));
        assert!(matches!(
            decide(&shell(Harness::Codex, "git merge --no-verify feature")),
            Answer::Deny { .. }
        ));
    }

    #[test]
    fn a_quoted_flag_is_still_the_flag() {
        assert!(matches!(
            decide(&shell(
                Harness::Claude,
                "git commit -m \"no --no-verify here\""
            )),
            Answer::Deny { .. }
        ));
    }

    #[test]
    fn the_flag_has_to_belong_to_the_guarded_command() {
        for allowed in [
            "git log -n 3",
            "git status -s",
            "git commit -m x && git log -n 3",
            "cargo test -- -n",
            "npm run build -n",
            "git diff --stat",
        ] {
            assert_eq!(
                decide(&shell(Harness::Claude, allowed)),
                Answer::Allow,
                "{allowed}"
            );
        }
        assert!(
            matches!(
                decide(&shell(
                    Harness::Claude,
                    "git log -n 3 && git commit --no-verify -m x"
                )),
                Answer::Deny { .. }
            ),
            "a denial after an allowed command is still a denial"
        );
    }

    #[test]
    fn a_write_to_a_path_the_stem_owns_is_refused() {
        let cases = [
            (
                Harness::Claude,
                "Write",
                json!({"file_path": "/work/repo/garden.lock"}),
                "garden.lock",
            ),
            (
                Harness::Claude,
                "Write",
                json!({"file_path": "garden.lock"}),
                "garden.lock",
            ),
            (
                Harness::Claude,
                "Edit",
                json!({"file_path": "/work/repo/.githooks/pre-commit"}),
                ".githooks/pre-commit",
            ),
            (
                Harness::Claude,
                "MultiEdit",
                json!({"file_path": ".plotplot/beds/weeder/garden.json"}),
                ".plotplot/beds/weeder/garden.json",
            ),
            (
                Harness::Gemini,
                "write_file",
                json!({"file_path": "/work/repo/.plotplot/bin/weeder"}),
                ".plotplot/bin/weeder",
            ),
            (
                Harness::Gemini,
                "replace",
                json!({"file_path": "garden.lock"}),
                "garden.lock",
            ),
            (
                Harness::Codex,
                "apply_patch",
                json!({"patch": "*** Begin Patch\n*** Update File: garden.lock\n*** End Patch\n"}),
                "garden.lock",
            ),
        ];
        for (harness, tool, input, guarded) in cases {
            let expected = guarded.to_owned();
            let answer = decide(&payload(harness, harness.before_tool_event(), tool, input));
            let reason = reason(&answer);
            assert!(
                reason.starts_with("plotplot: "),
                "{harness} {tool}: {reason}"
            );
            assert!(reason.contains(&expected), "{harness} {tool}: {reason}");
            assert!(
                reason.contains("run `plotplot "),
                "{harness} {tool}: {reason}"
            );
        }
    }

    #[test]
    fn the_tilth_write_tool_is_guarded_on_every_harness() {
        for harness in Harness::ALL {
            for tool in ["tilth_write", "mcp__tilth__tilth_write"] {
                let answer = decide(&payload(
                    harness,
                    harness.before_tool_event(),
                    tool,
                    json!({"path": ".githooks/pre-push"}),
                ));
                assert!(
                    matches!(answer, Answer::Deny { .. }),
                    "{harness} {tool} was allowed"
                );
            }
        }
    }

    #[test]
    fn a_shell_redirection_into_a_guarded_path_is_refused() {
        for command in [
            "echo x > .plotplot/bin/weeder",
            "echo x >.plotplot/bin/weeder",
            "echo x >> /work/repo/.githooks/pre-commit",
            "echo x | tee garden.lock",
            "printf 'season' > \"garden.lock\"",
        ] {
            assert!(
                matches!(
                    decide(&shell(Harness::Claude, command)),
                    Answer::Deny { .. }
                ),
                "{command} was allowed"
            );
        }
    }

    #[test]
    fn a_write_anywhere_else_is_allowed() {
        for input in [
            json!({"file_path": "/work/repo/src/main.rs"}),
            json!({"file_path": "src/main.rs"}),
            json!({"file_path": "/work/repo/docs/plans/stem.md"}),
            json!({"file_path": "/work/repo/.plotplot/friction/2026-09.jsonl"}),
            json!({"file_path": "/somewhere/else/garden.lock"}),
        ] {
            assert_eq!(
                decide(&payload(
                    Harness::Claude,
                    "PreToolUse",
                    "Write",
                    input.clone()
                )),
                Answer::Allow,
                "{input}"
            );
        }
        assert_eq!(
            decide(&shell(Harness::Claude, "echo x > src/main.rs")),
            Answer::Allow
        );
    }

    #[test]
    fn a_read_of_a_guarded_path_is_not_a_write() {
        assert_eq!(
            decide(&payload(
                Harness::Claude,
                "PreToolUse",
                "Read",
                json!({"file_path": "/work/repo/garden.lock"})
            )),
            Answer::Allow
        );
        assert_eq!(
            decide(&shell(Harness::Claude, "cat garden.lock")),
            Answer::Allow
        );
    }

    #[test]
    fn only_the_before_tool_event_can_deny() {
        for harness in Harness::ALL {
            for event in harness.events() {
                if *event == harness.before_tool_event() {
                    continue;
                }
                let answer = decide(&payload(
                    harness,
                    event,
                    "Write",
                    json!({"file_path": "/work/repo/garden.lock"}),
                ));
                assert_eq!(answer, Answer::Allow, "{harness} {event}");
            }
        }
    }

    #[test]
    fn every_fixture_payload_that_is_not_a_before_tool_event_is_allowed() {
        let mut seen = 0;
        for harness in Harness::ALL {
            let directory = payloads().join(harness.name());
            let entries = std::fs::read_dir(&directory)
                .unwrap_or_else(|e| panic!("{}: {e}", directory.display()));
            for entry in entries {
                let path = entry.expect("a readable directory entry").path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let json = std::fs::read_to_string(&path).expect("a readable fixture");
                let payload = parse_payload(harness, &json).expect("a parsed fixture");
                if payload.event == harness.before_tool_event() {
                    continue;
                }
                seen += 1;
                assert_eq!(decide(&payload), Answer::Allow, "{}", path.display());
            }
        }
        assert_eq!(seen, 21, "every fixture but the three before-tool ones");
    }

    #[test]
    fn the_gates_the_fixtures_run_are_allowed() {
        for harness in Harness::ALL {
            let directory = payloads().join(harness.name());
            let name = format!("{}.json", harness.before_tool_event());
            let json = std::fs::read_to_string(directory.join(&name))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            let payload = parse_payload(harness, &json).expect("a parsed fixture");
            assert_eq!(decide(&payload), Answer::Allow, "{harness} {name}");
        }
    }

    #[test]
    fn without_a_working_directory_only_a_relative_target_can_be_judged() {
        let relative = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Write",
            "tool_input":{"file_path":"garden.lock"}}"#;
        let payload = parse_payload(Harness::Claude, relative).expect("a parsed payload");
        assert!(matches!(decide(&payload), Answer::Deny { .. }));

        let absolute = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Write",
            "tool_input":{"file_path":"/work/repo/garden.lock"}}"#;
        let payload = parse_payload(Harness::Claude, absolute).expect("a parsed payload");
        assert_eq!(decide(&payload), Answer::Allow);
    }

    #[test]
    fn an_amend_is_not_a_skipped_verification() {
        for allowed in [
            "git commit --amend --no-edit",
            "git commit -am \"work\"",
            "git push -u origin main",
            "git push --tags",
            "git merge --squash feature",
        ] {
            assert_eq!(
                decide(&shell(Harness::Claude, allowed)),
                Answer::Allow,
                "{allowed}"
            );
        }
    }

    #[test]
    fn a_payload_with_no_tool_or_no_input_is_allowed_rather_than_guessed() {
        let json = r#"{"hook_event_name":"PreToolUse","session_id":"s","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert_eq!(decide(&payload), Answer::Allow);

        let json = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Write"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert_eq!(decide(&payload), Answer::Allow);
    }
}
