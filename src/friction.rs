//! The friction emitter: where the repository is hard, as one JSON line per event.
//!
//! The record shape, the envelope, the pinned kinds and the per-kind attributes are
//! `contracts/friction-profile.md`, exactly; `docs/plans/friction-ledger.md` §3 to §5 is the
//! reasoning. Nothing here asks a model anything and nothing here copies a field the profile
//! does not name: prompt text, tool output, file content and absolute home paths never reach
//! a record, because only the fields below are ever read.
//!
//! [`derive`] is pure over its arguments, so it cannot read a clock; [`emit`] stamps the
//! records it produces with the clock it was given and appends them. That split is why a
//! record leaves `derive` with an empty `time`: the time is a side effect, and side effects
//! live at the edge.
//!
//! This module also owns the payload readers the other hook-time faces need — the shell
//! command behind a vendor's shell tool, the paths a tool input names, and a path made
//! repo-relative — so that `deny.rs` and `receipt.rs` share one reading of a payload rather
//! than three.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli::{FrictionArgs, FrictionFace};
use crate::error::{Error, Result};
use crate::harness::{Harness, Payload, parse_payload};
use crate::layout;

/// `event.name` on every record this emitter writes.
pub const EVENT_NAME: &str = "plotplot.friction";

/// How far back a repeated tool input still counts as a retry, in before-tool payloads.
pub const RETRY_WINDOW: usize = 5;

// ---------------------------------------------------------------------------------------
// the clock
// ---------------------------------------------------------------------------------------

/// The one thing [`emit`] cannot compute: the moment it is running.
///
/// A parameter rather than a call to the system clock, so a test pins the time and the
/// journal it asserts on is a value. The CLI passes [`SystemClock`].
pub trait Now {
    /// The current instant as RFC 3339 in UTC, ending in `Z`, as the envelope requires.
    fn rfc3339_utc(&self) -> String;
}

/// The machine's clock, read in UTC.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SystemClock;

impl Now for SystemClock {
    fn rfc3339_utc(&self) -> String {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(since) => i64::try_from(since.as_millis()).unwrap_or(i64::MAX),
            Err(before) => i64::try_from(before.duration().as_millis())
                .map(|m| -m)
                .unwrap_or(i64::MIN),
        };
        rfc3339_from_unix_millis(millis)
    }
}

impl<F: Fn() -> String> Now for F {
    fn rfc3339_utc(&self) -> String {
        self()
    }
}

/// Unix milliseconds as an RFC 3339 UTC timestamp.
///
/// The civil date comes from Howard Hinnant's `civil_from_days`, which is total over every
/// `i64` and needs no calendar table.
fn rfc3339_from_unix_millis(millis: i64) -> String {
    let seconds = millis.div_euclid(1_000);
    let sub = millis.rem_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let secs_of_day = seconds.rem_euclid(86_400);

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{sub:03}Z",
        secs_of_day / 3_600,
        (secs_of_day % 3_600) / 60,
        secs_of_day % 60,
    )
}

// ---------------------------------------------------------------------------------------
// the record
// ---------------------------------------------------------------------------------------

/// The pinned kinds of `contracts/friction-profile.md`.
///
/// All thirteen are here because the vocabulary is the contract's, not the stem's; the
/// harness-session emitter below produces nine of them, and `gate.retry`, `worker.wedged`
/// and `model.call` come from pleach, umbel and mull writing into the same journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    #[serde(rename = "tool.denied")]
    ToolDenied,
    #[serde(rename = "tool.failed")]
    ToolFailed,
    #[serde(rename = "tool.retry")]
    ToolRetry,
    #[serde(rename = "file.reread")]
    FileReread,
    #[serde(rename = "search.fanout")]
    SearchFanout,
    #[serde(rename = "edit.churn")]
    EditChurn,
    #[serde(rename = "test.loop")]
    TestLoop,
    #[serde(rename = "stop.refused")]
    StopRefused,
    #[serde(rename = "context.compacted")]
    ContextCompacted,
    #[serde(rename = "session.ended")]
    SessionEnded,
    #[serde(rename = "gate.retry")]
    GateRetry,
    #[serde(rename = "worker.wedged")]
    WorkerWedged,
    #[serde(rename = "model.call")]
    ModelCall,
}

impl Kind {
    /// The dotted name the profile spells.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::ToolDenied => "tool.denied",
            Kind::ToolFailed => "tool.failed",
            Kind::ToolRetry => "tool.retry",
            Kind::FileReread => "file.reread",
            Kind::SearchFanout => "search.fanout",
            Kind::EditChurn => "edit.churn",
            Kind::TestLoop => "test.loop",
            Kind::StopRefused => "stop.refused",
            Kind::ContextCompacted => "context.compacted",
            Kind::SessionEnded => "session.ended",
            Kind::GateRetry => "gate.retry",
            Kind::WorkerWedged => "worker.wedged",
            Kind::ModelCall => "model.call",
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a path is, for the reducer that groups by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathKind {
    Prod,
    Test,
    Doc,
    Config,
    /// Outside the repository. The path itself is never recorded, only this.
    Outside,
}

impl PathKind {
    /// The lowercase name the profile spells.
    pub fn as_str(self) -> &'static str {
        match self {
            PathKind::Prod => "prod",
            PathKind::Test => "test",
            PathKind::Doc => "doc",
            PathKind::Config => "config",
            PathKind::Outside => "outside",
        }
    }
}

impl std::fmt::Display for PathKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One line of `.plotplot/friction/<yyyy-mm>.jsonl`.
///
/// Field names are the profile's, in the profile's order. An `Option<Option<T>>` is the
/// difference between "the profile does not ask for this key on this kind" (absent) and
/// "the key is required and the harness never told us" (present, `null`); the profile makes
/// that distinction explicitly for `gen_ai.request.model`, and the per-kind table repeats it
/// for the session-end token counts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// UTC, RFC 3339, ending in `Z`. Empty until [`emit`] stamps it.
    pub time: String,
    #[serde(rename = "event.name")]
    pub event_name: String,
    #[serde(rename = "plotplot.kind")]
    pub kind: Kind,
    #[serde(rename = "plotplot.harness")]
    pub harness: String,
    #[serde(
        rename = "plotplot.harness.version",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub harness_version: Option<String>,
    #[serde(
        rename = "gen_ai.request.model",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub model: Option<String>,
    #[serde(rename = "gen_ai.conversation.id")]
    pub conversation_id: String,
    #[serde(
        rename = "gen_ai.operation.name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_name: Option<String>,
    #[serde(
        rename = "gen_ai.tool.name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_name: Option<String>,
    #[serde(
        rename = "gen_ai.tool.call.id",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_call_id: Option<Option<String>>,
    #[serde(
        rename = "gen_ai.conversation.compacted",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compacted: Option<bool>,
    #[serde(
        rename = "gen_ai.usage.input_tokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub input_tokens: Option<Option<u64>>,
    #[serde(
        rename = "gen_ai.usage.output_tokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub output_tokens: Option<Option<u64>>,
    #[serde(
        rename = "plotplot.path",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub path: Option<String>,
    #[serde(
        rename = "plotplot.path.kind",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub path_kind: Option<PathKind>,
    #[serde(rename = "plotplot.rule", default)]
    pub rule: Option<String>,
    #[serde(
        rename = "error.type",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_type: Option<String>,
    #[serde(rename = "plotplot.count")]
    pub count: u64,
    #[serde(rename = "plotplot.agent.id", default)]
    pub agent_id: Option<String>,
}

impl Record {
    /// The same record with the envelope's `time` filled in.
    pub fn stamped(mut self, time: &str) -> Record {
        self.time = time.to_owned();
        self
    }
}

/// The `plotplot.harness` value for a vendor, as the profile spells it.
pub fn harness_label(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude => "claude-code",
        Harness::Gemini => "gemini-cli",
        Harness::Codex => "codex-cli",
    }
}

/// The harness's own version, when the payload carries one.
///
/// None of the three vendors put it in a hook payload as of the versions in the brief's §5,
/// so this is `None` in practice; the key it reads is the one a vendor would use if it
/// started, and the profile omits `plotplot.harness.version` rather than nulling it.
pub fn harness_version(payload: &Payload) -> Option<String> {
    let raw = payload.raw.as_object()?;
    if let Some(version) = raw.get("harness_version").and_then(Value::as_str) {
        return Some(version.to_owned());
    }
    raw.get("harness")?
        .get("version")?
        .as_str()
        .map(str::to_owned)
}

/// The token counts a payload carries, input then output; never estimated.
///
/// Read from a `usage` object with `input_tokens` and `output_tokens`. No installed harness
/// puts usage in a session-end payload today, so both are `None` on every fixture; a vendor
/// that starts is a change here and nowhere else.
pub fn usage_tokens(payload: &Payload) -> (Option<u64>, Option<u64>) {
    let usage = payload.raw.get("usage");
    let count = |key: &str| usage.and_then(|u| u.get(key)).and_then(Value::as_u64);
    (count("input_tokens"), count("output_tokens"))
}

// ---------------------------------------------------------------------------------------
// reading a payload
// ---------------------------------------------------------------------------------------

/// The shell command behind a vendor's shell tool, when this payload is one.
///
/// Claude's `Bash`, Gemini's `run_shell_command` and Codex's `shell`. Codex passes an argv
/// array; `["bash", "-lc", "<script>"]` is unwrapped to the script, because the script is
/// what a rule reads.
pub fn shell_command(payload: &Payload) -> Option<String> {
    if !is_shell_tool(payload.harness, payload.tool_name.as_deref()?) {
        return None;
    }
    let command = payload.tool_input.as_ref()?.get("command")?;
    if let Some(text) = command.as_str() {
        return Some(text.to_owned());
    }
    let words: Vec<&str> = command
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    match words.split_first() {
        Some((shell, rest))
            if matches!(*shell, "bash" | "sh" | "zsh")
                && rest
                    .first()
                    .is_some_and(|flag| flag.starts_with('-') && flag.contains('c')) =>
        {
            rest.get(1).map(|script| (*script).to_owned())
        }
        Some(_) => Some(words.join(" ")),
        None => None,
    }
}

/// Whether this vendor calls its shell tool by this name.
pub fn is_shell_tool(harness: Harness, tool: &str) -> bool {
    matches!(
        (harness, tool),
        (Harness::Claude, "Bash")
            | (Harness::Gemini, "run_shell_command")
            | (Harness::Codex, "shell")
    )
}

/// A tool name with an MCP prefix removed: `mcp__tilth__tilth_read` reads as `tilth_read`.
pub fn bare_tool_name(tool: &str) -> &str {
    let Some(rest) = tool.strip_prefix("mcp__") else {
        return tool;
    };
    match rest.rsplit("__").next() {
        Some(bare) if !bare.is_empty() => bare,
        _ => tool,
    }
}

/// A shell command as tokens, whitespace separating words and `;|&<>` forming their own.
///
/// Quoting is deliberately not interpreted: the rules that read these tokens match a flag or
/// a path by the token it is, and a quoted `--no-verify` is still a `--no-verify`.
pub fn shell_tokens(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut operator = String::new();
    for character in command.chars() {
        let is_operator = matches!(character, ';' | '|' | '&' | '<' | '>');
        if !is_operator && !operator.is_empty() {
            tokens.push(std::mem::take(&mut operator));
        }
        if character.is_whitespace() {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
        } else if is_operator {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            operator.push(character);
        } else {
            word.push(character);
        }
    }
    for last in [word, operator] {
        if !last.is_empty() {
            tokens.push(last);
        }
    }
    tokens
}

/// The tokens of one command, split at the operators that end a command.
pub fn shell_segments(tokens: &[String]) -> Vec<&[String]> {
    tokens
        .split(|token| matches!(token.as_str(), ";" | "|" | "||" | "&" | "&&"))
        .filter(|segment| !segment.is_empty())
        .collect()
}

/// A token with one layer of surrounding quotes removed, for a path written `"like this"`.
pub fn unquote(token: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = token
            .strip_prefix(quote)
            .and_then(|t| t.strip_suffix(quote))
        {
            return inner;
        }
    }
    token.trim_matches(['"', '\''])
}

/// Every path a tool input names, under the keys the three vendors and tilth use.
pub fn input_paths(input: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    for key in ["file_path", "absolute_path", "path", "filePath"] {
        if let Some(text) = input.get(key).and_then(Value::as_str) {
            paths.push(text.to_owned());
        }
    }
    if let Some(list) = input.get("paths").and_then(Value::as_array) {
        paths.extend(list.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    if let Some(patch) = input.get("patch").and_then(Value::as_str) {
        paths.extend(patch_paths(patch));
    }
    paths.sort();
    paths.dedup();
    paths
}

/// The files a Codex `apply_patch` names, from its `*** … File:` lines and nothing else.
pub fn patch_paths(patch: &str) -> Vec<String> {
    patch
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("*** ")?;
            for verb in ["Update File:", "Add File:", "Delete File:"] {
                if let Some(path) = rest.strip_prefix(verb) {
                    return Some(path.trim().to_owned());
                }
            }
            None
        })
        .collect()
}

/// A path as the ledger records it: relative to `base`, or outside the repository.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoPath {
    /// Inside `base`, forward-slash separated, never absolute.
    Inside(String),
    /// Outside `base`. The value is never recorded, only that it was outside.
    Outside,
}

/// Make a path the ledger can record, against the payload's own working directory.
pub fn repo_path(base: &Path, raw: &str) -> RepoPath {
    let path = Path::new(raw);
    let relative = if path.is_absolute() {
        match path.strip_prefix(base) {
            Ok(relative) => relative,
            Err(_) => return RepoPath::Outside,
        }
    } else {
        path
    };

    let mut parts: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return RepoPath::Outside;
            }
        }
    }
    if parts.is_empty() {
        return RepoPath::Outside;
    }
    RepoPath::Inside(parts.join("/"))
}

/// What a repo-relative path is, per the profile's five kinds.
pub fn path_kind(relative: &str) -> PathKind {
    let segments: Vec<&str> = relative.split('/').collect();
    let name = segments.last().copied().unwrap_or(relative);
    let directories = &segments[..segments.len().saturating_sub(1)];

    if directories
        .iter()
        .any(|segment| matches!(*segment, "test" | "tests" | "spec" | "__tests__"))
        || name.contains(".test.")
        || name.contains(".spec.")
    {
        return PathKind::Test;
    }
    let extension = match name.rsplit_once('.') {
        Some((_, extension)) => extension,
        // A name with no dot has no extension. That is what the file is called, not a
        // reading that failed, so it falls through to the kinds that do not need one.
        None => "",
    };
    let at_root = directories.is_empty();
    if matches!(extension, "md" | "txt" | "html")
        && (at_root || directories.first() == Some(&"docs"))
    {
        return PathKind::Doc;
    }
    if name.starts_with('.') || (at_root && matches!(extension, "toml" | "json" | "yaml" | "yml")) {
        return PathKind::Config;
    }
    PathKind::Prod
}

// ---------------------------------------------------------------------------------------
// session state
// ---------------------------------------------------------------------------------------

/// What one session has to remember for the kinds that count repetition.
///
/// Keys are repo-relative paths; a path outside the repository is keyed by the sha256 of its
/// text, so a re-read outside still counts without the state file holding a home path.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    /// How often each path was read.
    pub reads: BTreeMap<String, u64>,
    /// How often each path was edited.
    pub edits: BTreeMap<String, u64>,
    /// How often each test runner was run.
    pub tests: BTreeMap<String, u64>,
    /// How often each tool was called, for the receipt draft.
    pub tool_calls: BTreeMap<String, u64>,
    /// The last [`RETRY_WINDOW`] before-tool input hashes, oldest first.
    pub recent_inputs: Vec<String>,
    /// Whether this session has edited anything yet; searches before that are fanout.
    pub edited: bool,
}

impl SessionState {
    /// A session that has done nothing yet.
    pub fn new() -> SessionState {
        SessionState::default()
    }
}

// ---------------------------------------------------------------------------------------
// deriving records
// ---------------------------------------------------------------------------------------

/// What one tool call did, as far as the ledger cares.
enum Action {
    Read(Vec<String>),
    Search,
    Edit(Vec<String>),
    Test(String),
    Nothing,
}

/// Everything every record on this payload shares.
struct Envelope<'a> {
    payload: &'a Payload,
    base: PathBuf,
    conversation: String,
}

impl Envelope<'_> {
    fn record(&self, kind: Kind) -> Record {
        Record {
            time: String::new(),
            event_name: EVENT_NAME.to_owned(),
            kind,
            harness: harness_label(self.payload.harness).to_owned(),
            harness_version: harness_version(self.payload),
            model: self.payload.model.clone(),
            conversation_id: self.conversation.clone(),
            operation_name: None,
            tool_name: None,
            tool_call_id: None,
            compacted: None,
            input_tokens: None,
            output_tokens: None,
            path: None,
            path_kind: None,
            rule: None,
            error_type: None,
            count: 1,
            agent_id: self.payload.agent_id.clone(),
        }
    }

    /// The record with the path attributes the profile allows for it.
    fn with_path(&self, mut record: Record, path: &RepoPath) -> Record {
        match path {
            RepoPath::Inside(relative) => {
                record.path_kind = Some(path_kind(relative));
                record.path = Some(relative.clone());
            }
            RepoPath::Outside => record.path_kind = Some(PathKind::Outside),
        }
        record
    }

    fn tool(&self) -> Option<String> {
        self.payload.tool_name.clone()
    }
}

/// The records one payload earns, given what the session has done so far.
///
/// Pure over its arguments: it reads the payload and the state, updates the state, and
/// returns records whose `time` [`emit`] fills in. `root` is the repository root, used as the
/// base for making paths relative when the payload names no working directory of its own.
pub fn derive(root: &Path, payload: &Payload, state: &mut SessionState) -> Vec<Record> {
    let Some(conversation) = payload.session_id.clone() else {
        return Vec::new();
    };
    let envelope = Envelope {
        payload,
        base: payload.cwd.clone().unwrap_or_else(|| root.to_path_buf()),
        conversation,
    };
    let harness = payload.harness;
    let event = payload.event.as_str();

    if event == harness.before_tool_event() {
        return before_tool(&envelope, state);
    }
    if event == harness.session_end_event() {
        return vec![session_ended(&envelope)];
    }
    match event {
        "PostToolUse" | "PostToolUseFailure" | "AfterTool" => post_tool(&envelope, state),
        "PermissionDenied" => tool_denied(&envelope),
        "PreCompact" | "PreCompress" => vec![context_compacted(&envelope)],
        _ => Vec::new(),
    }
}

fn before_tool(envelope: &Envelope, state: &mut SessionState) -> Vec<Record> {
    let Some(tool) = envelope.tool() else {
        return Vec::new();
    };
    // A payload carrying no input hashes as the empty string, which no JSON value's own
    // rendering can produce, so "this call named no input" stays a value of its own and is
    // never confused with some input that happened to be short.
    let input = match envelope.payload.tool_input.as_ref() {
        Some(input) => input.to_string(),
        None => String::new(),
    };
    let hash = hex::encode(Sha256::digest(format!("{tool}\u{0}{input}").as_bytes()));

    let repeated = state.recent_inputs.iter().any(|seen| seen == &hash);
    state.recent_inputs.push(hash);
    let overflow = state.recent_inputs.len().saturating_sub(RETRY_WINDOW);
    state.recent_inputs.drain(..overflow);

    if !repeated {
        return Vec::new();
    }
    let mut record = envelope.record(Kind::ToolRetry);
    record.tool_name = Some(tool);
    record.tool_call_id = Some(envelope.payload.tool_call_id.clone());
    vec![record]
}

fn post_tool(envelope: &Envelope, state: &mut SessionState) -> Vec<Record> {
    let Some(tool) = envelope.tool() else {
        return Vec::new();
    };
    *state.tool_calls.entry(tool.clone()).or_insert(0) += 1;

    if let Some(error) = error_type(envelope.payload) {
        let mut record = envelope.record(Kind::ToolFailed);
        record.operation_name = Some("execute_tool".to_owned());
        record.tool_name = Some(tool);
        record.error_type = Some(error);
        return vec![first_path(envelope, record)];
    }

    match action(envelope.payload) {
        Action::Read(paths) => {
            repeated_paths(envelope, state, paths, Kind::FileReread, &tool, |s| {
                &mut s.reads
            })
        }
        Action::Edit(paths) => {
            state.edited = true;
            repeated_paths(envelope, state, paths, Kind::EditChurn, &tool, |s| {
                &mut s.edits
            })
        }
        Action::Search => {
            if state.edited {
                return Vec::new();
            }
            let mut record = envelope.record(Kind::SearchFanout);
            record.tool_name = Some(tool);
            vec![record]
        }
        Action::Test(runner) => {
            let seen = state.tests.entry(runner).or_insert(0);
            *seen += 1;
            if *seen < 2 {
                return Vec::new();
            }
            let mut record = envelope.record(Kind::TestLoop);
            record.tool_name = Some(tool);
            vec![record]
        }
        Action::Nothing => Vec::new(),
    }
}

/// One record per path this session has already touched under `counter`.
fn repeated_paths(
    envelope: &Envelope,
    state: &mut SessionState,
    paths: Vec<String>,
    kind: Kind,
    tool: &str,
    counter: fn(&mut SessionState) -> &mut BTreeMap<String, u64>,
) -> Vec<Record> {
    let mut records = Vec::new();
    for raw in paths {
        let path = repo_path(&envelope.base, &raw);
        let key = match &path {
            RepoPath::Inside(relative) => relative.clone(),
            RepoPath::Outside => hex::encode(Sha256::digest(raw.as_bytes())),
        };
        let seen = counter(state).entry(key).or_insert(0);
        *seen += 1;
        if *seen < 2 {
            continue;
        }
        let mut record = envelope.record(kind);
        record.tool_name = Some(tool.to_owned());
        records.push(envelope.with_path(record, &path));
    }
    records
}

fn tool_denied(envelope: &Envelope) -> Vec<Record> {
    let Some(tool) = envelope.tool() else {
        return Vec::new();
    };
    let mut record = envelope.record(Kind::ToolDenied);
    record.operation_name = Some("execute_tool".to_owned());
    record.tool_name = Some(tool);
    vec![first_path(envelope, record)]
}

fn context_compacted(envelope: &Envelope) -> Record {
    let mut record = envelope.record(Kind::ContextCompacted);
    record.compacted = Some(true);
    record
}

fn session_ended(envelope: &Envelope) -> Record {
    let (input, output) = usage_tokens(envelope.payload);
    let mut record = envelope.record(Kind::SessionEnded);
    record.input_tokens = Some(input);
    record.output_tokens = Some(output);
    record
}

/// The record with the first path its tool names, when the tool names one.
fn first_path(envelope: &Envelope, record: Record) -> Record {
    match tool_paths(envelope.payload).first() {
        Some(raw) => envelope.with_path(record, &repo_path(&envelope.base, raw)),
        None => record,
    }
}

/// Every path this payload's tool names, whether through its input or its command.
fn tool_paths(payload: &Payload) -> Vec<String> {
    if let Some(command) = shell_command(payload) {
        let tokens = shell_tokens(&command);
        let mut paths = redirection_targets(&tokens);
        if paths.is_empty() {
            paths = read_targets(&tokens);
        }
        return paths;
    }
    input_paths_of(payload)
}

/// The paths this payload's tool input names, and none when it carries no input.
///
/// Nothing is dropped here: a payload with no `tool_input` names no path, and no path is a
/// value the ledger has a meaning for, not a reading that failed. Where the difference
/// matters — a write whose target the boundary has to see — the caller reads `tool_input`
/// itself rather than this, which is what [`crate::deny`] does.
fn input_paths_of(payload: &Payload) -> Vec<String> {
    match payload.tool_input.as_ref() {
        Some(input) => input_paths(input),
        None => Vec::new(),
    }
}

/// The `error.type` for a failed tool call, or `None` when the call succeeded.
///
/// A vendor's error message is tool output, which the profile forbids, so the type is
/// structural: the exit code when there is one, and `tool_error` when there is not.
fn error_type(payload: &Payload) -> Option<String> {
    let code = payload
        .tool_response
        .as_ref()
        .and_then(|response| response.get("exit_code"))
        .and_then(Value::as_i64);
    let response_error = payload
        .tool_response
        .as_ref()
        .and_then(|response| response.get("error"))
        .is_some_and(|error| !error.is_null());

    let failed = payload.event == "PostToolUseFailure"
        || payload.tool_error.is_some()
        || response_error
        || matches!(code, Some(code) if code != 0);
    if !failed {
        return None;
    }
    match code {
        Some(code) if code != 0 => Some(format!("exit_code:{code}")),
        _ => Some("tool_error".to_owned()),
    }
}

const TEST_RUNNERS: [&[&str]; 9] = [
    &["cargo", "test"],
    &["npm", "test"],
    &["pnpm", "test"],
    &["bun", "test"],
    &["go", "test"],
    &["node", "--test"],
    &["pytest"],
    &["vitest"],
    &["jest"],
];

const READ_TOOLS: [&str; 3] = ["Read", "tilth_read", "read_file"];
const SEARCH_TOOLS: [&str; 5] = ["Grep", "Glob", "tilth_search", "grep_search", "glob"];
const EDIT_TOOLS: [&str; 7] = [
    "Edit",
    "Write",
    "MultiEdit",
    "tilth_write",
    "write_file",
    "replace",
    "apply_patch",
];

/// What a successful tool call did.
fn action(payload: &Payload) -> Action {
    let Some(tool) = payload.tool_name.as_deref() else {
        return Action::Nothing;
    };
    if let Some(command) = shell_command(payload) {
        return command_action(&command);
    }
    let bare = bare_tool_name(tool);
    if READ_TOOLS.contains(&bare) {
        return Action::Read(input_paths_of(payload));
    }
    if EDIT_TOOLS.contains(&bare) {
        return Action::Edit(input_paths_of(payload));
    }
    if SEARCH_TOOLS.contains(&bare) {
        return Action::Search;
    }
    Action::Nothing
}

/// What a shell command did: run tests, search, or read files.
fn command_action(command: &str) -> Action {
    let tokens = shell_tokens(command);
    let segments = shell_segments(&tokens);

    for segment in &segments {
        if let Some(runner) = test_runner(segment) {
            return Action::Test(runner);
        }
    }
    for segment in &segments {
        if segment.first().map(String::as_str) == Some("rg") {
            return Action::Search;
        }
    }
    // A command that writes is not a read, whatever it reads from: `cat > file` is a write.
    if !redirection_targets(&tokens).is_empty() {
        return Action::Nothing;
    }
    let paths = read_targets(&tokens);
    if paths.is_empty() {
        Action::Nothing
    } else {
        Action::Read(paths)
    }
}

/// The test runner a command segment starts with, as the key the session counts it by.
fn test_runner(segment: &[String]) -> Option<String> {
    let words: Vec<&str> = segment.iter().map(String::as_str).collect();
    TEST_RUNNERS
        .iter()
        .find(|runner| words.starts_with(runner))
        .map(|runner| runner.join(" "))
}

/// The files a command reads with `cat` or `sed -n`.
fn read_targets(tokens: &[String]) -> Vec<String> {
    let mut paths = Vec::new();
    for segment in shell_segments(tokens) {
        let Some((command, arguments)) = segment.split_first() else {
            continue;
        };
        match command.as_str() {
            "cat" => paths.extend(
                arguments
                    .iter()
                    .filter(|argument| !argument.starts_with('-'))
                    .map(|argument| unquote(argument).to_owned()),
            ),
            "sed" if arguments.iter().any(|argument| argument == "-n") => {
                if let Some(path) = arguments.last() {
                    if !path.starts_with('-') {
                        paths.push(unquote(path).to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    paths
}

/// The files a command writes through `>`, `>>` or `tee`.
pub fn redirection_targets(tokens: &[String]) -> Vec<String> {
    let mut paths = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let target = match token.as_str() {
            ">" | ">>" => tokens.get(index + 1),
            "tee" => tokens[index + 1..]
                .iter()
                .find(|argument| !argument.starts_with('-')),
            _ => None,
        };
        if let Some(target) = target {
            if !matches!(target.as_str(), ";" | "|" | "||" | "&" | "&&" | "<" | ">") {
                paths.push(unquote(target).to_owned());
            }
        }
    }
    paths
}

// ---------------------------------------------------------------------------------------
// the edge
// ---------------------------------------------------------------------------------------

/// Derive this payload's records, append them to the month's journal, and keep the session.
///
/// Returns the number of lines written. A payload with no session id writes nothing and
/// returns `Ok(0)`, because a record without `gen_ai.conversation.id` is not a record the
/// profile allows. The state file is pruned at the harness's session-end event, so
/// [`crate::receipt::draft`] must read it before this runs on that event.
pub fn emit(root: &Path, payload: &Payload, now: &dyn Now) -> Result<usize> {
    let Some(session) = payload.session_id.as_deref() else {
        return Ok(0);
    };
    let session = session_file_name(session)?;
    let time = now.rfc3339_utc();
    let month = month_of(&time)?;

    let state_path = layout::friction_state(root, session);
    let mut state = read_state(&state_path)?;
    let records = derive(root, payload, &mut state);
    append(root, &records, &time, &month)?;

    if payload.event == payload.harness.session_end_event() {
        prune_state(&state_path)?;
    } else {
        write_state(&state_path, &state)?;
    }
    Ok(records.len())
}

/// Why nothing this payload did can reach the journal, when the payload is the reason.
///
/// [`emit`] answers `Ok(0)` for two very different payloads: one that earned no record
/// because nothing about it was friction, and one that could earn none at all because the
/// profile keys every record by `gen_ai.conversation.id` and the harness sent no
/// `session_id`. A count cannot tell those apart, so the second says so here and the
/// dispatcher's trail puts it on stderr. Nothing about the answer to the harness changes:
/// a ledger that cannot write is not a reason to stop an agent, but it is a reason to say
/// out loud that the ledger is missing this session.
pub fn unrecorded_reason(payload: &Payload) -> Option<String> {
    if payload.session_id.is_some() {
        return None;
    }
    Some(format!(
        "plotplot: the {} {} payload carries no session_id, so nothing it did reaches the \
         friction journal; every record the profile allows is keyed by \
         gen_ai.conversation.id",
        payload.harness, payload.event
    ))
}

/// Record the stem's own refusal of a tool call, naming the rule that refused it.
///
/// The profile derives `tool.denied` from "our own PreToolUse decision" as well as from
/// Claude's `PermissionDenied`, and only the dispatcher knows about the first: [`derive`] sees
/// the payload alone, and a payload the stem is about to refuse looks exactly like one it is
/// about to allow. So the dispatcher tells the ledger, and the ledger writes one line.
///
/// This touches no session state. The same payload's before-tool records go through [`emit`],
/// which owns the state; a refusal adds a line, never a second reader and writer of the file.
///
/// # Errors
///
/// [`Error::Io`] when the journal cannot be written, [`Error::Json`] when a record cannot be
/// serialized, and [`Error::Harness`] when the clock's own timestamp names no month.
pub fn emit_denied(root: &Path, payload: &Payload, rule: &str, now: &dyn Now) -> Result<usize> {
    let time = now.rfc3339_utc();
    let month = month_of(&time)?;
    let records = derive_denied(root, payload, rule);
    append(root, &records, &time, &month)?;
    Ok(records.len())
}

/// The one `tool.denied` record a refusal by the stem's own deny list earns.
///
/// Pure over its arguments, like [`derive`], and empty for the same two reasons: a payload
/// with no session id has no `gen_ai.conversation.id`, and a payload naming no tool has none
/// of the per-kind attributes the profile requires for this kind.
pub fn derive_denied(root: &Path, payload: &Payload, rule: &str) -> Vec<Record> {
    let Some(conversation) = payload.session_id.clone() else {
        return Vec::new();
    };
    let envelope = Envelope {
        payload,
        base: payload.cwd.clone().unwrap_or_else(|| root.to_path_buf()),
        conversation,
    };
    let mut records = tool_denied(&envelope);
    for record in &mut records {
        record.rule = Some(rule.to_owned());
    }
    records
}

/// Append these records to the month's journal, stamped with the time they were derived at.
///
/// The one writer of the journal: [`emit`] and [`emit_denied`] both come through here, so the
/// file is created in one place and a line's shape cannot drift between the two.
fn append(root: &Path, records: &[Record], time: &str, month: &str) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    let journal = layout::friction_journal(root, month);

    let mut lines = String::new();
    for record in records.iter().map(|record| record.clone().stamped(time)) {
        let line = serde_json::to_string(&record).map_err(|source| Error::Json {
            path: Some(journal.clone()),
            source,
        })?;
        lines.push_str(&line);
        lines.push('\n');
    }

    create_dir(&layout::friction_dir(root))?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)
        .map_err(|source| Error::Io {
            path: journal.clone(),
            source,
        })?;
    file.write_all(lines.as_bytes())
        .map_err(|source| Error::Io {
            path: journal.clone(),
            source,
        })
}

/// `plotplot friction emit --harness <h>`: the face, from arguments and stdin to an exit code.
///
/// It writes nothing on stdout. The face is registered as a hook entry in every bundle, and a
/// vendor reads a hook's stdout as an answer, so the only thing this face says is its code.
pub fn run(
    root: &Path,
    args: &FrictionArgs,
    stdin: &str,
    _stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let FrictionFace::Emit(arg) = &args.face;
    let written =
        parse_payload(arg.harness, stdin).and_then(|payload| emit(root, &payload, &SystemClock));
    match written {
        Ok(_) => 0,
        Err(error) => {
            // The failure leaves on the code whether or not the sentence does: a caller that
            // has already closed this stream cannot be told anything, and nothing else here
            // can act on a stream that is gone.
            let _ = writeln!(stderr, "{error}");
            1
        }
    }
}

/// The session's state as it stands, or a fresh one when the session is new.
pub fn read_state(path: &Path) -> Result<SessionState> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SessionState::new());
        }
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    serde_json::from_str(&text).map_err(|source| Error::Json {
        path: Some(path.to_path_buf()),
        source,
    })
}

fn write_state(path: &Path, state: &SessionState) -> Result<()> {
    let Some(directory) = path.parent() else {
        return Err(Error::Io {
            path: path.to_path_buf(),
            source: std::io::Error::other("the state path has no directory"),
        });
    };
    create_dir(directory)?;
    let text = serde_json::to_string(state).map_err(|source| Error::Json {
        path: Some(path.to_path_buf()),
        source,
    })?;
    fs::write(path, text).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn prune_state(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// A session id safe to spell inside a file name, or an error naming why it is not.
///
/// The id arrives from a harness, so it is input: a `/` or a `..` in it would put the state
/// file somewhere other than `.plotplot/friction/state/`.
pub fn session_file_name(session: &str) -> Result<&str> {
    let usable = !session.is_empty()
        && session.len() <= 128
        && session
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !session.starts_with('.');
    if usable {
        Ok(session)
    } else {
        Err(Error::Harness {
            problem: format!(
                "the session id {session:?} is not a name the stem can write a file under"
            ),
        })
    }
}

/// The `yyyy-mm` of an RFC 3339 UTC timestamp, or an error naming what the clock produced.
fn month_of(time: &str) -> Result<String> {
    let refused = || Error::Harness {
        problem: format!("the clock produced {time:?}, which is not an RFC 3339 UTC timestamp"),
    };
    let month = time.get(..7).ok_or_else(refused)?;
    let shaped = month.as_bytes().iter().enumerate().all(|(index, byte)| {
        if index == 4 {
            *byte == b'-'
        } else {
            byte.is_ascii_digit()
        }
    });
    if !shaped || !time.ends_with('Z') {
        return Err(refused());
    }
    Ok(month.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::parse_payload;
    use serde_json::json;

    fn payloads() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/payloads")
    }

    fn fixture(harness: Harness, name: &str) -> Payload {
        let path = payloads().join(harness.name()).join(format!("{name}.json"));
        let json = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        parse_payload(harness, &json).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn root() -> &'static Path {
        Path::new("/work/repo")
    }

    fn kinds(records: &[Record]) -> Vec<&'static str> {
        records.iter().map(|record| record.kind.as_str()).collect()
    }

    /// A payload of any tool, as the vendor would shape it.
    fn tool_payload(harness: Harness, event: &str, tool: &str, input: Value) -> Payload {
        let json = json!({
            "hook_event_name": event,
            "session_id": "6f3c1b2a",
            "cwd": "/work/repo",
            "model": "claude-opus-5",
            "tool_name": tool,
            "tool_input": input,
            "tool_use_id": "toolu_01",
        })
        .to_string();
        parse_payload(harness, &json).expect("a parsed payload")
    }

    fn claude_post(tool: &str, input: Value) -> Payload {
        tool_payload(Harness::Claude, "PostToolUse", tool, input)
    }

    fn claude_bash(event: &str, command: &str) -> Payload {
        tool_payload(
            Harness::Claude,
            event,
            "Bash",
            json!({ "command": command }),
        )
    }

    // -----------------------------------------------------------------------------------
    // the profile's tables, written here as data
    // -----------------------------------------------------------------------------------

    /// `contracts/friction-profile.md`, "the envelope (issue 35)".
    const ENVELOPE: [&str; 6] = [
        "time",
        "event.name",
        "plotplot.kind",
        "plotplot.count",
        "plotplot.harness",
        "gen_ai.conversation.id",
    ];

    /// `contracts/friction-profile.md`, "the pinned kinds", column one.
    const PINNED_KINDS: [&str; 13] = [
        "tool.denied",
        "tool.failed",
        "tool.retry",
        "file.reread",
        "search.fanout",
        "edit.churn",
        "test.loop",
        "stop.refused",
        "context.compacted",
        "session.ended",
        "gate.retry",
        "worker.wedged",
        "model.call",
    ];

    /// `contracts/friction-profile.md`, "per-kind attributes beyond the envelope".
    const PER_KIND: [(&str, &[&str]); 13] = [
        (
            "tool.denied",
            &["gen_ai.operation.name", "gen_ai.tool.name"],
        ),
        (
            "tool.failed",
            &["gen_ai.operation.name", "gen_ai.tool.name", "error.type"],
        ),
        ("tool.retry", &["gen_ai.tool.name", "gen_ai.tool.call.id"]),
        ("file.reread", &["gen_ai.tool.name"]),
        ("search.fanout", &["gen_ai.tool.name"]),
        ("edit.churn", &["gen_ai.tool.name"]),
        ("test.loop", &["gen_ai.tool.name"]),
        ("stop.refused", &["plotplot.rule"]),
        ("context.compacted", &["gen_ai.conversation.compacted"]),
        (
            "session.ended",
            &["gen_ai.usage.input_tokens", "gen_ai.usage.output_tokens"],
        ),
        ("gate.retry", &["plotplot.node", "plotplot.gate"]),
        ("worker.wedged", &["plotplot.worker"]),
        (
            "model.call",
            &[
                "gen_ai.provider.name",
                "gen_ai.request.model",
                "gen_ai.usage.input_tokens",
                "gen_ai.usage.output_tokens",
            ],
        ),
    ];

    /// `contracts/friction-profile.md`, "the pinned kinds", the path column.
    const PATH_REQUIRED: [(&str, bool); 13] = [
        ("tool.denied", true),
        ("tool.failed", true),
        ("tool.retry", false),
        ("file.reread", true),
        ("search.fanout", false),
        ("edit.churn", true),
        ("test.loop", true),
        ("stop.refused", false),
        ("context.compacted", false),
        ("session.ended", false),
        ("gate.retry", false),
        ("worker.wedged", false),
        ("model.call", false),
    ];

    /// Where the profile's path column cannot be met by this emitter, and why.
    ///
    /// `test.loop` is a command, not a file, so no payload it derives from names a path;
    /// `tool.denied` and `tool.failed` name one only when the tool that was denied or failed
    /// named one. Reported to the umbrella agent rather than papered over: the record is
    /// still written, with the two path keys absent rather than invented.
    const PATH_GAPS: [&str; 3] = ["test.loop", "tool.denied", "tool.failed"];

    // -----------------------------------------------------------------------------------
    // what each fixture earns
    // -----------------------------------------------------------------------------------

    #[test]
    fn every_fixture_payload_earns_exactly_the_kinds_the_profile_names() {
        let table: [(Harness, &str, &[&str]); 26] = [
            (Harness::Claude, "PermissionDenied", &["tool.denied"]),
            (Harness::Claude, "PostToolUse", &[]),
            (Harness::Claude, "PostToolUseFailure", &["tool.failed"]),
            (Harness::Claude, "PreCompact", &["context.compacted"]),
            (Harness::Claude, "PreToolUse", &[]),
            (Harness::Claude, "SessionEnd", &["session.ended"]),
            (Harness::Claude, "SessionStart", &[]),
            (Harness::Claude, "Stop", &[]),
            (Harness::Claude, "UserPromptSubmit", &[]),
            (Harness::Claude, "leaky-PostToolUse", &[]),
            (Harness::Claude, "no-cwd-PreToolUse", &[]),
            (Harness::Claude, "no-input-PreToolUse", &[]),
            (Harness::Gemini, "AfterAgent", &[]),
            (Harness::Gemini, "AfterTool", &[]),
            (Harness::Gemini, "BeforeTool", &[]),
            (Harness::Gemini, "PreCompress", &["context.compacted"]),
            (Harness::Gemini, "SessionEnd", &["session.ended"]),
            (Harness::Gemini, "SessionStart", &[]),
            (Harness::Gemini, "leaky-AfterTool", &[]),
            (Harness::Codex, "PostToolUse", &[]),
            (Harness::Codex, "PreCompact", &["context.compacted"]),
            (Harness::Codex, "PreToolUse", &[]),
            (Harness::Codex, "SessionStart", &[]),
            (Harness::Codex, "Stop", &["session.ended"]),
            (Harness::Codex, "UserPromptSubmit", &[]),
            (Harness::Codex, "leaky-PostToolUse", &["tool.failed"]),
        ];
        for (harness, name, expected) in table {
            let mut state = SessionState::new();
            let records = derive(root(), &fixture(harness, name), &mut state);
            assert_eq!(kinds(&records), expected.to_vec(), "{harness}/{name}");
        }
    }

    #[test]
    fn a_failure_names_the_kind_of_error_and_never_the_message() {
        let mut state = SessionState::new();
        let records = derive(
            root(),
            &fixture(Harness::Claude, "PostToolUseFailure"),
            &mut state,
        );
        assert_eq!(records[0].error_type.as_deref(), Some("tool_error"));
        assert_eq!(records[0].path.as_deref(), Some("src/lock.rs"));
        assert_eq!(records[0].path_kind, Some(PathKind::Prod));

        let mut state = SessionState::new();
        let records = derive(
            root(),
            &fixture(Harness::Codex, "leaky-PostToolUse"),
            &mut state,
        );
        assert_eq!(records[0].error_type.as_deref(), Some("exit_code:1"));
        assert_eq!(records[0].path.as_deref(), Some("notes/onboarding.md"));
    }

    #[test]
    fn a_denial_names_the_tool_and_the_path_the_tool_aimed_at() {
        let mut state = SessionState::new();
        let records = derive(
            root(),
            &fixture(Harness::Claude, "PermissionDenied"),
            &mut state,
        );
        assert_eq!(records[0].tool_name.as_deref(), Some("Write"));
        assert_eq!(records[0].operation_name.as_deref(), Some("execute_tool"));
        assert_eq!(records[0].path.as_deref(), Some("garden.lock"));
        assert_eq!(records[0].path_kind, Some(PathKind::Prod));
        assert_eq!(records[0].rule, None, "the vendor's reason is not our rule");
    }

    #[test]
    fn a_compaction_says_so_in_the_conventions_own_attribute() {
        for (harness, name) in [
            (Harness::Claude, "PreCompact"),
            (Harness::Gemini, "PreCompress"),
            (Harness::Codex, "PreCompact"),
        ] {
            let mut state = SessionState::new();
            let records = derive(root(), &fixture(harness, name), &mut state);
            assert_eq!(records[0].compacted, Some(true), "{harness}/{name}");
        }
    }

    #[test]
    fn a_session_end_carries_token_counts_only_when_the_payload_does() {
        let mut state = SessionState::new();
        let records = derive(root(), &fixture(Harness::Claude, "SessionEnd"), &mut state);
        assert_eq!(records[0].input_tokens, Some(None));
        assert_eq!(records[0].output_tokens, Some(None));

        let json = json!({
            "hook_event_name": "SessionEnd",
            "session_id": "s",
            "cwd": "/work/repo",
            "usage": {"input_tokens": 12_000, "output_tokens": 900},
        })
        .to_string();
        let payload = parse_payload(Harness::Claude, &json).expect("a parsed payload");
        let records = derive(root(), &payload, &mut SessionState::new());
        assert_eq!(records[0].input_tokens, Some(Some(12_000)));
        assert_eq!(records[0].output_tokens, Some(Some(900)));
    }

    // -----------------------------------------------------------------------------------
    // what a session remembers
    // -----------------------------------------------------------------------------------

    #[test]
    fn the_second_read_of_one_path_is_a_reread_and_the_first_is_not() {
        let mut state = SessionState::new();
        let read = claude_post("Read", json!({"file_path": "/work/repo/src/lock.rs"}));
        assert!(derive(root(), &read, &mut state).is_empty());
        let records = derive(root(), &read, &mut state);
        assert_eq!(kinds(&records), ["file.reread"]);
        assert_eq!(records[0].path.as_deref(), Some("src/lock.rs"));
        assert_eq!(records[0].tool_name.as_deref(), Some("Read"));
        assert_eq!(kinds(&derive(root(), &read, &mut state)), ["file.reread"]);
    }

    #[test]
    fn every_vendors_read_tool_counts_towards_the_same_path() {
        let mut state = SessionState::new();
        let claude = claude_post("Read", json!({"file_path": "/work/repo/src/lock.rs"}));
        let tilth = claude_post("mcp__tilth__tilth_read", json!({"path": "src/lock.rs"}));
        let gemini = tool_payload(
            Harness::Gemini,
            "AfterTool",
            "read_file",
            json!({"absolute_path": "/work/repo/src/lock.rs"}),
        );
        let shell = claude_bash("PostToolUse", "sed -n '1,40p' src/lock.rs");

        assert!(derive(root(), &claude, &mut state).is_empty());
        for repeat in [&tilth, &gemini, &shell] {
            assert_eq!(kinds(&derive(root(), repeat, &mut state)), ["file.reread"]);
        }
        assert_eq!(state.reads.get("src/lock.rs"), Some(&4));
    }

    #[test]
    fn the_second_edit_of_one_path_is_churn() {
        let mut state = SessionState::new();
        let write = claude_post("Write", json!({"file_path": "src/lock.rs"}));
        let edit = claude_post("Edit", json!({"file_path": "/work/repo/src/lock.rs"}));
        assert!(derive(root(), &write, &mut state).is_empty());
        let records = derive(root(), &edit, &mut state);
        assert_eq!(kinds(&records), ["edit.churn"]);
        assert_eq!(records[0].path.as_deref(), Some("src/lock.rs"));
    }

    #[test]
    fn a_codex_patch_counts_every_file_it_names() {
        let patch = "*** Begin Patch\n*** Update File: src/lock.rs\n*** Add File: tests/lock.rs\n*** End Patch\n";
        let apply = tool_payload(
            Harness::Codex,
            "PostToolUse",
            "apply_patch",
            json!({ "patch": patch }),
        );
        let mut state = SessionState::new();
        assert!(derive(root(), &apply, &mut state).is_empty());
        let records = derive(root(), &apply, &mut state);
        assert_eq!(kinds(&records), ["edit.churn", "edit.churn"]);
        assert_eq!(records[0].path.as_deref(), Some("src/lock.rs"));
        assert_eq!(records[1].path.as_deref(), Some("tests/lock.rs"));
        assert_eq!(records[1].path_kind, Some(PathKind::Test));
    }

    #[test]
    fn a_second_run_of_one_test_runner_is_a_test_loop() {
        let mut state = SessionState::new();
        let first = claude_bash("PostToolUse", "cargo test --all-targets");
        let second = claude_bash("PostToolUse", "cargo test -- --nocapture");
        assert!(derive(root(), &first, &mut state).is_empty());
        let records = derive(root(), &second, &mut state);
        assert_eq!(kinds(&records), ["test.loop"]);
        assert_eq!(records[0].tool_name.as_deref(), Some("Bash"));
        assert_eq!(state.tests.get("cargo test"), Some(&2));
    }

    #[test]
    fn every_runner_the_profile_names_is_counted_separately() {
        let mut state = SessionState::new();
        for command in [
            "cargo test",
            "npm test",
            "pnpm test",
            "bun test",
            "go test ./...",
            "node --test",
            "pytest -q",
            "vitest run",
            "jest --ci",
        ] {
            let payload = claude_bash("PostToolUse", command);
            assert!(
                derive(root(), &payload, &mut state).is_empty(),
                "{command} looped on its first run"
            );
        }
        assert_eq!(state.tests.len(), 9, "{:?}", state.tests);
        let payload = claude_bash("PostToolUse", "cargo test");
        assert_eq!(kinds(&derive(root(), &payload, &mut state)), ["test.loop"]);
    }

    #[test]
    fn searches_before_the_first_edit_are_fanout_and_after_it_are_not() {
        let mut state = SessionState::new();
        let grep = claude_post("Grep", json!({"pattern": "derive"}));
        let rg = claude_bash("PostToolUse", "rg derive src");
        assert_eq!(kinds(&derive(root(), &grep, &mut state)), ["search.fanout"]);
        assert_eq!(kinds(&derive(root(), &rg, &mut state)), ["search.fanout"]);

        let edit = claude_post("Write", json!({"file_path": "src/lock.rs"}));
        assert!(derive(root(), &edit, &mut state).is_empty());
        assert!(derive(root(), &grep, &mut state).is_empty());
    }

    #[test]
    fn the_same_input_within_the_window_is_a_retry_and_outside_it_is_not() {
        let mut state = SessionState::new();
        let bash = claude_bash("PreToolUse", "cargo test --all-targets");
        assert!(derive(root(), &bash, &mut state).is_empty());
        let records = derive(root(), &bash, &mut state);
        assert_eq!(kinds(&records), ["tool.retry"]);
        assert_eq!(records[0].tool_name.as_deref(), Some("Bash"));
        assert_eq!(records[0].tool_call_id, Some(Some("toolu_01".to_owned())));

        let mut state = SessionState::new();
        assert!(derive(root(), &bash, &mut state).is_empty());
        for index in 0..RETRY_WINDOW {
            let other = claude_bash("PreToolUse", &format!("echo {index}"));
            assert!(derive(root(), &other, &mut state).is_empty());
        }
        assert!(
            derive(root(), &bash, &mut state).is_empty(),
            "a repeat beyond the window is not a retry"
        );
    }

    #[test]
    fn a_different_input_to_the_same_tool_is_not_a_retry() {
        let mut state = SessionState::new();
        assert!(derive(root(), &claude_bash("PreToolUse", "cargo test"), &mut state).is_empty());
        assert!(derive(root(), &claude_bash("PreToolUse", "cargo fmt"), &mut state).is_empty());
    }

    #[test]
    fn a_payload_without_a_session_earns_nothing() {
        let json = r#"{"hook_event_name":"SessionEnd","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert!(derive(root(), &payload, &mut SessionState::new()).is_empty());
    }

    #[test]
    fn a_post_tool_event_counts_the_call_for_the_receipt() {
        let mut state = SessionState::new();
        derive(
            root(),
            &claude_post("Read", json!({"file_path": "src/lock.rs"})),
            &mut state,
        );
        derive(
            root(),
            &fixture(Harness::Claude, "PostToolUseFailure"),
            &mut state,
        );
        derive(root(), &claude_bash("PreToolUse", "cargo test"), &mut state);
        assert_eq!(state.tool_calls.get("Read"), Some(&1));
        assert_eq!(state.tool_calls.get("Edit"), Some(&1));
        assert_eq!(
            state.tool_calls.len(),
            2,
            "a before-tool event is not a call"
        );
    }

    // -----------------------------------------------------------------------------------
    // what never reaches a record
    // -----------------------------------------------------------------------------------

    /// The strings the three leaky fixtures carry that no record may repeat.
    const LEAKS: [&str; 5] = [
        "Rewrite the onboarding email",
        "onboarding-v2",
        "running 24 tests",
        "/Users/",
        "someone",
    ];

    #[test]
    fn the_leaky_fixtures_leak_nothing() {
        let leaky = [
            (Harness::Claude, "leaky-PostToolUse"),
            (Harness::Gemini, "leaky-AfterTool"),
            (Harness::Codex, "leaky-PostToolUse"),
        ];
        let mut seen = 0;
        for (harness, name) in leaky {
            let payload = fixture(harness, name);
            let mut state = SessionState::new();
            // Twice, so the write that is silent the first time earns its churn record and
            // there is something to search.
            let mut records = derive(root(), &payload, &mut state);
            records.extend(derive(root(), &payload, &mut state));
            assert!(!records.is_empty(), "{harness}/{name} produced no record");

            for record in &records {
                seen += 1;
                let line =
                    serde_json::to_string(&record.clone().stamped("2026-09-09T00:00:00.000Z"))
                        .expect("a serializable record");
                for leak in LEAKS {
                    assert!(
                        !line.contains(leak),
                        "{harness}/{name} leaked {leak}: {line}"
                    );
                }
                if let Some(path) = &record.path {
                    assert!(!path.starts_with('/'), "{harness}/{name}: {path}");
                }
            }
        }
        assert_eq!(seen, 4, "one churn each and two failures from codex");
    }

    #[test]
    fn a_path_outside_the_repository_is_a_kind_and_never_a_value() {
        let mut state = SessionState::new();
        let read = claude_post("Read", json!({"file_path": "/Users/someone/secrets.md"}));
        assert!(derive(root(), &read, &mut state).is_empty());
        let records = derive(root(), &read, &mut state);
        assert_eq!(kinds(&records), ["file.reread"]);
        assert_eq!(records[0].path, None);
        assert_eq!(records[0].path_kind, Some(PathKind::Outside));

        let line = serde_json::to_string(&records[0]).expect("a serializable record");
        assert!(!line.contains("/Users/"), "{line}");
        assert!(!line.contains("secrets"), "{line}");
    }

    #[test]
    fn a_state_file_never_holds_a_path_from_outside_the_repository() {
        let mut state = SessionState::new();
        let read = claude_post("Read", json!({"file_path": "/Users/someone/secrets.md"}));
        derive(root(), &read, &mut state);
        let text = serde_json::to_string(&state).expect("a serializable state");
        assert!(!text.contains("secrets"), "{text}");
        assert!(!text.contains("/Users/"), "{text}");
    }

    // -----------------------------------------------------------------------------------
    // every record against the profile's tables
    // -----------------------------------------------------------------------------------

    /// Every record this emitter can produce, from the fixtures and from the sessions that
    /// need a second event to say anything.
    fn every_record() -> Vec<Record> {
        let mut records = Vec::new();
        for harness in Harness::ALL {
            let directory = payloads().join(harness.name());
            let entries =
                fs::read_dir(&directory).unwrap_or_else(|e| panic!("{}: {e}", directory.display()));
            for entry in entries {
                let path = entry.expect("a readable directory entry").path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let json = fs::read_to_string(&path).expect("a readable fixture");
                let payload = parse_payload(harness, &json).expect("a parsed fixture");
                let mut state = SessionState::new();
                records.extend(derive(root(), &payload, &mut state));
                records.extend(derive(root(), &payload, &mut state));
            }
        }

        let mut state = SessionState::new();
        for payload in [
            claude_post("Grep", json!({"pattern": "derive"})),
            claude_post("Read", json!({"file_path": "src/lock.rs"})),
            claude_post("Read", json!({"file_path": "src/lock.rs"})),
            claude_bash("PostToolUse", "cargo test"),
            claude_bash("PostToolUse", "cargo test"),
            claude_bash("PreToolUse", "cargo test"),
            claude_bash("PreToolUse", "cargo test"),
            claude_post("Write", json!({"file_path": "src/lock.rs"})),
            claude_post("Edit", json!({"file_path": "src/lock.rs"})),
        ] {
            records.extend(derive(root(), &payload, &mut state));
        }
        records
            .into_iter()
            .map(|record| record.stamped("2026-09-09T12:00:00.402Z"))
            .collect()
    }

    #[test]
    fn every_record_carries_the_envelope_and_its_kinds_own_attributes() {
        let records = every_record();
        assert!(records.len() > 20, "{}", records.len());

        for record in &records {
            let value: Value =
                serde_json::to_value(record).expect("a record serializes to an object");
            let object = value.as_object().expect("a record is a JSON object");
            let kind = record.kind.as_str();

            for key in ENVELOPE {
                assert!(object.contains_key(key), "{kind} has no {key}");
                assert!(!object[key].is_null(), "{kind} nulled {key}");
            }
            assert!(PINNED_KINDS.contains(&kind), "{kind} is not a pinned kind");
            assert_eq!(object["event.name"], EVENT_NAME);
            assert!(object["time"].as_str().is_some_and(|t| t.ends_with('Z')));
            assert!(object["plotplot.count"].as_u64().is_some_and(|c| c >= 1));
            assert!(object["plotplot.harness"].as_str().is_some());
            assert!(object["gen_ai.conversation.id"].as_str().is_some());

            let (_, required) = PER_KIND
                .iter()
                .find(|(name, _)| *name == kind)
                .unwrap_or_else(|| panic!("{kind} is not in the per-kind table"));
            for key in *required {
                assert!(object.contains_key(*key), "{kind} has no {key}");
            }

            let (_, path_required) = PATH_REQUIRED
                .iter()
                .find(|(name, _)| *name == kind)
                .unwrap_or_else(|| panic!("{kind} is not in the kinds table"));
            if *path_required {
                if !PATH_GAPS.contains(&kind) {
                    assert!(
                        object.contains_key("plotplot.path.kind"),
                        "{kind} has no plotplot.path.kind"
                    );
                }
            } else {
                assert!(
                    !object.contains_key("plotplot.path"),
                    "{kind} names a path the profile does not give it"
                );
            }
            if let Some(path) = object.get("plotplot.path").and_then(Value::as_str) {
                assert!(!path.starts_with('/'), "{kind}: {path}");
                assert!(!path.contains(".."), "{kind}: {path}");
                assert_eq!(
                    object["plotplot.path.kind"],
                    json!(path_kind(path).as_str()),
                    "{kind}: {path}"
                );
            }
            if object.get("plotplot.path.kind") == Some(&json!("outside")) {
                assert!(!object.contains_key("plotplot.path"), "{kind}");
            }
        }
    }

    #[test]
    fn every_kind_this_emitter_produces_is_covered_by_that_check() {
        let mut produced: Vec<&str> = every_record()
            .iter()
            .map(|record| record.kind.as_str())
            .collect();
        produced.sort_unstable();
        produced.dedup();
        assert_eq!(
            produced,
            [
                "context.compacted",
                "edit.churn",
                "file.reread",
                "search.fanout",
                "session.ended",
                "test.loop",
                "tool.denied",
                "tool.failed",
                "tool.retry",
            ]
        );
    }

    #[test]
    fn the_pinned_tables_line_up_with_each_other() {
        for kind in PINNED_KINDS {
            assert!(PER_KIND.iter().any(|(name, _)| *name == kind), "{kind}");
            assert!(
                PATH_REQUIRED.iter().any(|(name, _)| *name == kind),
                "{kind}"
            );
        }
        assert_eq!(PINNED_KINDS.len(), PER_KIND.len());
        assert_eq!(PINNED_KINDS.len(), PATH_REQUIRED.len());
    }

    // -----------------------------------------------------------------------------------
    // the pieces
    // -----------------------------------------------------------------------------------

    #[test]
    fn a_payload_the_profile_cannot_key_says_why_rather_than_counting_zero_in_silence() {
        for harness in Harness::ALL {
            let json = json!({
                "hook_event_name": harness.before_tool_event(),
                "cwd": "/work/repo",
                "tool_name": "Write",
                "tool_input": {"file_path": "src/lock.rs"},
            })
            .to_string();
            let payload = parse_payload(harness, &json).expect("a parsed payload");

            let reason = unrecorded_reason(&payload).unwrap_or_else(|| {
                panic!("{harness}: a payload with no session_id names no reason")
            });
            assert!(reason.contains("session_id"), "{reason}");
            assert!(reason.contains("gen_ai.conversation.id"), "{reason}");
            assert!(reason.contains(harness.name()), "{reason}");
            assert!(reason.contains(harness.before_tool_event()), "{reason}");

            // And the count alone cannot say it: derive earns nothing either way.
            assert!(derive(root(), &payload, &mut SessionState::new()).is_empty());
            assert!(derive_denied(root(), &payload, "deny.no-verify").is_empty());
        }
    }

    #[test]
    fn a_payload_the_profile_can_key_leaves_no_reason_to_report() {
        for (harness, name) in [
            (Harness::Claude, "PreToolUse"),
            (Harness::Claude, "SessionEnd"),
            (Harness::Gemini, "BeforeTool"),
            (Harness::Codex, "Stop"),
        ] {
            assert_eq!(
                unrecorded_reason(&fixture(harness, name)),
                None,
                "{harness}/{name}"
            );
        }
    }

    #[test]
    fn emitting_a_payload_the_profile_cannot_key_writes_nothing_and_fails_at_nothing() {
        let temp = tempfile::tempdir().expect("a temporary root");
        let root = temp.path();
        let json = json!({
            "hook_event_name": "SessionEnd",
            "cwd": "/work/repo",
        })
        .to_string();
        let payload = parse_payload(Harness::Claude, &json).expect("a parsed payload");

        let clock = || "2026-09-09T12:00:00.402Z".to_owned();
        let written = emit(root, &payload, &clock).expect("emitting is not an error");
        assert_eq!(written, 0);
        assert!(!layout::friction_dir(root).exists(), "nothing was written");
        assert!(unrecorded_reason(&payload).is_some(), "and it says why");
    }

    #[test]
    fn a_record_leaves_derive_unstamped_and_emit_stamps_it() {
        let mut state = SessionState::new();
        let records = derive(root(), &fixture(Harness::Claude, "SessionEnd"), &mut state);
        assert_eq!(records[0].time, "");
        let stamped = records[0].clone().stamped("2026-09-09T12:00:00.402Z");
        assert_eq!(stamped.time, "2026-09-09T12:00:00.402Z");
    }

    #[test]
    fn the_system_clock_reads_utc_and_ends_in_z() {
        for (millis, expected) in [
            (0, "1970-01-01T00:00:00.000Z"),
            (-1, "1969-12-31T23:59:59.999Z"),
            (946_684_799_999, "1999-12-31T23:59:59.999Z"),
            (951_782_400_000, "2000-02-29T00:00:00.000Z"),
            (1_788_955_200_402, "2026-09-09T12:00:00.402Z"),
        ] {
            assert_eq!(rfc3339_from_unix_millis(millis), expected);
        }
        let now = SystemClock.rfc3339_utc();
        assert!(now.ends_with('Z'), "{now}");
        assert_eq!(now.len(), 24, "{now}");
        assert!(month_of(&now).is_ok(), "{now}");
    }

    #[test]
    fn a_closure_is_a_clock_too() {
        let clock = || "2026-09-09T12:00:00.402Z".to_owned();
        assert_eq!(clock.rfc3339_utc(), "2026-09-09T12:00:00.402Z");
    }

    #[test]
    fn the_month_comes_from_the_timestamp_and_a_broken_clock_is_an_error() {
        assert_eq!(
            month_of("2026-09-09T12:00:00.402Z").expect("a well-shaped timestamp"),
            "2026-09"
        );
        for broken in [
            "",
            "2026",
            "not a time",
            "2026/09/09T00:00:00Z",
            "2026-09-09T12:00:00",
        ] {
            assert!(month_of(broken).is_err(), "{broken}");
        }
    }

    #[test]
    fn a_session_id_that_is_not_a_file_name_is_refused() {
        assert!(session_file_name("6f3c1b2a-8d47-4f0e-9b31-2a5c7e91d044").is_ok());
        for refused in [
            "",
            "../../etc/passwd",
            "a/b",
            ".hidden",
            "a b",
            &"x".repeat(129),
        ] {
            assert!(session_file_name(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn a_path_is_read_against_the_payloads_own_working_directory() {
        let base = Path::new("/work/repo");
        assert_eq!(
            repo_path(base, "/work/repo/src/lock.rs"),
            RepoPath::Inside("src/lock.rs".to_owned())
        );
        assert_eq!(
            repo_path(base, "./src/lock.rs"),
            RepoPath::Inside("src/lock.rs".to_owned())
        );
        assert_eq!(repo_path(base, "/etc/passwd"), RepoPath::Outside);
        assert_eq!(repo_path(base, "../outside.rs"), RepoPath::Outside);
        assert_eq!(repo_path(base, "/work/repo"), RepoPath::Outside);
    }

    #[test]
    fn a_path_kind_is_one_of_the_five() {
        for (path, kind) in [
            ("src/lock.rs", PathKind::Prod),
            ("notes/onboarding.md", PathKind::Prod),
            ("tests/faces.rs", PathKind::Test),
            ("src/spec/thing.rs", PathKind::Test),
            ("src/__tests__/thing.js", PathKind::Test),
            ("src/lock.test.ts", PathKind::Test),
            ("src/lock.spec.ts", PathKind::Test),
            ("docs/plans/stem.md", PathKind::Doc),
            ("docs/tend2/stem.tend2.html", PathKind::Doc),
            ("README.md", PathKind::Doc),
            ("notes.txt", PathKind::Doc),
            ("Cargo.toml", PathKind::Config),
            ("package.json", PathKind::Config),
            ("action.yml", PathKind::Config),
            (".gitignore", PathKind::Config),
            ("src/deep/config.toml", PathKind::Prod),
            ("garden.lock", PathKind::Prod),
        ] {
            assert_eq!(path_kind(path), kind, "{path}");
        }
    }

    // -----------------------------------------------------------------------------------
    // the stem's own refusal
    // -----------------------------------------------------------------------------------

    #[test]
    fn the_stems_own_refusal_earns_one_tool_denied_record_carrying_the_rule() {
        let payload = claude_bash("PreToolUse", "git commit --no-verify -m x");
        let records = derive_denied(root(), &payload, "deny.no-verify");

        assert_eq!(kinds(&records), ["tool.denied"]);
        assert_eq!(records[0].rule.as_deref(), Some("deny.no-verify"));
        assert_eq!(records[0].operation_name.as_deref(), Some("execute_tool"));
        assert_eq!(records[0].tool_name.as_deref(), Some("Bash"));
        assert_eq!(records[0].conversation_id, "6f3c1b2a");
        assert_eq!(records[0].count, 1);
        assert_eq!(records[0].event_name, EVENT_NAME);
    }

    #[test]
    fn a_refused_write_carries_the_path_it_was_refused_for() {
        let payload = tool_payload(
            Harness::Claude,
            "PreToolUse",
            "Write",
            json!({ "file_path": "/work/repo/.githooks/pre-commit" }),
        );
        let records = derive_denied(root(), &payload, "deny.stem-owned-path");

        assert_eq!(records[0].path.as_deref(), Some(".githooks/pre-commit"));
        assert!(records[0].path_kind.is_some());
    }

    #[test]
    fn a_refusal_the_profile_cannot_name_earns_no_record() {
        // No session id: the record would have no `gen_ai.conversation.id`, which the
        // profile requires non-null on every line.
        let json = r#"{"hook_event_name":"PreToolUse","cwd":"/work/repo","tool_name":"Bash"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert!(derive_denied(root(), &payload, "deny.no-verify").is_empty());

        // No tool: `gen_ai.tool.name` is required for this kind and cannot be invented.
        let json = r#"{"hook_event_name":"PreToolUse","session_id":"s","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert!(derive_denied(root(), &payload, "deny.no-verify").is_empty());
    }

    #[test]
    fn a_denied_record_carries_the_same_envelope_a_derived_one_does() {
        let payload = fixture(Harness::Claude, "PermissionDenied");
        let mut state = SessionState::new();
        let derived = derive(root(), &payload, &mut state);
        let denied = derive_denied(root(), &payload, "deny.no-verify");

        assert_eq!(kinds(&derived), kinds(&denied));
        assert_eq!(derived[0].harness, denied[0].harness);
        assert_eq!(derived[0].conversation_id, denied[0].conversation_id);
        assert_eq!(derived[0].model, denied[0].model);
        // The one difference: the vendor's own denial names no rule of ours, ours does.
        assert_eq!(derived[0].rule, None);
        assert_eq!(denied[0].rule.as_deref(), Some("deny.no-verify"));
    }

    #[test]
    fn a_codex_shell_payload_reads_as_the_script_it_runs() {
        let payload = fixture(Harness::Codex, "PreToolUse");
        assert_eq!(
            shell_command(&payload).as_deref(),
            Some("cargo fmt --check")
        );

        let payload = tool_payload(
            Harness::Codex,
            "PreToolUse",
            "shell",
            json!({"command": ["ls", "-la"]}),
        );
        assert_eq!(shell_command(&payload).as_deref(), Some("ls -la"));

        let payload = claude_post("Read", json!({"file_path": "a"}));
        assert_eq!(shell_command(&payload), None);
    }

    #[test]
    fn a_command_reads_as_tokens_with_its_operators_split_off() {
        assert_eq!(
            shell_tokens("echo x >.plotplot/bin/weeder"),
            ["echo", "x", ">", ".plotplot/bin/weeder"]
        );
        assert_eq!(
            shell_tokens("a && b || c ; d | e"),
            ["a", "&&", "b", "||", "c", ";", "d", "|", "e"]
        );
        assert_eq!(shell_tokens("  "), Vec::<String>::new());
        assert_eq!(shell_tokens("cat >> f"), ["cat", ">>", "f"]);
    }

    #[test]
    fn an_mcp_tool_reads_as_the_tool_behind_the_prefix() {
        assert_eq!(bare_tool_name("mcp__tilth__tilth_read"), "tilth_read");
        assert_eq!(bare_tool_name("tilth_read"), "tilth_read");
        assert_eq!(bare_tool_name("Read"), "Read");
        assert_eq!(bare_tool_name("mcp__"), "mcp__");
    }

    #[test]
    fn a_command_that_writes_is_never_read_as_a_read() {
        let mut state = SessionState::new();
        let payload = claude_bash("PostToolUse", "cat > src/lock.rs <<'EOF'\nhello\nEOF");
        assert!(derive(root(), &payload, &mut state).is_empty());
        assert!(derive(root(), &payload, &mut state).is_empty());
        assert!(state.reads.is_empty(), "{:?}", state.reads);
    }

    #[test]
    fn the_state_survives_a_round_trip_through_its_file_shape() {
        let mut state = SessionState::new();
        derive(
            root(),
            &claude_post("Read", json!({"file_path": "src/lock.rs"})),
            &mut state,
        );
        let text = serde_json::to_string(&state).expect("a serializable state");
        let back: SessionState = serde_json::from_str(&text).expect("a readable state");
        assert_eq!(back, state);
        assert_eq!(
            serde_json::from_str::<SessionState>("{}").expect("an empty state"),
            SessionState::new()
        );
    }

    #[test]
    fn a_corrupt_state_file_is_an_error_and_not_a_fresh_session() {
        let temp = tempfile::tempdir().expect("a temporary directory");
        let path = temp.path().join("state.json");
        fs::write(&path, "{ not json").expect("a written file");
        assert!(matches!(read_state(&path), Err(Error::Json { .. })));
        assert_eq!(
            read_state(&temp.path().join("absent.json")).expect("a fresh session"),
            SessionState::new()
        );
    }
}
