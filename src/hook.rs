//! The dispatcher: the one program every hook entry in every bundle calls.
//!
//! The contract with beds, ruled by the umbrella on 2026-09-08: `<bed> hook <harness>` reads
//! the vendor's payload on stdin, writes the vendor's answer on stdout, and exits 0 to allow,
//! 2 to deny, 3 when it cannot judge. The dispatcher speaks that contract on both sides: a
//! vendor calls it the same way, and it calls each registered bed the same way, in parallel,
//! and merges the answers with deny over cannot-judge over allow.
//!
//! Two things run before any bed and two after. Before: the payload is read, and the stem's
//! own hard-limit deny list decides; a refusal there answers the vendor and no bed is called.
//! After the merge, and never changing its code: the friction emitter, always, and the
//! receipt draft on the harness's session-end event. Those two write files; a failure in
//! either is a line on stderr, because a ledger that cannot be written is not a reason to
//! block an agent's work.
//!
//! Nothing here reads the current directory: the repository root is a parameter, and every
//! path under it comes from [`crate::layout`].

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crate::bed::Bed;
use crate::cli::HookArgs;
use crate::deny;
use crate::friction::{self, SystemClock};
use crate::harness::{Answer, Harness, Payload, exit_code, parse_payload, render_answer};
use crate::layout;
use crate::manifest;
use crate::receipt;

/// The tool call may proceed.
pub const ALLOW: i32 = 0;
/// The tool call is refused, or the stop is.
pub const DENY: i32 = 2;
/// Nobody could judge: the answer is missing rather than permissive.
pub const CANNOT: i32 = 3;
/// An argument the stem refuses, the code clap exits with for a bad value.
pub const USAGE: i32 = 2;

/// What a bed gets on a stop or a session end, where the vendor's own budget is tightest.
pub const STOP_TIMEOUT: Duration = Duration::from_secs(4);
/// What a bed gets on every other event.
pub const TOOL_TIMEOUT: Duration = Duration::from_secs(8);

/// How often a waiting dispatcher looks to see whether a bed has answered.
const POLL: Duration = Duration::from_millis(2);
/// How long a bed's output is still collected after the bed itself has ended. A child that
/// left a grandchild holding the pipe open does not get to hold the dispatcher with it.
const DRAIN: Duration = Duration::from_millis(250);
/// The same, for a bed that was killed at the timeout: its own end of the pipe went with it,
/// so anything still holding that pipe open is a grandchild, and waiting on one would spend
/// the very budget the timeout exists to protect.
const KILLED_DRAIN: Duration = Duration::from_millis(50);

/// A bed registered for this event, resolved to the program that answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registered {
    pub bed: String,
    /// `.plotplot/bin/<binary>`, the judge the lock pinned and `lock verify` fetched.
    pub program: PathBuf,
}

/// What one bed said.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BedResult {
    pub bed: String,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// What the dispatcher says, ready for the three streams a process has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dispatch {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Dispatch {
    /// Nothing to say.
    fn allow() -> Dispatch {
        Dispatch {
            code: ALLOW,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// The dispatcher itself could not judge, for the reason given.
    fn cannot(reason: String) -> Dispatch {
        Dispatch {
            code: CANNOT,
            stdout: String::new(),
            stderr: format!("{reason}\n"),
        }
    }
}

/// `plotplot hook <harness> <event>`: the face, from arguments and stdin to an exit code.
///
/// The event is checked against the vendor's own list (§5) rather than against the payload,
/// because a bundle naming an event a vendor does not fire is a fault in the bundle, and it
/// is reported the way clap reports a bad value: exit 2, the valid values named.
pub fn run(
    root: &Path,
    args: &HookArgs,
    stdin: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let harness = args.harness;
    if !harness.has_event(&args.event) {
        let _ = writeln!(
            stderr,
            "error: invalid value '{}' for '<EVENT>': {harness} fires {}",
            args.event,
            harness.events().join(", ")
        );
        return USAGE;
    }

    let answer = dispatch(root, harness, &args.event, stdin);
    // A stream the vendor has already closed cannot be told anything; the exit code still
    // carries the answer, so the code is what this returns either way.
    let _ = write!(stdout, "{}", answer.stdout);
    let _ = write!(stderr, "{}", answer.stderr);
    answer.code
}

/// One hook event, from the payload on stdin to the answer the vendor reads.
pub fn dispatch(root: &Path, harness: Harness, event: &str, payload_json: &str) -> Dispatch {
    let payload = match parse_payload(harness, payload_json) {
        Ok(payload) => payload,
        Err(error) => return Dispatch::cannot(error.to_string()),
    };

    let mut answer = judge(root, harness, event, payload_json, &payload);
    trail(root, event, &payload, &mut answer.stderr);
    answer
}

/// The answer itself: the stem's deny list, then the beds that registered for this event.
fn judge(
    root: &Path,
    harness: Harness,
    event: &str,
    payload_json: &str,
    payload: &Payload,
) -> Dispatch {
    let decision = deny::decide(payload);
    if let Answer::Deny { reason } | Answer::Block { reason } | Answer::Cannot { reason } =
        &decision
    {
        return Dispatch {
            code: exit_code(&decision),
            stdout: render_answer(harness, event, &decision),
            stderr: format!("{reason}\n"),
        };
    }

    let beds = match manifest::load_beds(root) {
        Ok(beds) => beds,
        Err(error) => return Dispatch::cannot(error.to_string()),
    };
    let registered = registered(root, &beds, harness, event, payload.tool_name.as_deref());
    if registered.is_empty() {
        return Dispatch::allow();
    }
    merge(&fan_out(
        harness,
        payload_json,
        &registered,
        timeout_for(harness, event),
    ))
}

/// The two faces the stem owns, after the answer and never changing it.
///
/// The receipt draft goes first on the session-end event: it counts from the friction state
/// that [`crate::friction::emit`] prunes there, which is the ordering `receipt.rs` documents.
fn trail(root: &Path, event: &str, payload: &Payload, stderr: &mut String) {
    if event == payload.harness.session_end_event() {
        if let Err(error) = receipt::draft(root, payload) {
            stderr.push_str(&format!("{error}\n"));
        }
    }
    if let Err(error) = friction::emit(root, payload, &SystemClock) {
        stderr.push_str(&format!("{error}\n"));
    }
}

/// The beds this event belongs to, in the order [`crate::manifest::load_beds`] gives them.
///
/// A bed is registered when it declared this event on this harness and its matcher, if it
/// declared one, is the tool the payload names. A bed with hook entries always has a binary
/// (`manifest::to_bed` refuses one without), so a bed with none registers nothing.
pub fn registered(
    root: &Path,
    beds: &[Bed],
    harness: Harness,
    event: &str,
    tool: Option<&str>,
) -> Vec<Registered> {
    beds.iter()
        .filter(|bed| {
            bed.hooks.get(&harness).is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry.event == event
                        && match &entry.matcher {
                            Some(matcher) => tool == Some(matcher.as_str()),
                            None => true,
                        }
                })
            })
        })
        .filter_map(|bed| {
            bed.binary.as_ref().map(|binary| Registered {
                bed: bed.name.clone(),
                program: layout::judge_binary(root, binary),
            })
        })
        .collect()
}

/// How long a bed has to answer: less where the vendor's own budget is tightest.
pub fn timeout_for(harness: Harness, event: &str) -> Duration {
    if event == "Stop" || event == harness.session_end_event() {
        STOP_TIMEOUT
    } else {
        TOOL_TIMEOUT
    }
}

/// Ask every registered bed at once, and wait no longer than the timeout for any of them.
///
/// One thread per bed, so two beds that each take 300 ms take 300 ms together. The child
/// inherits this process's working directory, which is the repository the vendor is running
/// in, so a bed reads the same tree the agent is editing.
pub fn fan_out(
    harness: Harness,
    payload_json: &str,
    beds: &[Registered],
    timeout: Duration,
) -> Vec<BedResult> {
    if beds.is_empty() {
        return Vec::new();
    }
    std::thread::scope(|scope| {
        let calls: Vec<_> = beds
            .iter()
            .map(|bed| {
                (
                    bed,
                    scope.spawn(move || call(harness, payload_json, bed, timeout)),
                )
            })
            .collect();
        calls
            .into_iter()
            .map(|(bed, call)| {
                call.join().unwrap_or_else(|_| BedResult {
                    bed: bed.bed.clone(),
                    code: CANNOT,
                    stdout: String::new(),
                    stderr: format!("{}: the dispatcher's thread for it ended badly", bed.bed),
                })
            })
            .collect()
    })
}

/// One bed, called the way the contract says: `<bed> hook <harness>`, payload on stdin.
fn call(harness: Harness, payload_json: &str, bed: &Registered, timeout: Duration) -> BedResult {
    let cannot = |reason: String| BedResult {
        bed: bed.bed.clone(),
        code: CANNOT,
        stdout: String::new(),
        stderr: reason,
    };

    let mut child = match Command::new(&bed.program)
        .arg("hook")
        .arg(harness.name())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(source) => {
            return cannot(format!(
                "{}: {} could not be run: {source}",
                bed.bed,
                bed.program.display()
            ));
        }
    };

    // The payload goes out on its own thread: a bed that answers without reading its stdin
    // would otherwise hold the dispatcher on a full pipe.
    if let Some(mut stdin) = child.stdin.take() {
        let payload = payload_json.to_owned();
        std::thread::spawn(move || {
            let _ = stdin.write_all(payload.as_bytes());
        });
    }
    let stdout = reader(child.stdout.take());
    let stderr = reader(child.stderr.take());

    let ended = wait_until(&mut child, timeout);
    // One budget for both streams, not one each, so a bed can never cost twice the grace.
    let deadline = Instant::now()
        + match ended {
            Ended::TimedOut => KILLED_DRAIN,
            _ => DRAIN,
        };
    let out = drain(stdout, deadline);
    let err = drain(stderr, deadline);

    match ended {
        Ended::Exited(Some(code)) => BedResult {
            bed: bed.bed.clone(),
            code,
            stdout: out,
            stderr: err,
        },
        Ended::Exited(None) => cannot(format!(
            "{}: it was killed by a signal before it answered\n{err}",
            bed.bed
        )),
        Ended::TimedOut => cannot(format!(
            "{}: it did not answer within {} s and was stopped\n{err}",
            bed.bed,
            timeout.as_secs()
        )),
        Ended::Failed(problem) => cannot(format!("{}: {problem}\n{err}", bed.bed)),
    }
}

/// How a child ended.
enum Ended {
    /// It exited; `None` when a signal killed it and there is no code to read.
    Exited(Option<i32>),
    /// It was still running at the timeout and was killed.
    TimedOut,
    /// The dispatcher could not wait for it.
    Failed(String),
}

/// Wait for a child, killing it at the deadline.
fn wait_until(child: &mut Child, timeout: Duration) -> Ended {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ended::Exited(status.code()),
            Ok(None) => {}
            Err(source) => return Ended::Failed(source.to_string()),
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ended::TimedOut;
        }
        std::thread::sleep(POLL);
    }
}

/// Read one of a child's streams to its end, off the thread that waits for the child.
fn reader<R: Read + Send + 'static>(source: Option<R>) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    if let Some(mut source) = source {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = source.read_to_end(&mut bytes);
            let _ = sender.send(String::from_utf8_lossy(&bytes).into_owned());
        });
    }
    receiver
}

/// What a stream carried, or nothing when the pipe outlived the child that owned it.
fn drain(stream: Receiver<String>, deadline: Instant) -> String {
    let left = deadline.saturating_duration_since(Instant::now());
    stream.recv_timeout(left).unwrap_or_default()
}

/// Merge what the beds said: deny over cannot-judge over allow.
///
/// The first denial's stdout is the answer the vendor reads, because a vendor reads one
/// answer; its reason leads on stderr and the other denials follow it. With no denial, a bed
/// that could not judge makes the whole dispatch unable to judge, each reason on stderr, so
/// a boundary that failed reads as failed rather than as permission. A code that is neither
/// 0, 2 nor 3 is not an answer the contract has, and is read as unable to judge for the same
/// reason.
pub fn merge(results: &[BedResult]) -> Dispatch {
    let denials: Vec<&BedResult> = results
        .iter()
        .filter(|result| result.code == DENY)
        .collect();
    if let Some(first) = denials.first() {
        return Dispatch {
            code: DENY,
            stdout: first.stdout.clone(),
            stderr: reasons(&denials),
        };
    }

    let unable: Vec<&BedResult> = results
        .iter()
        .filter(|result| result.code != ALLOW)
        .collect();
    if unable.is_empty() {
        Dispatch::allow()
    } else {
        Dispatch {
            code: CANNOT,
            stdout: String::new(),
            stderr: reasons(&unable),
        }
    }
}

/// Each bed's reason, one after another, and a line naming the bed when it gave none.
fn reasons(results: &[&BedResult]) -> String {
    let mut text = String::new();
    for result in results {
        let reason = result.stderr.trim_end_matches('\n');
        if reason.is_empty() {
            text.push_str(&format!(
                "{}: exited {} with no reason\n",
                result.bed, result.code
            ));
        } else {
            text.push_str(reason);
            text.push('\n');
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bed::HookEntry;
    use std::collections::BTreeMap;

    fn bed(name: &str, entries: &[(Harness, &str, Option<&str>)]) -> Bed {
        let mut hooks: BTreeMap<Harness, Vec<HookEntry>> = BTreeMap::new();
        for (harness, event, matcher) in entries {
            hooks.entry(*harness).or_default().push(HookEntry {
                event: (*event).to_owned(),
                matcher: matcher.map(str::to_owned),
            });
        }
        Bed {
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            binary: Some(name.to_owned()),
            skill: None,
            hooks,
            git_hooks: Vec::new(),
            mcp: None,
            check: None,
        }
    }

    fn result(bed: &str, code: i32, stdout: &str, stderr: &str) -> BedResult {
        BedResult {
            bed: bed.to_owned(),
            code,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }

    #[test]
    fn a_stop_or_session_end_gives_a_bed_less_time_than_a_tool_call() {
        assert_eq!(timeout_for(Harness::Claude, "Stop"), STOP_TIMEOUT);
        assert_eq!(timeout_for(Harness::Claude, "SessionEnd"), STOP_TIMEOUT);
        assert_eq!(timeout_for(Harness::Gemini, "SessionEnd"), STOP_TIMEOUT);
        assert_eq!(timeout_for(Harness::Codex, "Stop"), STOP_TIMEOUT);
        assert_eq!(timeout_for(Harness::Claude, "PreToolUse"), TOOL_TIMEOUT);
        assert_eq!(timeout_for(Harness::Gemini, "BeforeTool"), TOOL_TIMEOUT);
        assert_eq!(timeout_for(Harness::Codex, "PostToolUse"), TOOL_TIMEOUT);
    }

    #[test]
    fn a_matcher_selects_the_tool_it_names_and_nothing_else() {
        let root = Path::new("/work/repo");
        let beds = vec![
            bed("weeder", &[(Harness::Claude, "PreToolUse", Some("Bash"))]),
            bed("petals", &[(Harness::Claude, "PreToolUse", None)]),
        ];

        let names = |tool| {
            registered(root, &beds, Harness::Claude, "PreToolUse", tool)
                .into_iter()
                .map(|entry| entry.bed)
                .collect::<Vec<_>>()
        };
        assert_eq!(names(Some("Bash")), ["weeder", "petals"]);
        assert_eq!(names(Some("Read")), ["petals"]);
        assert_eq!(names(None), ["petals"]);
    }

    #[test]
    fn a_registered_bed_resolves_to_the_fetched_judge() {
        let root = Path::new("/work/repo");
        let beds = vec![bed("weeder", &[(Harness::Claude, "Stop", None)])];
        assert_eq!(
            registered(root, &beds, Harness::Claude, "Stop", None),
            vec![Registered {
                bed: "weeder".to_owned(),
                program: root.join(".plotplot/bin/weeder"),
            }]
        );
        assert!(registered(root, &beds, Harness::Claude, "SessionEnd", None).is_empty());
        assert!(registered(root, &beds, Harness::Codex, "Stop", None).is_empty());
    }

    #[test]
    fn a_bed_with_no_binary_registers_nothing() {
        let root = Path::new("/work/repo");
        let mut beds = vec![bed("pollen", &[(Harness::Claude, "Stop", None)])];
        beds[0].binary = None;
        assert!(registered(root, &beds, Harness::Claude, "Stop", None).is_empty());
    }

    #[test]
    fn no_bed_answering_is_an_allowance_with_nothing_on_either_stream() {
        assert_eq!(merge(&[]), Dispatch::allow());
        assert_eq!(
            merge(&[
                result("weeder", ALLOW, "", ""),
                result("tend2", ALLOW, "", "")
            ]),
            Dispatch::allow()
        );
    }

    #[test]
    fn the_first_denial_is_the_answer_and_the_others_follow_it_on_stderr() {
        let merged = merge(&[
            result("apple", ALLOW, "", ""),
            result(
                "weeder",
                DENY,
                "{\"decision\":\"deny\"}",
                "weeder: a test was deleted\n",
            ),
            result(
                "tend2",
                DENY,
                "{\"decision\":\"deny\"}",
                "tend2: the claim is unproven\n",
            ),
            result("petals", CANNOT, "", "petals: no tokens\n"),
        ]);
        assert_eq!(merged.code, DENY);
        assert_eq!(merged.stdout, "{\"decision\":\"deny\"}");
        assert_eq!(
            merged.stderr,
            "weeder: a test was deleted\ntend2: the claim is unproven\n"
        );
    }

    #[test]
    fn a_bed_that_could_not_judge_makes_the_dispatch_unable_to_judge() {
        let merged = merge(&[
            result("apple", ALLOW, "", ""),
            result("weeder", CANNOT, "", "weeder: the index is stale\n"),
            result("tend2", CANNOT, "", "tend2: no map here\n"),
        ]);
        assert_eq!(merged.code, CANNOT);
        assert_eq!(merged.stdout, "");
        assert_eq!(
            merged.stderr,
            "weeder: the index is stale\ntend2: no map here\n"
        );
    }

    #[test]
    fn a_code_the_contract_does_not_have_is_read_as_unable_to_judge() {
        let merged = merge(&[result("weeder", 127, "", "")]);
        assert_eq!(merged.code, CANNOT);
        assert_eq!(merged.stderr, "weeder: exited 127 with no reason\n");
    }

    #[test]
    fn a_denial_that_gave_no_reason_still_names_itself() {
        let merged = merge(&[result("weeder", DENY, "{}", "")]);
        assert_eq!(merged.code, DENY);
        assert_eq!(merged.stdout, "{}");
        assert_eq!(merged.stderr, "weeder: exited 2 with no reason\n");
    }

    #[test]
    fn a_bed_that_is_not_there_cannot_judge_and_names_the_path() {
        let missing = Registered {
            bed: "weeder".to_owned(),
            program: PathBuf::from("/work/repo/.plotplot/bin/weeder"),
        };
        let results = fan_out(
            Harness::Claude,
            "{\"hook_event_name\":\"PreToolUse\"}",
            std::slice::from_ref(&missing),
            TOOL_TIMEOUT,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code, CANNOT);
        assert!(
            results[0]
                .stderr
                .contains("/work/repo/.plotplot/bin/weeder"),
            "{}",
            results[0].stderr
        );
    }

    #[test]
    fn asking_no_bed_at_all_runs_nothing() {
        assert!(fan_out(Harness::Claude, "{}", &[], TOOL_TIMEOUT).is_empty());
    }
}
