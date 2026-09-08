//! The three git configuration entries a planted repository needs.
//!
//! `core.hooksPath` is what makes `.githooks/` the law rather than four files nobody runs.
//! The two receipts refspecs are what make a receipt travel: a note under
//! `refs/notes/plotplot/receipts` is not fetched or pushed with the branch unless the remote
//! is told to carry it.
//!
//! The reading and writing here go through the `git` binary rather than a config parser,
//! because git's own precedence, includes and conditional includes are the truth about what
//! a hook will see. Every command is scoped with `--local`, so a value inherited from the
//! planter's own global configuration is never mistaken for the repository's.

use std::path::Path;
use std::process::{Command, Output};

use crate::error::{Error, Result};
use crate::layout;

/// Where git looks for the hooks it runs.
pub const HOOKS_PATH_KEY: &str = "core.hooksPath";
/// What origin brings down on a fetch.
pub const FETCH_KEY: &str = "remote.origin.fetch";
/// What origin takes up on a push.
pub const PUSH_KEY: &str = "remote.origin.push";

/// The ref receipts live on, in-toto statements attached to the commits they attest.
pub const RECEIPTS_REF: &str = "refs/notes/plotplot/receipts";

/// The entries a planted repository carries, in the order `init` writes them.
///
/// The values are the same in every repository, so `root` is read only as the question being
/// asked: `core.hooksPath` is relative to the working tree git runs the hook in, and the two
/// refspecs name a ref, not a place.
pub fn desired(_root: &Path) -> Vec<(String, String)> {
    vec![
        (HOOKS_PATH_KEY.to_owned(), layout::GITHOOKS_DIR.to_owned()),
        (
            FETCH_KEY.to_owned(),
            format!("+{RECEIPTS_REF}:{RECEIPTS_REF}"),
        ),
        (
            PUSH_KEY.to_owned(),
            format!("{RECEIPTS_REF}:{RECEIPTS_REF}"),
        ),
    ]
}

/// Every value the repository at `root` currently holds for the three keys, absent keys
/// omitted and multi-valued keys in git's own order.
///
/// The values are the repository's own: a `core.hooksPath` the planter set globally is not
/// reported here, because it is not what this repository carries.
///
/// # Errors
///
/// [`Error::Git`] when git cannot be run, when `root` is not a repository, or when git
/// answers with bytes that are not UTF-8.
pub fn read(root: &Path) -> Result<Vec<(String, String)>> {
    let mut current = Vec::new();
    for (key, _) in desired(root) {
        let output = run(root, &["config", "--local", "--get-all", &key])?;
        // git answers 1 for a key it does not hold, which is not a failure to read.
        if output.status.code() == Some(1) {
            continue;
        }
        if !output.status.success() {
            return Err(failed(
                root,
                &["config", "--local", "--get-all", &key],
                &output,
            ));
        }
        let values = String::from_utf8(output.stdout).map_err(|_| Error::Git {
            command: command_line(root, &["config", "--local", "--get-all", &key]),
            stderr: "git answered with bytes that are not utf-8".to_owned(),
        })?;
        for value in values.lines() {
            current.push((key.clone(), value.to_owned()));
        }
    }
    Ok(current)
}

/// The entries of `desired` that `current` does not already hold, in `desired`'s order.
///
/// A key held with a different value counts as missing: `core.hooksPath` pointing somewhere
/// else is not the law this repository claims.
pub fn missing(
    desired: &[(String, String)],
    current: &[(String, String)],
) -> Vec<(String, String)> {
    desired
        .iter()
        .filter(|entry| !current.contains(entry))
        .cloned()
        .collect()
}

/// Set the entries the repository at `root` does not already hold.
///
/// The refspecs are added, because a remote carries several and the ones already there are
/// somebody's; `core.hooksPath` is set, because a repository has one. An entry already
/// present is left alone, so applying the same entries twice writes nothing the second time.
///
/// This is the one function in `plant` that changes the world.
///
/// # Errors
///
/// [`Error::Git`] when git cannot be run, when `root` is not a repository, or when git
/// refuses to set an entry.
pub fn apply(root: &Path, entries: &[(String, String)]) -> Result<()> {
    let current = read(root)?;
    for (key, value) in missing(entries, &current) {
        let args = if multi_valued(&key) {
            vec!["config", "--local", "--add", key.as_str(), value.as_str()]
        } else {
            vec!["config", "--local", key.as_str(), value.as_str()]
        };
        let output = run(root, &args)?;
        if !output.status.success() {
            return Err(failed(root, &args, &output));
        }
    }
    Ok(())
}

/// Whether git may hold more than one value for `key`, which decides between `--add` and a
/// plain set.
fn multi_valued(key: &str) -> bool {
    key == FETCH_KEY || key == PUSH_KEY
}

/// Run git in `root` and hand back what it said, whatever its status.
fn run(root: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|source| Error::Git {
            command: command_line(root, args),
            stderr: source.to_string(),
        })
}

/// The error for a git command that ran and refused.
fn failed(root: &Path, args: &[&str], output: &Output) -> Error {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    let stderr = if stderr.is_empty() {
        match output.status.code() {
            Some(code) => format!("git exited {code} and said nothing"),
            None => "git was killed by a signal".to_owned(),
        }
    } else {
        stderr.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    Error::Git {
        command: command_line(root, args),
        stderr,
    }
}

/// The command as an error should name it.
fn command_line(root: &Path, args: &[&str]) -> String {
    format!("git -C {} {}", root.display(), args.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    #[test]
    fn the_desired_entries_are_the_hooks_path_and_the_two_receipts_refspecs() {
        assert_eq!(
            desired(Path::new("/work/repo")),
            vec![
                entry("core.hooksPath", ".githooks"),
                entry(
                    "remote.origin.fetch",
                    "+refs/notes/plotplot/receipts:refs/notes/plotplot/receipts"
                ),
                entry(
                    "remote.origin.push",
                    "refs/notes/plotplot/receipts:refs/notes/plotplot/receipts"
                ),
            ]
        );
    }

    #[test]
    fn the_hooks_path_is_the_directory_the_layout_names() {
        let hooks_path = desired(Path::new("/work/repo"))
            .into_iter()
            .find(|(key, _)| key == HOOKS_PATH_KEY)
            .map(|(_, value)| value);
        assert_eq!(hooks_path.as_deref(), Some(layout::GITHOOKS_DIR));
    }

    #[test]
    fn the_desired_entries_do_not_depend_on_which_repository_is_asked() {
        assert_eq!(
            desired(Path::new("/work/repo")),
            desired(Path::new("/elsewhere"))
        );
    }

    #[test]
    fn only_the_refspecs_are_multi_valued() {
        assert!(multi_valued(FETCH_KEY));
        assert!(multi_valued(PUSH_KEY));
        assert!(!multi_valued(HOOKS_PATH_KEY));
    }

    #[test]
    fn everything_is_missing_from_an_empty_configuration() {
        let desired = desired(Path::new("/work/repo"));
        assert_eq!(missing(&desired, &[]), desired);
    }

    #[test]
    fn nothing_is_missing_once_every_entry_is_held() {
        let desired = desired(Path::new("/work/repo"));
        assert!(missing(&desired, &desired).is_empty());
    }

    #[test]
    fn a_key_held_with_another_value_is_still_missing() {
        let desired = desired(Path::new("/work/repo"));
        let current = vec![
            entry("core.hooksPath", ".git/hooks"),
            entry("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"),
        ];
        assert_eq!(missing(&desired, &current), desired);
    }

    #[test]
    fn a_refspec_beside_the_remotes_own_is_not_missing() {
        let desired = desired(Path::new("/work/repo"));
        let current = vec![
            entry("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"),
            desired[1].clone(),
        ];
        assert_eq!(
            missing(&desired, &current),
            vec![desired[0].clone(), desired[2].clone()]
        );
    }

    #[test]
    fn missing_keeps_the_order_it_was_asked_in() {
        let desired = desired(Path::new("/work/repo"));
        let current = vec![desired[1].clone()];
        assert_eq!(
            missing(&desired, &current),
            vec![desired[0].clone(), desired[2].clone()]
        );
    }

    #[test]
    fn the_command_line_an_error_names_is_the_one_that_ran() {
        assert_eq!(
            command_line(
                Path::new("/work/repo"),
                &["config", "--local", "core.hooksPath"]
            ),
            "git -C /work/repo config --local core.hooksPath"
        );
    }
}
