//! The one error type the whole crate returns.
//!
//! Every variant carries what a reader needs to act: the path when a file was involved, the
//! bed when a manifest was, the command when git was. `Display` is one line and puts the
//! path first, because that line is what `main.rs` prints on stderr.

use std::fmt;
use std::path::PathBuf;

/// What can go wrong anywhere in the stem.
#[derive(Debug)]
pub enum Error {
    /// A file could not be read or written.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// JSON that could not be parsed. `path` is `None` when the JSON came from stdin or a
    /// string rather than a file.
    Json {
        path: Option<PathBuf>,
        source: serde_json::Error,
    },
    /// TOML that could not be parsed, or that the lock schema refused.
    Toml { path: PathBuf, message: String },
    /// A `garden.json` the contracts or the stem refused.
    Manifest { bed: String, problem: String },
    /// A hook payload the stem could not read as one of the three vendors' shapes.
    Harness { problem: String },
    /// A git command that failed.
    Git { command: String, stderr: String },
    /// A planted bed that cannot do what its manifest claims.
    Bed { bed: String, problem: String },
    /// Output a gate offered as SARIF that the SARIF 2.1.0 schema will not accept.
    Sarif { gate: String, problem: String },
    /// An `AGENTS.md` whose plotplot markers do not make one replaceable block, so the stem
    /// cannot say which bytes the garden block owns.
    Agents { problem: String },
    /// A `.github/CODEOWNERS` whose plotplot markers do not make one replaceable region, so
    /// the stem cannot say which lines are its own.
    Codeowners { problem: String },
    /// A url that did not give up its bytes: a status outside 2xx, or a transport that
    /// failed before any status arrived.
    Fetch { url: String, problem: String },
    /// Bytes whose digest is not the digest the lock pins. The stem refuses to run them.
    Checksum {
        judge: String,
        expected: String,
        actual: String,
    },
    /// An artifact the stem could not unpack, or one in a format it does not unpack.
    Archive { path: PathBuf, problem: String },
    /// A harness that would not take the bundle `init` generated for it, or a project-scope
    /// configuration file of that harness's that the stem could not read or merge into.
    Install {
        harness: crate::harness::Harness,
        problem: String,
    },
    /// A receipt the stem could not seal, read or believe, named by the commit it is about.
    Receipt { commit: String, problem: String },
}

/// The crate's result type; failure always lives here rather than in a panic.
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Json { path, source } => match path {
                Some(path) => write!(f, "{}: {source}", path.display()),
                None => write!(f, "{source}"),
            },
            Error::Toml { path, message } => write!(f, "{}: {message}", path.display()),
            Error::Manifest { bed, problem } | Error::Bed { bed, problem } => {
                write!(f, "{bed}: {problem}")
            }
            Error::Sarif { gate, problem } => write!(f, "{gate}: {problem}"),
            Error::Harness { problem }
            | Error::Agents { problem }
            | Error::Codeowners { problem } => write!(f, "{problem}"),
            Error::Git { command, stderr } => write!(f, "{command}: {stderr}"),
            Error::Fetch { url, problem } => write!(f, "{url}: {problem}"),
            Error::Checksum {
                judge,
                expected,
                actual,
            } => write!(f, "{judge}: expected {expected}, got {actual}"),
            Error::Archive { path, problem } => write!(f, "{}: {problem}", path.display()),
            Error::Install { harness, problem } => write!(f, "{harness}: {problem}"),
            Error::Receipt { commit, problem } => write!(f, "{commit}: {problem}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Json { source, .. } => Some(source),
            Error::Toml { .. }
            | Error::Manifest { .. }
            | Error::Harness { .. }
            | Error::Git { .. }
            | Error::Bed { .. }
            | Error::Sarif { .. }
            | Error::Agents { .. }
            | Error::Codeowners { .. }
            | Error::Fetch { .. }
            | Error::Checksum { .. }
            | Error::Archive { .. }
            | Error::Install { .. }
            | Error::Receipt { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn io_displays_the_path_first() {
        let error = Error::Io {
            path: PathBuf::from("/work/repo/garden.lock"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
        };
        assert_eq!(error.to_string(), "/work/repo/garden.lock: no such file");
    }

    #[test]
    fn json_without_a_path_displays_the_source_alone() {
        let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let message = source.to_string();
        let error = Error::Json { path: None, source };
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn json_with_a_path_displays_the_path_first() {
        let source = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let message = source.to_string();
        let error = Error::Json {
            path: Some(PathBuf::from(
                "/work/repo/.plotplot/beds/weeder/garden.json",
            )),
            source,
        };
        assert_eq!(
            error.to_string(),
            format!("/work/repo/.plotplot/beds/weeder/garden.json: {message}")
        );
    }

    #[test]
    fn toml_displays_the_path_first() {
        let error = Error::Toml {
            path: PathBuf::from("garden.lock"),
            message: "expected a table".to_owned(),
        };
        assert_eq!(error.to_string(), "garden.lock: expected a table");
    }

    #[test]
    fn the_bed_named_errors_name_their_bed() {
        let manifest = Error::Manifest {
            bed: "weeder".to_owned(),
            problem: "declares hooks for unknown harness \"cursor\"".to_owned(),
        };
        assert_eq!(
            manifest.to_string(),
            "weeder: declares hooks for unknown harness \"cursor\""
        );
        let bed = Error::Bed {
            bed: "weeder".to_owned(),
            problem: "is not on disk".to_owned(),
        };
        assert_eq!(bed.to_string(), "weeder: is not on disk");
        let sarif = Error::Sarif {
            gate: "weeder".to_owned(),
            problem: "its output is not JSON".to_owned(),
        };
        assert_eq!(sarif.to_string(), "weeder: its output is not JSON");
    }

    #[test]
    fn harness_and_git_display_on_one_line() {
        let harness = Error::Harness {
            problem: "the payload has no hook_event_name".to_owned(),
        };
        assert_eq!(harness.to_string(), "the payload has no hook_event_name");
        let git = Error::Git {
            command: "git config core.hooksPath".to_owned(),
            stderr: "not a git repository".to_owned(),
        };
        assert_eq!(
            git.to_string(),
            "git config core.hooksPath: not a git repository"
        );
    }

    #[test]
    fn a_malformed_marker_pair_displays_its_problem_alone() {
        let agents = Error::Agents {
            problem: "AGENTS.md has <!-- plotplot:begin --> without <!-- plotplot:end -->"
                .to_owned(),
        };
        assert_eq!(
            agents.to_string(),
            "AGENTS.md has <!-- plotplot:begin --> without <!-- plotplot:end -->"
        );
    }

    #[test]
    fn a_fetch_failure_names_the_url_first_and_then_what_happened() {
        let error = Error::Fetch {
            url: "https://example.invalid/weeder.tar.gz".to_owned(),
            problem: "the server answered 404".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "https://example.invalid/weeder.tar.gz: the server answered 404"
        );
    }

    #[test]
    fn a_checksum_failure_names_the_judge_and_both_digests() {
        let error = Error::Checksum {
            judge: "tilth".to_owned(),
            expected: "38c36e".to_owned(),
            actual: "d05426".to_owned(),
        };
        assert_eq!(error.to_string(), "tilth: expected 38c36e, got d05426");
    }

    #[test]
    fn an_archive_failure_displays_the_path_first() {
        let error = Error::Archive {
            path: PathBuf::from("https://example.invalid/weeder.zip"),
            problem: "zip is not unpacked on this platform".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "https://example.invalid/weeder.zip: zip is not unpacked on this platform"
        );
    }

    #[test]
    fn every_display_is_one_line() {
        let errors = [
            Error::Io {
                path: PathBuf::from("a"),
                source: std::io::Error::other("b"),
            },
            Error::Toml {
                path: PathBuf::from("a"),
                message: "b".to_owned(),
            },
            Error::Manifest {
                bed: "a".to_owned(),
                problem: "b".to_owned(),
            },
            Error::Harness {
                problem: "b".to_owned(),
            },
            Error::Git {
                command: "a".to_owned(),
                stderr: "b".to_owned(),
            },
            Error::Bed {
                bed: "a".to_owned(),
                problem: "b".to_owned(),
            },
            Error::Sarif {
                gate: "a".to_owned(),
                problem: "b".to_owned(),
            },
            Error::Agents {
                problem: "b".to_owned(),
            },
            Error::Codeowners {
                problem: "b".to_owned(),
            },
            Error::Fetch {
                url: "a".to_owned(),
                problem: "b".to_owned(),
            },
            Error::Checksum {
                judge: "a".to_owned(),
                expected: "b".to_owned(),
                actual: "c".to_owned(),
            },
            Error::Archive {
                path: PathBuf::from("a"),
                problem: "b".to_owned(),
            },
            Error::Install {
                harness: crate::harness::Harness::Gemini,
                problem: "b".to_owned(),
            },
            Error::Receipt {
                commit: "a".to_owned(),
                problem: "b".to_owned(),
            },
        ];
        for error in &errors {
            assert!(!error.to_string().contains('\n'), "{error}");
        }
    }

    #[test]
    fn an_install_failure_names_the_harness_first() {
        let error = Error::Install {
            harness: crate::harness::Harness::Claude,
            problem: "`claude plugin install` exited 1: no such marketplace".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "claude: `claude plugin install` exited 1: no such marketplace"
        );
    }

    #[test]
    fn a_receipt_failure_names_the_commit_first() {
        let error = Error::Receipt {
            commit: "3f2a1b4".to_owned(),
            problem: "carries no receipt".to_owned(),
        };
        assert_eq!(error.to_string(), "3f2a1b4: carries no receipt");
    }

    #[test]
    fn the_error_is_a_std_error() {
        fn assert_std_error<E: std::error::Error>(_: &E) {}
        assert_std_error(&Error::Harness {
            problem: "b".to_owned(),
        });
    }
}
