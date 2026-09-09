//! The receipt: what a session can say about itself the moment it ends, and what a commit
//! carries about what produced and verified it.
//!
//! `docs/plans/receipts.md` §4 puts the draft at the session-end event, inside Claude Code's
//! 1.5-second budget, so this face writes counters and nothing else: no network, no git, no
//! model. The fields are §3's, and anything the payload cannot supply is `null` rather than
//! an estimate. Signing, the subject digest and the verification block belong to
//! `receipt seal` and `receipt sign`, which are not this face.
//!
//! Ordering the dispatcher owes this face: on the session-end event the draft must be written
//! before [`crate::friction::emit`], because emit prunes the session's friction state and the
//! tool counts and the summary digest come from it. When the state is already gone the draft
//! is still written, with no counts and no digest, which is the documented empty case.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::cli::{ReceiptArgs, ReceiptFace, SealArgs, ShowArgs, VerifyArgs};
use crate::error::{Error, Result};
use crate::friction::{
    Now, SessionState, SystemClock, harness_label, harness_version, session_file_name, usage_tokens,
};
use crate::harness::{Payload, parse_payload};
use crate::plant::gitconfig;
use crate::{layout, sarif};

/// The principal a session driven from a terminal has, in tend2's vocabulary.
pub const PRINCIPAL_CLI: &str = "cli";

/// `.plotplot/receipts/drafts/<session_id>.json`, the unsigned v0 draft.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    pub harness: DraftHarness,
    /// The models the payload named; empty when it named none.
    pub models: Vec<String>,
    pub principal: String,
    pub sessions: Vec<String>,
    pub cost: Cost,
    pub friction: Friction,
}

/// Which harness produced the session, and at which version when it says.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftHarness {
    pub name: String,
    pub version: Option<String>,
}

/// What the session cost, in the units a hook can know. Never estimated.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    /// Wall time is not derivable from a session-end payload alone, so it is always null here.
    pub wall_seconds: Option<u64>,
    pub tool_calls: BTreeMap<String, u64>,
}

/// The digest of the session's friction state, when the ledger kept one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Friction {
    pub summary: Option<Sha256Digest>,
}

/// A sha256, hex encoded, in the `{"sha256": …}` shape the predicate uses throughout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sha256Digest {
    pub sha256: String,
}

/// The draft a payload and a session's friction state make, as a value.
///
/// Pure: the caller supplies the state and the digest of the bytes it was read from, and
/// nothing here touches the disk.
pub fn build(
    payload: &Payload,
    state: Option<&SessionState>,
    summary: Option<String>,
) -> Result<Draft> {
    let session = payload.session_id.clone().ok_or_else(|| Error::Harness {
        problem: format!(
            "the {} {} payload has no session_id, and a receipt draft is named by its session",
            payload.harness, payload.event
        ),
    })?;
    let (input_tokens, output_tokens) = usage_tokens(payload);

    Ok(Draft {
        harness: DraftHarness {
            name: harness_label(payload.harness).to_owned(),
            version: harness_version(payload),
        },
        models: payload.model.clone().into_iter().collect(),
        principal: PRINCIPAL_CLI.to_owned(),
        sessions: vec![session],
        cost: Cost {
            input_tokens,
            output_tokens,
            wall_seconds: None,
            tool_calls: state
                .map(|state| state.tool_calls.clone())
                .unwrap_or_default(),
        },
        friction: Friction {
            summary: summary.map(|sha256| Sha256Digest { sha256 }),
        },
    })
}

/// Write this session's draft, replacing any draft the session already has.
///
/// Returns the path written.
pub fn draft(root: &Path, payload: &Payload) -> Result<PathBuf> {
    let session = payload
        .session_id
        .as_deref()
        .ok_or_else(|| Error::Harness {
            problem: format!(
                "the {} {} payload has no session_id, and a receipt draft is named by its session",
                payload.harness, payload.event
            ),
        })?;
    let session = session_file_name(session)?;

    let state_path = layout::friction_state(root, session);
    let (state, summary) = match fs::read(&state_path) {
        Ok(bytes) => {
            let state: SessionState =
                serde_json::from_slice(&bytes).map_err(|source| Error::Json {
                    path: Some(state_path.clone()),
                    source,
                })?;
            (Some(state), Some(hex::encode(Sha256::digest(&bytes))))
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => (None, None),
        Err(source) => {
            return Err(Error::Io {
                path: state_path,
                source,
            });
        }
    };

    let draft = build(payload, state.as_ref(), summary)?;
    let text = serde_json::to_string_pretty(&draft)
        .map_err(|source| Error::Json { path: None, source })?;

    let directory = layout::receipt_drafts_dir(root);
    fs::create_dir_all(&directory).map_err(|source| Error::Io {
        path: directory,
        source,
    })?;
    let path = layout::receipt_draft(root, session);
    fs::write(&path, format!("{text}\n")).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

/// `plotplot receipt <face>`: from arguments and stdin to an exit code.
///
/// `draft` is the only face that reads stdin, and the only one that writes nothing on
/// stdout: it is registered as a hook entry in every bundle, and a vendor reads a hook's
/// stdout as an answer, so the only thing it says is its code.
pub fn run(
    root: &Path,
    args: &ReceiptArgs,
    stdin: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    match &args.face {
        ReceiptFace::Draft(arg) => {
            let written =
                parse_payload(arg.harness, stdin).and_then(|payload| draft(root, &payload));
            match written {
                Ok(_) => 0,
                Err(error) => {
                    let _ = writeln!(stderr, "{error}");
                    1
                }
            }
        }
        ReceiptFace::Seal(arg) => run_seal(root, arg, stdout, stderr),
        ReceiptFace::Verify(arg) => run_verify(root, arg, stdout, stderr),
        ReceiptFace::Show(arg) => run_show(root, arg, stdout, stderr),
    }
}

// ---------------------------------------------------------------------------------------
// the sealed receipt (v0)
// ---------------------------------------------------------------------------------------

/// The in-toto Attestation Framework's Statement v1, the shape the law picked over a receipt
/// schema of the garden's own.
pub const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";

/// The predicate this statement carries. Its JSON Schema is the contracts' to own; this
/// binary writes `docs/plans/receipts.md` §3's shape, key for key and in §3's order.
pub const PREDICATE_TYPE: &str = "https://plotplot.ai/receipt/v1";

/// What seals a receipt, as the predicate's `producer.name`.
pub const PRODUCER: &str = "plotplot";

/// The note's last line, which carries the digest of every byte above it.
pub const DIGEST_PREFIX: &str = "sha256: ";

/// The one host whose urls have a canonical short form the garden uses.
const GITHUB_HOST: &str = "github.com";

/// Where the loops live, and so where a stamp can be added.
const LOOPS_DIR: &str = "docs/tend2";

/// The suffix of a loop file, which is what tells one from a page beside it.
const LOOP_SUFFIX: &str = ".tend2.html";

/// The judge whose SARIF a receipt carries, when the repository has it fetched.
const WEEDER: &str = "weeder";

/// Every commit in the range carries a receipt, and every receipt is the one git re-derives.
pub const VERIFIED: i32 = 0;
/// Some commit carries no receipt, or one that does not survive being recomputed.
pub const REFUSED: i32 = 3;
/// git could not say what a revision is, so nothing was judged.
pub const FAILED: i32 = 1;

/// An in-toto Statement v1 about one commit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: Predicate,
}

/// What the statement is about: the repository by its canonical url, the change by its own
/// hashes, so verification needs the repository and nothing else.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: Digest,
}

/// The DigestSet in-toto already defines for git objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub git_commit: String,
    pub git_tree: String,
}

/// `docs/plans/receipts.md` §3, in §3's order: what produced the change, what it touched,
/// what verified it, what it cost.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Predicate {
    pub producer: Producer,
    pub created_at: String,
    /// The harness the folded drafts name, or `null` when this commit folded no draft.
    pub harness: Option<DraftHarness>,
    pub models: Option<Vec<String>>,
    pub principal: Option<String>,
    pub sessions: Vec<String>,
    /// The conductor and its node. No face of this binary fills it in; a run under a
    /// conductor is what a later slice writes, and what is read back here unchanged.
    pub conductor: Option<Value>,
    pub changed: Vec<String>,
    pub verification: Verification,
    pub cost: Cost,
    pub friction: Friction,
}

/// The tool that sealed, and its version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
}

/// What judged the change, as far as this repository can prove at seal time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// weeder's own log, digested and counted, or `null` when no pinned weeder answered.
    pub weeder: Option<WeederResult>,
    pub tend2: Vec<Stamp>,
    /// pleach's gates. Nothing in this binary knows them; a run under pleach is what fills
    /// this in, and what is read back here unchanged.
    pub pleach: Option<Value>,
    /// The commands run with their exit codes. A v0 draft counts tool calls by name and
    /// records no command and no exit code, so this list is empty until something that knows
    /// both writes a draft that carries them. An entry is never derived from a count.
    pub commands: Vec<CommandRun>,
}

/// weeder's answer over this commit: the log's digest and what it found.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeederResult {
    pub sarif: Sha256Digest,
    pub block: usize,
    pub warn: usize,
}

/// One tend2 stamp this commit's diff added: which check, and the sha the verifier stamped.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    pub check: String,
    pub sha: String,
}

/// One command a session ran, and what it exited with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandRun {
    pub command: String,
    pub exit: Option<i32>,
}

/// What the drafts of one commit add up to.
#[derive(Clone, Debug, PartialEq)]
pub struct Folded {
    pub harness: Option<DraftHarness>,
    pub models: Option<Vec<String>>,
    pub principal: Option<String>,
    pub sessions: Vec<String>,
    pub cost: Cost,
    pub friction: Friction,
}

/// What a seal did to the note that was already there, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sealed {
    /// The commit carried no receipt, and now carries one.
    Written,
    /// The statement is the one the note already holds, so nothing was written.
    Unchanged,
    /// The statement differs from the one the note held, and the note now holds this one.
    Replaced,
}

impl Sealed {
    /// The word `seal` prints after the commit.
    pub fn word(self) -> &'static str {
        match self {
            Sealed::Written => "sealed",
            Sealed::Unchanged => "unchanged",
            Sealed::Replaced => "replaced",
        }
    }
}

/// What one seal did, for the face to print.
#[derive(Clone, Debug, PartialEq)]
pub struct Seal {
    pub commit: String,
    pub short: String,
    pub outcome: Sealed,
    /// What was in the way of a fuller receipt, when something was: a pinned weeder that did
    /// not answer with SARIF. Said on stderr rather than guessed into the predicate.
    pub aside: Option<String>,
}

/// What `verify` found for one commit.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// The note is there, and every digest in it is the one git and sha256 recompute.
    Ok,
    /// The commit carries no note on the receipts ref.
    Missing,
    /// The note is there and does not survive being recomputed.
    Invalid(String),
    /// The note is a v0 receipt, and the caller asked for a signature.
    Unsigned,
}

impl Verdict {
    /// The rest of the line `verify` prints after the short sha.
    pub fn line(&self) -> String {
        match self {
            Verdict::Ok => "receipt ok".to_owned(),
            Verdict::Missing => "no receipt".to_owned(),
            Verdict::Invalid(problem) => problem.clone(),
            Verdict::Unsigned => "unsigned (v0)".to_owned(),
        }
    }

    /// Whether this verdict lets the run exit 0.
    pub fn passes(&self) -> bool {
        matches!(self, Verdict::Ok)
    }
}

// ------------------------------------------------------------------ the pure parts

/// The repository's canonical name for a subject, from the url `origin` is at.
///
/// A GitHub url in any of the forms git writes it — https, ssh, the scp-like short form,
/// with or without a user, a port or the `.git` suffix — becomes `github.com/<owner>/<repo>`,
/// so two clones of the same repository name the same subject. Every other url is the
/// subject exactly as the remote gives it: shortening a host whose layout the stem does not
/// know would invent an identity.
pub fn canonical_url(remote: &str) -> String {
    let url = remote.trim();
    let scheme_less = ["https://", "http://", "ssh://", "git://", "git+ssh://"]
        .into_iter()
        .find_map(|scheme| url.strip_prefix(scheme));

    // With a scheme the authority ends at the first slash; without one this is the scp-like
    // form `git@host:owner/repo`, whose authority ends at the first colon.
    let split = match scheme_less {
        Some(rest) => rest.split_once('/'),
        None => url.split_once(':'),
    };
    let Some((authority, path)) = split else {
        return url.to_owned();
    };

    let host = authority.rsplit_once('@').map_or(authority, |(_, it)| it);
    let host = host.split_once(':').map_or(host, |(it, _)| it);
    if !host.eq_ignore_ascii_case(GITHUB_HOST) {
        return url.to_owned();
    }

    let path = path.trim_start_matches('/').trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut segments = path.split('/');
    match (segments.next(), segments.next(), segments.next()) {
        (Some(owner), Some(repo), None) if !owner.is_empty() && !repo.is_empty() => {
            format!("{GITHUB_HOST}/{owner}/{repo}")
        }
        // Anything else under github.com is not one repository, so it keeps its own url.
        _ => url.to_owned(),
    }
}

/// Everything the drafts of one commit say, folded in the order they are given.
///
/// The rules are the plan's: what no draft supplies is `null` and never an estimate. A token
/// count is the sum only when every folded draft reports one, because a sum over the drafts
/// that happened to report is not the total it would be read as.
pub fn fold(drafts: &[Draft]) -> Folded {
    let first = drafts.first();

    let mut models: Vec<String> = Vec::new();
    let mut sessions: Vec<String> = Vec::new();
    let mut tool_calls: BTreeMap<String, u64> = BTreeMap::new();
    for draft in drafts {
        for model in &draft.models {
            if !models.contains(model) {
                models.push(model.clone());
            }
        }
        for session in &draft.sessions {
            if !sessions.contains(session) {
                sessions.push(session.clone());
            }
        }
        for (tool, count) in &draft.cost.tool_calls {
            let total = tool_calls.entry(tool.clone()).or_insert(0);
            *total = total.saturating_add(*count);
        }
    }

    Folded {
        harness: first.map(|draft| draft.harness.clone()),
        models: first.map(|_| models),
        principal: first.map(|draft| draft.principal.clone()),
        sessions,
        cost: Cost {
            input_tokens: summed(drafts, |cost| cost.input_tokens),
            output_tokens: summed(drafts, |cost| cost.output_tokens),
            // A session-end payload cannot say how long the session ran, so no draft carries
            // it and no sum of drafts invents it.
            wall_seconds: summed(drafts, |cost| cost.wall_seconds),
            tool_calls,
        },
        friction: Friction {
            summary: friction_summary(drafts),
        },
    }
}

/// One of the drafts' token counts, summed, and `None` unless every draft reported one.
fn summed(drafts: &[Draft], pick: fn(&Cost) -> Option<u64>) -> Option<u64> {
    if drafts.is_empty() {
        return None;
    }
    let mut total: u64 = 0;
    for draft in drafts {
        total = total.checked_add(pick(&draft.cost)?)?;
    }
    Some(total)
}

/// The digest over the drafts' friction summary digests, in the order they were folded.
///
/// One line per digest, so the same set in another order is another digest and a receipt
/// names the exact session summaries it stands on. `None` when no draft kept one.
fn friction_summary(drafts: &[Draft]) -> Option<Sha256Digest> {
    let digests: Vec<&str> = drafts
        .iter()
        .filter_map(|draft| draft.friction.summary.as_ref())
        .map(|digest| digest.sha256.as_str())
        .collect();
    if digests.is_empty() {
        return None;
    }
    let mut hasher = Sha256::new();
    for digest in digests {
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    Some(Sha256Digest {
        sha256: hex::encode(hasher.finalize()),
    })
}

/// The statement for one commit, from everything already gathered. Pure over its arguments.
#[allow(clippy::too_many_arguments)]
pub fn statement(
    name: &str,
    commit: &str,
    tree: &str,
    created_at: &str,
    folded: Folded,
    changed: Vec<String>,
    verification: Verification,
) -> Statement {
    Statement {
        statement_type: STATEMENT_TYPE.to_owned(),
        subject: vec![Subject {
            name: name.to_owned(),
            digest: Digest {
                git_commit: commit.to_owned(),
                git_tree: tree.to_owned(),
            },
        }],
        predicate_type: PREDICATE_TYPE.to_owned(),
        predicate: Predicate {
            producer: Producer {
                name: PRODUCER.to_owned(),
                version: crate::VERSION.to_owned(),
            },
            created_at: created_at.to_owned(),
            harness: folded.harness,
            models: folded.models,
            principal: folded.principal,
            sessions: folded.sessions,
            conductor: None,
            changed,
            verification,
            cost: folded.cost,
            friction: folded.friction,
        },
    }
}

/// The note's bytes: the statement, then one line carrying the sha256 of everything above.
///
/// # Errors
///
/// [`Error::Json`] when the statement cannot be serialized.
pub fn note_text(statement: &Statement) -> Result<String> {
    let json = serde_json::to_string_pretty(statement)
        .map_err(|source| Error::Json { path: None, source })?;
    let bytes = format!("{json}\n");
    let digest = hex::encode(Sha256::digest(bytes.as_bytes()));
    Ok(format!("{bytes}{DIGEST_PREFIX}{digest}\n"))
}

/// A note split into the bytes that were hashed and the digest the note claims for them.
///
/// The statement's bytes are everything up to and including the newline before the last
/// line, which is what `sed '$d'` leaves and what [`note_text`] hashed.
pub fn split_note(note: &str) -> Option<(&str, &str)> {
    let body = note.strip_suffix('\n').unwrap_or(note);
    let break_at = body.rfind('\n')?;
    let digest = body[break_at + 1..].strip_prefix(DIGEST_PREFIX)?;
    Some((&note[..=break_at], digest))
}

/// Whether two statements differ only in the moment they were sealed.
///
/// `createdAt` moves every time the clock is read, so it cannot be part of the question
/// "does this commit already carry this receipt".
pub fn same_but_for_the_moment(existing: &Statement, fresh: &Statement) -> bool {
    let mut fresh = fresh.clone();
    fresh.predicate.created_at = existing.predicate.created_at.clone();
    &fresh == existing
}

/// Every added line of a unified diff that stamps a check, with the file it was added to.
///
/// A stamp is a checklist line the diff adds that carries both `[x]` and an `@<sha>` token:
/// the box the verifier ticked and the evidence it ticked it on.
pub fn added_stamp_lines(diff: &str) -> Vec<(String, String)> {
    let mut file: Option<String> = None;
    let mut added = Vec::new();
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            file = is_loop(path).then(|| path.to_owned());
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        let Some(content) = line.strip_prefix('+') else {
            continue;
        };
        let Some(path) = &file else { continue };
        if is_stamped(content) && stamp_sha(content).is_some() {
            added.push((path.clone(), content.to_owned()));
        }
    }
    added
}

/// Whether a path is one of the loop files a stamp can land in.
fn is_loop(path: &str) -> bool {
    path.starts_with(&format!("{LOOPS_DIR}/")) && path.ends_with(LOOP_SUFFIX)
}

/// Whether a line is a checklist line with its box ticked.
fn is_stamped(line: &str) -> bool {
    line.trim_start().starts_with("- [x]")
}

/// Whether a line is a checklist line at all, ticked or not.
fn is_check(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("- [x]") || line.starts_with("- [ ]")
}

/// The sha a stamp carries: the first `@<hex>` token on the line.
pub fn stamp_sha(line: &str) -> Option<&str> {
    line.split_whitespace().find_map(|token| {
        let sha = token.strip_prefix('@')?;
        let hex = (4..=40).contains(&sha.len()) && sha.chars().all(|c| c.is_ascii_hexdigit());
        hex.then_some(sha)
    })
}

/// The loop a file belongs to: the file name without its suffix.
pub fn loop_name(path: &str) -> Option<&str> {
    path.rsplit('/').next()?.strip_suffix(LOOP_SUFFIX)
}

/// Which check a stamped line is, counting checklist lines in the file it now sits in.
///
/// tend2 addresses a check by its position in the loop, which is what `tend2 verify
/// --check <n>` takes, so `<loop>:c<n>` is the same address the verifier was called with.
pub fn check_id(path: &str, line: &str, file: &str) -> Option<String> {
    let name = loop_name(path)?;
    let mut seen = 0;
    for candidate in file.lines() {
        if !is_check(candidate) {
            continue;
        }
        seen += 1;
        if candidate == line {
            return Some(format!("{name}:c{seen}"));
        }
    }
    None
}

// ------------------------------------------------------------------ the edge: git

/// Run git in `root` and hand back whatever it said, however it exited.
///
/// The stem talks to git through the binary rather than a library, so what a hook sees and
/// what this reads are the same git, with the same configuration and the same includes.
fn git_output(root: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|source| Error::Git {
            command: spelled(root, args),
            stderr: crate::manifest::one_line(&source.to_string()),
        })
}

/// Run git in `root` and hand back its stdout, refusing anything but success.
///
/// # Errors
///
/// [`Error::Git`] when git could not be run or exited non-zero, carrying what it said.
fn git_text(root: &Path, args: &[&str]) -> Result<String> {
    let output = git_output(root, args)?;
    if !output.status.success() {
        return Err(Error::Git {
            command: spelled(root, args),
            stderr: crate::manifest::one_line(&String::from_utf8_lossy(&output.stderr)),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The command as an error should name it.
fn spelled(root: &Path, args: &[&str]) -> String {
    format!("git -C {} {}", root.display(), args.join(" "))
}

/// The full hash of a revision's commit, refusing anything that is not one.
pub fn commit_of(root: &Path, rev: &str) -> Result<String> {
    Ok(
        git_text(root, &["rev-parse", &format!("{rev}^{{commit}}")])?
            .trim()
            .to_owned(),
    )
}

/// The hash of the tree a commit points at.
pub fn tree_of(root: &Path, rev: &str) -> Result<String> {
    Ok(git_text(root, &["rev-parse", &format!("{rev}^{{tree}}")])?
        .trim()
        .to_owned())
}

/// The abbreviation git itself would print for a commit, so a line names it the way every
/// other tool in the repository does.
fn short_of(root: &Path, commit: &str) -> Result<String> {
    Ok(git_text(root, &["rev-parse", "--short", commit])?
        .trim()
        .to_owned())
}

/// The commits `verify` was asked about, oldest first.
///
/// # Errors
///
/// [`Error::Git`] when git cannot resolve the revision or the range.
pub fn commits(root: &Path, rev: Option<&str>, range: Option<&str>) -> Result<Vec<String>> {
    if let Some(range) = range {
        return Ok(git_text(root, &["rev-list", "--reverse", range])?
            .lines()
            .map(str::to_owned)
            .collect());
    }
    let rev = rev.unwrap_or("HEAD");
    Ok(vec![commit_of(root, rev)?])
}

/// The note one commit carries on the receipts ref, or `None` when it carries none.
///
/// Read as the blob git holds rather than through `git notes show`, so what is hashed here
/// is what is stored, byte for byte.
///
/// # Errors
///
/// [`Error::Git`] when git could not be run or failed for a reason other than the absence of
/// a note; [`Error::Receipt`] when the note's bytes are not UTF-8.
pub fn read_note(root: &Path, commit: &str) -> Result<Option<String>> {
    let listed = git_output(
        root,
        &["notes", "--ref", gitconfig::RECEIPTS_REF, "list", commit],
    )?;
    // git answers 1 for an object with no note, which is an answer and not a failure.
    if listed.status.code() == Some(1) {
        return Ok(None);
    }
    if !listed.status.success() {
        return Err(Error::Git {
            command: spelled(
                root,
                &["notes", "--ref", gitconfig::RECEIPTS_REF, "list", commit],
            ),
            stderr: crate::manifest::one_line(&String::from_utf8_lossy(&listed.stderr)),
        });
    }
    let blob = String::from_utf8_lossy(&listed.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned();
    if blob.is_empty() {
        return Ok(None);
    }

    let bytes = git_output(root, &["cat-file", "blob", &blob])?;
    if !bytes.status.success() {
        return Err(Error::Git {
            command: spelled(root, &["cat-file", "blob", &blob]),
            stderr: crate::manifest::one_line(&String::from_utf8_lossy(&bytes.stderr)),
        });
    }
    String::from_utf8(bytes.stdout)
        .map(Some)
        .map_err(|_| Error::Receipt {
            commit: commit.to_owned(),
            problem: "its note is not UTF-8, so it is not a receipt this stem wrote".to_owned(),
        })
}

/// Attach `text` to `commit` on the receipts ref, replacing any note already there.
///
/// The bytes go in on stdin (`-F -`), so sealing never writes a scratch file into somebody's
/// repository on its way to writing a note.
///
/// # Errors
///
/// [`Error::Git`] when git could not be run or refused the note.
pub fn write_note(root: &Path, commit: &str, text: &str) -> Result<()> {
    let args = [
        "notes",
        "--ref",
        gitconfig::RECEIPTS_REF,
        "add",
        "-f",
        "-F",
        "-",
        commit,
    ];
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::Git {
            command: spelled(root, &args),
            stderr: crate::manifest::one_line(&source.to_string()),
        })?;

    match child.stdin.take() {
        Some(mut stdin) => stdin
            .write_all(text.as_bytes())
            .map_err(|source| Error::Git {
                command: spelled(root, &args),
                stderr: crate::manifest::one_line(&source.to_string()),
            })?,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Git {
                command: spelled(root, &args),
                stderr: "git took no stdin, so the note could not be handed to it".to_owned(),
            });
        }
    }

    let output = child.wait_with_output().map_err(|source| Error::Git {
        command: spelled(root, &args),
        stderr: crate::manifest::one_line(&source.to_string()),
    })?;
    if !output.status.success() {
        return Err(Error::Git {
            command: spelled(root, &args),
            stderr: crate::manifest::one_line(&String::from_utf8_lossy(&output.stderr)),
        });
    }
    Ok(())
}

/// The url `origin` is at, or `None` when the repository has no such remote.
fn origin_url(root: &Path) -> Result<Option<String>> {
    let output = git_output(root, &["config", "--get", "remote.origin.url"])?;
    if !output.status.success() {
        return Ok(None);
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok((!url.is_empty()).then_some(url))
}

/// The name the subject takes: the repository's canonical url.
///
/// # Errors
///
/// [`Error::Receipt`] when the repository has no `origin`. A receipt names the repository it
/// is about, and a repository nobody has pushed anywhere has no name to give that is not
/// this machine's own path, which the plan forbids a receipt to carry.
fn subject_name(root: &Path, commit: &str) -> Result<String> {
    match origin_url(root)? {
        Some(url) => Ok(canonical_url(&url)),
        None => Err(Error::Receipt {
            commit: commit.to_owned(),
            problem: "this repository has no origin remote, so a receipt has no canonical name \
                      for its subject; give it one with `git remote add origin <url>`"
                .to_owned(),
        }),
    }
}

/// The paths one commit changed.
fn changed_files(root: &Path, commit: &str) -> Result<Vec<String>> {
    let text = git_text(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            commit,
        ],
    )?;
    Ok(text.lines().map(str::to_owned).collect())
}

/// The stamps one commit's diff added to the loops.
fn stamps(root: &Path, commit: &str) -> Result<Vec<Stamp>> {
    let diff = git_text(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "diff-tree",
            "--no-commit-id",
            "-r",
            "-p",
            commit,
            "--",
            LOOPS_DIR,
        ],
    )?;

    let mut stamps = Vec::new();
    let mut files: BTreeMap<String, String> = BTreeMap::new();
    for (path, line) in added_stamp_lines(&diff) {
        let file = match files.get(&path) {
            Some(file) => file,
            None => {
                let text = git_text(root, &["show", &format!("{commit}:{path}")])?;
                files.entry(path.clone()).or_insert(text)
            }
        };
        let (Some(check), Some(sha)) = (check_id(&path, &line, file), stamp_sha(&line)) else {
            continue;
        };
        stamps.push(Stamp {
            check,
            sha: sha.to_owned(),
        });
    }
    Ok(stamps)
}

/// What the pinned weeder says about this commit, when the repository has one to ask.
///
/// A judge that is not fetched is not a finding, and a judge that answers with something
/// other than SARIF is not a count: both leave the block `null` and the second says why on
/// the caller's stderr.
fn weeder_verification(root: &Path, commit: &str) -> (Option<WeederResult>, Option<String>) {
    let program = layout::judge_binary(root, WEEDER);
    if !program.exists() {
        return (None, None);
    }

    let output = Command::new(&program)
        .args([
            "check",
            "--format",
            "sarif",
            "--base",
            &format!("{commit}^"),
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .output();
    let output = match output {
        Ok(output) => output,
        Err(source) => {
            return (
                None,
                Some(format!("{} could not be run: {source}", program.display())),
            );
        }
    };

    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    match sarif::validate_from(WEEDER, &printed) {
        Ok(log) => (
            Some(WeederResult {
                sarif: Sha256Digest {
                    sha256: hex::encode(Sha256::digest(printed.as_bytes())),
                },
                block: sarif::block_count(&log),
                warn: sarif::level_count(&log, sarif::WARNING_LEVEL),
            }),
            None,
        ),
        Err(problem) => (
            None,
            Some(format!(
                "the receipt for {commit} carries no weeder result: {problem}"
            )),
        ),
    }
}

/// Every draft this commit's receipt folds in, by session, oldest name first.
///
/// Both the drafts waiting to be sealed and the drafts this same commit already sealed: the
/// second is what makes sealing a commit twice fold the same set and so say `unchanged`. A
/// session that drafted again after the seal shadows its own sealed copy, because the newer
/// draft is what that session now says about itself.
fn load_drafts(root: &Path, commit: &str) -> Result<Vec<(PathBuf, Draft)>> {
    let mut by_session: BTreeMap<String, PathBuf> = BTreeMap::new();
    for directory in [
        layout::receipt_sealed_dir(root, commit),
        layout::receipt_drafts_dir(root),
    ] {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(Error::Io {
                    path: directory,
                    source,
                });
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|it| it == "json") {
                let name = path
                    .file_name()
                    .map(|it| it.to_string_lossy().into_owned())
                    .unwrap_or_default();
                by_session.insert(name, path);
            }
        }
    }

    let mut drafts = Vec::new();
    for path in by_session.into_values() {
        let bytes = fs::read(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let draft: Draft = serde_json::from_slice(&bytes).map_err(|source| Error::Json {
            path: Some(path.clone()),
            source,
        })?;
        drafts.push((path, draft));
    }
    Ok(drafts)
}

/// Move the drafts a receipt folded in under the commit that folded them, so the next commit
/// does not fold the same session's counts a second time.
fn put_away(root: &Path, commit: &str, folded: &[(PathBuf, Draft)]) -> Result<()> {
    let waiting = layout::receipt_drafts_dir(root);
    let sealed = layout::receipt_sealed_dir(root, commit);
    let mut made = false;
    for (path, _) in folded {
        if !path.starts_with(&waiting) {
            continue;
        }
        if !made {
            fs::create_dir_all(&sealed).map_err(|source| Error::Io {
                path: sealed.clone(),
                source,
            })?;
            made = true;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        fs::rename(path, sealed.join(name)).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
    }
    Ok(())
}

// ------------------------------------------------------------------ the three faces

/// Seal one commit's receipt onto the receipts ref.
///
/// # Errors
///
/// [`Error::Git`] when git cannot resolve the revision or refuses the note, [`Error::Io`] or
/// [`Error::Json`] when a draft cannot be read, and [`Error::Receipt`] when the repository
/// has no canonical name to be the subject of a statement.
pub fn seal(root: &Path, rev: &str, now: &dyn Now) -> Result<Seal> {
    let commit = commit_of(root, rev)?;
    let tree = tree_of(root, &commit)?;
    let short = short_of(root, &commit)?;
    let name = subject_name(root, &short)?;

    let drafts = load_drafts(root, &commit)?;
    let folded = fold(&drafts.iter().map(|(_, it)| it.clone()).collect::<Vec<_>>());
    let changed = changed_files(root, &commit)?;
    let (weeder, aside) = weeder_verification(root, &commit);
    let verification = Verification {
        weeder,
        tend2: stamps(root, &commit)?,
        pleach: None,
        commands: Vec::new(),
    };

    let fresh = statement(
        &name,
        &commit,
        &tree,
        &now.rfc3339_utc(),
        folded,
        changed,
        verification,
    );

    let held = read_note(root, &commit)?.and_then(|note| readable(&note));
    let outcome = match held {
        Some(existing) if same_but_for_the_moment(&existing, &fresh) => Sealed::Unchanged,
        Some(_) => Sealed::Replaced,
        None => Sealed::Written,
    };
    if outcome != Sealed::Unchanged {
        write_note(root, &commit, &note_text(&fresh)?)?;
    }
    put_away(root, &commit, &drafts)?;

    Ok(Seal {
        commit,
        short,
        outcome,
        aside,
    })
}

/// The statement a note holds, when the note is one this stem could have written and its own
/// digest line is the digest of the bytes above it.
fn readable(note: &str) -> Option<Statement> {
    let (bytes, claimed) = split_note(note)?;
    if hex::encode(Sha256::digest(bytes.as_bytes())) != claimed {
        return None;
    }
    serde_json::from_str(bytes).ok()
}

/// What one commit's receipt is worth: read it, recompute every digest in it, and say.
///
/// # Errors
///
/// [`Error::Git`] when git could not answer, which is a failure to judge rather than a
/// judgement; the caller exits 1 on it and 3 on any verdict but [`Verdict::Ok`].
pub fn check(root: &Path, commit: &str, require_signed: bool) -> Result<Verdict> {
    let Some(note) = read_note(root, commit)? else {
        return Ok(Verdict::Missing);
    };
    let Some((bytes, claimed)) = split_note(&note) else {
        return Ok(Verdict::Invalid(
            "receipt malformed: its last line is not the statement's sha256".to_owned(),
        ));
    };

    let actual = hex::encode(Sha256::digest(bytes.as_bytes()));
    if actual != claimed {
        return Ok(Verdict::Invalid(format!(
            "receipt digest mismatch: the note claims sha256 {claimed}, its statement hashes \
             to {actual}"
        )));
    }

    let statement: Statement = match serde_json::from_str(bytes) {
        Ok(statement) => statement,
        Err(problem) => {
            return Ok(Verdict::Invalid(format!(
                "receipt malformed: {}",
                crate::manifest::one_line(&problem.to_string())
            )));
        }
    };
    let Some(subject) = statement.subject.first() else {
        return Ok(Verdict::Invalid(
            "receipt malformed: its statement names no subject".to_owned(),
        ));
    };

    let tree = tree_of(root, commit)?;
    if subject.digest.git_commit != commit {
        return Ok(Verdict::Invalid(format!(
            "receipt subject gitCommit is {}, git says {commit}",
            subject.digest.git_commit
        )));
    }
    if subject.digest.git_tree != tree {
        return Ok(Verdict::Invalid(format!(
            "receipt subject gitTree is {}, git says {tree}",
            subject.digest.git_tree
        )));
    }

    if require_signed {
        return Ok(Verdict::Unsigned);
    }
    Ok(Verdict::Ok)
}

/// The predicate one commit's receipt carries, for a person to read.
///
/// # Errors
///
/// [`Error::Git`] when git could not answer, and [`Error::Receipt`] when the commit carries
/// no receipt or one whose statement cannot be read.
pub fn show(root: &Path, rev: &str) -> Result<String> {
    let commit = commit_of(root, rev)?;
    let short = short_of(root, &commit)?;
    let refuse = |problem: String| Error::Receipt {
        commit: short.clone(),
        problem,
    };

    let Some(note) = read_note(root, &commit)? else {
        return Err(refuse("carries no receipt".to_owned()));
    };
    let Some((bytes, _)) = split_note(&note) else {
        return Err(refuse(
            "carries a note whose last line is not the statement's sha256".to_owned(),
        ));
    };
    let statement: Statement = serde_json::from_str(bytes).map_err(|source| {
        refuse(format!(
            "carries a note the stem cannot read as a statement: {}",
            crate::manifest::one_line(&source.to_string())
        ))
    })?;

    let text = serde_json::to_string_pretty(&statement.predicate)
        .map_err(|source| Error::Json { path: None, source })?;
    Ok(format!("{text}\n"))
}

// ------------------------------------------------------------------ the faces' exit codes

/// `plotplot receipt seal [--commit <rev>]`.
fn run_seal(root: &Path, args: &SealArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match seal(root, &args.commit, &SystemClock) {
        Ok(sealed) => {
            if let Some(aside) = &sealed.aside {
                let _ = writeln!(stderr, "{aside}");
            }
            match writeln!(stdout, "{} receipt {}", sealed.short, sealed.outcome.word()) {
                Ok(()) => VERIFIED,
                Err(error) => {
                    let _ = writeln!(stderr, "stdout: {error}");
                    FAILED
                }
            }
        }
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            FAILED
        }
    }
}

/// `plotplot receipt verify (<rev> | --range <a>..<b>) [--require-signed]`.
fn run_verify(
    root: &Path,
    args: &VerifyArgs,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let commits = match commits(root, args.rev.as_deref(), args.range.as_deref()) {
        Ok(commits) => commits,
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return FAILED;
        }
    };

    let mut code = VERIFIED;
    for commit in &commits {
        let short = match short_of(root, commit) {
            Ok(short) => short,
            Err(error) => {
                let _ = writeln!(stderr, "{error}");
                return FAILED;
            }
        };
        let verdict = match check(root, commit, args.require_signed) {
            Ok(verdict) => verdict,
            // A note the stem cannot read at all is a refusal like any other bad receipt;
            // only git failing to answer leaves the range unjudged.
            Err(Error::Receipt { problem, .. }) => Verdict::Invalid(problem),
            Err(error) => {
                let _ = writeln!(stderr, "{error}");
                return FAILED;
            }
        };
        if !verdict.passes() {
            code = REFUSED;
        }
        if let Err(error) = writeln!(stdout, "{short} {}", verdict.line()) {
            let _ = writeln!(stderr, "stdout: {error}");
            return FAILED;
        }
    }
    code
}

/// `plotplot receipt show <rev>`.
fn run_show(root: &Path, args: &ShowArgs, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match show(root, &args.rev) {
        Ok(text) => match write!(stdout, "{text}") {
            Ok(()) => VERIFIED,
            Err(error) => {
                let _ = writeln!(stderr, "stdout: {error}");
                FAILED
            }
        },
        Err(error @ Error::Git { .. }) => {
            let _ = writeln!(stderr, "{error}");
            FAILED
        }
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            REFUSED
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::{Harness, parse_payload};
    use serde_json::{Value, json};

    fn session_end(harness: Harness, extra: Value) -> Payload {
        let mut object = json!({
            "hook_event_name": harness.session_end_event(),
            "session_id": "6f3c1b2a",
            "cwd": "/work/repo",
        });
        if let (Some(object), Some(extra)) = (object.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                object.insert(key.clone(), value.clone());
            }
        }
        parse_payload(harness, &object.to_string()).expect("a parsed payload")
    }

    fn value(draft: &Draft) -> Value {
        serde_json::to_value(draft).expect("a serializable draft")
    }

    #[test]
    fn the_draft_names_the_fields_the_receipts_plan_spells() {
        let payload = session_end(Harness::Claude, json!({"model": "claude-opus-5"}));
        let draft = build(&payload, None, None).expect("a draft");
        let value = value(&draft);
        let object = value.as_object().expect("a draft is a JSON object");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "cost",
                "friction",
                "harness",
                "models",
                "principal",
                "sessions"
            ]
        );
        let mut cost: Vec<&str> = object["cost"]
            .as_object()
            .expect("cost is an object")
            .keys()
            .map(String::as_str)
            .collect();
        cost.sort_unstable();
        assert_eq!(
            cost,
            ["inputTokens", "outputTokens", "toolCalls", "wallSeconds"]
        );
        assert_eq!(object["harness"]["name"], "claude-code");
        assert_eq!(object["principal"], "cli");
        assert_eq!(object["models"], json!(["claude-opus-5"]));
        assert_eq!(object["sessions"], json!(["6f3c1b2a"]));
    }

    #[test]
    fn every_harness_names_itself_as_the_profile_does() {
        for (harness, name) in [
            (Harness::Claude, "claude-code"),
            (Harness::Gemini, "gemini-cli"),
            (Harness::Codex, "codex-cli"),
        ] {
            let draft = build(&session_end(harness, json!({})), None, None).expect("a draft");
            assert_eq!(draft.harness.name, name);
        }
    }

    #[test]
    fn a_payload_that_names_no_model_carries_no_models() {
        let draft = build(&session_end(Harness::Gemini, json!({})), None, None).expect("a draft");
        assert!(draft.models.is_empty());
        assert_eq!(value(&draft)["models"], json!([]));
    }

    #[test]
    fn what_the_payload_is_silent_about_is_null_and_never_estimated() {
        let draft = build(&session_end(Harness::Claude, json!({})), None, None).expect("a draft");
        assert_eq!(draft.harness.version, None);
        assert_eq!(draft.cost.input_tokens, None);
        assert_eq!(draft.cost.output_tokens, None);
        assert_eq!(draft.cost.wall_seconds, None);
        assert!(draft.cost.tool_calls.is_empty());
        assert_eq!(draft.friction.summary, None);

        let value = value(&draft);
        for null in ["inputTokens", "outputTokens", "wallSeconds"] {
            assert!(value["cost"][null].is_null(), "{null}");
        }
        assert!(value["harness"]["version"].is_null());
        assert!(value["friction"]["summary"].is_null());
        assert_eq!(value["cost"]["toolCalls"], json!({}));
    }

    #[test]
    fn a_payload_that_carries_its_version_and_its_usage_says_so() {
        let payload = session_end(
            Harness::Claude,
            json!({
                "harness_version": "2.1.265",
                "usage": {"input_tokens": 41_000, "output_tokens": 2_100},
            }),
        );
        let draft = build(&payload, None, None).expect("a draft");
        assert_eq!(draft.harness.version.as_deref(), Some("2.1.265"));
        assert_eq!(draft.cost.input_tokens, Some(41_000));
        assert_eq!(draft.cost.output_tokens, Some(2_100));
    }

    #[test]
    fn the_friction_state_is_where_the_tool_counts_come_from() {
        let mut state = SessionState::new();
        state.tool_calls.insert("Bash".to_owned(), 41);
        state.tool_calls.insert("Edit".to_owned(), 12);
        let draft = build(
            &session_end(Harness::Claude, json!({})),
            Some(&state),
            Some("9c1d".to_owned()),
        )
        .expect("a draft");

        assert_eq!(draft.cost.tool_calls.get("Bash"), Some(&41));
        assert_eq!(draft.cost.tool_calls.get("Edit"), Some(&12));
        assert_eq!(
            draft.friction.summary,
            Some(Sha256Digest {
                sha256: "9c1d".to_owned()
            })
        );
        assert_eq!(
            value(&draft)["friction"]["summary"],
            json!({"sha256": "9c1d"})
        );
    }

    #[test]
    fn a_payload_with_no_session_cannot_be_drafted() {
        let json = r#"{"hook_event_name":"SessionEnd","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert!(matches!(
            build(&payload, None, None),
            Err(Error::Harness { .. })
        ));
    }

    #[test]
    fn a_draft_survives_a_round_trip_through_its_file_shape() {
        let draft = build(
            &session_end(Harness::Claude, json!({"model": "claude-opus-5"})),
            None,
            None,
        )
        .expect("a draft");
        let text = serde_json::to_string(&draft).expect("a serializable draft");
        let back: Draft = serde_json::from_str(&text).expect("a readable draft");
        assert_eq!(back, draft);
    }

    // ------------------------------------------------------------------ the sealed receipt

    /// A clock the test pins, so `createdAt` is a value rather than a moving target.
    fn pinned() -> impl Now {
        || "2026-09-09T12:00:00.000Z".to_owned()
    }

    fn draft_of(session: &str, model: Option<&str>, tokens: Option<u64>) -> Draft {
        Draft {
            harness: DraftHarness {
                name: "claude-code".to_owned(),
                version: Some("2.1.265".to_owned()),
            },
            models: model.map(str::to_owned).into_iter().collect(),
            principal: PRINCIPAL_CLI.to_owned(),
            sessions: vec![session.to_owned()],
            cost: Cost {
                input_tokens: tokens,
                output_tokens: tokens,
                wall_seconds: None,
                tool_calls: BTreeMap::from([("Bash".to_owned(), 3)]),
            },
            friction: Friction {
                summary: Some(Sha256Digest {
                    sha256: format!("{session}-digest"),
                }),
            },
        }
    }

    #[test]
    fn a_github_url_in_any_of_gits_forms_is_the_same_subject() {
        for url in [
            "https://github.com/jahala/weeder",
            "https://github.com/jahala/weeder.git",
            "https://github.com/jahala/weeder/",
            "http://github.com/jahala/weeder.git",
            "https://jahala@github.com/jahala/weeder.git",
            "git@github.com:jahala/weeder.git",
            "git@github.com:jahala/weeder",
            "ssh://git@github.com/jahala/weeder.git",
            "git://github.com/jahala/weeder.git",
            "https://GitHub.com/jahala/weeder.git",
            "  https://github.com/jahala/weeder.git  ",
        ] {
            assert_eq!(canonical_url(url), "github.com/jahala/weeder", "{url}");
        }
    }

    #[test]
    fn a_url_the_stem_cannot_shorten_honestly_is_the_url_as_given() {
        for url in [
            "https://gitlab.com/jahala/weeder.git",
            "https://example.invalid/jahala/fixture.git",
            "git@codeberg.org:jahala/weeder.git",
            // Under github.com, but not one repository.
            "https://github.com/jahala",
            "https://github.com/jahala/weeder/tree/main",
            // Not a url at all.
            "/srv/git/weeder.git",
            "weeder",
        ] {
            assert_eq!(canonical_url(url), url, "{url}");
        }
    }

    #[test]
    fn a_commit_that_folded_no_draft_says_null_and_never_a_number() {
        let folded = fold(&[]);
        assert_eq!(folded.harness, None);
        assert_eq!(folded.models, None);
        assert_eq!(folded.principal, None);
        assert!(folded.sessions.is_empty());
        assert_eq!(folded.cost.input_tokens, None);
        assert_eq!(folded.cost.output_tokens, None);
        assert_eq!(folded.cost.wall_seconds, None);
        assert!(folded.cost.tool_calls.is_empty());
        assert_eq!(folded.friction.summary, None);
    }

    #[test]
    fn folding_one_draft_carries_what_that_draft_said() {
        let folded = fold(&[draft_of("aaa", Some("claude-opus-5"), Some(1_000))]);
        assert_eq!(
            folded.harness.map(|it| it.name).as_deref(),
            Some("claude-code")
        );
        assert_eq!(
            folded.models.as_deref(),
            Some(["claude-opus-5".to_owned()].as_slice())
        );
        assert_eq!(folded.principal.as_deref(), Some("cli"));
        assert_eq!(folded.sessions, ["aaa"]);
        assert_eq!(folded.cost.input_tokens, Some(1_000));
        assert_eq!(folded.cost.tool_calls.get("Bash"), Some(&3));
    }

    #[test]
    fn folding_two_drafts_adds_the_counts_and_keeps_each_session() {
        let folded = fold(&[
            draft_of("aaa", Some("claude-opus-5"), Some(1_000)),
            draft_of("bbb", Some("gpt-5"), Some(250)),
        ]);
        assert_eq!(folded.sessions, ["aaa", "bbb"]);
        assert_eq!(
            folded.models.as_deref(),
            Some(["claude-opus-5".to_owned(), "gpt-5".to_owned()].as_slice())
        );
        assert_eq!(folded.cost.input_tokens, Some(1_250));
        assert_eq!(folded.cost.tool_calls.get("Bash"), Some(&6));
    }

    #[test]
    fn a_draft_that_reports_no_tokens_makes_the_total_unknown_rather_than_partial() {
        let folded = fold(&[
            draft_of("aaa", None, Some(1_000)),
            draft_of("bbb", None, None),
        ]);
        assert_eq!(folded.cost.input_tokens, None);
        assert_eq!(folded.cost.output_tokens, None);
        // What is known is still known: the tool counts are a sum of what was counted.
        assert_eq!(folded.cost.tool_calls.get("Bash"), Some(&6));
    }

    #[test]
    fn drafts_that_named_no_model_leave_an_empty_list_and_not_a_null() {
        let folded = fold(&[draft_of("aaa", None, None)]);
        assert_eq!(folded.models, Some(Vec::new()));
    }

    #[test]
    fn the_friction_digest_is_over_the_drafts_digests_in_order() {
        let first = draft_of("aaa", None, None);
        let second = draft_of("bbb", None, None);
        let one_way = fold(&[first.clone(), second.clone()]).friction.summary;
        let other_way = fold(&[second, first.clone()]).friction.summary;
        assert!(one_way.is_some());
        assert_ne!(one_way, other_way, "the order of the drafts is part of it");

        let mut hasher = Sha256::new();
        hasher.update(b"aaa-digest\n");
        assert_eq!(
            fold(&[first]).friction.summary,
            Some(Sha256Digest {
                sha256: hex::encode(hasher.finalize())
            })
        );
    }

    fn a_statement() -> Statement {
        statement(
            "github.com/jahala/weeder",
            "3f2a1b4c5d6e7f8091a2b3c4d5e6f708192a3b4c",
            "9c1d0f0e6a1b2c3d4e5f60718293a4b5c6d7e8f9",
            "2026-09-09T12:00:00.000Z",
            fold(&[draft_of("aaa", Some("claude-opus-5"), Some(10))]),
            vec!["src/lib.rs".to_owned()],
            Verification {
                weeder: None,
                tend2: Vec::new(),
                pleach: None,
                commands: Vec::new(),
            },
        )
    }

    /// The keys of a pretty-printed object's body, in the order they were written. `Value`
    /// is a sorted map, so only the bytes can answer the question §3 asks.
    fn written_keys(text: &str, indent: usize) -> Vec<String> {
        let prefix = " ".repeat(indent);
        text.lines()
            .filter_map(|line| {
                let rest = line.strip_prefix(&prefix)?.strip_prefix('"')?;
                if rest.starts_with(' ') {
                    return None;
                }
                Some(rest.split_once("\":")?.0.to_owned())
            })
            .collect()
    }

    #[test]
    fn the_statement_is_written_in_the_plans_key_order() {
        let text = serde_json::to_string_pretty(&a_statement()).expect("a statement");
        assert_eq!(
            written_keys(&text, 2),
            ["_type", "subject", "predicateType", "predicate"]
        );
        assert_eq!(
            written_keys(&text, 4),
            [
                "producer",
                "createdAt",
                "harness",
                "models",
                "principal",
                "sessions",
                "conductor",
                "changed",
                "verification",
                "cost",
                "friction"
            ]
        );
    }

    #[test]
    fn a_statement_survives_a_round_trip_through_its_note() {
        let statement = a_statement();
        let note = note_text(&statement).expect("a note");
        let (bytes, digest) = split_note(&note).expect("a note splits");
        assert_eq!(hex::encode(Sha256::digest(bytes.as_bytes())), digest);
        let back: Statement = serde_json::from_str(bytes).expect("a readable statement");
        assert_eq!(back, statement);
    }

    #[test]
    fn the_hashed_bytes_are_everything_above_the_last_line() {
        let note = note_text(&a_statement()).expect("a note");
        let (bytes, _) = split_note(&note).expect("a note splits");
        assert!(bytes.ends_with("}\n"), "{bytes}");
        assert_eq!(note.lines().count(), bytes.lines().count() + 1);
        assert!(note.ends_with('\n'));
    }

    #[test]
    fn a_note_with_no_digest_line_does_not_split() {
        assert!(split_note("{}\n").is_none());
        assert!(split_note("{}\nnot a digest line\n").is_none());
        assert!(split_note("").is_none());
    }

    #[test]
    fn two_statements_that_differ_only_in_when_they_were_sealed_are_the_same_receipt() {
        let existing = a_statement();
        let mut fresh = existing.clone();
        fresh.predicate.created_at = "2027-01-01T00:00:00.000Z".to_owned();
        assert!(same_but_for_the_moment(&existing, &fresh));

        fresh.predicate.changed.push("src/other.rs".to_owned());
        assert!(!same_but_for_the_moment(&existing, &fresh));
    }

    #[test]
    fn a_stamp_is_an_added_ticked_line_in_a_loop_and_nothing_else() {
        let diff = concat!(
            "diff --git a/docs/tend2/rules-block.tend2.html b/docs/tend2/rules-block.tend2.html\n",
            "--- a/docs/tend2/rules-block.tend2.html\n",
            "+++ b/docs/tend2/rules-block.tend2.html\n",
            "@@ -1,3 +1,3 @@\n",
            "-- [ ] (code) the third check · rules.sh three\n",
            "+- [x] (code) the third check · rules.sh three @9853abb · by cli\n",
            "+- [ ] (code) a fourth check nobody stamped · rules.sh four\n",
            "+a plain added line mentioning [x] and @9853abb outside a check\n",
            "diff --git a/README.md b/README.md\n",
            "--- a/README.md\n",
            "+++ b/README.md\n",
            "+- [x] a ticked line in a file that is not a loop @9853abb\n",
        );
        let added = added_stamp_lines(diff);
        assert_eq!(added.len(), 1, "{added:?}");
        assert_eq!(added[0].0, "docs/tend2/rules-block.tend2.html");
        assert!(added[0].1.contains("the third check"));
    }

    #[test]
    fn a_stamps_sha_is_the_first_at_token_that_could_be_one() {
        assert_eq!(stamp_sha("- [x] a · b @9853abb · by cli"), Some("9853abb"));
        assert_eq!(stamp_sha("- [x] a @jahala · b @9853abb"), Some("9853abb"));
        assert_eq!(stamp_sha("- [x] a · b"), None);
        assert_eq!(stamp_sha("- [x] a @zzz"), None);
    }

    #[test]
    fn a_check_is_addressed_the_way_the_verifier_was_called() {
        let file = concat!(
            "## Tests\n",
            "- [ ] (code) one · a.sh\n",
            "- [x] (code) two · b.sh @1234abc\n",
            "- [ ] (code) three · c.sh\n",
            "\n",
            "## Tried\n",
            "- 2026-09-09 a Tried line is not a check\n",
        );
        assert_eq!(
            check_id(
                "docs/tend2/rules-block.tend2.html",
                "- [x] (code) two · b.sh @1234abc",
                file
            )
            .as_deref(),
            Some("rules-block:c2")
        );
        assert_eq!(
            check_id("docs/tend2/rules-block.tend2.html", "- [x] absent", file),
            None
        );
        assert_eq!(
            loop_name("docs/tend2/rules-block.tend2.html"),
            Some("rules-block")
        );
        assert_eq!(loop_name("docs/tend2/loop.css"), None);
    }

    // ------------------------------------------------------------- against a real repository

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git runs on the build machine");
        assert!(
            output.status.success(),
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    /// A repository with one commit and an origin, which is the least a receipt needs.
    fn repository(origin: Option<&str>) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("a temp directory");
        let root = tmp.path().join("repo");
        fs::create_dir_all(&root).expect("the repository directory");
        git(tmp.path(), &["init", "-q", "repo"]);
        git(&root, &["config", "user.email", "unit@plotplot.invalid"]);
        git(&root, &["config", "user.name", "plotplot unit"]);
        if let Some(origin) = origin {
            git(&root, &["remote", "add", "origin", origin]);
        }
        fs::write(root.join("README.md"), "# unit\n").expect("a file to commit");
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "one"]);
        (tmp, root)
    }

    #[test]
    fn the_subject_digests_are_the_ones_git_holds() {
        let (_tmp, root) = repository(Some("https://github.com/jahala/fixture.git"));
        let sealed = seal(&root, "HEAD", &pinned()).expect("a sealed receipt");
        assert_eq!(sealed.outcome, Sealed::Written);

        let note = read_note(&root, &sealed.commit)
            .expect("git answers")
            .expect("a note");
        let (bytes, _) = split_note(&note).expect("a note splits");
        let statement: Statement = serde_json::from_str(bytes).expect("a statement");
        let subject = statement.subject.first().expect("one subject");
        assert_eq!(subject.name, "github.com/jahala/fixture");
        assert_eq!(
            subject.digest.git_commit,
            git(&root, &["rev-parse", "HEAD"])
        );
        assert_eq!(
            subject.digest.git_tree,
            git(&root, &["rev-parse", "HEAD^{tree}"])
        );
        assert_eq!(statement.predicate.created_at, "2026-09-09T12:00:00.000Z");
    }

    #[test]
    fn sealing_the_same_commit_twice_changes_nothing() {
        let (_tmp, root) = repository(Some("https://github.com/jahala/fixture.git"));
        let first = seal(&root, "HEAD", &pinned()).expect("a sealed receipt");
        let before = read_note(&root, &first.commit).expect("git answers");

        // A clock that moved: only `createdAt` differs, and that is not a new receipt.
        let later = || "2027-01-01T00:00:00.000Z".to_owned();
        let second = seal(&root, "HEAD", &later).expect("a second seal");
        assert_eq!(second.outcome, Sealed::Unchanged);
        assert_eq!(
            read_note(&root, &first.commit).expect("git answers"),
            before
        );
    }

    #[test]
    fn a_sealed_commit_verifies_and_a_bare_one_does_not() {
        let (_tmp, root) = repository(Some("https://github.com/jahala/fixture.git"));
        let commit = commit_of(&root, "HEAD").expect("a commit");
        assert_eq!(
            check(&root, &commit, false).expect("git answers"),
            Verdict::Missing
        );

        seal(&root, "HEAD", &pinned()).expect("a sealed receipt");
        assert_eq!(
            check(&root, &commit, false).expect("git answers"),
            Verdict::Ok
        );
        assert_eq!(
            check(&root, &commit, true).expect("git answers"),
            Verdict::Unsigned
        );
    }

    #[test]
    fn a_repository_with_no_origin_is_refused_rather_than_named_after_this_machine() {
        let (_tmp, root) = repository(None);
        match seal(&root, "HEAD", &pinned()) {
            Err(Error::Receipt { problem, .. }) => {
                assert!(problem.contains("origin"), "{problem}");
            }
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_revision_git_does_not_know_is_a_git_error_and_never_a_panic() {
        let (_tmp, root) = repository(Some("https://github.com/jahala/fixture.git"));
        assert!(matches!(
            commit_of(&root, "no-such-revision"),
            Err(Error::Git { .. })
        ));
        assert!(matches!(
            commits(&root, None, Some("no..such")),
            Err(Error::Git { .. })
        ));
    }

    #[test]
    fn the_note_a_seal_wrote_is_the_note_git_hands_back() {
        let (_tmp, root) = repository(Some("git@github.com:jahala/fixture.git"));
        let sealed = seal(&root, "HEAD", &pinned()).expect("a sealed receipt");
        let held = read_note(&root, &sealed.commit)
            .expect("git answers")
            .expect("a note");
        let shown = git(
            &root,
            &[
                "notes",
                "--ref",
                gitconfig::RECEIPTS_REF,
                "show",
                &sealed.commit,
            ],
        );
        assert_eq!(held.trim_end(), shown);
        assert!(held.ends_with('\n'), "the note ends with one newline");
    }
}
