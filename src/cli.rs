//! The shape of the command line, and the one place a face's `Result` becomes an exit code.
//!
//! A face with work of its own keeps that work in its own module. `version` has none beyond
//! reading the lock and laying out five lines, so it lives here.

use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::lock::{Lock, read_lock};
use crate::{VERSION, bundle, doctor};

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
    /// The generated vendor bundles.
    Bundle {
        #[command(subcommand)]
        command: BundleFace,
    },
    /// Prove the garden is planted: one line per check, exit 0 when every check passes.
    Doctor,
}

/// What `plotplot bundle` can be asked to do.
#[derive(Debug, Subcommand)]
pub enum BundleFace {
    /// Regenerate `.plotplot/bundles/<harness>/` from the planted beds' manifests.
    Build {
        /// The harness to regenerate for; all three when omitted.
        #[arg(value_name = "claude|gemini|codex", value_parser = harness)]
        harness: Option<Harness>,
    },
}

/// One harness name from the command line, refused by the same reader a manifest goes
/// through so the three names are spelled in one place.
fn harness(name: &str) -> Result<Harness> {
    name.parse()
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
        Face::Doctor => match home() {
            Ok(home) => doctor::run(root, &home, stdout, stderr),
            Err(error) => {
                let _ = writeln!(stderr, "{error}");
                1
            }
        },
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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/fixtures/garden.lock");
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
