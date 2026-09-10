//! `plotplot check`: every gate the planted manifests declare, and one SARIF log.
//!
//! The stem does not judge anything. It resolves each bed's declared check command to the
//! judge the lock pinned under `.plotplot/bin/`, runs it in the repository root, reads its
//! output as SARIF 2.1.0, and concatenates what it gets. The version that judged is the
//! version the lock names, so a gate is never reached through `PATH` (the plan, §4).
//!
//! Fail closed. A gate that is not there, that will not run, or that prints something the
//! SARIF schema refuses makes the whole run unable to judge (exit 3), never permission. A
//! run that carries a block-level result exits 2. Both at once is 3: not being able to judge
//! is the worse of the two answers, because a block is a decision and a gate that could not
//! run is a decision missing.
//!
//! Nothing here reads the current directory: the repository root is a parameter, and the
//! only paths under it come from [`crate::layout`].

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::bed::Bed;
use crate::cli::CheckArgs;
use crate::error::{Error, Result};
use crate::{layout, manifest, sarif};

/// Every gate ran, and none of them blocked.
pub const CLEAN: i32 = 0;
/// A run carried a block-level result.
pub const BLOCKED: i32 = 2;
/// A gate could not run, so the answer is missing rather than permissive.
pub const CANNOT: i32 = 3;

/// How long one gate has to answer.
///
/// Generous next to a hook's budget, because nothing is waiting on a check the way a vendor
/// waits on a hook: a gate reads a whole tree, and some of them ask git for a diff first.
/// What the timeout is for is a gate that hangs, which would otherwise hang the run.
pub const TIMEOUT: Duration = Duration::from_secs(120);

/// The flag the gates that declared `check_strict` are asked for.
pub const STRICT: &str = "--strict";

/// How often a waiting run looks to see whether a gate has answered.
const POLL: Duration = Duration::from_millis(20);

/// How long a gate's output is still collected after the gate itself has ended.
const DRAIN: Duration = Duration::from_millis(250);

/// The characters that make a command line mean something only a shell can work out.
///
/// The stem spawns a gate directly, with no shell in between, so a command carrying one of
/// these cannot be run the way whoever wrote it meant. Refusing it names the bed; running it
/// with the characters taken literally would judge the tree with the wrong command and call
/// the answer a gate's.
const SHELL_CHARACTERS: [char; 19] = [
    '"', '\'', '\\', '|', '&', ';', '<', '>', '(', ')', '$', '`', '*', '?', '[', ']', '{', '}', '~',
];

/// One bed's gate, resolved to the judge the lock pinned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    pub bed: String,
    /// `.plotplot/bin/<binary>`, never a program found on `PATH`.
    pub program: PathBuf,
    /// The rest of the declared check command, split on whitespace.
    pub args: Vec<String>,
    /// Whether this bed's manifest declares `check_strict: true`, and so whether `--strict`
    /// reaches it when the run is strict. A gate that never said it takes the flag is never
    /// handed it (`manifest::CHECK_STRICT`).
    pub takes_strict: bool,
}

impl Gate {
    /// The arguments one gate is called with in this run.
    pub fn arguments(&self, strict: bool) -> Vec<String> {
        let mut args = self.args.clone();
        if strict && self.takes_strict {
            args.push(STRICT.to_owned());
        }
        args
    }
}

/// What one gate had to say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateOutcome {
    /// The log it printed, already accepted by the SARIF 2.1.0 schema.
    Sarif(Value),
    /// It did not answer, and why.
    CouldNotRun { bed: String, problem: String },
}

/// The gates the planted beds declare, in the order the beds are given.
///
/// A bed declares its gate as a command; the stem replaces the command's first word with the
/// judge under `.plotplot/bin/` and takes the rest as arguments.
///
/// # Errors
///
/// [`Error::Bed`] naming the bed when its check command is empty, when it needs a shell to
/// mean what it says, or when the bed declares a gate but names no binary to run it, which
/// leaves the stem with a gate it can never call.
pub fn gates(root: &Path, beds: &[Bed], strict_checkers: &[String]) -> Result<Vec<Gate>> {
    let mut gates = Vec::new();
    for bed in beds {
        let Some(command) = &bed.check else {
            continue;
        };
        let refuse = |problem: String| Error::Bed {
            bed: bed.name.clone(),
            problem,
        };

        if let Some(character) = command.chars().find(|c| SHELL_CHARACTERS.contains(c)) {
            return Err(refuse(format!(
                "declares the check command \"{command}\", which carries {character} and so \
                 needs a shell to mean what it says; the stem runs a gate directly, so a check \
                 command is a program and its arguments and nothing else"
            )));
        }

        let mut words = command.split_whitespace();
        let Some(_) = words.next() else {
            return Err(refuse(
                "declares a check command of nothing but whitespace".to_owned(),
            ));
        };
        let Some(binary) = &bed.binary else {
            return Err(refuse(format!(
                "declares the check command \"{command}\" but names neither faces.cli nor \
                 install.binary_name, so the stem has no pinned judge to run it with"
            )));
        };

        gates.push(Gate {
            bed: bed.name.clone(),
            program: layout::judge_binary(root, binary),
            args: words.map(str::to_owned).collect(),
            takes_strict: strict_checkers.iter().any(|name| name == &bed.name),
        });
    }
    Ok(gates)
}

/// Run every gate in the repository root, in the order given, and collect what each said.
///
/// One at a time: the gates read the same working tree, several of them ask git about it,
/// and nothing waits on a check the way a vendor waits on a hook. Each gate's stdout is the
/// log it is judged on, whatever it exited with, because a gate that finds a block exits
/// non-zero and has still answered.
pub fn run(root: &Path, gates: &[Gate], strict: bool, timeout: Duration) -> Vec<GateOutcome> {
    gates
        .iter()
        .map(|gate| call(root, gate, strict, timeout))
        .collect()
}

/// The exit code the outcomes add up to: 3 if any gate could not run, else 2 if any run
/// blocks, else 0.
pub fn code(outcomes: &[GateOutcome]) -> i32 {
    if outcomes
        .iter()
        .any(|outcome| matches!(outcome, GateOutcome::CouldNotRun { .. }))
    {
        return CANNOT;
    }
    let blocks: usize = outcomes
        .iter()
        .map(|outcome| match outcome {
            GateOutcome::Sarif(log) => sarif::block_count(log),
            GateOutcome::CouldNotRun { .. } => 0,
        })
        .sum();
    if blocks > 0 { BLOCKED } else { CLEAN }
}

/// One SARIF log carrying the run of every gate that answered.
///
/// A gate that could not run contributes no run: the log says what was judged, and the exit
/// code and stderr say what was not. Inventing an empty run for a gate that never spoke
/// would read as a gate that found nothing.
pub fn merged(outcomes: &[GateOutcome]) -> Value {
    let logs: Vec<Value> = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            GateOutcome::Sarif(log) => Some(log.clone()),
            GateOutcome::CouldNotRun { .. } => None,
        })
        .collect();
    sarif::merge(&logs)
}

/// The table a person reads: one row per gate, and a header naming the columns.
///
/// A gate that could not run has no counts to show, so its two count columns carry a dash
/// and the last column carries the reason. That is the whole difference between a gate that
/// found nothing and a gate that was never asked.
pub fn render(outcomes: &[GateOutcome]) -> String {
    let rows: Vec<(String, String, String, String)> = outcomes
        .iter()
        .map(|outcome| match outcome {
            GateOutcome::Sarif(log) => (
                sarif::tool_name(log).unwrap_or(sarif::UNNAMED).to_owned(),
                sarif::result_count(log).to_string(),
                sarif::block_count(log).to_string(),
                String::new(),
            ),
            GateOutcome::CouldNotRun { bed, problem } => {
                (bed.clone(), "-".to_owned(), "-".to_owned(), problem.clone())
            }
        })
        .collect();

    let header = ("bed", "results", "blocks", "could not run");
    let width = |column: fn(&(String, String, String, String)) -> &String, head: &str| {
        rows.iter()
            .map(|row| column(row).len())
            .chain(std::iter::once(head.len()))
            .max()
            .unwrap_or_default()
    };
    let bed = width(|row| &row.0, header.0);
    let results = width(|row| &row.1, header.1);
    let blocks = width(|row| &row.2, header.2);

    let mut table = String::new();
    let _ = writeln!(
        table,
        "{:bed$}  {:>results$}  {:>blocks$}  {}",
        header.0, header.1, header.2, header.3
    );
    for row in &rows {
        let _ = writeln!(
            table,
            "{:bed$}  {:>results$}  {:>blocks$}  {}",
            row.0,
            row.1,
            row.2,
            row.3.trim_end()
        );
    }
    table
}

/// How a run's answer is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The merged log, which is what another program reads.
    Sarif,
    /// The table, which is what a person reads.
    Table,
}

impl Format {
    /// The two names `--format` takes.
    pub const NAMES: [&'static str; 2] = ["sarif", "table"];
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Format, String> {
        match value {
            "sarif" => Ok(Format::Sarif),
            "table" => Ok(Format::Table),
            other => Err(format!(
                "\"{other}\" is not a format the stem writes; it writes {}",
                Format::NAMES.join(" or ")
            )),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Sarif => f.write_str("sarif"),
            Format::Table => f.write_str("table"),
        }
    }
}

/// `plotplot check`: the face, from the planted manifests to an exit code.
///
/// Named apart from [`run`], which is the gate runner this calls.
///
/// Every gate that could not run says so on stderr whichever format was asked for. The table
/// carries the same reason in its own column, and the repetition is the point: exit 3 is the
/// stem refusing to say a tree is clean, and that reason must survive a run whose stdout was
/// captured into a log file.
pub fn run_face(
    root: &Path,
    args: &CheckArgs,
    format: Format,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let outcomes = match plan(root) {
        Ok(gates) => run(root, &gates, args.strict, TIMEOUT),
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return CANNOT;
        }
    };

    let text = match format {
        Format::Sarif => match serde_json::to_string_pretty(&merged(&outcomes)) {
            Ok(json) => format!("{json}\n"),
            Err(error) => {
                let _ = writeln!(stderr, "the merged log could not be written: {error}");
                return CANNOT;
            }
        },
        Format::Table => render(&outcomes),
    };
    if let Err(error) = write!(stdout, "{text}") {
        let _ = writeln!(stderr, "stdout: {error}");
        return CANNOT;
    }

    for outcome in &outcomes {
        if let GateOutcome::CouldNotRun { bed, problem } = outcome {
            let _ = writeln!(stderr, "{bed}: {problem}");
        }
    }
    code(&outcomes)
}

/// The gates a planted repository declares, read from its manifests.
///
/// # Errors
///
/// Whatever [`crate::manifest::load_beds`], [`crate::manifest::strict_checkers`] and
/// [`gates`] refuse.
fn plan(root: &Path) -> Result<Vec<Gate>> {
    let beds = manifest::load_beds(root)?;
    let strict_checkers = manifest::strict_checkers(root)?;
    gates(root, &beds, &strict_checkers)
}

/// One gate, run in the repository root with its output captured.
fn call(root: &Path, gate: &Gate, strict: bool, timeout: Duration) -> GateOutcome {
    let cannot = |problem: String| GateOutcome::CouldNotRun {
        bed: gate.bed.clone(),
        problem,
    };

    let mut child = match Command::new(&gate.program)
        .args(gate.arguments(strict))
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(source) => {
            return cannot(format!(
                "{} could not be run: {source}",
                gate.program.display()
            ));
        }
    };

    let stdout = reader(child.stdout.take());
    let stderr = reader(child.stderr.take());
    let ended = wait_until(&mut child, timeout);
    let deadline = Instant::now() + DRAIN;
    let printed = drain(stdout, deadline);
    let said = manifest::one_line(&drain(stderr, deadline));

    let code = match ended {
        Ended::Exited(code) => code,
        Ended::TimedOut => {
            return cannot(format!(
                "did not answer within {} s and was stopped{}",
                timeout.as_secs(),
                aside(&said)
            ));
        }
        Ended::Failed(problem) => {
            return cannot(format!(
                "could not be waited for: {problem}{}",
                aside(&said)
            ));
        }
    };

    match sarif::validate_from(&gate.bed, &printed) {
        Ok(log) => GateOutcome::Sarif(log),
        Err(Error::Sarif { problem, .. }) => cannot(match code {
            Some(0) => format!("{problem}{}", aside(&said)),
            Some(code) => format!("exited {code} and {problem}{}", aside(&said)),
            None => format!(
                "was killed by a signal before it answered, and {problem}{}",
                aside(&said)
            ),
        }),
        Err(other) => cannot(other.to_string()),
    }
}

/// What a gate said on stderr, appended to a problem, or nothing when it said nothing.
fn aside(said: &str) -> String {
    if said.is_empty() {
        String::new()
    } else {
        format!("; it said: {said}")
    }
}

/// How a gate ended.
enum Ended {
    /// It exited; `None` when a signal killed it and there is no code to read.
    Exited(Option<i32>),
    /// It was still running at the timeout and was killed.
    TimedOut,
    /// The stem could not wait for it.
    Failed(String),
}

/// Wait for a gate, killing it at the deadline.
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

/// Read one of a gate's streams to its end, off the thread that waits for the gate.
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

/// What a stream carried, or nothing when the pipe outlived the gate that owned it.
fn drain(stream: Receiver<String>, deadline: Instant) -> String {
    let left = deadline.saturating_duration_since(Instant::now());
    stream.recv_timeout(left).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bed(name: &str, binary: Option<&str>, check: Option<&str>) -> Bed {
        Bed {
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            binary: binary.map(str::to_owned),
            skill: None,
            hooks: std::collections::BTreeMap::new(),
            git_hooks: Vec::new(),
            mcp: None,
            check: check.map(str::to_owned),
        }
    }

    fn log_of(tool: &str, levels: &[&str]) -> Value {
        json!({
            "$schema": sarif::SCHEMA_URL,
            "version": sarif::VERSION,
            "runs": [{
                "tool": { "driver": { "name": tool } },
                "results": levels.iter().map(|level| json!({
                    "level": level,
                    "message": { "text": "a finding" }
                })).collect::<Vec<_>>()
            }]
        })
    }

    #[test]
    fn a_gate_is_the_pinned_judge_and_the_rest_of_the_command() {
        let root = Path::new("/work/repo");
        let beds = [bed(
            "weeder",
            Some("weeder"),
            Some("weeder check --format sarif"),
        )];
        let gates = gates(root, &beds, &[]).expect("a plain command");
        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].bed, "weeder");
        assert_eq!(
            gates[0].program,
            root.join(".plotplot/bin/weeder"),
            "the pinned judge, never PATH"
        );
        assert_eq!(gates[0].args, ["check", "--format", "sarif"]);
        assert!(!gates[0].takes_strict);
    }

    #[test]
    fn the_commands_first_word_is_replaced_even_when_it_is_not_the_beds_name() {
        let root = Path::new("/work/repo");
        let beds = [bed("tend2", Some("tend2"), Some("node lint docs/tend2"))];
        let gates = gates(root, &beds, &[]).expect("a plain command");
        assert_eq!(gates[0].program, root.join(".plotplot/bin/tend2"));
        assert_eq!(gates[0].args, ["lint", "docs/tend2"]);
    }

    #[test]
    fn a_bed_with_no_check_declares_no_gate() {
        let root = Path::new("/work/repo");
        let beds = [
            bed("pollen", Some("pollen"), None),
            bed("weeder", Some("weeder"), Some("weeder check")),
        ];
        let gates = gates(root, &beds, &[]).expect("one gate");
        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].bed, "weeder");
    }

    #[test]
    fn a_check_command_needing_a_shell_names_the_bed() {
        let root = Path::new("/work/repo");
        for command in [
            "weeder check --scope \"src/**\"",
            "weeder check | tee out",
            "weeder check && echo done",
            "weeder check > out",
            "weeder check --scope src/*.rs",
            "weeder check --config $HOME/weeder.toml",
            "weeder check --config ~/weeder.toml",
        ] {
            let beds = [bed("weeder", Some("weeder"), Some(command))];
            let error = gates(root, &beds, &[]).expect_err(command);
            let message = error.to_string();
            assert!(message.starts_with("weeder: "), "{message}");
            assert!(message.contains("needs a shell"), "{message}");
        }
    }

    #[test]
    fn a_bed_that_declares_a_gate_and_no_binary_names_the_bed() {
        let root = Path::new("/work/repo");
        let beds = [bed("petals", None, Some("petals check --format sarif"))];
        let error = gates(root, &beds, &[]).expect_err("no judge to run it with");
        let message = error.to_string();
        assert!(message.starts_with("petals: "), "{message}");
        assert!(message.contains("no pinned judge"), "{message}");
    }

    #[test]
    fn a_check_command_of_whitespace_names_the_bed() {
        let root = Path::new("/work/repo");
        let beds = [bed("weeder", Some("weeder"), Some("   "))];
        let error = gates(root, &beds, &[]).expect_err("nothing to run");
        assert!(error.to_string().contains("whitespace"), "{error}");
    }

    #[test]
    fn strict_reaches_only_the_gates_that_declared_it() {
        let root = Path::new("/work/repo");
        let beds = [
            bed("petals", Some("petals"), Some("petals check")),
            bed("weeder", Some("weeder"), Some("weeder check")),
        ];
        let gates = gates(root, &beds, &["weeder".to_owned()]).expect("two gates");
        assert!(!gates[0].takes_strict);
        assert!(gates[1].takes_strict);

        assert_eq!(gates[0].arguments(true), ["check"]);
        assert_eq!(gates[1].arguments(true), ["check", "--strict"]);
        assert_eq!(gates[0].arguments(false), ["check"]);
        assert_eq!(gates[1].arguments(false), ["check"]);
    }

    #[test]
    fn a_run_that_blocks_is_two_and_a_clean_run_is_zero() {
        assert_eq!(code(&[]), CLEAN);
        assert_eq!(
            code(&[GateOutcome::Sarif(log_of("weeder", &["warning", "note"]))]),
            CLEAN
        );
        assert_eq!(
            code(&[GateOutcome::Sarif(log_of("weeder", &["warning", "error"]))]),
            BLOCKED
        );
    }

    #[test]
    fn a_gate_that_could_not_run_is_three_even_beside_a_block() {
        let outcomes = [
            GateOutcome::Sarif(log_of("weeder", &["error"])),
            GateOutcome::CouldNotRun {
                bed: "petals".to_owned(),
                problem: "it is not there".to_owned(),
            },
        ];
        assert_eq!(code(&outcomes), CANNOT);
        assert_eq!(code(&outcomes[1..]), CANNOT);
    }

    #[test]
    fn the_merged_log_carries_only_the_gates_that_answered() {
        let outcomes = [
            GateOutcome::Sarif(log_of("petals", &["note"])),
            GateOutcome::CouldNotRun {
                bed: "tilth".to_owned(),
                problem: "it is not there".to_owned(),
            },
            GateOutcome::Sarif(log_of("weeder", &["error"])),
        ];
        let merged = merged(&outcomes);
        let runs = merged["runs"].as_array().expect("runs");
        assert_eq!(runs.len(), 2);
        assert_eq!(
            runs.iter()
                .map(|run| run.pointer("/tool/driver/name").and_then(Value::as_str))
                .collect::<Vec<_>>(),
            [Some("petals"), Some("weeder")]
        );
    }

    #[test]
    fn the_table_has_a_header_and_one_row_per_gate() {
        let outcomes = [
            GateOutcome::Sarif(log_of("petals", &["note", "error"])),
            GateOutcome::CouldNotRun {
                bed: "tilth".to_owned(),
                problem: "it is not there".to_owned(),
            },
        ];
        let table = render(&outcomes);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 3, "{table}");
        assert_eq!(
            lines[0].split_whitespace().collect::<Vec<_>>(),
            ["bed", "results", "blocks", "could", "not", "run"]
        );
        assert_eq!(
            lines[1].split_whitespace().collect::<Vec<_>>(),
            ["petals", "2", "1"]
        );
        assert_eq!(
            lines[2].split_whitespace().collect::<Vec<_>>(),
            ["tilth", "-", "-", "it", "is", "not", "there"]
        );
    }

    #[test]
    fn the_table_of_no_gates_is_its_header_alone() {
        assert_eq!(render(&[]).lines().count(), 1);
    }

    #[test]
    fn the_two_formats_round_trip_through_their_names() {
        for format in [Format::Sarif, Format::Table] {
            assert_eq!(format.to_string().parse::<Format>(), Ok(format));
        }
        let error = "json".parse::<Format>().expect_err("two names, no more");
        assert!(error.contains("sarif or table"), "{error}");
    }

    #[test]
    fn a_gate_that_is_not_there_could_not_run_and_names_its_path() {
        // A temporary root, never the crate's own: this repository is planted with the stem
        // and carries a real weeder under .plotplot/bin/.
        let temp = tempfile::tempdir().expect("a temp root");
        let root = temp.path();
        let gate = Gate {
            bed: "weeder".to_owned(),
            program: root.join(".plotplot/bin/weeder"),
            args: vec!["check".to_owned()],
            takes_strict: false,
        };
        let outcomes = run(root, &[gate], false, TIMEOUT);
        match &outcomes[0] {
            GateOutcome::CouldNotRun { bed, problem } => {
                assert_eq!(bed, "weeder");
                assert!(problem.contains(".plotplot/bin/weeder"), "{problem}");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(code(&outcomes), CANNOT);
    }
}
