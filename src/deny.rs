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
//!
//! And a hard limit fails closed. A write whose target this module cannot work out — a
//! payload with no `tool_input`, an input naming no path, an absolute path with no `cwd` to
//! judge it against — is refused rather than allowed, because a boundary that answers
//! "allow" whenever it is blindfolded is a boundary anything can blindfold. Reads, searches
//! and shell commands without a redirection write nothing, so the gap costs them nothing and
//! they are still allowed.

use std::path::Path;

use serde_json::Value;

use crate::friction::{
    RepoPath, bare_tool_name, input_paths, patch_paths, redirection_targets, repo_path,
    shell_command, shell_segments, shell_tokens, unquote,
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

/// Which hard limit refused a tool call.
///
/// The friction profile's `tool.denied` record carries `plotplot.rule`, so the refusal has to
/// be nameable and not only explainable: the sentence a vendor shows the agent is prose, and
/// prose is not something a ledger can count. A blindfolded refusal has its own name, so the
/// ledger can tell a write that aimed at the stem's own files from a write nobody could aim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// A `git commit`, `git push` or `git merge` carrying `--no-verify` or `-n`.
    SkippedVerification,
    /// A write to `garden.lock`, `.githooks/`, `.plotplot/bin/` or `.plotplot/beds/`.
    StemOwnedPath,
    /// A write whose target the payload does not let this module work out.
    WriteTargetUnjudged,
}

impl Rule {
    /// The rule's name as the friction journal records it, in the `deny.` namespace so a
    /// reader of the ledger can tell the stem's own refusals from a bed's.
    pub fn name(self) -> &'static str {
        match self {
            Rule::SkippedVerification => "deny.no-verify",
            Rule::StemOwnedPath => "deny.stem-owned-path",
            Rule::WriteTargetUnjudged => "deny.write-target-unjudged",
        }
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(self.name())
    }
}

/// What the stem says about this tool call.
///
/// [`Answer::Allow`] for every event but the vendor's before-tool event, because a decision
/// on any other event cannot stop anything and a hook that denies too late is noise.
pub fn decide(payload: &Payload) -> Answer {
    decide_with_rule(payload).0
}

/// The same decision, with the rule that made it when the answer is a refusal.
///
/// [`decide`] is the shape the module contract fixes and the shape a caller who only needs an
/// answer wants; the dispatcher needs the name too, so both come from here and the two can
/// never disagree.
pub fn decide_with_rule(payload: &Payload) -> (Answer, Option<Rule>) {
    if payload.event != payload.harness.before_tool_event() {
        return (Answer::Allow, None);
    }
    let Some(tool) = payload.tool_name.as_deref() else {
        return (Answer::Allow, None);
    };

    if let Some(command) = shell_command(payload) {
        if let Some(verb) = skipped_verification(&command) {
            return (
                Answer::Deny {
                    reason: skip_reason(&verb),
                },
                Some(Rule::SkippedVerification),
            );
        }
        // A command with no redirection writes no file, so there is no target to miss.
        let tokens = shell_tokens(&command);
        let mut targets = redirection_targets(&tokens);
        // Except `apply_patch`, which writes through a patch rather than through a
        // redirection wherever it is called from.
        if tokens.first().map(String::as_str) == Some(APPLY_PATCH) {
            if let Some(input) = payload.tool_input.as_ref() {
                targets.extend(patch_targets(input));
            }
        }
        return answer(judge(payload, targets));
    }

    if is_writing_tool(payload.harness, tool) {
        let Some(input) = payload.tool_input.as_ref() else {
            return answer(Target::Unjudged(NO_PATH_REASON.to_owned()));
        };
        let mut paths = input_paths(input);
        if bare_tool_name(tool) == APPLY_PATCH {
            paths.extend(patch_targets(input));
            paths.sort();
            paths.dedup();
        }
        if paths.is_empty() {
            return answer(Target::Unjudged(NO_PATH_REASON.to_owned()));
        }
        return answer(judge(payload, paths));
    }
    (Answer::Allow, None)
}

/// The tool that writes by patch rather than by naming a file.
const APPLY_PATCH: &str = "apply_patch";

/// The files a patch names, wherever in this tool input the patch text sits.
///
/// Codex takes `apply_patch` as a freeform tool on every model profile of the installed
/// 0.133.0 (`"apply_patch_tool_type": "freeform"`, six of six), so the patch reaches a hook
/// as text and not under a field the stem could name: it may be the whole argument, or sit
/// under `input`, or under `patch`, or inside the command array. What the vendor's own
/// grammar does fix is the patch itself, whose file operations are `*** Add File:`,
/// `*** Update File:` and `*** Delete File:` headers. So every string in the input is read
/// for those headers, and the shape around them is left to the vendor to change as it likes.
fn patch_targets(input: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    read_patch_headers(input, &mut paths);
    paths
}

/// Every `*** … File:` header in every string under this value.
///
/// Depth is bounded by `serde_json`'s own nesting limit, which refuses a document deeper than
/// it will recurse, so a payload cannot spend this module's stack.
fn read_patch_headers(value: &Value, paths: &mut Vec<String>) {
    match value {
        Value::String(text) => paths.extend(patch_paths(text)),
        Value::Array(items) => {
            for item in items {
                read_patch_headers(item, paths);
            }
        }
        Value::Object(fields) => {
            for field in fields.values() {
                read_patch_headers(field, paths);
            }
        }
        _ => {}
    }
}

/// What this module could see of a write's targets.
enum Target {
    /// A target the stem owns, as its repo-relative path.
    Guarded(String),
    /// Every target was judged, and none of them is the stem's.
    Clear,
    /// A target could not be judged, for the reason given.
    Unjudged(String),
}

/// The answer a judged set of targets earns, and the rule behind it.
fn answer(target: Target) -> (Answer, Option<Rule>) {
    match target {
        Target::Guarded(relative) => (
            Answer::Deny {
                reason: guarded_reason(&relative),
            },
            Some(Rule::StemOwnedPath),
        ),
        Target::Unjudged(reason) => (Answer::Deny { reason }, Some(Rule::WriteTargetUnjudged)),
        Target::Clear => (Answer::Allow, None),
    }
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

/// Judge these targets: the first one the stem owns, or the first one that cannot be judged.
///
/// A relative target is judged against the payload's working directory, and against the
/// repository the dispatcher was started in when the payload names none. An absolute target
/// with no working directory has no base at all: there is nothing to say whether it is inside
/// this repository or another one, so it is unjudged rather than guessed at.
fn judge(payload: &Payload, targets: Vec<String>) -> Target {
    for target in targets {
        let judged = match (payload.cwd.as_deref(), Path::new(&target).is_absolute()) {
            // No base, and a path that needs one: there is nothing here to say whether this
            // is this repository's `garden.lock` or another repository's.
            (None, true) => return Target::Unjudged(no_working_directory_reason(&target)),
            // No base, and a path that needs none: a relative target is already spelled from
            // the directory the dispatcher was started in, which is the planted repository.
            (None, false) => repo_path(Path::new(""), &target),
            (Some(base), _) => repo_path(base, &target),
        };
        let RepoPath::Inside(relative) = judged else {
            continue;
        };
        if GUARDED_PATHS
            .iter()
            .any(|guarded| relative == *guarded || relative.starts_with(&format!("{guarded}/")))
        {
            return Target::Guarded(relative);
        }
    }
    Target::Clear
}

/// One sentence: the rule, then the way through it.
fn skip_reason(verb: &str) -> String {
    format!(
        "plotplot: `git {verb}` carrying --no-verify or -n skips the gate the garden enforces at \
         the boundary; run `plotplot check`, fix what it names, and {verb} without the flag \
         (`--dry-run` if a dry run was what you meant)."
    )
}

/// The reason a write earns when the payload never names a path at all.
const NO_PATH_REASON: &str = "plotplot: cannot judge the target of a write: the payload names \
     no path; a boundary that cannot see the target refuses it";

/// The reason a write earns when the payload gives no working directory to judge it against.
fn no_working_directory_reason(path: &str) -> String {
    format!(
        "plotplot: cannot judge the target of a write: the payload has no working directory \
         to judge {path} against"
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
    fn without_a_working_directory_a_relative_target_is_still_judged() {
        let relative = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Write",
            "tool_input":{"file_path":"garden.lock"}}"#;
        let payload = parse_payload(Harness::Claude, relative).expect("a parsed payload");
        assert!(matches!(decide(&payload), Answer::Deny { .. }));

        let elsewhere = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Write",
            "tool_input":{"file_path":"src/main.rs"}}"#;
        let payload = parse_payload(Harness::Claude, elsewhere).expect("a parsed payload");
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
    fn a_payload_naming_no_tool_at_all_is_allowed_rather_than_guessed() {
        let json = r#"{"hook_event_name":"PreToolUse","session_id":"s","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert_eq!(decide(&payload), Answer::Allow);
    }

    // -----------------------------------------------------------------------------------
    // a boundary that cannot see the target refuses it
    // -----------------------------------------------------------------------------------

    /// The reason a write whose target the payload never names earns.
    const NO_PATH_REASON: &str = "plotplot: cannot judge the target of a write: the payload \
         names no path; a boundary that cannot see the target refuses it";

    /// The reason a write the payload gives no working directory for earns.
    fn no_cwd_reason(path: &str) -> String {
        format!(
            "plotplot: cannot judge the target of a write: the payload has no working \
             directory to judge {path} against"
        )
    }

    /// Every vendor's own writing tools, and the MCP one they share.
    const WRITING_TOOLS: [(Harness, &str); 8] = [
        (Harness::Claude, "Write"),
        (Harness::Claude, "Edit"),
        (Harness::Claude, "MultiEdit"),
        (Harness::Gemini, "write_file"),
        (Harness::Gemini, "replace"),
        (Harness::Codex, "apply_patch"),
        (Harness::Claude, "tilth_write"),
        (Harness::Claude, "mcp__tilth__tilth_write"),
    ];

    #[test]
    fn a_write_whose_payload_carries_no_input_is_refused() {
        for (harness, tool) in WRITING_TOOLS {
            let json = format!(
                r#"{{"hook_event_name":"{}","session_id":"s","cwd":"/work/repo",
                    "tool_name":"{tool}"}}"#,
                harness.before_tool_event()
            );
            let payload = parse_payload(harness, &json).expect("a parsed payload");
            let (answer, rule) = decide_with_rule(&payload);
            assert_eq!(reason(&answer), NO_PATH_REASON, "{harness} {tool}");
            assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{harness} {tool}");
        }
    }

    #[test]
    fn a_write_whose_input_names_no_path_is_refused() {
        for (harness, tool) in WRITING_TOOLS {
            for input in [
                json!({}),
                json!({"content": "season = \"2026.09\"\n"}),
                json!({"old_string": "a", "new_string": "b"}),
                json!({"patch": "*** Begin Patch\n*** End Patch\n"}),
            ] {
                let payload = payload(harness, harness.before_tool_event(), tool, input.clone());
                let (answer, rule) = decide_with_rule(&payload);
                assert_eq!(reason(&answer), NO_PATH_REASON, "{harness} {tool} {input}");
                assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{harness} {tool}");
            }
        }
    }

    #[test]
    fn a_write_with_an_absolute_target_and_no_working_directory_is_refused() {
        for (harness, tool) in WRITING_TOOLS {
            let json = format!(
                r#"{{"hook_event_name":"{}","session_id":"s","tool_name":"{tool}",
                    "tool_input":{{"file_path":"/work/repo/src/main.rs"}}}}"#,
                harness.before_tool_event()
            );
            let payload = parse_payload(harness, &json).expect("a parsed payload");
            let (answer, rule) = decide_with_rule(&payload);
            assert_eq!(
                reason(&answer),
                no_cwd_reason("/work/repo/src/main.rs"),
                "{harness} {tool}"
            );
            assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{harness} {tool}");
        }
    }

    #[test]
    fn a_redirection_with_an_absolute_target_and_no_working_directory_is_refused() {
        for harness in Harness::ALL {
            let input = match harness {
                Harness::Codex => json!({"command": ["bash", "-lc", "echo x > /work/repo/out"]}),
                _ => json!({"command": "echo x > /work/repo/out"}),
            };
            let tool = match harness {
                Harness::Claude => "Bash",
                Harness::Gemini => "run_shell_command",
                Harness::Codex => "shell",
            };
            let json = serde_json::json!({
                "hook_event_name": harness.before_tool_event(),
                "session_id": "s",
                "tool_name": tool,
                "tool_input": input,
            })
            .to_string();
            let payload = parse_payload(harness, &json).expect("a parsed payload");
            let (answer, rule) = decide_with_rule(&payload);
            assert_eq!(
                reason(&answer),
                no_cwd_reason("/work/repo/out"),
                "{harness}"
            );
            assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{harness}");
        }
    }

    #[test]
    fn a_shell_command_with_no_redirection_needs_no_working_directory() {
        for command in ["cargo test", "cat /work/repo/src/main.rs", "git status -s"] {
            let json = serde_json::json!({
                "hook_event_name": "PreToolUse",
                "session_id": "s",
                "tool_name": "Bash",
                "tool_input": {"command": command},
            })
            .to_string();
            let payload = parse_payload(Harness::Claude, &json).expect("a parsed payload");
            assert_eq!(decide(&payload), Answer::Allow, "{command}");
        }
    }

    #[test]
    fn a_read_or_a_search_missing_the_same_fields_is_still_allowed() {
        for (tool, input) in [
            ("Read", None),
            ("Read", Some(json!({}))),
            ("Grep", Some(json!({"pattern": "derive"}))),
            ("Glob", None),
            ("tilth_read", Some(json!({}))),
        ] {
            let mut json = serde_json::json!({
                "hook_event_name": "PreToolUse",
                "session_id": "s",
                "tool_name": tool,
            });
            if let Some(input) = input {
                json["tool_input"] = input;
            }
            let payload =
                parse_payload(Harness::Claude, &json.to_string()).expect("a parsed payload");
            assert_eq!(decide(&payload), Answer::Allow, "{tool}");
        }
    }

    #[test]
    fn the_two_unjudgeable_fixtures_are_refused_and_name_the_same_rule() {
        for (name, expected) in [
            ("no-input-PreToolUse", NO_PATH_REASON.to_owned()),
            ("no-cwd-PreToolUse", no_cwd_reason("/work/repo/garden.lock")),
        ] {
            let path = payloads().join("claude").join(format!("{name}.json"));
            let json = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let payload = parse_payload(Harness::Claude, &json).expect("a parsed fixture");
            let (answer, rule) = decide_with_rule(&payload);
            assert_eq!(reason(&answer), expected, "{name}");
            assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{name}");
        }
    }

    /// Codex takes `apply_patch` as a freeform tool on every model profile in the installed
    /// 0.133.0 (`"apply_patch_tool_type": "freeform"`, six of six), so the patch arrives as
    /// text rather than under a field the stem can name. The `*** … File:` headers are the
    /// part of it the vendor's own grammar fixes, so they are what the boundary reads.
    #[test]
    fn a_freeform_patch_is_judged_wherever_the_vendor_puts_its_text() {
        let patch = "*** Begin Patch\n*** Update File: garden.lock\n@@\n-a\n+b\n*** End Patch\n";
        for input in [
            json!(patch),
            json!({"input": patch}),
            json!({"patch": patch}),
            json!({"command": ["apply_patch", patch]}),
            json!({"arguments": {"input": patch}}),
        ] {
            let answer = decide(&payload(
                Harness::Codex,
                "PreToolUse",
                "apply_patch",
                input.clone(),
            ));
            let reason = reason(&answer);
            assert!(reason.contains("garden.lock"), "{input}: {reason}");
            assert!(reason.contains("plotplot lock update"), "{input}: {reason}");
        }
    }

    #[test]
    fn a_patch_run_through_the_shell_tool_is_judged_the_same_way() {
        let patch = "*** Begin Patch\n*** Update File: .githooks/pre-push\n+x\n*** End Patch\n";
        let answer = decide(&payload(
            Harness::Codex,
            "PreToolUse",
            "shell",
            json!({"command": ["apply_patch", patch]}),
        ));
        assert!(reason(&answer).contains(".githooks/pre-push"), "{answer:?}");

        // And a shell command that is not a patch stays a shell command.
        assert_eq!(
            decide(&payload(
                Harness::Codex,
                "PreToolUse",
                "shell",
                json!({"command": ["bash", "-lc", "cargo test"]})
            )),
            Answer::Allow
        );
    }

    #[test]
    fn a_freeform_patch_touching_nothing_the_stem_owns_is_allowed() {
        let patch = "*** Begin Patch\n*** Add File: src/lock.rs\n+fn main() {}\n*** End Patch\n";
        for input in [json!(patch), json!({"input": patch})] {
            assert_eq!(
                decide(&payload(
                    Harness::Codex,
                    "PreToolUse",
                    "apply_patch",
                    input.clone()
                )),
                Answer::Allow,
                "{input}"
            );
        }
    }

    #[test]
    fn a_patch_naming_no_file_at_all_is_refused() {
        for input in [
            json!("*** Begin Patch\n*** End Patch\n"),
            json!({"input": "not a patch"}),
        ] {
            let (answer, rule) = decide_with_rule(&payload(
                Harness::Codex,
                "PreToolUse",
                "apply_patch",
                input.clone(),
            ));
            assert_eq!(reason(&answer), NO_PATH_REASON, "{input}");
            assert_eq!(rule, Some(Rule::WriteTargetUnjudged), "{input}");
        }
    }

    #[test]
    fn only_a_patch_is_read_for_headers_it_carries() {
        // A file whose contents happen to spell a patch is a file, not a patch: the tool
        // that writes it names its own target, and that is the target judged.
        let content = "*** Begin Patch\n*** Update File: garden.lock\n*** End Patch\n";
        assert_eq!(
            decide(&payload(
                Harness::Claude,
                "PreToolUse",
                "Write",
                json!({"file_path": "docs/patches/example.patch", "content": content})
            )),
            Answer::Allow
        );
    }

    #[test]
    fn a_target_the_payload_can_see_is_judged_rather_than_refused() {
        // The working directory is there, the path is outside the repository: judged, and
        // not the stem's. That is an answer, not a gap.
        assert_eq!(
            decide(&payload(
                Harness::Claude,
                "PreToolUse",
                "Write",
                json!({"file_path": "/somewhere/else/garden.lock"})
            )),
            Answer::Allow
        );
    }

    // -----------------------------------------------------------------------------------
    // the rule behind the refusal, which the friction ledger records
    // -----------------------------------------------------------------------------------

    #[test]
    fn each_refusal_names_the_rule_that_made_it_and_an_allowance_names_none() {
        assert_eq!(
            decide_with_rule(&shell(Harness::Claude, "git push --no-verify")).1,
            Some(Rule::SkippedVerification)
        );
        assert_eq!(
            decide_with_rule(&shell(Harness::Claude, "echo x > .plotplot/bin/weeder")).1,
            Some(Rule::StemOwnedPath)
        );
        assert_eq!(
            decide_with_rule(&payload(
                Harness::Claude,
                "PreToolUse",
                "Write",
                json!({ "file_path": "garden.lock" })
            ))
            .1,
            Some(Rule::StemOwnedPath)
        );
        assert_eq!(
            decide_with_rule(&shell(Harness::Claude, "cargo test")).1,
            None
        );
    }

    #[test]
    fn every_rule_name_is_distinct_and_lives_in_the_deny_namespace() {
        let mut names: Vec<&str> = Vec::new();
        for rule in [
            Rule::SkippedVerification,
            Rule::StemOwnedPath,
            Rule::WriteTargetUnjudged,
        ] {
            assert!(rule.name().starts_with("deny."), "{rule}");
            assert_eq!(rule.to_string(), rule.name());
            names.push(rule.name());
        }
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "{names:?}");
        assert_eq!(
            Rule::WriteTargetUnjudged.name(),
            "deny.write-target-unjudged"
        );
    }

    #[test]
    fn the_answer_is_the_same_whichever_of_the_two_deciders_is_asked() {
        for payload in [
            shell(Harness::Gemini, "git commit -n -m x"),
            shell(Harness::Codex, "printf x > .githooks/pre-commit"),
            shell(Harness::Claude, "cargo clippy"),
            payload(
                Harness::Claude,
                "PostToolUse",
                "Write",
                json!({ "file_path": "garden.lock" }),
            ),
        ] {
            assert_eq!(decide(&payload), decide_with_rule(&payload).0);
        }
    }

    #[test]
    fn a_refusal_always_carries_a_rule_and_an_allowance_never_does() {
        for command in [
            "git commit --no-verify -m x",
            "git merge -n feature",
            "echo x >> .plotplot/beds/weeder/garden.json",
            "cargo test",
            "git log -n 3",
        ] {
            let (answer, rule) = decide_with_rule(&shell(Harness::Claude, command));
            assert_eq!(
                matches!(answer, Answer::Deny { .. }),
                rule.is_some(),
                "{command}"
            );
        }
    }
}
