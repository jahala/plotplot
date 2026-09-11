//! The platform half of the law: what `init --github` projects onto GitHub, and what
//! `doctor --platform` reads back.
//!
//! The repository holds the truth in files, and the platform enforces it on people. A git
//! hook binds a person until `--no-verify`; only a ruleset binds past that (jahala/plotplot
//! issue 7). So `init --github` writes the stem's region of `.github/CODEOWNERS` and applies
//! one ruleset on the default branch, and `doctor --platform` asks GitHub which rules are in
//! force there, with read-only calls.
//!
//! The ruleset requires a check the pull request gate reports, so it is applied only once the
//! default branch's workflow reports it (jahala/plotplot issue 33): a ruleset requiring a check
//! no workflow on the branch reports lets no pull request merge, the one that would fix the
//! workflow included.
//!
//! GitHub is reached through `gh` behind [`Gh`]. The body, the comparison, the region and the
//! read-back are pure functions over strings and JSON values with unit tests of their own;
//! `scripts/fit/stem.sh platform` runs the real binary against a recording `gh`.

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

pub use crate::doctor::live::Ran;
use crate::doctor::live::{LiveFinding, Verdict};
use crate::error::{Error, Result};
use crate::layout;
use crate::manifest::one_line;
use crate::plant::region::{self, Markers};

/// The status check the ruleset requires, and the name of the pull request gate's job that
/// reports it (a contract ruled on 2026-09-10). The ruleset names the check, so a job renamed
/// without it silently unprotects the branch; the workflow, the body and the read-back all
/// read this one constant.
pub const REQUIRED_CHECK: &str = "garden";

/// What the stem calls the one ruleset it applies, and how it finds it again.
pub const RULESET_NAME: &str = "plotplot: the default branch";

/// The program the platform is reached through.
pub const GH: &str = "gh";

/// Where the stem's lines in `.github/CODEOWNERS` start. Every line outside the region is
/// somebody else's.
pub const BEGIN: &str = "# plotplot:begin";
/// Where they end.
pub const END: &str = "# plotplot:end";

/// The paths the region names the owner for, in the order it names them: weeder's C1
/// guardrail files (the git hooks, the lock, the hard limits' files, weeder's own
/// configuration, the three harnesses' settings), the stem's own files, the pull request gate
/// and the map. Each is anchored at the root, and a directory ends in `/`.
pub const GUARDED: [&str; 12] = [
    "/.githooks/",
    "/garden.lock",
    "/garden.json",
    "/AGENTS.md",
    "/CLAUDE.md",
    "/weeder.toml",
    "/.claude/settings.json",
    "/.gemini/settings.json",
    "/.codex/hooks.json",
    "/.github/workflows/plotplot-check.yml",
    "/.github/CODEOWNERS",
    "/docs/tend2/",
];

/// The fields that decide what a ruleset enforces, and the only ones compared.
const COMPARED: [&str; 4] = ["target", "enforcement", "conditions", "rules"];

// ---------------------------------------------------------------------- the repository

/// A repository on github.com, as its origin url names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub owner: String,
    pub name: String,
}

impl Repository {
    /// `<owner>/<name>`, the way GitHub's paths spell it.
    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// The repository an origin url names on github.com, or `None` for any other host or shape.
///
/// Three spellings, each with or without `.git` and a trailing slash: https, ssh in scp form
/// (`git@github.com:`), and ssh as a url. Both segments go into an API path, so each has to be
/// a name GitHub itself accepts, and a url with anything after the name is not a repository.
pub fn github_repository(origin: &str) -> Option<Repository> {
    let rest = [
        "https://github.com/",
        "git@github.com:",
        "ssh://git@github.com/",
    ]
    .iter()
    .find_map(|prefix| origin.trim().strip_prefix(prefix))?;
    let rest = rest.trim_end_matches('/');
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let (owner, name) = rest.split_once('/')?;
    if is_github_name(owner) && is_github_name(name) {
        Some(Repository {
            owner: owner.to_owned(),
            name: name.to_owned(),
        })
    } else {
        None
    }
}

/// Letters, digits, `-`, `_` and `.`, and never a segment a path would read as a directory.
fn is_github_name(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Why the platform cannot be reached from here: each is one line a planter can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unreachable {
    /// No `gh` on the search path.
    NoGh,
    /// The repository has no `origin` remote.
    NoOrigin,
    /// `origin` is not a repository on github.com.
    NotGithub { origin: String },
}

impl fmt::Display for Unreachable {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unreachable::NoGh => write!(out, "{GH} is not on PATH"),
            Unreachable::NoOrigin => out.write_str("this repository has no origin remote"),
            Unreachable::NotGithub { origin } => {
                write!(out, "origin {origin} is not a repository on github.com")
            }
        }
    }
}

/// The `gh` and the repository to act on, or why there are none.
///
/// `gh` is asked about first: without it nothing on the platform can be read or written,
/// whatever the origin says.
pub fn reach<G>(
    gh: Option<G>,
    origin: Option<&str>,
) -> std::result::Result<(G, Repository), Unreachable> {
    let gh = gh.ok_or(Unreachable::NoGh)?;
    let origin = origin.ok_or(Unreachable::NoOrigin)?;
    let repository = github_repository(origin).ok_or_else(|| Unreachable::NotGithub {
        origin: origin.to_owned(),
    })?;
    Ok((gh, repository))
}

// ---------------------------------------------------------------------------- the seam

/// One call to GitHub's API. The real implementation runs `gh`; a test double answers from
/// canned JSON so everything around the call can be judged without a network.
pub trait Gh {
    /// The method, the path under the API root, and the JSON body when there is one.
    fn api(&self, method: &str, path: &str, body: Option<&str>) -> Ran;
    /// A read of a file's raw contents: GitHub answers the file's bytes, not JSON about it.
    fn raw(&self, path: &str) -> Ran;
}

/// The Accept header that makes GitHub's contents endpoint answer a file's bytes rather than
/// JSON carrying them in base64.
pub const RAW: &str = "Accept: application/vnd.github.raw+json";

/// The real `gh`: the executable found on the search path, one process per call, with the
/// planter's own login.
pub struct GhCli {
    pub program: PathBuf,
}

impl Gh for GhCli {
    fn api(&self, method: &str, path: &str, body: Option<&str>) -> Ran {
        self.run(
            &call_args(method, path, body.is_some()),
            body,
            &call_line(method, path),
        )
    }

    fn raw(&self, path: &str) -> Ran {
        self.run(&raw_args(path), None, &raw_line(path))
    }
}

impl GhCli {
    /// One gh process: its arguments, the body on stdin when there is one, and the line that
    /// names the call when gh could not be run at all.
    fn run(&self, args: &[String], body: Option<&str>, line: &str) -> Ran {
        let failed = |error: std::io::Error| Ran {
            code: 1,
            stdout: String::new(),
            stderr: format!("{line}: {error}\n"),
        };
        let mut child = match Command::new(&self.program)
            .args(args)
            .stdin(if body.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => return failed(error),
        };

        // The body goes in whole and the pipe closes when `stdin` drops, which is how gh
        // knows the input has ended.
        let written = match (body, child.stdin.take()) {
            (Some(body), Some(mut stdin)) => stdin.write_all(body.as_bytes()),
            _ => Ok(()),
        };
        let ran = match child.wait_with_output() {
            Ok(output) => Ran {
                code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(error) => return failed(error),
        };
        // A gh that failed has said why in its own words; one that succeeded without the
        // whole body cannot have applied it, and that is said instead.
        match written {
            Err(error) if ran.code == 0 => failed(error),
            _ => ran,
        }
    }
}

/// gh's arguments for one call: `api <path>` for a read, `api -X <METHOD> <path> --input -`
/// for a call carrying a body on stdin.
pub fn call_args(method: &str, path: &str, has_body: bool) -> Vec<String> {
    let mut args = vec!["api".to_owned()];
    if method != "GET" || has_body {
        args.extend(["-X".to_owned(), method.to_owned()]);
    }
    args.push(path.to_owned());
    if has_body {
        args.extend(["--input".to_owned(), "-".to_owned()]);
    }
    args
}

/// The call as a planter would type it, which is the line that names a failure.
pub fn call_line(method: &str, path: &str) -> String {
    if method == "GET" {
        format!("{GH} api {path}")
    } else {
        format!("{GH} api -X {method} {path}")
    }
}

/// gh's arguments for a raw read: `api -H "<RAW>" <path>`, a GET.
pub fn raw_args(path: &str) -> Vec<String> {
    vec![
        "api".to_owned(),
        "-H".to_owned(),
        RAW.to_owned(),
        path.to_owned(),
    ]
}

/// A raw read as a planter would type it.
pub fn raw_line(path: &str) -> String {
    format!("{GH} api -H \"{RAW}\" {path}")
}

// ------------------------------------------------------------------------- the ruleset

/// The ruleset `init --github` applies, as GitHub's rulesets API takes it.
///
/// Deletion and force pushes refused, and the pull request gate's check required, on the
/// default branch, with nobody allowed past. Linear history is deliberately absent: the law
/// merges every pull request with a merge commit, and GitHub's linear-history rule refuses
/// those.
pub fn desired_ruleset() -> Value {
    json!({
        "name": RULESET_NAME,
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": ["~DEFAULT_BRANCH"], "exclude": []}},
        "rules": [
            {"type": "deletion"},
            {"type": "non_fast_forward"},
            {"type": "required_status_checks", "parameters": {
                "strict_required_status_checks_policy": false,
                "required_status_checks": [{"context": REQUIRED_CHECK}],
            }},
        ],
    })
}

/// The body on gh's stdin: compact JSON, keys in one order, the same bytes every run.
pub fn ruleset_body() -> String {
    desired_ruleset().to_string()
}

/// Whether a ruleset GitHub holds enforces what the desired one does.
///
/// Target, enforcement, conditions and rules are compared. GitHub hands a ruleset back with
/// fields of its own: an id, a source and links at the top, `do_not_enforce_on_create` beside
/// the status checks, an `integration_id` on a check (read 2026-09-11 off public rulesets).
/// Those are not differences, or every run would put the same body again. Every key the
/// desired ruleset names must hold an equal value, a key only GitHub adds is left out, and
/// every list must hold the same entries, in any order, and nothing more: a rule somebody
/// added, or one taken away, is a difference.
pub fn same_ruleset(held: &Value) -> bool {
    let desired = desired_ruleset();
    COMPARED
        .iter()
        .all(|field| match (desired.get(field), held.get(field)) {
            (Some(want), Some(have)) => covers(want, have),
            _ => false,
        })
}

/// Whether `have` says everything `want` says, in the sense [`same_ruleset`] gives.
fn covers(want: &Value, have: &Value) -> bool {
    match (want, have) {
        (Value::Object(want), Value::Object(have)) => want
            .iter()
            .all(|(key, value)| have.get(key).is_some_and(|held| covers(value, held))),
        (Value::Array(want), Value::Array(have)) => {
            if want.len() != have.len() {
                return false;
            }
            let mut taken = vec![false; have.len()];
            want.iter().all(|value| {
                let found = have.iter().enumerate().position(|(index, held)| {
                    taken.get(index) == Some(&false) && covers(value, held)
                });
                match found.and_then(|index| taken.get_mut(index)) {
                    Some(slot) => {
                        *slot = true;
                        true
                    }
                    None => false,
                }
            })
        }
        _ => want == have,
    }
}

/// The id of the ruleset called [`RULESET_NAME`] in the list GitHub answered.
///
/// # Errors
///
/// What is wrong with the answer when it is not a list of rulesets.
pub fn find_ruleset(list: &Value) -> std::result::Result<Option<u64>, String> {
    let rulesets = list
        .as_array()
        .ok_or_else(|| "the answer is not a list of rulesets".to_owned())?;
    for ruleset in rulesets {
        if ruleset.get("name").and_then(Value::as_str) == Some(RULESET_NAME) {
            return ruleset
                .get("id")
                .and_then(Value::as_u64)
                .map(Some)
                .ok_or_else(|| format!("the ruleset called \"{RULESET_NAME}\" carries no id"));
        }
    }
    Ok(None)
}

/// Why the ruleset was not applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unapplied {
    /// gh exited non-zero. What it wrote is printed as it wrote it.
    Refused { call: String, ran: Ran },
    /// gh exited zero with an answer that is not the shape GitHub documents.
    Unreadable {
        call: String,
        problem: String,
        ran: Ran,
    },
    /// The default branch's workflow reports no job named [`REQUIRED_CHECK`], so a ruleset
    /// requiring that check would let no pull request merge (jahala/plotplot issue 33).
    Ungated {
        /// The gate as the default branch holds it.
        gate: Gate,
        /// `Ok(Some(id))` when the stem's ruleset already requires the check, which is that
        /// deadlock now; `Ok(None)` when it does not; `Err` with gh's word when the rulesets
        /// could not be read.
        deadlock: std::result::Result<Option<u64>, String>,
    },
}

/// Apply the ruleset on `repository`: create it when it is absent, put the desired body to it
/// when it says something else, and leave it alone when it already says this.
///
/// `gate` is the pull request gate as the default branch holds it ([`read_gate`]). A ruleset
/// requiring [`REQUIRED_CHECK`] is applied only when that gate already reports the check: a
/// rename lands the workflow on the default branch first, and the ruleset moves after. Asked
/// otherwise, it refuses before anything is listed, compared or written; the only calls a
/// refusal makes are the reads that say whether the stem's ruleset already requires the check.
///
/// `Ok(Some(line))` when a call changed the platform, `Ok(None)` when no call was needed.
///
/// # Errors
///
/// [`Unapplied::Ungated`] when the gate does not report the check, else [`Unapplied`] naming
/// the first call that did not do what it was for.
pub fn apply_ruleset(
    gh: &dyn Gh,
    repository: &Repository,
    gate: &Gate,
) -> std::result::Result<Option<String>, Unapplied> {
    if !gate.reports(REQUIRED_CHECK) {
        return Err(Unapplied::Ungated {
            gate: gate.clone(),
            deadlock: requiring(gh, repository),
        });
    }

    let slug = repository.slug();
    let list_path = format!("repos/{slug}/rulesets");
    let list = answer(gh, &list_path)?;
    let found = find_ruleset(&list.0).map_err(|problem| Unapplied::Unreadable {
        call: call_line("GET", &list_path),
        problem,
        ran: list.1,
    })?;

    let (method, path) = match found {
        None => ("POST", list_path),
        Some(id) => {
            let path = format!("repos/{slug}/rulesets/{id}");
            if same_ruleset(&answer(gh, &path)?.0) {
                return Ok(None);
            }
            ("PUT", path)
        }
    };

    let ran = gh.api(method, &path, Some(&ruleset_body()));
    if ran.code != 0 {
        return Err(Unapplied::Refused {
            call: call_line(method, &path),
            ran,
        });
    }
    Ok(Some(format!(
        "ruleset \"{RULESET_NAME}\" applied on {slug}"
    )))
}

/// One read, parsed, with the answer kept for a refusal to quote.
fn answer(gh: &dyn Gh, path: &str) -> std::result::Result<(Value, Ran), Unapplied> {
    let call = call_line("GET", path);
    let ran = gh.api("GET", path, None);
    if ran.code != 0 {
        return Err(Unapplied::Refused { call, ran });
    }
    match serde_json::from_str(&ran.stdout) {
        Ok(value) => Ok((value, ran)),
        Err(error) => Err(Unapplied::Unreadable {
            call,
            problem: one_line(&error.to_string()),
            ran,
        }),
    }
}

/// The id of the stem's ruleset when it is enforced and already requires [`REQUIRED_CHECK`],
/// read with two read-only calls; gh's word when the rulesets could not be read. A ruleset that
/// is disabled or only evaluated blocks no merge, so it is no deadlock.
fn requiring(gh: &dyn Gh, repository: &Repository) -> std::result::Result<Option<u64>, String> {
    let slug = repository.slug();
    let (list, _) = answer(gh, &format!("repos/{slug}/rulesets")).map_err(|u| reason(&u))?;
    let Some(id) = find_ruleset(&list)? else {
        return Ok(None);
    };
    let (held, _) = answer(gh, &format!("repos/{slug}/rulesets/{id}")).map_err(|u| reason(&u))?;
    let enforced = held.get("enforcement").and_then(Value::as_str) == Some("active");
    let requires = enforced
        && held
            .get("rules")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|rule| {
                rule.get("type").and_then(Value::as_str) == Some("required_status_checks")
            })
            .any(|rule| contexts(rule).contains(&REQUIRED_CHECK));
    Ok(requires.then_some(id))
}

/// What `init` prints on stderr for a ruleset that was not applied.
///
/// For a call that failed: one line naming the call, then what gh wrote on stderr and on
/// stdout, as it wrote them. For a gate that does not report the check: one line naming the
/// branch, what it holds and the fix, and a second when the stem's ruleset already requires
/// the check, or when whether it does could not be read.
pub fn unapplied_text(unapplied: &Unapplied) -> String {
    let (head, ran) = match unapplied {
        Unapplied::Refused { call, ran } => (format!("{call} could not be applied:"), ran),
        Unapplied::Unreadable { call, problem, ran } => (
            format!(
                "{call} answered what the stem cannot read ({problem}), so nothing was applied:"
            ),
            ran,
        ),
        Unapplied::Ungated { gate, deadlock } => return ungated_text(gate, deadlock),
    };
    format!("{head}\n{}{}", verbatim(&ran.stderr), verbatim(&ran.stdout))
}

fn ungated_text(gate: &Gate, deadlock: &std::result::Result<Option<u64>, String>) -> String {
    let branch = &gate.branch;
    let mut text = format!(
        "{}; land the workflow on {branch} first, then run plotplot init --github again\n",
        gate.missing()
    );
    match deadlock {
        Ok(Some(id)) => text.push_str(&format!(
            "ruleset {id} already requires \"{REQUIRED_CHECK}\", so {branch} takes no merge \
             until that workflow lands\n"
        )),
        Ok(None) => {}
        Err(reason) => text.push_str(&format!(
            "whether ruleset \"{RULESET_NAME}\" already requires \"{REQUIRED_CHECK}\" could \
             not be read: {reason}\n"
        )),
    }
    text
}

/// A failed call as one line: gh's own first word, or the call and its exit when gh said
/// nothing; for an answer the stem could not read, the call and the problem.
fn reason(unapplied: &Unapplied) -> String {
    match unapplied {
        Unapplied::Refused { call, ran } => ran
            .first_line()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{call} exited {}", ran.code)),
        Unapplied::Unreadable { call, problem, .. } => {
            format!("{call} answered what the stem cannot read ({problem})")
        }
        Unapplied::Ungated { gate, .. } => gate.missing(),
    }
}

/// gh's words untouched, ending in a newline so the next line starts on its own.
fn verbatim(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text.to_owned()
    } else {
        format!("{text}\n")
    }
}

// -------------------------------------------------------------------------- CODEOWNERS

/// The stem's region of `.github/CODEOWNERS`: the markers, and one line per guarded path
/// naming `@owner`.
pub fn codeowners_region(owner: &str) -> String {
    let mut region = format!("{BEGIN}\n");
    for path in GUARDED {
        region.push_str(&format!("{path} @{owner}\n"));
    }
    region.push_str(END);
    region
}

/// `codeowners` with `region` planted in it, every line outside the markers left as it was.
///
/// # Errors
///
/// [`Error::Codeowners`] when the markers do not make exactly one region.
pub fn codeowners_in(codeowners: &str, region: &str) -> Result<String> {
    let markers = Markers {
        file: layout::CODEOWNERS,
        begin: BEGIN,
        end: END,
    };
    region::replace_in(codeowners, region, markers).map_err(|problem| Error::Codeowners { problem })
}

// ---------------------------------------------------------------------------- the gate

/// One job the pull request gate declares: its key under `jobs:`, and the name it reports its
/// check under, which is what a ruleset requires.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub key: String,
    pub name: String,
}

/// The jobs a workflow declares, read without a YAML parser.
///
/// After a line that is exactly `jobs:`, a line of two spaces, a key of letters, digits, `-`
/// and `_`, and a colon starts a job; a line of four spaces, `name: ` and some text under it
/// is the name that job reports, trimmed, with a pair of surrounding single or double quotes
/// removed. A job with no `name:` reports its key, which is what GitHub does. A line with no
/// indent ends the section; blank lines and comments do not.
///
/// That is enough because the file is one the stem itself writes ([`crate::init::workflow`]).
/// A hand-edited workflow that hides its job's name from this reader (a flow mapping, a
/// comment after the key, a name folded onto the next line) is a workflow the doctor reports
/// as naming no `garden` job, which is the honest reading: the stem cannot tell that the check
/// is reported, so it does not say so.
pub fn gate_jobs(workflow: &str) -> Vec<Job> {
    let mut jobs: Vec<Job> = Vec::new();
    let mut inside = false;
    // Whether the job being read has met its `name:` line; the first one is its name.
    let mut named = false;
    for line in workflow.lines() {
        if !inside {
            inside = line == "jobs:";
            continue;
        }
        let content = line.trim_start();
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        if let Some(key) = job_key(line) {
            jobs.push(Job {
                key: key.to_owned(),
                name: key.to_owned(),
            });
            named = false;
        } else if let Some(text) = line.strip_prefix("    name: ")
            && let Some(job) = jobs.last_mut()
            && !named
        {
            let name = unquote(text.trim());
            if !name.is_empty() {
                job.name = name.to_owned();
            }
            named = true;
        }
    }
    jobs
}

/// The key a line of two spaces, a job id and a colon declares, and nothing else.
fn job_key(line: &str) -> Option<&str> {
    let key = line.strip_prefix("  ")?.strip_suffix(':')?;
    let id = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
    id.then_some(key)
}

/// `text` without one pair of surrounding single or double quotes.
fn unquote(text: &str) -> &str {
    ['"', '\'']
        .into_iter()
        .find_map(|quote| text.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(text)
}

/// Whether any of `jobs` reports `check`.
pub fn reports(jobs: &[Job], check: &str) -> bool {
    jobs.iter().any(|job| job.name == check)
}

/// The pull request gate as the default branch holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    /// The default branch.
    pub branch: String,
    /// The jobs its workflow declares, or `None` when the branch holds no workflow.
    pub jobs: Option<Vec<Job>>,
}

impl Gate {
    /// Whether the workflow on the branch reports `check`.
    pub fn reports(&self, check: &str) -> bool {
        self.jobs
            .as_deref()
            .is_some_and(|jobs| reports(jobs, check))
    }

    /// What the branch holds in place of a job reporting [`REQUIRED_CHECK`], as one clause:
    /// no workflow, or the jobs it declares and the names they report.
    pub fn missing(&self) -> String {
        let branch = &self.branch;
        let file = crate::init::WORKFLOW;
        let Some(jobs) = &self.jobs else {
            return format!("{branch} has no {file}");
        };
        let found = if jobs.is_empty() {
            "none".to_owned()
        } else {
            jobs.iter()
                .map(|job| {
                    if job.name == job.key {
                        job.key.clone()
                    } else {
                        format!("{}, reported as \"{}\"", job.key, job.name)
                    }
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        format!("{branch}'s {file} names no job \"{REQUIRED_CHECK}\" (jobs: {found})")
    }
}

/// Where the contents endpoint serves the gate's workflow as `branch` holds it.
pub fn workflow_path(repository: &Repository, branch: &str) -> String {
    format!(
        "repos/{}/contents/{}?ref={}",
        repository.slug(),
        crate::init::WORKFLOW,
        branch_query(branch)
    )
}

/// A branch name as a query value: what [`branch_segment`] escapes, and the two characters git
/// allows in a branch name that a query would read as something else.
pub fn branch_query(branch: &str) -> String {
    branch_segment(branch)
        .replace('&', "%26")
        .replace('+', "%2B")
}

/// The repository's default branch, read with `GET repos/<owner>/<repo>`: the read
/// `doctor --platform` and `init --github` both start from.
///
/// # Errors
///
/// [`Unapplied`] naming the call when gh refused it or its answer names no default branch.
pub fn default_branch(
    gh: &dyn Gh,
    repository: &Repository,
) -> std::result::Result<String, Unapplied> {
    let path = format!("repos/{}", repository.slug());
    let (about, ran) = answer(gh, &path)?;
    match about.get("default_branch").and_then(Value::as_str) {
        Some(branch) if !branch.is_empty() => Ok(branch.to_owned()),
        _ => Err(Unapplied::Unreadable {
            call: call_line("GET", &path),
            problem: "it names no default_branch".to_owned(),
            ran,
        }),
    }
}

/// The gate's workflow as `branch` holds it, read raw: its text, or `None` when the branch
/// holds no such file (gh's first line carries 404, or the answer is empty).
///
/// # Errors
///
/// [`Unapplied::Refused`] with gh's words when the read failed for any other reason.
pub fn workflow_on(
    gh: &dyn Gh,
    repository: &Repository,
    branch: &str,
) -> std::result::Result<Option<String>, Unapplied> {
    let path = workflow_path(repository, branch);
    let ran = gh.raw(&path);
    if ran.code != 0 {
        if ran.first_line().is_some_and(|line| line.contains("404")) {
            return Ok(None);
        }
        return Err(Unapplied::Refused {
            call: raw_line(&path),
            ran,
        });
    }
    if ran.stdout.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(ran.stdout))
}

/// The pull request gate on the default branch, with two read-only calls: the branch, then
/// the workflow there.
///
/// # Errors
///
/// [`Unapplied`] naming the call that could not be made or read.
pub fn read_gate(gh: &dyn Gh, repository: &Repository) -> std::result::Result<Gate, Unapplied> {
    let branch = default_branch(gh, repository)?;
    let jobs = workflow_on(gh, repository, &branch)?.map(|text| gate_jobs(&text));
    Ok(Gate { branch, jobs })
}

// ------------------------------------------------------------------------ the read-back

/// The four questions `doctor --platform` asks, in the order it prints the answers: the three
/// rules in force on the default branch, then whether the gate there reports the check they
/// require.
pub const READ_BACK: [&str; 4] = ["required check", "force push", "deletion", "gate job"];

/// The one line the platform table carries when nothing could be read.
pub const UNAVAILABLE: &str = "platform";

/// The platform table when nothing could be read: one line saying why. Never a guess.
pub fn unavailable(reason: impl Into<String>) -> Vec<LiveFinding> {
    vec![LiveFinding {
        check: UNAVAILABLE,
        verdict: Verdict::Unavailable,
        detail: reason.into(),
    }]
}

/// Ask GitHub which rules are in force on the default branch and whether the gate there
/// reports the check they require, with three read-only calls, and read the answers back as
/// the platform table.
///
/// The branch and the rules come first, and without either the table is one `unavailable`
/// line. The workflow is read last; a read of it that fails for a reason other than a missing
/// file makes that line alone `unavailable`.
pub fn read(gh: &dyn Gh, repository: &Repository) -> Vec<LiveFinding> {
    let branch = match default_branch(gh, repository) {
        Ok(branch) => branch,
        Err(unread) => return unavailable(reason(&unread)),
    };
    let path = format!(
        "repos/{}/rules/branches/{}",
        repository.slug(),
        branch_segment(&branch)
    );
    let rules = match read_only(gh, &path) {
        Ok(rules) => rules,
        Err(reason) => return unavailable(reason),
    };
    let mut findings = read_back(&branch, &rules);
    if findings.iter().any(|finding| finding.check == UNAVAILABLE) {
        return findings;
    }
    findings.push(match workflow_on(gh, repository, &branch) {
        Ok(text) => gate_line(&Gate {
            jobs: text.map(|text| gate_jobs(&text)),
            branch,
        }),
        Err(unread) => LiveFinding {
            check: READ_BACK[3],
            verdict: Verdict::Unavailable,
            detail: reason(&unread),
        },
    });
    findings
}

/// The gate as the default branch holds it, read back as the table's `gate job` line.
pub fn gate_line(gate: &Gate) -> LiveFinding {
    if gate.reports(REQUIRED_CHECK) {
        let file = crate::init::WORKFLOW
            .rsplit('/')
            .next()
            .unwrap_or(crate::init::WORKFLOW);
        finding(
            READ_BACK[3],
            true,
            format!("{}'s {file} reports {REQUIRED_CHECK}", gate.branch),
        )
    } else {
        finding(READ_BACK[3], false, gate.missing())
    }
}

/// One read-only call: the answer, or gh's first line when the call failed.
fn read_only(gh: &dyn Gh, path: &str) -> std::result::Result<Value, String> {
    let ran = gh.api("GET", path, None);
    if ran.code != 0 {
        return Err(ran
            .first_line()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{} exited {}", call_line("GET", path), ran.code)));
    }
    serde_json::from_str(&ran.stdout).map_err(|error| {
        format!(
            "{} answered what is not JSON: {}",
            call_line("GET", path),
            one_line(&error.to_string())
        )
    })
}

/// A branch name as a path segment: the two characters git allows in a branch name that a
/// url would read as something else are escaped, and the rest is left as it is.
pub fn branch_segment(branch: &str) -> String {
    branch.replace('%', "%25").replace('#', "%23")
}

/// The rules in force on `branch`, as GitHub's `rules/branches` endpoint lists them, read
/// back as the first three answers [`READ_BACK`] names.
pub fn read_back(branch: &str, rules: &Value) -> Vec<LiveFinding> {
    let Some(rules) = rules.as_array() else {
        return unavailable(format!("the rules in force on {branch} are not a list"));
    };
    let of_type = |kind: &str| -> Vec<&Value> {
        rules
            .iter()
            .filter(|rule| rule.get("type").and_then(Value::as_str) == Some(kind))
            .collect()
    };

    let checks = of_type("required_status_checks");
    let required = match checks
        .iter()
        .find(|rule| contexts(rule).contains(&REQUIRED_CHECK))
    {
        Some(rule) => finding(
            READ_BACK[0],
            true,
            format!("{branch} requires {REQUIRED_CHECK}{}", from(rule)),
        ),
        None => {
            let found: Vec<&str> = checks.iter().flat_map(|rule| contexts(rule)).collect();
            let found = if found.is_empty() {
                "none".to_owned()
            } else {
                found.join(", ")
            };
            finding(
                READ_BACK[0],
                false,
                format!("{branch} requires {found} instead of {REQUIRED_CHECK}"),
            )
        }
    };

    let force = match of_type("non_fast_forward").first() {
        Some(rule) => finding(
            READ_BACK[1],
            true,
            format!("{branch} refuses force pushes{}", from(rule)),
        ),
        None => finding(
            READ_BACK[1],
            false,
            format!("{branch} takes force pushes: no non_fast_forward rule is in force"),
        ),
    };

    let deletion = match of_type("deletion").first() {
        Some(rule) => finding(
            READ_BACK[2],
            true,
            format!("{branch} refuses deletion{}", from(rule)),
        ),
        None => finding(
            READ_BACK[2],
            false,
            format!("{branch} can be deleted: no deletion rule is in force"),
        ),
    };

    vec![required, force, deletion]
}

/// The contexts one `required_status_checks` rule requires.
fn contexts(rule: &Value) -> Vec<&str> {
    rule.get("parameters")
        .and_then(|parameters| parameters.get("required_status_checks"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|check| check.get("context").and_then(Value::as_str))
        .collect()
}

/// `, from ruleset <id>`, naming where a rule in force came from.
fn from(rule: &Value) -> String {
    match rule.get("ruleset_id").and_then(Value::as_u64) {
        Some(id) => format!(", from ruleset {id}"),
        None => ", from a rule naming no ruleset".to_owned(),
    }
}

fn finding(check: &'static str, ok: bool, detail: String) -> LiveFinding {
    LiveFinding {
        check,
        verdict: if ok { Verdict::Ok } else { Verdict::Fail },
        detail,
    }
}

/// A `gh` that answers from canned JSON per method and path and remembers every call in
/// order. It stands in for GitHub alone: the body, the comparison, the region and the
/// read-back it feeds are the real functions.
#[cfg(test)]
pub(crate) mod recorded {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use super::{Gh, Ran};

    /// One call as the double saw it: method, path, and the body on stdin when there was one.
    /// A read of raw file contents is recorded, and answered, under the method `RAW`.
    pub type Call = (String, String, Option<String>);

    /// The method a raw read is recorded under: a read, never a write.
    pub const RAW_READ: &str = "RAW";

    #[derive(Default)]
    pub struct Recorded {
        answers: BTreeMap<(String, String), Ran>,
        calls: RefCell<Vec<Call>>,
    }

    impl Recorded {
        /// A successful answer carrying `json` on stdout.
        pub fn answer(mut self, method: &str, path: &str, json: &str) -> Recorded {
            self.answers.insert(
                (method.to_owned(), path.to_owned()),
                Ran {
                    code: 0,
                    stdout: json.to_owned(),
                    stderr: String::new(),
                },
            );
            self
        }

        /// A successful raw read carrying the file's `text` on stdout.
        pub fn raw_answer(self, path: &str, text: &str) -> Recorded {
            self.answer(RAW_READ, path, text)
        }

        /// An answer exactly as gh gave it, failure included.
        pub fn ran(mut self, method: &str, path: &str, ran: Ran) -> Recorded {
            self.answers
                .insert((method.to_owned(), path.to_owned()), ran);
            self
        }

        pub fn calls(&self) -> Vec<Call> {
            self.calls.borrow().clone()
        }

        /// The calls that would change something on the platform.
        pub fn writes(&self) -> Vec<Call> {
            self.calls()
                .into_iter()
                .filter(|(method, _, _)| method != "GET" && method != RAW_READ)
                .collect()
        }

        fn call(&self, method: &str, path: &str, body: Option<&str>) -> Ran {
            self.calls.borrow_mut().push((
                method.to_owned(),
                path.to_owned(),
                body.map(str::to_owned),
            ));
            self.answers
                .get(&(method.to_owned(), path.to_owned()))
                .cloned()
                .unwrap_or_else(|| Ran {
                    code: 1,
                    stdout: String::new(),
                    stderr: format!("the double holds no answer for {method} {path}\n"),
                })
        }
    }

    impl Gh for Recorded {
        fn api(&self, method: &str, path: &str, body: Option<&str>) -> Ran {
            self.call(method, path, body)
        }

        fn raw(&self, path: &str) -> Ran {
            self.call(RAW_READ, path, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::recorded::Recorded;
    use super::*;
    use crate::doctor::live::Verdict;

    fn repo() -> Repository {
        Repository {
            owner: "example-owner".to_owned(),
            name: "example-repo".to_owned(),
        }
    }

    /// The gate as the stem writes it, on the default branch: one job reporting `garden`.
    fn gated() -> Gate {
        Gate {
            branch: "main".to_owned(),
            jobs: Some(gate_jobs(&crate::init::workflow(false))),
        }
    }

    const LIST: &str = "repos/example-owner/example-repo/rulesets";
    const ONE: &str = "repos/example-owner/example-repo/rulesets/41";

    // ------------------------------------------------------------------ the ruleset

    #[test]
    fn the_desired_body_is_byte_stable_json_with_three_rules_in_order() {
        let body = ruleset_body();
        assert_eq!(
            body,
            concat!(
                r#"{"bypass_actors":[],"#,
                r#""conditions":{"ref_name":{"exclude":[],"include":["~DEFAULT_BRANCH"]}},"#,
                r#""enforcement":"active","#,
                r#""name":"plotplot: the default branch","#,
                r#""rules":[{"type":"deletion"},{"type":"non_fast_forward"},"#,
                r#"{"parameters":{"required_status_checks":[{"context":"garden"}],"#,
                r#""strict_required_status_checks_policy":false},"type":"required_status_checks"}],"#,
                r#""target":"branch"}"#,
            )
        );
        assert_eq!(body, ruleset_body(), "the same bytes every time");
        assert!(!body.contains("required_linear_history"), "{body}");
        assert!(!body.contains("linear"), "{body}");
    }

    #[test]
    fn the_required_check_is_the_one_constant_the_body_names() {
        let desired = desired_ruleset();
        assert_eq!(
            desired["rules"][2]["parameters"]["required_status_checks"][0]["context"],
            REQUIRED_CHECK
        );
        assert_eq!(REQUIRED_CHECK, "garden");
    }

    #[test]
    fn a_ruleset_github_handed_back_with_its_own_fields_is_the_same_ruleset() {
        // What GitHub answers for a ruleset it holds: its own ids, links and defaults added,
        // the lists and keys in its own order.
        let held: Value = serde_json::from_str(
            r#"{"id":41,"name":"plotplot: the default branch","target":"branch",
                "source_type":"Repository","source":"example-owner/example-repo",
                "enforcement":"active","bypass_actors":[],
                "conditions":{"ref_name":{"exclude":[],"include":["~DEFAULT_BRANCH"]}},
                "rules":[
                  {"type":"non_fast_forward"},
                  {"type":"required_status_checks","parameters":{
                     "strict_required_status_checks_policy":false,
                     "do_not_enforce_on_create":false,
                     "required_status_checks":[{"context":"garden","integration_id":15368}]}},
                  {"type":"deletion"}],
                "node_id":"RRS_x","current_user_can_bypass":"never"}"#,
        )
        .expect("a ruleset as GitHub shapes one");
        assert!(same_ruleset(&held));
    }

    #[test]
    fn a_ruleset_that_says_something_else_is_a_different_ruleset() {
        let desired = desired_ruleset();
        let mut changes: Vec<(&str, Value)> = Vec::new();

        let mut evaluate = desired.clone();
        evaluate["enforcement"] = Value::from("evaluate");
        changes.push(("enforcement", evaluate));

        let mut linear = desired.clone();
        if let Some(rules) = linear["rules"].as_array_mut() {
            rules.push(json!({"type": "required_linear_history"}));
        }
        changes.push(("a rule added", linear));

        let mut fewer = desired.clone();
        if let Some(rules) = fewer["rules"].as_array_mut() {
            rules.remove(0);
        }
        changes.push(("a rule taken away", fewer));

        let mut renamed = desired.clone();
        renamed["rules"][2]["parameters"]["required_status_checks"][0]["context"] =
            Value::from("plotplot check --strict");
        changes.push(("another context", renamed));

        let mut strict = desired.clone();
        strict["rules"][2]["parameters"]["strict_required_status_checks_policy"] =
            Value::from(true);
        changes.push(("strict", strict));

        let mut every = desired.clone();
        every["conditions"]["ref_name"]["include"] = json!(["~ALL"]);
        changes.push(("every branch", every));

        let mut tag = desired.clone();
        tag["target"] = Value::from("tag");
        changes.push(("target", tag));

        let mut bare = desired.clone();
        if let Some(object) = bare.as_object_mut() {
            object.remove("conditions");
        }
        changes.push(("no conditions", bare));

        for (what, held) in changes {
            assert!(!same_ruleset(&held), "{what}: {held}");
        }
        assert!(same_ruleset(&desired));
    }

    #[test]
    fn an_absent_ruleset_is_posted_with_the_desired_body() {
        let gh = Recorded::default().answer(
            "GET",
            LIST,
            r#"[{"id":7,"name":"somebody else's","target":"branch"}]"#,
        );
        let gh = gh.answer("POST", LIST, r#"{"id":41}"#);

        let applied = apply_ruleset(&gh, &repo(), &gated()).expect("applied");
        assert_eq!(
            applied.as_deref(),
            Some("ruleset \"plotplot: the default branch\" applied on example-owner/example-repo")
        );
        assert_eq!(
            gh.calls(),
            [
                ("GET".to_owned(), LIST.to_owned(), None),
                ("POST".to_owned(), LIST.to_owned(), Some(ruleset_body())),
            ]
        );
    }

    #[test]
    fn a_present_and_equal_ruleset_makes_no_writing_call() {
        let gh = Recorded::default()
            .answer(
                "GET",
                LIST,
                r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
            )
            .answer("GET", ONE, &ruleset_body());

        assert_eq!(apply_ruleset(&gh, &repo(), &gated()).expect("read"), None);
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
        assert_eq!(gh.calls().len(), 2, "{:?}", gh.calls());
    }

    #[test]
    fn a_present_and_different_ruleset_is_put_to_its_id() {
        let mut held = desired_ruleset();
        held["enforcement"] = Value::from("disabled");
        let gh = Recorded::default()
            .answer(
                "GET",
                LIST,
                r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
            )
            .answer("GET", ONE, &held.to_string())
            .answer("PUT", ONE, r#"{"id":41}"#);

        let applied = apply_ruleset(&gh, &repo(), &gated()).expect("applied");
        assert!(applied.is_some());
        assert_eq!(
            gh.writes(),
            [("PUT".to_owned(), ONE.to_owned(), Some(ruleset_body()))]
        );
    }

    #[test]
    fn a_refused_write_carries_ghs_own_words_and_the_call() {
        let refusal = Ran {
            code: 1,
            stdout: r#"{"message":"Validation Failed","status":"422"}"#.to_owned(),
            stderr: "gh: Validation Failed (HTTP 422)\n".to_owned(),
        };
        let gh = Recorded::default()
            .answer("GET", LIST, "[]")
            .ran("POST", LIST, refusal.clone());

        let unapplied = apply_ruleset(&gh, &repo(), &gated()).expect_err("gh refused");
        assert_eq!(
            unapplied,
            Unapplied::Refused {
                call: "gh api -X POST repos/example-owner/example-repo/rulesets".to_owned(),
                ran: refusal,
            }
        );
        assert_eq!(
            unapplied_text(&unapplied),
            "gh api -X POST repos/example-owner/example-repo/rulesets could not be applied:\n\
             gh: Validation Failed (HTTP 422)\n\
             {\"message\":\"Validation Failed\",\"status\":\"422\"}\n"
        );
    }

    #[test]
    fn a_list_that_is_not_a_list_is_unreadable_and_nothing_is_written() {
        let gh = Recorded::default().answer("GET", LIST, r#"{"message":"Moved"}"#);
        let unapplied = apply_ruleset(&gh, &repo(), &gated()).expect_err("not a list");
        assert!(
            matches!(&unapplied, Unapplied::Unreadable { call, .. } if call == "gh api repos/example-owner/example-repo/rulesets"),
            "{unapplied:?}"
        );
        assert!(gh.writes().is_empty());
        assert!(unapplied_text(&unapplied).contains(r#"{"message":"Moved"}"#));
    }

    #[test]
    fn gh_is_called_as_a_planter_would_type_it() {
        assert_eq!(call_args("GET", "repos/o/r", false), ["api", "repos/o/r"]);
        assert_eq!(
            call_args("POST", "repos/o/r/rulesets", true),
            ["api", "-X", "POST", "repos/o/r/rulesets", "--input", "-"]
        );
        assert_eq!(call_line("GET", "repos/o/r"), "gh api repos/o/r");
        assert_eq!(
            call_line("PUT", "repos/o/r/rulesets/41"),
            "gh api -X PUT repos/o/r/rulesets/41"
        );
    }

    // ------------------------------------------------------------------ the origin

    #[test]
    fn the_owner_is_read_from_https_and_ssh_origins_with_and_without_git() {
        for origin in [
            "https://github.com/example-owner/example-repo.git",
            "https://github.com/example-owner/example-repo",
            "https://github.com/example-owner/example-repo/",
            "git@github.com:example-owner/example-repo.git",
            "git@github.com:example-owner/example-repo",
            "ssh://git@github.com/example-owner/example-repo.git",
        ] {
            assert_eq!(github_repository(origin), Some(repo()), "{origin}");
        }
        assert_eq!(repo().slug(), "example-owner/example-repo");
    }

    #[test]
    fn an_origin_off_github_or_out_of_shape_names_no_repository() {
        for origin in [
            "https://example.invalid/jahala/fixture.git",
            "https://gitlab.com/example-owner/example-repo.git",
            "git@gitlab.com:example-owner/example-repo.git",
            "https://github.com/example-owner",
            "https://github.com/example-owner/example-repo/tree/main",
            "https://github.com/example-owner/../x",
            "https://github.com/example owner/example-repo",
            "https://github.com/example-owner/example-repo?x=1",
            "/work/a/local/clone",
            "",
        ] {
            assert_eq!(github_repository(origin), None, "{origin}");
        }
    }

    #[test]
    fn reach_asks_for_gh_first_and_then_for_a_github_origin() {
        let github = Some("https://github.com/example-owner/example-repo.git");
        assert_eq!(reach(None::<()>, github), Err(Unreachable::NoGh));
        assert_eq!(reach(None::<()>, None), Err(Unreachable::NoGh));
        assert_eq!(reach(Some(()), None), Err(Unreachable::NoOrigin));
        assert_eq!(
            reach(Some(()), Some("https://example.invalid/o/r.git")),
            Err(Unreachable::NotGithub {
                origin: "https://example.invalid/o/r.git".to_owned()
            })
        );
        assert_eq!(reach(Some(()), github), Ok(((), repo())));

        for why in [
            Unreachable::NoGh,
            Unreachable::NoOrigin,
            Unreachable::NotGithub {
                origin: "https://example.invalid/o/r.git".to_owned(),
            },
        ] {
            let line = why.to_string();
            assert!(!line.contains('\n'), "{line}");
            assert!(!line.contains("  "), "{line}");
        }
        assert!(Unreachable::NoGh.to_string().contains("gh"));
        assert!(
            Unreachable::NotGithub {
                origin: "https://example.invalid/o/r.git".to_owned()
            }
            .to_string()
            .contains("github.com")
        );
    }

    // ------------------------------------------------------------------ CODEOWNERS

    #[test]
    fn the_region_names_the_owner_for_the_twelve_guarded_paths_in_order() {
        let region = codeowners_region("example-owner");
        let lines: Vec<&str> = region.lines().collect();
        assert_eq!(
            lines,
            [
                "# plotplot:begin",
                "/.githooks/ @example-owner",
                "/garden.lock @example-owner",
                "/garden.json @example-owner",
                "/AGENTS.md @example-owner",
                "/CLAUDE.md @example-owner",
                "/weeder.toml @example-owner",
                "/.claude/settings.json @example-owner",
                "/.gemini/settings.json @example-owner",
                "/.codex/hooks.json @example-owner",
                "/.github/workflows/plotplot-check.yml @example-owner",
                "/.github/CODEOWNERS @example-owner",
                "/docs/tend2/ @example-owner",
                "# plotplot:end",
            ]
        );
        assert!(!region.ends_with('\n'), "the markers bound the region");
    }

    #[test]
    fn each_guarded_path_is_the_one_its_own_module_spells() {
        use crate::{init, install, layout};
        assert_eq!(GUARDED[0], format!("/{}/", layout::GITHOOKS_DIR));
        assert_eq!(GUARDED[1], format!("/{}", layout::GARDEN_LOCK));
        assert_eq!(GUARDED[2], format!("/{}", layout::GARDEN_JSON));
        assert_eq!(GUARDED[3], format!("/{}", layout::AGENTS_MD));
        assert_eq!(GUARDED[6], format!("/{}", install::CLAUDE_SETTINGS));
        assert_eq!(GUARDED[7], format!("/{}", install::GEMINI_SETTINGS));
        assert_eq!(GUARDED[8], format!("/{}", install::CODEX_HOOKS));
        assert_eq!(GUARDED[9], format!("/{}", init::WORKFLOW));
        assert_eq!(GUARDED[10], format!("/{}", layout::CODEOWNERS));
    }

    #[test]
    fn a_file_with_no_region_gets_it_after_one_blank_line() {
        let region = codeowners_region("o");
        assert_eq!(
            codeowners_in("* @someone\n", &region).expect("planted"),
            format!("* @someone\n\n{region}\n")
        );
        assert_eq!(
            codeowners_in("", &region).expect("planted"),
            format!("{region}\n")
        );
    }

    #[test]
    fn the_region_is_idempotent_and_foreign_lines_stay_byte_for_byte() {
        let above = "# Owners this repository keeps for itself.\n\n";
        let below = "\n*.md @someone\n";
        let old = "# plotplot:begin\n/garden.lock @previous-owner\n# plotplot:end";
        let start = format!("{above}{old}{below}");

        let region = codeowners_region("example-owner");
        let once = codeowners_in(&start, &region).expect("the first planting");
        assert_eq!(once, format!("{above}{region}{below}"));
        let twice = codeowners_in(&once, &region).expect("the second planting");
        assert_eq!(once, twice);
    }

    #[test]
    fn markers_that_do_not_make_one_region_are_refused_naming_the_file() {
        let region = codeowners_region("o");
        for broken in [
            format!("{BEGIN}\n/x @o\n"),
            format!("/x @o\n{END}\n"),
            format!("{END}\n{BEGIN}\n"),
            format!("{BEGIN}\n{END}\n{BEGIN}\n{END}\n"),
        ] {
            match codeowners_in(&broken, &region) {
                Err(Error::Codeowners { problem }) => {
                    assert!(problem.starts_with(crate::layout::CODEOWNERS), "{problem}");
                    assert!(!problem.contains('\n'), "{problem}");
                }
                other => panic!("{broken:?} was not refused: {other:?}"),
            }
        }
    }

    // ------------------------------------------------------------------ the read-back

    const REPOSITORY: &str = "repos/example-owner/example-repo";
    const RULES: &str = "repos/example-owner/example-repo/rules/branches/main";

    /// The rules GitHub reports in force on a branch the desired ruleset protects, as its
    /// `rules/branches` endpoint shapes them.
    const IN_FORCE: &str = r#"[
        {"type":"deletion","ruleset_source_type":"Repository","ruleset_source":"example-owner/example-repo","ruleset_id":41},
        {"type":"non_fast_forward","ruleset_source_type":"Repository","ruleset_source":"example-owner/example-repo","ruleset_id":41},
        {"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"garden"}],"strict_required_status_checks_policy":false},"ruleset_source_type":"Repository","ruleset_source":"example-owner/example-repo","ruleset_id":41}
    ]"#;

    fn verdicts(findings: &[LiveFinding]) -> Vec<(&str, Verdict)> {
        findings
            .iter()
            .map(|finding| (finding.check, finding.verdict))
            .collect()
    }

    fn rules(json: &str) -> Value {
        serde_json::from_str(json).expect("rules as GitHub shapes them")
    }

    #[test]
    fn the_rules_in_force_read_back_as_three_ok_lines_naming_branch_and_ruleset() {
        let findings = read_back("main", &rules(IN_FORCE));
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Ok),
                ("force push", Verdict::Ok),
                ("deletion", Verdict::Ok),
            ]
        );
        for finding in &findings {
            assert!(finding.detail.contains("main"), "{finding:?}");
            assert!(finding.detail.contains("ruleset 41"), "{finding:?}");
            assert!(!finding.detail.contains("  "), "{finding:?}");
        }
        assert!(findings[0].detail.contains("garden"), "{findings:?}");
    }

    #[test]
    fn no_rules_in_force_fails_all_three_and_says_what_is_required_instead() {
        let findings = read_back("main", &rules("[]"));
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Fail),
                ("force push", Verdict::Fail),
                ("deletion", Verdict::Fail),
            ]
        );
        assert!(findings[0].detail.contains("none"), "{findings:?}");
    }

    #[test]
    fn another_required_context_fails_the_required_check_and_names_it() {
        let other = IN_FORCE.replace(
            r#"{"context":"garden"}"#,
            r#"{"context":"ci"},{"context":"lint"}"#,
        );
        let findings = read_back("main", &rules(&other));
        assert_eq!(findings[0].verdict, Verdict::Fail, "{findings:?}");
        assert!(findings[0].detail.contains("ci, lint"), "{findings:?}");
        assert_eq!(findings[1].verdict, Verdict::Ok);
        assert_eq!(findings[2].verdict, Verdict::Ok);
    }

    #[test]
    fn a_missing_force_push_rule_fails_that_line_alone() {
        let mut held = rules(IN_FORCE);
        if let Some(list) = held.as_array_mut() {
            list.retain(|rule| rule["type"] != "non_fast_forward");
        }
        let findings = read_back("main", &held);
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Ok),
                ("force push", Verdict::Fail),
                ("deletion", Verdict::Ok),
            ]
        );
    }

    #[test]
    fn a_missing_deletion_rule_fails_that_line_alone() {
        let mut held = rules(IN_FORCE);
        if let Some(list) = held.as_array_mut() {
            list.retain(|rule| rule["type"] != "deletion");
        }
        let findings = read_back("main", &held);
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Ok),
                ("force push", Verdict::Ok),
                ("deletion", Verdict::Fail),
            ]
        );
    }

    #[test]
    fn the_read_back_asks_three_read_only_questions_and_no_more() {
        let gh = Recorded::default()
            .answer(
                "GET",
                REPOSITORY,
                r#"{"default_branch":"main","name":"example-repo"}"#,
            )
            .answer("GET", RULES, IN_FORCE)
            .raw_answer(WORKFLOW_ON_MAIN, &crate::init::workflow(false));
        let findings = read(&gh, &repo());
        assert_eq!(findings.len(), READ_BACK.len(), "{findings:?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.verdict == Verdict::Ok),
            "{findings:?}"
        );
        assert_eq!(
            gh.calls(),
            [
                ("GET".to_owned(), REPOSITORY.to_owned(), None),
                ("GET".to_owned(), RULES.to_owned(), None),
                ("RAW".to_owned(), WORKFLOW_ON_MAIN.to_owned(), None),
            ]
        );
    }

    #[test]
    fn a_failing_call_is_one_unavailable_line_carrying_ghs_first_line() {
        let gh = Recorded::default().ran(
            "GET",
            REPOSITORY,
            Ran {
                code: 1,
                stdout: r#"{"message":"Not Found"}"#.to_owned(),
                stderr: "gh: Not Found (HTTP 404)\n".to_owned(),
            },
        );
        let findings = read(&gh, &repo());
        assert_eq!(verdicts(&findings), [("platform", Verdict::Unavailable)]);
        assert_eq!(findings[0].detail, "gh: Not Found (HTTP 404)");
    }

    #[test]
    fn an_answer_with_no_default_branch_is_unavailable_and_asks_nothing_more() {
        let gh = Recorded::default().answer("GET", REPOSITORY, r#"{"name":"example-repo"}"#);
        let findings = read(&gh, &repo());
        assert_eq!(verdicts(&findings), [("platform", Verdict::Unavailable)]);
        assert!(
            findings[0].detail.contains("default_branch"),
            "{findings:?}"
        );
        assert_eq!(gh.calls().len(), 1);
    }

    #[test]
    fn a_branch_name_is_escaped_where_a_path_would_read_it_otherwise() {
        assert_eq!(branch_segment("main"), "main");
        assert_eq!(branch_segment("release/2026"), "release/2026");
        assert_eq!(branch_segment("a#b%c"), "a%23b%25c");
    }

    // ------------------------------------------------------------------ the gate

    /// The stem's own workflow with its job named as master's was before the rename.
    fn renamed(workflow: &str) -> String {
        let text = workflow.replace("    name: garden\n", "    name: plotplot check --strict\n");
        assert_ne!(text, workflow, "the name line was there to change");
        text
    }

    fn job(key: &str, name: &str) -> Job {
        Job {
            key: key.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn the_workflow_init_writes_declares_one_job_garden_reporting_garden() {
        for own in [true, false] {
            let jobs = gate_jobs(&crate::init::workflow(own));
            assert_eq!(jobs, [job("garden", "garden")]);
            assert!(reports(&jobs, REQUIRED_CHECK));
        }
    }

    #[test]
    fn a_job_whose_name_line_says_something_else_reports_that() {
        let jobs = gate_jobs(&renamed(&crate::init::workflow(true)));
        assert_eq!(jobs, [job("garden", "plotplot check --strict")]);
        assert!(!reports(&jobs, REQUIRED_CHECK));
        assert!(reports(&jobs, "plotplot check --strict"));
    }

    #[test]
    fn a_job_with_no_name_line_reports_its_key() {
        let workflow = "\
name: ci
on: push

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - name: run the linter
        run: make lint
  garden:
    runs-on: ubuntu-latest
    name: plotplot check --strict
";
        assert_eq!(
            gate_jobs(workflow),
            [
                job("lint", "lint"),
                job("garden", "plotplot check --strict")
            ]
        );
    }

    #[test]
    fn a_file_with_no_jobs_section_declares_no_job() {
        for workflow in [
            "",
            "name: ci\non: push\n",
            // Indented, `jobs:` is some other mapping's key.
            "name: ci\nenv:\n  jobs:\n  garden:\n    name: garden\n",
            // `jobs:` with anything after it on the line is not the line this reads.
            "jobs: {garden: {name: garden}}\n",
        ] {
            assert_eq!(gate_jobs(workflow), [], "{workflow:?}");
        }
    }

    #[test]
    fn quotes_around_a_name_are_removed() {
        let workflow = "jobs:\n  one:\n    name: \"garden\"\n  two:\n    name: 'plotplot check --strict'\n  three:\n    name:   spaced out  \n";
        assert_eq!(
            gate_jobs(workflow),
            [
                job("one", "garden"),
                job("two", "plotplot check --strict"),
                job("three", "spaced out"),
            ]
        );
    }

    #[test]
    fn a_steps_name_line_is_never_a_job_or_a_jobs_name() {
        let workflow = "\
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - name: garden
        run: plotplot check --strict
      - name: plotplot check --strict
";
        let jobs = gate_jobs(workflow);
        assert_eq!(jobs, [job("check", "check")]);
        assert!(!reports(&jobs, REQUIRED_CHECK));
    }

    #[test]
    fn a_top_level_line_ends_the_jobs_section_and_comments_do_not() {
        let workflow = "\
jobs:
  # A comment at the jobs' own indent.
# A comment at the top.

  garden:
    name: garden
env:
  other:
    name: not a job
";
        assert_eq!(gate_jobs(workflow), [job("garden", "garden")]);
    }

    #[test]
    fn a_key_that_is_not_a_job_id_starts_no_job() {
        let workflow =
            "jobs:\n  a job:\n    name: garden\n  garden: # trailing\n    name: garden\n";
        assert_eq!(gate_jobs(workflow), []);
    }

    const WORKFLOW_ON_MAIN: &str =
        "repos/example-owner/example-repo/contents/.github/workflows/plotplot-check.yml?ref=main";
    const ABOUT: &str = r#"{"default_branch":"main","name":"example-repo"}"#;

    fn not_found() -> Ran {
        Ran {
            code: 1,
            stdout: r#"{"message":"Not Found","status":"404"}"#.to_owned(),
            stderr: "gh: Not Found (HTTP 404)\n".to_owned(),
        }
    }

    #[test]
    fn the_file_is_read_raw_as_a_planter_would_type_it() {
        assert_eq!(
            raw_args("repos/o/r/contents/x.yml?ref=main"),
            [
                "api",
                "-H",
                "Accept: application/vnd.github.raw+json",
                "repos/o/r/contents/x.yml?ref=main"
            ]
        );
        assert_eq!(
            raw_line("repos/o/r/contents/x.yml?ref=main"),
            "gh api -H \"Accept: application/vnd.github.raw+json\" repos/o/r/contents/x.yml?ref=main"
        );
        assert_eq!(workflow_path(&repo(), "main"), WORKFLOW_ON_MAIN);
    }

    #[test]
    fn a_branch_in_the_query_is_escaped_where_a_query_would_read_it_otherwise() {
        assert_eq!(branch_query("main"), "main");
        assert_eq!(branch_query("release/2026"), "release/2026");
        assert_eq!(branch_query("a#b%c&d+e"), "a%23b%25c%26d%2Be");
    }

    #[test]
    fn the_gate_is_read_from_the_default_branch_with_two_read_only_calls() {
        let gh = Recorded::default()
            .answer("GET", REPOSITORY, ABOUT)
            .raw_answer(WORKFLOW_ON_MAIN, &crate::init::workflow(false));
        let gate = read_gate(&gh, &repo()).expect("read");
        assert_eq!(
            gate,
            Gate {
                branch: "main".to_owned(),
                jobs: Some(vec![job("garden", "garden")]),
            }
        );
        assert!(gate.reports(REQUIRED_CHECK));
        assert_eq!(
            gh.calls(),
            [
                ("GET".to_owned(), REPOSITORY.to_owned(), None),
                ("RAW".to_owned(), WORKFLOW_ON_MAIN.to_owned(), None),
            ]
        );
    }

    #[test]
    fn a_404_or_an_empty_answer_is_a_branch_with_no_workflow() {
        for ran in [
            not_found(),
            Ran::default(),
            Ran {
                code: 0,
                stdout: "\n".to_owned(),
                stderr: String::new(),
            },
        ] {
            let gh = Recorded::default().answer("GET", REPOSITORY, ABOUT).ran(
                "RAW",
                WORKFLOW_ON_MAIN,
                ran.clone(),
            );
            let gate = read_gate(&gh, &repo()).expect("read");
            assert_eq!(gate.jobs, None, "{ran:?}");
            assert!(!gate.reports(REQUIRED_CHECK));
        }
    }

    #[test]
    fn a_failed_read_of_the_file_that_is_not_a_404_is_refused_with_ghs_words() {
        let forbidden = Ran {
            code: 1,
            stdout: String::new(),
            stderr: "gh: Resource not accessible by integration (HTTP 403)\n".to_owned(),
        };
        let gh = Recorded::default().answer("GET", REPOSITORY, ABOUT).ran(
            "RAW",
            WORKFLOW_ON_MAIN,
            forbidden.clone(),
        );
        assert_eq!(
            read_gate(&gh, &repo()),
            Err(Unapplied::Refused {
                call: raw_line(WORKFLOW_ON_MAIN),
                ran: forbidden,
            })
        );
    }

    #[test]
    fn a_repository_answer_with_no_default_branch_is_unreadable_and_reads_no_file() {
        let gh = Recorded::default().answer("GET", REPOSITORY, r#"{"name":"example-repo"}"#);
        let unread = read_gate(&gh, &repo()).expect_err("no branch");
        assert!(
            matches!(&unread, Unapplied::Unreadable { problem, .. } if problem.contains("default_branch")),
            "{unread:?}"
        );
        assert_eq!(gh.calls().len(), 1, "{:?}", gh.calls());
    }

    fn gate_with(jobs: Option<Vec<Job>>) -> Gate {
        Gate {
            branch: "master".to_owned(),
            jobs,
        }
    }

    #[test]
    fn a_gate_reporting_the_old_name_is_refused_before_any_ruleset_is_written() {
        let gh = Recorded::default().answer("GET", LIST, "[]");
        let gate = gate_with(Some(vec![job("check", "plotplot check --strict")]));

        let unapplied = apply_ruleset(&gh, &repo(), &gate).expect_err("refused");
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
        assert_eq!(
            unapplied_text(&unapplied),
            "master's .github/workflows/plotplot-check.yml names no job \"garden\" \
             (jobs: check, reported as \"plotplot check --strict\"); land the workflow on master \
             first, then run plotplot init --github again\n"
        );
    }

    #[test]
    fn a_branch_with_no_workflow_is_refused_naming_the_missing_file() {
        let gh = Recorded::default().answer("GET", LIST, "[]");
        let unapplied = apply_ruleset(&gh, &repo(), &gate_with(None)).expect_err("refused");
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
        assert_eq!(
            unapplied_text(&unapplied),
            "master has no .github/workflows/plotplot-check.yml; land the workflow on master \
             first, then run plotplot init --github again\n"
        );
    }

    #[test]
    fn a_workflow_with_no_jobs_is_refused_saying_it_found_none() {
        let gh = Recorded::default().answer("GET", LIST, "[]");
        let unapplied =
            apply_ruleset(&gh, &repo(), &gate_with(Some(Vec::new()))).expect_err("refused");
        assert!(
            unapplied_text(&unapplied).starts_with(
                "master's .github/workflows/plotplot-check.yml names no job \"garden\" (jobs: none); "
            ),
            "{}",
            unapplied_text(&unapplied)
        );
    }

    #[test]
    fn a_ruleset_already_requiring_the_check_adds_the_deadlock_line() {
        let gh = Recorded::default()
            .answer(
                "GET",
                LIST,
                r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
            )
            .answer("GET", ONE, &ruleset_body());
        let gate = gate_with(Some(vec![job("check", "plotplot check --strict")]));

        let unapplied = apply_ruleset(&gh, &repo(), &gate).expect_err("refused");
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
        let text = unapplied_text(&unapplied);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "{text}");
        assert_eq!(
            lines[1],
            "ruleset 41 already requires \"garden\", so master takes no merge until that workflow lands"
        );
    }

    #[test]
    fn a_ruleset_of_the_stems_name_requiring_something_else_adds_no_deadlock_line() {
        let mut held = desired_ruleset();
        held["rules"][2]["parameters"]["required_status_checks"][0]["context"] =
            Value::from("plotplot check --strict");
        let gh = Recorded::default()
            .answer(
                "GET",
                LIST,
                r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
            )
            .answer("GET", ONE, &held.to_string());
        let unapplied = apply_ruleset(&gh, &repo(), &gate_with(None)).expect_err("refused");
        assert_eq!(unapplied_text(&unapplied).lines().count(), 1);
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
    }

    #[test]
    fn a_ruleset_requiring_the_check_that_is_not_enforced_adds_no_deadlock_line() {
        for enforcement in ["disabled", "evaluate"] {
            let mut held = desired_ruleset();
            held["enforcement"] = Value::from(enforcement);
            let gh = Recorded::default()
                .answer(
                    "GET",
                    LIST,
                    r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
                )
                .answer("GET", ONE, &held.to_string());
            let unapplied = apply_ruleset(&gh, &repo(), &gate_with(None)).expect_err("refused");
            assert_eq!(
                unapplied_text(&unapplied).lines().count(),
                1,
                "{enforcement}"
            );
        }
    }

    #[test]
    fn rulesets_that_cannot_be_read_are_said_so_on_the_refusals_second_line() {
        let gh = Recorded::default().ran("GET", LIST, not_found());
        let unapplied = apply_ruleset(&gh, &repo(), &gate_with(None)).expect_err("refused");
        let text = unapplied_text(&unapplied);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "{text}");
        assert_eq!(
            lines[1],
            "whether ruleset \"plotplot: the default branch\" already requires \"garden\" \
             could not be read: gh: Not Found (HTTP 404)"
        );
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
    }

    #[test]
    fn a_gate_reporting_garden_proceeds_to_the_post() {
        let gh = Recorded::default()
            .answer("GET", LIST, "[]")
            .answer("POST", LIST, r#"{"id":41}"#);
        let gate = gate_with(Some(vec![job("garden", "garden")]));
        assert!(
            apply_ruleset(&gh, &repo(), &gate)
                .expect("applied")
                .is_some()
        );
        assert_eq!(
            gh.writes(),
            [("POST".to_owned(), LIST.to_owned(), Some(ruleset_body()))]
        );
    }

    // ------------------------------------------------------------ the doctor's fourth line

    fn doctor_gh(workflow: Ran) -> Recorded {
        Recorded::default()
            .answer("GET", REPOSITORY, ABOUT)
            .answer("GET", RULES, IN_FORCE)
            .ran("RAW", WORKFLOW_ON_MAIN, workflow)
    }

    fn file(text: &str) -> Ran {
        Ran {
            code: 0,
            stdout: text.to_owned(),
            stderr: String::new(),
        }
    }

    #[test]
    fn the_gate_job_line_is_ok_when_the_default_branchs_workflow_reports_garden() {
        let gh = doctor_gh(file(&crate::init::workflow(false)));
        let findings = read(&gh, &repo());
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Ok),
                ("force push", Verdict::Ok),
                ("deletion", Verdict::Ok),
                ("gate job", Verdict::Ok),
            ]
        );
        assert_eq!(
            findings[3].detail,
            "main's plotplot-check.yml reports garden"
        );
        assert!(gh.writes().is_empty());
        assert_eq!(gh.calls().len(), 3, "{:?}", gh.calls());
    }

    #[test]
    fn the_gate_job_line_fails_naming_the_jobs_the_workflow_reports_instead() {
        let old = renamed(&crate::init::workflow(true)).replace("\n  garden:\n", "\n  check:\n");
        let findings = read(&doctor_gh(file(&old)), &repo());
        assert_eq!(findings.len(), 4, "{findings:?}");
        assert_eq!(findings[3].check, "gate job");
        assert_eq!(findings[3].verdict, Verdict::Fail);
        assert_eq!(
            findings[3].detail,
            "main's .github/workflows/plotplot-check.yml names no job \"garden\" \
             (jobs: check, reported as \"plotplot check --strict\")"
        );
        assert!(
            findings[..3]
                .iter()
                .all(|finding| finding.verdict == Verdict::Ok)
        );
    }

    #[test]
    fn the_gate_job_line_fails_when_the_default_branch_has_no_workflow() {
        let findings = read(&doctor_gh(not_found()), &repo());
        assert_eq!(findings[3].verdict, Verdict::Fail, "{findings:?}");
        assert_eq!(
            findings[3].detail,
            "main has no .github/workflows/plotplot-check.yml"
        );
    }

    #[test]
    fn a_failed_read_of_the_workflow_is_unavailable_for_that_line_alone() {
        let findings = read(
            &doctor_gh(Ran {
                code: 1,
                stdout: String::new(),
                stderr: "gh: Bad credentials (HTTP 401)\n".to_owned(),
            }),
            &repo(),
        );
        assert_eq!(
            verdicts(&findings),
            [
                ("required check", Verdict::Ok),
                ("force push", Verdict::Ok),
                ("deletion", Verdict::Ok),
                ("gate job", Verdict::Unavailable),
            ]
        );
        assert_eq!(findings[3].detail, "gh: Bad credentials (HTTP 401)");
    }
}
