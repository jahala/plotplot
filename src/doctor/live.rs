//! `plotplot doctor --live`: the half of the command that proves a hook fires by firing it.
//!
//! Static mode reads what is on disk and compares it to what the stem would write. That
//! catches a bundle nobody installed; it cannot catch a vendor renaming an event, changing a
//! payload's shape, or loading a project's hooks only under a condition nobody noticed. The
//! only proof that a boundary holds is a real session walking into it, so live mode drives
//! one through umbel: it asks the worker for a single shell command the stem's own deny list
//! refuses, and then reads what the dispatcher wrote.
//!
//! What proves it is never the worker's word. A worker can say it was refused when it was
//! not, and did on the run that found Codex 0.133.0's second name for its shell tool. Proof
//! is three files the dispatcher itself writes: a `tool.denied` record carrying the rule that
//! refused, a `session.ended` record, and the session's receipt draft. Each is one finding.
//!
//! A harness that cannot start a session, or whose worker sits at a login prompt or a
//! provider error, is reported `unavailable` with what umbel or the pane said. That is not a
//! failure of the stem and is never silence: the table carries a line for every harness asked
//! for, always.
//!
//! umbel is driven behind [`Driver`] so the parsing and the verdict have unit tests. The
//! proof itself has none: no double writes a record here. `scripts/fit/stem.sh doctor-live`
//! runs the real thing against real binaries, which is the only place the claim is earned.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::deny::Rule;
use crate::doctor;
use crate::error::{Error, Result};
use crate::friction::{Kind, Record, harness_label};
use crate::harness::Harness;
use crate::{install, layout};

/// The program every session is driven through.
pub const UMBEL: &str = "umbel";

/// What the worker is asked to do: one shell command the deny list refuses, then stop.
///
/// `git commit --no-verify` never commits, because the boundary refuses it before git is
/// reached; and were the boundary gone, the fixture it runs in has nothing staged. The
/// wording forbids a retry, so a worker that is refused does not spend a session arguing.
pub const PROMPT: &str = "Run exactly this one shell command, once: git commit --no-verify -m probe\n\nDo not run any other command, do not edit any file, and do not retry or work around it if it is refused. Then stop and say only DONE.";

/// How long a worker has to answer, when `--timeout` does not say.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);

/// How long a worker may leave its pane still before it is taken to be stuck.
///
/// umbel leaves this net off by default, because a worker may be inside a long silent tool
/// call. This probe's worker is not: it runs one command that is refused in milliseconds, so
/// a pane that has not moved in this long is a worker at a login prompt or a provider error,
/// and waiting out the deadline only makes the doctor slower at saying so.
pub const IDLE: Duration = Duration::from_secs(20);

/// How long the session-end record is waited for after the session is ended.
///
/// The session-end hook runs while the harness is shutting down, so it lands after `kill`
/// returns rather than before it. Measured at two to three seconds on this machine for both
/// Claude Code and Codex; the budget is what it takes for a slow shutdown, not a guess at
/// whether a record is coming.
pub const SETTLE: Duration = Duration::from_secs(10);

/// How often the journal is looked at while settling.
const POLL: Duration = Duration::from_millis(200);

/// How many lines of the worker's pane are read looking for what the vendor said.
const PANE_LINES: &str = "120";

/// What a pane says when the vendor, and not the stem, is why nothing was proved.
///
/// Each is a phrase one of the three CLIs prints in its own pane, matched case-insensitively;
/// the line carrying it is quoted verbatim into the report, so the reader sees the vendor's
/// own words and not this list. Every entry was read off a real pane on this machine.
pub const PROVIDER_TROUBLE: [&str; 6] = [
    "404",
    "not logged in",
    "/login",
    "authenticate",
    "no authentication method",
    "rate limit",
];

// ---------------------------------------------------------------------------------- findings

/// What live mode can say about one question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Proved, by a file the dispatcher wrote.
    Ok,
    /// The session ran and the proof is not there.
    Fail,
    /// No session to judge: the harness is not planted here, or the vendor would not run.
    Unavailable,
}

impl Verdict {
    /// The word the table prints.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Ok => "ok",
            Verdict::Fail => "fail",
            Verdict::Unavailable => "unavailable",
        }
    }
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(self.as_str())
    }
}

/// One live question, its answer, and the record or the reason behind it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveFinding {
    pub check: &'static str,
    pub verdict: Verdict,
    pub detail: String,
}

/// The four lines one harness can earn, in the order the table prints them.
///
/// Spelled out rather than formatted so every name is a `&'static str` and the table's first
/// column can never carry a string built from something a vendor said.
const LINES: [[&str; 4]; 3] = [
    [
        "claude session",
        "claude before-tool",
        "claude session-end",
        "claude receipt draft",
    ],
    [
        "gemini session",
        "gemini before-tool",
        "gemini session-end",
        "gemini receipt draft",
    ],
    [
        "codex session",
        "codex before-tool",
        "codex session-end",
        "codex receipt draft",
    ],
];

/// The four line names live mode uses for one harness: the session, then one per event class.
pub fn lines(harness: Harness) -> [&'static str; 4] {
    match harness {
        Harness::Claude => LINES[0],
        Harness::Gemini => LINES[1],
        Harness::Codex => LINES[2],
    }
}

fn finding(check: &'static str, verdict: Verdict, detail: impl Into<String>) -> LiveFinding {
    LiveFinding {
        check,
        verdict,
        detail: detail.into(),
    }
}

/// The live table: check, verdict, detail, in three aligned columns.
pub fn render(findings: &[LiveFinding]) -> String {
    let width = findings
        .iter()
        .map(|finding| finding.check.len())
        .max()
        .unwrap_or_default();

    let mut table = String::new();
    for finding in findings {
        let _ = writeln!(
            table,
            "{:width$}  {:11}  {}",
            finding.check,
            finding.verdict.as_str(),
            finding.detail
        );
    }
    table
}

/// Whether any live question was answered `fail`: a session that ran and did not prove.
///
/// `unavailable` is not a failure. A vendor that will not run is a fact about the machine,
/// and reporting it as a broken stem would teach a reader to ignore the exit code.
pub fn failed(findings: &[LiveFinding]) -> bool {
    findings
        .iter()
        .any(|finding| finding.verdict == Verdict::Fail)
}

// ------------------------------------------------------------------------------ the seam

/// What one umbel command answered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ran {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    /// The first line with anything on it, which is umbel's own word about what happened;
    /// on a timeout the pane snapshot follows it and is read separately.
    pub fn first_line(&self) -> Option<&str> {
        self.stderr
            .lines()
            .chain(self.stdout.lines())
            .map(str::trim)
            .find(|line| !line.is_empty())
    }
}

/// Driving one session. The real implementation runs `umbel`; a test double plays back
/// recorded output so the parsing and the verdict can be judged without a vendor.
pub trait Driver {
    fn spawn(&self, session: &str, harness: Harness, root: &Path) -> Ran;
    fn send(&self, session: &str, prompt: &str) -> Ran;
    fn wait(&self, session: &str, timeout: Duration) -> Ran;
    fn capture(&self, session: &str) -> Ran;
    fn kill(&self, session: &str) -> Ran;
}

/// The real driver: umbel on `PATH`, one process per verb.
pub struct Umbel;

impl Umbel {
    fn run(args: &[&str]) -> Ran {
        match Command::new(UMBEL).args(args).output() {
            Ok(output) => Ran {
                code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(error) => Ran {
                code: 1,
                stdout: String::new(),
                stderr: format!("{UMBEL} {}: {error}", args.join(" ")),
            },
        }
    }
}

impl Driver for Umbel {
    /// `umbel spawn <session> --cwd <root> --provider <harness> --unattended`.
    ///
    /// Unattended is what keeps a worker off a permission prompt nobody is there to answer.
    /// `--allowed-tools` goes only to Claude Code: umbel refuses the flag for the other two,
    /// and the probe needs one tool.
    fn spawn(&self, session: &str, harness: Harness, root: &Path) -> Ran {
        let root = root.display().to_string();
        let mut args = vec![
            "spawn",
            session,
            "--cwd",
            &root,
            "--provider",
            harness.name(),
            "--unattended",
        ];
        if harness == Harness::Claude {
            args.extend(["--allowed-tools", "Bash"]);
        }
        Umbel::run(&args)
    }

    fn send(&self, session: &str, prompt: &str) -> Ran {
        Umbel::run(&["send", session, "--prompt", prompt])
    }

    fn wait(&self, session: &str, timeout: Duration) -> Ran {
        let timeout = format!("{}s", timeout.as_secs());
        let idle = format!("{}s", IDLE.as_secs());
        Umbel::run(&[
            "wait",
            session,
            "--timeout",
            &timeout,
            "--idle-timeout",
            &idle,
        ])
    }

    fn capture(&self, session: &str) -> Ran {
        Umbel::run(&["capture", session, "--lines", PANE_LINES])
    }

    /// Ending the session is also what fires the session-end event, so this is part of the
    /// probe and not only its cleanup. Only sessions this doctor spawned are ever named
    /// (`docs/building-the-garden.md` §2: never kill what you did not start).
    fn kill(&self, session: &str) -> Ran {
        Umbel::run(&["kill", session])
    }
}

/// What this doctor calls the session it is about to start, so a conductor sharing the
/// machine can tell whose it is and whose it is not.
pub fn session_name(harness: Harness, pid: u32) -> String {
    format!("plotplot-doctor-{}-{pid}", harness.name())
}

// ---------------------------------------------------------------------- what can be driven

/// The project-scope file that says a harness is planted here, or `None` for a harness whose
/// project scope is a directory the vendor writes.
fn project_scope(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude => install::CLAUDE_SETTINGS,
        Harness::Gemini => install::GEMINI_SETTINGS,
        Harness::Codex => install::CODEX_HOOKS,
    }
}

/// Whether a harness's project-scope configuration carries the stem, and what it says.
///
/// The static findings prove the bundles are generated and current; this asks the narrower
/// question live mode needs answered before it starts anything, which is whether this vendor
/// would load the stem's hooks in this repository at all. Claude Code enables the plugin,
/// the other two carry the dispatcher's own command.
pub fn planted(harness: Harness, config: Option<&str>) -> std::result::Result<String, String> {
    let file = project_scope(harness);
    let Some(text) = config else {
        return Err(format!("not planted here: {file} is not there"));
    };
    let document: Value = serde_json::from_str(text)
        .map_err(|error| format!("{file}: {}", crate::manifest::one_line(&error.to_string())))?;

    if harness == Harness::Claude {
        let id = format!("{}@{}", crate::bundle::BUNDLE_NAME, install::MARKETPLACE);
        return match document.get("enabledPlugins").and_then(|p| p.get(&id)) {
            Some(Value::Bool(true)) => Ok(format!("{file} enables {id}")),
            _ => Err(format!("not planted here: {file} does not enable {id}")),
        };
    }

    let wanted = format!("hook {}", harness.name());
    let events = document.get("hooks").and_then(Value::as_object);
    let registered = events
        .into_iter()
        .flatten()
        .filter(|(_, groups)| carries(groups, &wanted))
        .count();
    if registered == 0 {
        Err(format!(
            "not planted here: {file} registers no `plotplot hook {}` entry",
            harness.name()
        ))
    } else {
        Ok(format!("{file} registers {registered} dispatcher entries"))
    }
}

/// Whether any command under one event's groups names the dispatcher.
fn carries(groups: &Value, wanted: &str) -> bool {
    groups
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .filter_map(|entry| entry.get("command").and_then(Value::as_str))
        .any(|command| command.contains(wanted))
}

/// Why no proof is expected of this session, or `None` when the vendor gave no reason and the
/// missing proof is the stem's to answer for.
///
/// umbel's own exit code comes first, because it is the one signal that means the worker
/// never got to the tool call: 126 blocked on input, 125 dead, 124 the deadline, 123 a pane
/// that stopped moving.
/// Then the pane, which is where a provider's own refusal shows up while umbel still sees a
/// finished turn.
pub fn unavailable_reason(waited: &Ran, pane: &str) -> Option<String> {
    let trouble = pane_trouble(pane);
    if waited.code == 0 {
        return trouble;
    }
    let said = waited
        .first_line()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("umbel wait exited {}", waited.code));
    Some(match trouble {
        Some(trouble) => format!("{said}; the pane said: {trouble}"),
        None => said,
    })
}

/// The first line of a pane naming something the vendor, and not the stem, is answerable for.
pub fn pane_trouble(pane: &str) -> Option<String> {
    pane.lines()
        .find(|line| {
            let lowered = line.to_lowercase();
            PROVIDER_TROUBLE
                .iter()
                .any(|marker| lowered.contains(marker))
        })
        .map(tidy)
}

/// One pane line as a table cell: the vendor's own words, with the box it was drawn inside
/// and the padding that centred it taken off. Nothing is added and no word is changed; a
/// reader who wants the pane itself has umbel's own snapshot on stderr.
fn tidy(line: &str) -> String {
    line.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim_matches(|c: char| c == '│' || c == '|' || c == '║' || c.is_whitespace())
        .trim()
        .to_owned()
}

// ----------------------------------------------------------------------------- the verdict

/// What the records a probe added prove, one finding per event class.
///
/// Pure over its arguments: the records are the lines the journal gained while the session
/// ran, and `drafts` the session ids `.plotplot/receipts/drafts/` holds. The session id comes
/// out of the records themselves, which is the only place the harness reported it.
pub fn proofs(harness: Harness, added: &[Record], drafts: &[String]) -> Vec<LiveFinding> {
    let names = lines(harness);
    let label = harness_label(harness);
    let mine: Vec<&Record> = added
        .iter()
        .filter(|record| record.harness == label)
        .collect();

    let refused = Rule::SkippedVerification.name();
    let denied = mine
        .iter()
        .find(|record| record.kind == Kind::ToolDenied && record.rule.as_deref() == Some(refused));
    let session = denied
        .map(|record| record.conversation_id.clone())
        .or_else(|| {
            mine.iter()
                .find(|record| record.kind == Kind::SessionEnded)
                .map(|record| record.conversation_id.clone())
        });

    let before_tool = match denied {
        Some(record) => finding(names[1], Verdict::Ok, as_line(record)),
        None => finding(
            names[1],
            Verdict::Fail,
            format!(
                "the journal gained no {} record carrying {refused}, so the before-tool hook \
                 did not refuse: {}",
                Kind::ToolDenied,
                summary(&mine)
            ),
        ),
    };

    let ended = session.as_ref().and_then(|session| {
        mine.iter()
            .find(|record| record.kind == Kind::SessionEnded && &record.conversation_id == session)
    });
    let session_end = match ended {
        Some(record) => finding(names[2], Verdict::Ok, as_line(record)),
        None => finding(
            names[2],
            Verdict::Fail,
            format!(
                "the journal gained no {} record{}, so the session-end hook did not fire",
                Kind::SessionEnded,
                for_session(session.as_deref())
            ),
        ),
    };

    let drafted = session
        .as_ref()
        .filter(|session| drafts.iter().any(|held| &held == session));
    let draft = match drafted {
        Some(session) => finding(
            names[3],
            Verdict::Ok,
            layout::receipt_draft(Path::new("."), session)
                .display()
                .to_string(),
        ),
        None => finding(
            names[3],
            Verdict::Fail,
            format!(
                "no receipt draft{}, so the draft face did not fire",
                for_session(session.as_deref())
            ),
        ),
    };

    vec![before_tool, session_end, draft]
}

/// One record as the journal holds it, which is the proof itself and not a summary of it.
fn as_line(record: &Record) -> String {
    serde_json::to_string(record)
        .unwrap_or_else(|error| format!("{} for {} ({error})", record.kind, record.conversation_id))
}

/// What the journal did gain, so a failure says what was there instead of only what was not.
fn summary(records: &[&Record]) -> String {
    if records.is_empty() {
        return "the journal gained nothing at all".to_owned();
    }
    let kinds: Vec<&str> = records.iter().map(|record| record.kind.as_str()).collect();
    format!("it gained {}", kinds.join(", "))
}

/// ` for <session>`, or nothing when no record named a session.
fn for_session(session: Option<&str>) -> String {
    session
        .map(|session| format!(" for {session}"))
        .unwrap_or_default()
}

// -------------------------------------------------------------------------------- the edge

/// Every journal file's length now, so what one session added can be told from what was there.
fn offsets(root: &Path) -> Result<BTreeMap<PathBuf, u64>> {
    let directory = layout::friction_dir(root);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeMap::new());
        }
        Err(source) => {
            return Err(Error::Io {
                path: directory,
                source,
            });
        }
    };

    let mut lengths = BTreeMap::new();
    for entry in entries {
        let path = entry
            .map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        if path.extension().is_some_and(|kind| kind == "jsonl") {
            let length = fs::metadata(&path)
                .map_err(|source| Error::Io {
                    path: path.clone(),
                    source,
                })?
                .len();
            lengths.insert(path, length);
        }
    }
    Ok(lengths)
}

/// The lines every journal gained since those lengths were taken.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Appended {
    pub records: Vec<Record>,
    /// Lines the journal gained that are not records the profile's shape can read.
    pub unparsed: Vec<String>,
}

fn appended(root: &Path, before: &BTreeMap<PathBuf, u64>) -> Result<Appended> {
    let mut gained = Appended::default();
    for (path, length) in offsets(root)? {
        let bytes = fs::read(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        // A journal that shrank was rewritten under us; everything in it is new.
        let from = before.get(&path).copied().unwrap_or_default().min(length);
        let tail = bytes.get(from as usize..).unwrap_or_default();
        for line in String::from_utf8_lossy(tail).lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Record>(line) {
                Ok(record) => gained.records.push(record),
                Err(_) => gained.unparsed.push(line.to_owned()),
            }
        }
    }
    Ok(gained)
}

/// The session ids `.plotplot/receipts/drafts/` holds a draft for.
fn drafts(root: &Path) -> Result<Vec<String>> {
    let directory = layout::receipt_drafts_dir(root);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(Error::Io {
                path: directory,
                source,
            });
        }
    };

    let mut held = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        if path.extension().is_some_and(|kind| kind == "json")
            && let Some(session) = path.file_stem().and_then(|stem| stem.to_str())
        {
            held.push(session.to_owned());
        }
    }
    held.sort();
    Ok(held)
}

/// Wait for the session-end record, which lands while the harness is shutting down and so
/// after `kill` has returned. Whatever the journal holds when the budget runs out is what is
/// judged; nothing is assumed to be on its way.
fn settle(
    root: &Path,
    before: &BTreeMap<PathBuf, u64>,
    harness: Harness,
    budget: Duration,
) -> Result<Appended> {
    let label = harness_label(harness);
    let deadline = Instant::now() + budget;
    loop {
        let gained = appended(root, before)?;
        let ended = gained
            .records
            .iter()
            .any(|record| record.harness == label && record.kind == Kind::SessionEnded);
        if ended || Instant::now() >= deadline {
            return Ok(gained);
        }
        std::thread::sleep(POLL);
    }
}

// ----------------------------------------------------------------------------- the probe

/// One harness, from the project-scope question to the findings its session earned.
///
/// Every session this starts is ended here, on every path out, because a worker left in a
/// pane is a worker somebody else has to reason about.
fn probe(
    root: &Path,
    harness: Harness,
    timeout: Duration,
    driver: &dyn Driver,
    stderr: &mut dyn Write,
) -> Result<Vec<LiveFinding>> {
    let names = lines(harness);
    let config = read_project_scope(root, harness)?;
    let planted = match planted(harness, config.as_deref()) {
        Ok(detail) => detail,
        Err(reason) => return Ok(vec![finding(names[0], Verdict::Unavailable, reason)]),
    };

    let session = session_name(harness, std::process::id());
    let before = offsets(root)?;

    let _ = writeln!(
        stderr,
        "{harness}: {planted}; spawning {session} through {UMBEL}"
    );
    let spawned = driver.spawn(&session, harness, root);
    if spawned.code != 0 {
        let said = spawned
            .first_line()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{UMBEL} spawn exited {}", spawned.code));
        return Ok(vec![finding(
            names[0],
            Verdict::Unavailable,
            format!("no session started: {said}"),
        )]);
    }

    let sent = driver.send(&session, PROMPT);
    let waited = if sent.code == 0 {
        driver.wait(&session, timeout)
    } else {
        sent
    };
    let pane = driver.capture(&session).stdout;
    let killed = driver.kill(&session);
    if killed.code != 0 {
        let _ = writeln!(
            stderr,
            "{harness}: {UMBEL} kill {session} exited {}: {}",
            killed.code,
            killed.first_line().unwrap_or_default()
        );
    }

    let gained = settle(root, &before, harness, SETTLE)?;
    for line in &gained.unparsed {
        let _ = writeln!(
            stderr,
            "{harness}: the journal gained a line the profile cannot read: {line}"
        );
    }

    let held = drafts(root)?;
    let proofs = proofs(harness, &gained.records, &held);
    let proved = proofs.iter().any(|proof| proof.verdict == Verdict::Ok);
    if !proved && let Some(reason) = unavailable_reason(&waited, &pane) {
        return Ok(vec![finding(
            names[0],
            Verdict::Unavailable,
            format!("session {session}: {reason}"),
        )]);
    }

    let ran = match session_id(&gained.records, harness) {
        Some(id) => format!("session {session}, which the harness called {id}"),
        None => format!("session {session}, which reported no session id"),
    };
    let mut findings = vec![finding(names[0], Verdict::Ok, ran)];
    findings.extend(proofs);
    Ok(findings)
}

/// The session id this harness reported, out of the records it earned.
fn session_id(records: &[Record], harness: Harness) -> Option<String> {
    let label = harness_label(harness);
    records
        .iter()
        .find(|record| record.harness == label)
        .map(|record| record.conversation_id.clone())
}

/// The harness's project-scope configuration, when this repository has one.
fn read_project_scope(root: &Path, harness: Harness) -> Result<Option<String>> {
    let path = root.join(project_scope(harness));
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::Io { path, source }),
    }
}

/// Every chosen harness, one after another.
///
/// One at a time on purpose: two live sessions in one repository would write into one journal
/// and each would have to guess which lines were its own.
pub fn probe_all(
    root: &Path,
    harnesses: &[Harness],
    timeout: Duration,
    driver: &dyn Driver,
    stderr: &mut dyn Write,
) -> Vec<LiveFinding> {
    let mut findings = Vec::new();
    for harness in harnesses {
        match probe(root, *harness, timeout, driver, stderr) {
            Ok(earned) => findings.extend(earned),
            Err(error) => findings.push(finding(
                lines(*harness)[0],
                Verdict::Fail,
                error.to_string(),
            )),
        }
    }
    findings
}

/// `plotplot doctor --live`: the static table, then the live one.
///
/// Static findings come first and are printed whatever live mode goes on to say, because a
/// bundle that is not on disk explains a hook that did not fire. The exit code is 3 when a
/// static check failed or a session that ran did not prove what it should have.
pub fn run(
    root: &Path,
    home: &Path,
    harnesses: &[Harness],
    timeout: Duration,
    driver: &dyn Driver,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let code = doctor::run(root, home, stdout, stderr);
    if doctor::unplanted(root).is_some() || code == 1 {
        return code;
    }

    let findings = probe_all(root, harnesses, timeout, driver, stderr);
    if let Err(error) = write!(stdout, "\n{}", render(&findings)) {
        let _ = writeln!(stderr, "stdout: {error}");
        return 1;
    }
    if failed(&findings) || code != 0 { 3 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A driver that answers from recorded umbel output and remembers what it was asked.
    ///
    /// It never writes a record: the proof is the dispatcher's to write, and a double that
    /// wrote one would prove only that this file can read what it just made up. What it does
    /// cover is the parsing (umbel's exit codes, a pane's words) and the verdict.
    struct Recorded {
        spawn: Ran,
        send: Ran,
        wait: Ran,
        pane: String,
        calls: RefCell<Vec<String>>,
    }

    impl Recorded {
        fn new(wait: Ran, pane: &str) -> Recorded {
            Recorded {
                spawn: Ran::default(),
                send: Ran::default(),
                wait,
                pane: pane.to_owned(),
                calls: RefCell::new(Vec::new()),
            }
        }

        fn note(&self, call: String) {
            self.calls.borrow_mut().push(call);
        }

        fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl Driver for Recorded {
        fn spawn(&self, session: &str, harness: Harness, _root: &Path) -> Ran {
            self.note(format!("spawn {session} {harness}"));
            self.spawn.clone()
        }
        fn send(&self, session: &str, prompt: &str) -> Ran {
            self.note(format!("send {session} {}", prompt.lines().count()));
            self.send.clone()
        }
        fn wait(&self, session: &str, timeout: Duration) -> Ran {
            self.note(format!(
                "wait {session} {}s idle {}s",
                timeout.as_secs(),
                IDLE.as_secs()
            ));
            self.wait.clone()
        }
        fn capture(&self, session: &str) -> Ran {
            self.note(format!("capture {session}"));
            Ran {
                code: 0,
                stdout: self.pane.clone(),
                stderr: String::new(),
            }
        }
        fn kill(&self, session: &str) -> Ran {
            self.note(format!("kill {session}"));
            Ran::default()
        }
    }

    /// The two records a Claude probe earned on this machine on 2026-09-09, as the journal
    /// holds them. Copied from a real run, so the verdict is judged against the profile's
    /// own bytes and not against a shape invented here.
    const DENIED: &str = r#"{"time":"2026-09-09T12:14:23.424Z","event.name":"plotplot.friction","plotplot.kind":"tool.denied","plotplot.harness":"claude-code","gen_ai.conversation.id":"98f16765-ab3a-4763-99fb-c30204d9e6df","gen_ai.operation.name":"execute_tool","gen_ai.tool.name":"Bash","plotplot.rule":"deny.no-verify","plotplot.count":1,"plotplot.agent.id":null}"#;
    const ENDED: &str = r#"{"time":"2026-09-09T12:14:28.106Z","event.name":"plotplot.friction","plotplot.kind":"session.ended","plotplot.harness":"claude-code","gen_ai.conversation.id":"98f16765-ab3a-4763-99fb-c30204d9e6df","gen_ai.usage.input_tokens":null,"gen_ai.usage.output_tokens":null,"plotplot.rule":null,"plotplot.count":1,"plotplot.agent.id":null}"#;
    const SESSION: &str = "98f16765-ab3a-4763-99fb-c30204d9e6df";

    fn record(json: &str) -> Record {
        serde_json::from_str(json).expect("a record the profile can read")
    }

    fn verdicts(findings: &[LiveFinding]) -> Vec<(&str, Verdict)> {
        findings
            .iter()
            .map(|finding| (finding.check, finding.verdict))
            .collect()
    }

    // ------------------------------------------------------------------ the verdict

    #[test]
    fn the_three_records_a_session_wrote_prove_the_three_event_classes() {
        let added = [record(DENIED), record(ENDED)];
        let drafts = [SESSION.to_owned()];
        let proofs = proofs(Harness::Claude, &added, &drafts);

        assert_eq!(
            verdicts(&proofs),
            [
                ("claude before-tool", Verdict::Ok),
                ("claude session-end", Verdict::Ok),
                ("claude receipt draft", Verdict::Ok),
            ]
        );
        // Each `ok` carries the record that proved it, not a sentence about it.
        assert!(proofs[0].detail.contains("deny.no-verify"), "{proofs:?}");
        assert!(proofs[0].detail.contains(SESSION), "{proofs:?}");
        assert!(proofs[1].detail.contains("session.ended"), "{proofs:?}");
        assert!(proofs[2].detail.contains(SESSION), "{proofs:?}");
    }

    #[test]
    fn a_session_that_wrote_nothing_fails_all_three_and_says_so() {
        let proofs = proofs(Harness::Claude, &[], &[]);
        assert_eq!(
            verdicts(&proofs),
            [
                ("claude before-tool", Verdict::Fail),
                ("claude session-end", Verdict::Fail),
                ("claude receipt draft", Verdict::Fail),
            ]
        );
        assert!(
            proofs[0]
                .detail
                .contains("the journal gained nothing at all"),
            "{proofs:?}"
        );
    }

    #[test]
    fn a_refusal_without_a_draft_proves_two_of_three() {
        let added = [record(DENIED), record(ENDED)];
        let proofs = proofs(Harness::Claude, &added, &[]);
        assert_eq!(
            verdicts(&proofs),
            [
                ("claude before-tool", Verdict::Ok),
                ("claude session-end", Verdict::Ok),
                ("claude receipt draft", Verdict::Fail),
            ]
        );
        assert!(proofs[2].detail.contains(SESSION), "{proofs:?}");
    }

    #[test]
    fn another_harnesss_records_prove_nothing_about_this_one() {
        let added = [record(DENIED), record(ENDED)];
        let proofs = proofs(Harness::Codex, &added, &[SESSION.to_owned()]);
        assert!(
            proofs.iter().all(|proof| proof.verdict == Verdict::Fail),
            "{proofs:?}"
        );
        assert_eq!(proofs[0].check, "codex before-tool");
    }

    #[test]
    fn a_denial_by_another_rule_is_not_the_before_tool_proof() {
        let other = record(DENIED).clone();
        let mut other = other;
        other.rule = Some("deny.stem-owned-path".to_owned());
        let proofs = proofs(Harness::Claude, &[other], &[]);
        assert_eq!(proofs[0].verdict, Verdict::Fail);
        assert!(
            proofs[0].detail.contains("it gained tool.denied"),
            "{proofs:?}"
        );
    }

    // ------------------------------------------------------------------ availability

    #[test]
    fn umbels_own_exit_codes_are_what_blocked_and_dead_mean() {
        for (code, said) in [
            (
                126,
                "umbel: session 'x' is waiting for input — Claude is waiting for your input",
            ),
            (
                125,
                "umbel: wait failed — session 'x' died before completing its turn.",
            ),
            (124, "umbel: wait timed out. Last tmux pane:"),
        ] {
            let waited = Ran {
                code,
                stdout: String::new(),
                stderr: format!("{said}\n"),
            };
            assert_eq!(
                unavailable_reason(&waited, "").as_deref(),
                Some(said),
                "exit {code}"
            );
        }
    }

    #[test]
    fn a_finished_turn_with_a_provider_error_in_the_pane_is_unavailable() {
        let pane = "\n■ unexpected status 404 Not Found: The model `gpt-5.5` does not exist or you\ndo not have access to it.\n";
        let reason = unavailable_reason(&Ran::default(), pane).expect("a reason");
        assert!(reason.contains("404"), "{reason}");
        assert!(reason.contains("gpt-5.5"), "{reason}");
    }

    #[test]
    fn a_pane_line_reaches_the_table_without_the_box_it_was_drawn_in() {
        let pane = "│   No authentication method selected.                       │\n";
        assert_eq!(
            pane_trouble(pane).as_deref(),
            Some("No authentication method selected.")
        );
    }

    #[test]
    fn a_gemini_pane_at_its_sign_in_picker_is_unavailable() {
        let pane = "│   How would you like to authenticate for this project?   │\n│   No authentication method selected.   │\n";
        let waited = Ran {
            code: 124,
            stdout: String::new(),
            stderr: "umbel: wait timed out. Last tmux pane:\n".to_owned(),
        };
        let reason = unavailable_reason(&waited, pane).expect("a reason");
        assert!(reason.contains("wait timed out"), "{reason}");
        assert!(reason.contains("authenticate"), "{reason}");
    }

    #[test]
    fn a_finished_turn_with_a_quiet_pane_has_no_reason_and_the_stem_answers_for_it() {
        assert_eq!(unavailable_reason(&Ran::default(), "all well\n"), None);
    }

    // ------------------------------------------------------------------ what is planted

    #[test]
    fn claude_is_planted_when_the_projects_settings_enable_the_plugin() {
        let settings = r#"{"enabledPlugins": {"plotplot@plotplot-local": true}}"#;
        assert!(planted(Harness::Claude, Some(settings)).is_ok());

        let off = r#"{"enabledPlugins": {"plotplot@plotplot-local": false}}"#;
        let refused = planted(Harness::Claude, Some(off)).expect_err("not enabled");
        assert!(refused.contains("does not enable"), "{refused}");

        let absent = planted(Harness::Claude, None).expect_err("no settings file");
        assert!(absent.contains(install::CLAUDE_SETTINGS), "{absent}");
    }

    #[test]
    fn the_other_two_are_planted_when_their_project_file_calls_the_dispatcher() {
        let hooks = r#"{"hooks": {"PreToolUse": [{"hooks": [
            {"type": "command", "command": "\"/repo/.plotplot/bundles/codex/bin/plotplot\" hook codex PreToolUse"}
        ]}]}}"#;
        let detail = planted(Harness::Codex, Some(hooks)).expect("planted");
        assert!(detail.contains("1 dispatcher entries"), "{detail}");

        // The same file, registering somebody else's hook.
        let other = r#"{"hooks": {"PreToolUse": [{"hooks": [
            {"type": "command", "command": "/usr/local/bin/something else"}
        ]}]}}"#;
        let refused = planted(Harness::Codex, Some(other)).expect_err("not the stem");
        assert!(refused.contains("plotplot hook codex"), "{refused}");

        let broken = planted(Harness::Gemini, Some("{oh no")).expect_err("not JSON");
        assert!(broken.contains(install::GEMINI_SETTINGS), "{broken}");
    }

    // ------------------------------------------------------------------ the probe

    /// A repository planted far enough for live mode: the one project file the probe reads.
    fn planted_repo(harness: Harness, config: &str) -> tempfile::TempDir {
        let temp = tempfile::tempdir().expect("a temporary root");
        let path = temp.path().join(project_scope(harness));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the project directory");
        }
        fs::write(&path, config).expect("the project configuration");
        temp
    }

    const CODEX_HOOKS: &str = r#"{"hooks": {"PreToolUse": [{"hooks": [
        {"type": "command", "command": "plotplot hook codex PreToolUse"}
    ]}]}}"#;

    #[test]
    fn an_unplanted_harness_earns_one_line_and_no_session_is_started() {
        let temp = tempfile::tempdir().expect("a temporary root");
        let driver = Recorded::new(Ran::default(), "");
        let mut stderr = Vec::new();

        let findings = probe_all(
            temp.path(),
            &[Harness::Gemini],
            DEFAULT_TIMEOUT,
            &driver,
            &mut stderr,
        );
        assert_eq!(
            verdicts(&findings),
            [("gemini session", Verdict::Unavailable)]
        );
        assert!(driver.calls().is_empty(), "{:?}", driver.calls());
    }

    #[test]
    fn a_probe_spawns_sends_waits_captures_and_always_kills_what_it_started() {
        let temp = planted_repo(Harness::Codex, CODEX_HOOKS);
        let driver = Recorded::new(Ran::default(), "");
        let mut stderr = Vec::new();

        let findings = probe_all(
            temp.path(),
            &[Harness::Codex],
            Duration::from_secs(30),
            &driver,
            &mut stderr,
        );

        let session = session_name(Harness::Codex, std::process::id());
        assert_eq!(
            driver.calls(),
            [
                format!("spawn {session} codex"),
                format!("send {session} 3"),
                format!("wait {session} 30s idle {}s", IDLE.as_secs()),
                format!("capture {session}"),
                format!("kill {session}"),
            ]
        );
        // Nothing wrote a record, so the session ran and proved nothing: three failures.
        assert_eq!(
            verdicts(&findings),
            [
                ("codex session", Verdict::Ok),
                ("codex before-tool", Verdict::Fail),
                ("codex session-end", Verdict::Fail),
                ("codex receipt draft", Verdict::Fail),
            ]
        );
        assert!(failed(&findings));
    }

    #[test]
    fn a_worker_blocked_at_a_prompt_is_unavailable_and_is_still_killed() {
        let temp = planted_repo(Harness::Codex, CODEX_HOOKS);
        let waited = Ran {
            code: 126,
            stdout: String::new(),
            stderr: "umbel: session is waiting for input — Please run /login\n".to_owned(),
        };
        let driver = Recorded::new(waited, "  Not logged in · Please run /login\n");
        let mut stderr = Vec::new();

        let findings = probe_all(
            temp.path(),
            &[Harness::Codex],
            DEFAULT_TIMEOUT,
            &driver,
            &mut stderr,
        );
        assert_eq!(
            verdicts(&findings),
            [("codex session", Verdict::Unavailable)]
        );
        assert!(findings[0].detail.contains("/login"), "{findings:?}");
        assert!(
            !failed(&findings),
            "a vendor that will not run is not a failed stem"
        );
        let session = session_name(Harness::Codex, std::process::id());
        assert!(
            driver.calls().contains(&format!("kill {session}")),
            "{:?}",
            driver.calls()
        );
    }

    #[test]
    fn a_spawn_that_refuses_earns_one_line_and_kills_nothing() {
        let temp = planted_repo(Harness::Codex, CODEX_HOOKS);
        let mut driver = Recorded::new(Ran::default(), "");
        driver.spawn = Ran {
            code: 1,
            stdout: String::new(),
            stderr: "Provider 'codex' has no unattended mode\n".to_owned(),
        };
        let mut stderr = Vec::new();

        let findings = probe_all(
            temp.path(),
            &[Harness::Codex],
            DEFAULT_TIMEOUT,
            &driver,
            &mut stderr,
        );
        assert_eq!(
            verdicts(&findings),
            [("codex session", Verdict::Unavailable)]
        );
        assert!(
            findings[0].detail.contains("no session started"),
            "{findings:?}"
        );
        assert_eq!(driver.calls().len(), 1, "{:?}", driver.calls());
    }

    // ------------------------------------------------------------------ the table

    #[test]
    fn the_table_carries_one_aligned_line_per_finding() {
        let findings = [
            finding(
                "claude session",
                Verdict::Ok,
                "session plotplot-doctor-claude-1",
            ),
            finding("gemini session", Verdict::Unavailable, "not planted here"),
        ];
        let table = render(&findings);
        assert_eq!(
            table,
            "claude session  ok           session plotplot-doctor-claude-1\n\
             gemini session  unavailable  not planted here\n"
        );
    }

    #[test]
    fn a_session_name_says_whose_it_is() {
        assert_eq!(
            session_name(Harness::Claude, 4242),
            "plotplot-doctor-claude-4242"
        );
        // umbel takes lowercase, digits and dashes, up to 63 characters.
        for harness in Harness::ALL {
            let name = session_name(harness, u32::MAX);
            assert!(name.len() <= 63, "{name}");
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{name}"
            );
        }
    }
}
