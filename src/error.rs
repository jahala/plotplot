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
            Error::Harness { problem } => write!(f, "{problem}"),
            Error::Git { command, stderr } => write!(f, "{command}: {stderr}"),
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
            | Error::Bed { .. } => None,
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
        ];
        for error in &errors {
            assert!(!error.to_string().contains('\n'), "{error}");
        }
    }

    #[test]
    fn the_error_is_a_std_error() {
        fn assert_std_error<E: std::error::Error>(_: &E) {}
        assert_std_error(&Error::Harness {
            problem: "b".to_owned(),
        });
    }
}
