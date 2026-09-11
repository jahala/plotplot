//! The shape of the command line, and the one place a face's `Result` becomes an exit code.
//!
//! A face with work of its own keeps that work in its own module. `version` has none beyond
//! reading the lock and laying out five lines, so it lives here.

use std::ffi::OsString;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Args as ClapArgs, Parser, Subcommand};

use crate::check::Format;
use crate::doctor::live;
use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::lock::{Lock, read_lock};
use crate::{VERSION, bundle, check, doctor, friction, hook, init, lock, receipt};

/// `plotplot`, the stem of the garden.
#[derive(Debug, Parser)]
#[command(
    name = "plotplot",
    version,
    about = "Plant the garden in this repository, and prove it is planted."
)]
pub struct Args {
    #[command(subcommand)]
    pub face: Face,
}

/// The faces the binary answers to. One variant lands per face as its node lands.
#[derive(Debug, Subcommand)]
pub enum Face {
    /// Print the stem's version, and the season and judges `garden.lock` pins.
    Version,
    /// Plant the garden in this repository: the lock, the bundles, the hooks, the block.
    Init(InitArgs),
    /// Dispatch one hook event to the beds that registered for it (stdin: the payload).
    Hook(HookArgs),
    /// The friction emitter on its own (stdin: the payload).
    Friction(FrictionArgs),
    /// The receipt writer on its own (stdin: the payload).
    Receipt(ReceiptArgs),
    /// The generated vendor bundles.
    Bundle {
        #[command(subcommand)]
        command: BundleFace,
    },
    /// Run every gate the planted manifests declare and merge their findings into one log.
    Check(CheckArgs),
    /// Prove the garden is planted: one line per check, exit 0 when every check passes.
    Doctor(DoctorArgs),
    /// The pinned judges `garden.lock` names.
    Lock {
        #[command(subcommand)]
        command: LockFace,
    },
}

/// What `plotplot lock` can be asked to do.
#[derive(Debug, Subcommand)]
pub enum LockFace {
    /// Resolve every judge for this platform, fetching what is absent and refusing bytes
    /// whose digest is not the one the lock pins.
    Verify,
}

/// What `plotplot bundle` can be asked to do.
#[derive(Debug, Subcommand)]
pub enum BundleFace {
    /// Regenerate `.plotplot/bundles/<harness>/` from the planted beds' manifests.
    Build {
        /// The harness to regenerate for; all three when omitted.
        #[arg(value_name = "claude|gemini|codex", value_parser = harness_value)]
        harness: Option<Harness>,
    },
}

/// `plotplot init [--harness …] [--beds …] [--profile …] [--lock <path>] [--github]`.
#[derive(Debug, ClapArgs)]
pub struct InitArgs {
    /// The harnesses to plant; what is detected on PATH and already configured here when
    /// this is not given.
    #[arg(
        long,
        value_name = "claude,gemini,codex",
        value_delimiter = ',',
        value_parser = harness_value
    )]
    pub harness: Option<Vec<Harness>>,
    /// The beds to plant, overriding the profile's list.
    #[arg(long, value_name = "a,b", value_delimiter = ',')]
    pub beds: Option<Vec<String>>,
    /// Which beds the profile plants: weeder alone, or every judge the lock pins.
    #[arg(long, value_name = "minimal|full", default_value = "minimal")]
    pub profile: Profile,
    /// The `garden.lock` template to copy from, needed when this repository pins none yet.
    #[arg(long, value_name = "path")]
    pub lock: Option<PathBuf>,
    /// After planting, write the stem's region of .github/CODEOWNERS and apply the default
    /// branch's ruleset through gh: the garden check required, force pushes and deletion
    /// refused.
    #[arg(long)]
    pub github: bool,
}

/// Which beds `init` plants when `--beds` does not say.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Profile {
    /// weeder with its hooks, the garden block, the lock, the git hooks and the PR gate
    /// (jahala/plotplot issue 17).
    Minimal,
    /// Every judge the lock pins.
    Full,
}

/// `plotplot doctor [--platform] [--live] [--harness claude,gemini,codex] [--timeout <s>]`.
#[derive(Debug, ClapArgs)]
pub struct DoctorArgs {
    /// After the static findings, read back through gh, with read-only calls, whether GitHub
    /// requires the garden check and refuses force pushes and deletion on the default branch.
    #[arg(long)]
    pub platform: bool,
    /// After the static findings, drive one real session per harness through umbel and prove
    /// from the friction journal and the receipt drafts that each hook fired.
    #[arg(long)]
    pub live: bool,
    /// The harnesses live mode drives; all three when this is not given, each reported as
    /// unavailable when it is not planted here.
    #[arg(
        long,
        value_name = "claude,gemini,codex",
        value_delimiter = ',',
        value_parser = harness_value
    )]
    pub harness: Option<Vec<Harness>>,
    /// How long one worker has to answer, in seconds.
    #[arg(long, value_name = "s")]
    pub timeout: Option<u64>,
}

impl DoctorArgs {
    /// The harnesses live mode drives, in `Harness::ALL`'s order and without repeats.
    pub fn harnesses(&self) -> Vec<Harness> {
        match &self.harness {
            None => Harness::ALL.to_vec(),
            Some(asked) => Harness::ALL
                .into_iter()
                .filter(|harness| asked.contains(harness))
                .collect(),
        }
    }

    /// How long one worker has to answer.
    pub fn timeout(&self) -> Duration {
        self.timeout
            .map_or(live::DEFAULT_TIMEOUT, Duration::from_secs)
    }
}

/// `plotplot check [--strict] [--format sarif|table]`.
#[derive(Debug, ClapArgs)]
pub struct CheckArgs {
    /// Ask for the strictest judgement of every gate that declares it takes one.
    #[arg(long)]
    pub strict: bool,
    /// How to write the answer; a table at a terminal and SARIF everywhere else.
    #[arg(long, value_name = "sarif|table", value_parser = format_value)]
    pub format: Option<Format>,
}

/// `plotplot hook <harness> <event>`, the call every hook entry in every bundle makes.
#[derive(Debug, ClapArgs)]
pub struct HookArgs {
    /// The harness whose payload is on stdin.
    #[arg(value_parser = harness_value)]
    pub harness: Harness,
    /// The event, spelled the way that harness spells it.
    pub event: String,
}

/// `plotplot friction …`.
#[derive(Debug, ClapArgs)]
pub struct FrictionArgs {
    #[command(subcommand)]
    pub face: FrictionFace,
}

/// The friction faces the stem answers to.
#[derive(Debug, Subcommand)]
pub enum FrictionFace {
    /// Append this payload's friction records to the month's journal.
    Emit(HarnessArg),
}

/// `plotplot receipt …`.
#[derive(Debug, ClapArgs)]
pub struct ReceiptArgs {
    #[command(subcommand)]
    pub face: ReceiptFace,
}

/// The receipt faces the stem answers to.
#[derive(Debug, Subcommand)]
pub enum ReceiptFace {
    /// Write this session's unsigned receipt draft.
    Draft(HarnessArg),
    /// Attach a commit's receipt to `refs/notes/plotplot/receipts`.
    Seal(SealArgs),
    /// Recompute a commit's receipt, or every receipt in a range.
    Verify(VerifyArgs),
    /// Print the predicate a commit's receipt carries.
    Show(ShowArgs),
}

/// `plotplot receipt seal [--commit <rev>]`.
#[derive(Debug, ClapArgs)]
pub struct SealArgs {
    /// The commit to seal; `HEAD` when it is not given, which is what the post-commit hook
    /// means by the commit that just happened.
    #[arg(long, value_name = "rev", default_value = "HEAD")]
    pub commit: String,
}

/// `plotplot receipt verify (<rev> | --range <a>..<b>) [--require-signed]`.
#[derive(Debug, ClapArgs)]
#[command(group(clap::ArgGroup::new("commits").required(true).args(["rev", "range"])))]
pub struct VerifyArgs {
    /// The commit whose receipt to recompute.
    #[arg(value_name = "rev")]
    pub rev: Option<String>,
    /// Every commit a range holds, as git spells one.
    #[arg(long, value_name = "a..b")]
    pub range: Option<String>,
    /// Refuse a receipt that carries no signature. Every v0 receipt is unsigned, so this
    /// refuses all of them until signing lands.
    #[arg(long)]
    pub require_signed: bool,
}

/// `plotplot receipt show <rev>`.
#[derive(Debug, ClapArgs)]
pub struct ShowArgs {
    /// The commit whose receipt to print.
    #[arg(value_name = "rev")]
    pub rev: String,
}

/// The harness a face reading stdin needs told, because a payload does not name its vendor.
#[derive(Debug, ClapArgs)]
pub struct HarnessArg {
    /// claude, gemini or codex.
    #[arg(long, value_parser = harness_value)]
    pub harness: Harness,
}

/// One harness name from the command line, read by the same parser a manifest goes through
/// so the three names are spelled in one place; clap prints the error beside the bad value.
fn harness_value(value: &str) -> std::result::Result<Harness, String> {
    value.parse::<Harness>().map_err(|error| error.to_string())
}

/// One format name from the command line, read by the same parser the two names are spelled
/// in; clap prints the error beside the bad value.
fn format_value(value: &str) -> std::result::Result<Format, String> {
    value.parse::<Format>()
}

/// Run one face. The only exit codes this returns are the face's own; `main` turns the
/// number into a process status and nothing else.
pub fn run(args: Args, root: &Path, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match args.face {
        Face::Version => match read_lock(root) {
            Ok(lock) => match write!(stdout, "{}", version_text(lock.as_ref())) {
                Ok(()) => 0,
                Err(error) => {
                    let _ = writeln!(stderr, "stdout: {error}");
                    1
                }
            },
            Err(error) => {
                let _ = writeln!(stderr, "{error}");
                1
            }
        },
        Face::Hook(face) => match payload_on_stdin() {
            Ok(payload) => hook::run(root, &face, &payload, stdout, stderr),
            Err(error) => {
                let _ = writeln!(stderr, "stdin: {error}");
                hook::CANNOT
            }
        },
        Face::Friction(face) => match payload_on_stdin() {
            Ok(payload) => friction::run(root, &face, &payload, stdout, stderr),
            Err(error) => {
                let _ = writeln!(stderr, "stdin: {error}");
                1
            }
        },
        // Only `draft` is a hook face, and only a hook face has a payload waiting on stdin.
        // Seal, verify and show are run from a git hook or a terminal, where reading stdin
        // would block on nothing.
        Face::Receipt(face) => match &face.face {
            ReceiptFace::Draft(_) => match payload_on_stdin() {
                Ok(payload) => receipt::run(root, &face, &payload, stdout, stderr),
                Err(error) => {
                    let _ = writeln!(stderr, "stdin: {error}");
                    1
                }
            },
            ReceiptFace::Seal(_) | ReceiptFace::Verify(_) | ReceiptFace::Show(_) => {
                receipt::run(root, &face, "", stdout, stderr)
            }
        },
        Face::Bundle {
            command: BundleFace::Build { harness },
        } => {
            let harnesses = match harness {
                Some(one) => vec![one],
                None => Harness::ALL.to_vec(),
            };
            match bundle::running_stem() {
                Ok(stem) => bundle::run(root, &stem, &harnesses, stdout, stderr),
                Err(error) => {
                    let _ = writeln!(stderr, "{error}");
                    1
                }
            }
        }
        Face::Init(face) => init::run(root, &face, search_path().as_deref(), stdout, stderr),
        Face::Check(face) => {
            let format = face.format.unwrap_or_else(chosen_format);
            check::run_face(root, &face, format, stdout, stderr)
        }
        Face::Lock { command } => lock::run(root, &command, stdout, stderr),
        Face::Doctor(face) => match home() {
            Ok(home) => {
                doctor::run_face(root, &home, &face, search_path().as_deref(), stdout, stderr)
            }
            Err(error) => {
                let _ = writeln!(stderr, "{error}");
                1
            }
        },
    }
}

/// The vendor's payload, read whole from stdin.
///
/// The three hook-time faces are filters, and [`run`]'s shape has nowhere for their input to
/// be passed in, so this is where the stem reads it. A payload that is not UTF-8 is not one
/// of the three vendors' JSON, and the error says so with the reason the reader gave.
fn payload_on_stdin() -> std::io::Result<String> {
    let mut payload = String::new();
    std::io::stdin().read_to_string(&mut payload)?;
    Ok(payload)
}

/// The directories a command name is looked for in, as the process was started with.
///
/// Read here, at the edge, beside the payload on stdin and the planter's home: `init` is
/// told where to look for the three harness CLIs and for gh, and `doctor --platform` for gh,
/// rather than asking the environment themselves, so a test can hand them a directory of its
/// own.
fn search_path() -> Option<OsString> {
    std::env::var_os("PATH")
}

/// What `--format` defaults to: a table for a person at a terminal, SARIF for anything that
/// reads the answer rather than looks at it.
///
/// Read here, at the edge, beside the other two facts this process learns from around it:
/// the payload on stdin and the planter's home. The face itself is told the format.
fn chosen_format() -> Format {
    if std::io::stdout().is_terminal() {
        Format::Table
    } else {
        Format::Sarif
    }
}

/// The planter's home directory, where Codex records the projects it trusts.
///
/// Read here, at the edge, and passed in: `doctor` is told where to look rather than asking
/// the environment, so a test can point it at a temporary home and the build machine's own
/// Codex configuration is never read.
///
/// # Errors
///
/// [`Error::Io`] when `HOME` is not set. `doctor` refuses rather than guessing, because a
/// guess would report Codex's trust from a place nobody configured.
fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| Error::Io {
            path: PathBuf::from("$HOME"),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "doctor reads Codex's project trust from the home directory, and HOME is not set",
            ),
        })
}

/// What `plotplot version` prints: the binary's own version, then, when the repository is
/// planted, the season and each judge the lock pins, sorted by name.
pub fn version_text(lock: Option<&Lock>) -> String {
    let mut text = format!("plotplot {VERSION}\n");
    if let Some(lock) = lock {
        text.push_str(&format!("season {}\n", lock.season));
        for (name, judge) in &lock.judges {
            match &judge.npm {
                Some(package) => {
                    text.push_str(&format!("{name} {} (npm {package})\n", judge.version));
                }
                None => text.push_str(&format!("{name} {}\n", judge.version)),
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_lock() -> Lock {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/garden.lock");
        let toml = std::fs::read_to_string(&path).expect("the fixture lock");
        crate::lock::parse_lock_at(&path, &toml).expect("a parsed lock")
    }

    #[test]
    fn without_a_lock_the_version_is_the_binarys_own() {
        assert_eq!(version_text(None), "plotplot 0.1.0\n");
    }

    #[test]
    fn with_a_lock_the_season_and_every_judge_follow_sorted_by_name() {
        assert_eq!(
            version_text(Some(&fixture_lock())),
            concat!(
                "plotplot 0.1.0\n",
                "season 2026.09\n",
                "tend2 1.0.0 (npm @plotplot/tend2)\n",
                "tilth 0.10.1\n",
                "weeder 0.1.0\n",
            )
        );
    }

    #[test]
    fn the_first_line_is_the_same_with_and_without_a_lock() {
        let lock = fixture_lock();
        let planted = version_text(Some(&lock));
        assert_eq!(planted.lines().next(), version_text(None).lines().next());
    }

    #[test]
    fn a_judge_with_no_npm_package_carries_no_marker() {
        let lock = fixture_lock();
        let text = version_text(Some(&lock));
        assert!(text.contains("\ntilth 0.10.1\n"), "{text}");
        assert!(
            text.contains("\ntend2 1.0.0 (npm @plotplot/tend2)\n"),
            "{text}"
        );
    }

    #[test]
    fn run_writes_the_version_to_the_stream_it_is_given() {
        let root = tempfile::tempdir().expect("a temp root");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(
            Args {
                face: Face::Version,
            },
            root.path(),
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, 0);
        assert_eq!(String::from_utf8_lossy(&stdout), "plotplot 0.1.0\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn run_reports_a_lock_it_cannot_read_on_stderr_and_answers_one() {
        let root = tempfile::tempdir().expect("a temp root");
        let path = crate::layout::garden_lock(root.path());
        std::fs::write(&path, "season = \"2026.09\"\n").expect("a lock the contracts refuse");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(
            Args {
                face: Face::Version,
            },
            root.path(),
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, 1);
        assert!(stdout.is_empty());
        let stderr = String::from_utf8_lossy(&stderr);
        assert!(
            stderr.starts_with(&format!("{}: ", path.display())),
            "{stderr}"
        );
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
    }

    #[test]
    fn the_argument_parser_is_well_formed() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
