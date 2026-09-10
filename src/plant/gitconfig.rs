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
/// What origin brings down on a fetch. Read, never set: see [`desired`].
pub const FETCH_KEY: &str = "remote.origin.fetch";
/// What origin takes up on a push. The stem never sets it: with any push refspec configured,
/// `git push` sends only what the refspecs name and not the current branch, and the receipts
/// ref does not exist until the first seal, so a push refspec for it broke every push in a
/// freshly planted repository. Notes travel when a seal pushes the ref by name.
pub const PUSH_KEY: &str = "remote.origin.push";

/// The ref receipts live on, in-toto statements attached to the commits they attest.
pub const RECEIPTS_REF: &str = "refs/notes/plotplot/receipts";

/// The entries a planted repository carries: `core.hooksPath`, and nothing else.
///
/// No refspec for the receipts ref, in either direction. A fetch refspec naming a ref the
/// remote does not have makes `git fetch` fail outright until the first receipt is pushed,
/// and a push refspec makes `git push` send that ref alone and not the branch; both were
/// met on the first planted repository. Notes travel when asked: [`fetch_receipts`] brings
/// the ref by name when the remote has it, and a seal pushes it by name.
///
/// The value is the same in every repository, so `root` is read only as the question being
/// asked: `core.hooksPath` is relative to the working tree git runs the hook in.
pub fn desired(_root: &Path) -> Vec<(String, String)> {
    vec![(HOOKS_PATH_KEY.to_owned(), layout::GITHOOKS_DIR.to_owned())]
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
    let keys: Vec<String> = desired(root).into_iter().map(|(key, _)| key).collect();
    read_keys(root, &keys)
}

/// Every value the repository at `root` holds for `keys`, absent keys omitted.
///
/// # Errors
///
/// [`Error::Git`] when git cannot read the configuration.
pub fn read_keys(root: &Path, keys: &[String]) -> Result<Vec<(String, String)>> {
    let mut current = Vec::new();
    for key in keys {
        let key = key.as_str();
        let output = run(root, &["config", "--local", "--get-all", key])?;
        // git answers 1 for a key it does not hold, which is not a failure to read.
        if output.status.code() == Some(1) {
            continue;
        }
        if !output.status.success() {
            return Err(failed(
                root,
                &["config", "--local", "--get-all", key],
                &output,
            ));
        }
        let values = String::from_utf8(output.stdout).map_err(|_| Error::Git {
            command: command_line(root, &["config", "--local", "--get-all", key]),
            stderr: "git answered with bytes that are not utf-8".to_owned(),
        })?;
        for value in values.lines() {
            current.push((key.to_owned(), value.to_owned()));
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

/// What [`fetch_receipts`] found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipts {
    /// The remote has the ref and it is now local, new or moved.
    Fetched,
    /// The remote has the ref and the local one already matched it.
    AlreadyHere,
    /// The remote has no receipts ref yet; nothing to bring.
    NoneOnRemote,
    /// The repository has no `origin`; there is nowhere to ask.
    NoOrigin,
    /// Origin could not be asked (offline, gone, refusing); what git said, on one line.
    Unreachable(String),
}

/// Bring `refs/notes/plotplot/receipts` from origin by name, when origin has it.
///
/// This is the "when asked" of the receipts plan: nothing in the repository's config makes
/// git fetch or push the ref on its own. A remote that cannot be asked is reported, not
/// failed on: planting a repository offline is a thing that happens.
///
/// # Errors
///
/// [`Error::Git`] when git itself cannot run, or a fetch of a ref the remote just listed
/// fails.
pub fn fetch_receipts(root: &Path) -> Result<Receipts> {
    let remotes = run(root, &["remote"])?;
    if !remotes.status.success() {
        return Err(failed(root, &["remote"], &remotes));
    }
    if !String::from_utf8_lossy(&remotes.stdout)
        .lines()
        .any(|line| line.trim() == "origin")
    {
        return Ok(Receipts::NoOrigin);
    }
    let listing = ["ls-remote", "--exit-code", "origin", RECEIPTS_REF];
    let listed = run(root, &listing)?;
    match listed.status.code() {
        Some(0) => {}
        // `--exit-code`: 2 when the remote answered and has no such ref.
        Some(2) => return Ok(Receipts::NoneOnRemote),
        _ => {
            let Error::Git { stderr, .. } = failed(root, &listing, &listed) else {
                return Ok(Receipts::Unreachable(
                    "origin could not be asked".to_owned(),
                ));
            };
            return Ok(Receipts::Unreachable(stderr));
        }
    }
    let before = local_ref(root, RECEIPTS_REF)?;
    let refspec = format!("+{RECEIPTS_REF}:{RECEIPTS_REF}");
    let fetching = ["fetch", "--quiet", "origin", refspec.as_str()];
    let fetched = run(root, &fetching)?;
    if !fetched.status.success() {
        return Err(failed(root, &fetching, &fetched));
    }
    let after = local_ref(root, RECEIPTS_REF)?;
    Ok(if before == after {
        Receipts::AlreadyHere
    } else {
        Receipts::Fetched
    })
}

/// The commit a local ref points at, or `None` when the ref is not there.
fn local_ref(root: &Path, reference: &str) -> Result<Option<String>> {
    let args = ["rev-parse", "--verify", "--quiet", reference];
    let output = run(root, &args)?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        ))
    } else {
        Ok(None)
    }
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
    fn the_desired_entries_are_the_hooks_path_and_no_refspec_at_all() {
        let desired = desired(Path::new("/work/repo"));
        assert_eq!(desired, vec![entry("core.hooksPath", ".githooks")]);
        assert!(
            desired
                .iter()
                .all(|(key, _)| key != PUSH_KEY && key != FETCH_KEY),
            "a fetch refspec for an absent ref fails every fetch; a push refspec sends the ref alone"
        );
    }

    #[test]
    fn fetching_receipts_from_a_remote_that_has_none_is_not_an_error() {
        let origin = tempfile::tempdir().expect("an origin");
        assert!(
            std::process::Command::new("git")
                .args(["init", "-q", "--bare"])
                .arg(origin.path())
                .status()
                .expect("git runs")
                .success()
        );
        let repo = tempfile::tempdir().expect("a repository");
        for args in [
            vec!["init", "-q"],
            vec![
                "remote",
                "add",
                "origin",
                origin.path().to_str().expect("utf-8"),
            ],
        ] {
            assert!(
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(repo.path())
                    .args(&args)
                    .status()
                    .expect("git runs")
                    .success()
            );
        }
        assert_eq!(
            fetch_receipts(repo.path()).expect("a remote without receipts is a plain answer"),
            Receipts::NoneOnRemote
        );
        let alone = tempfile::tempdir().expect("a repository with no origin");
        assert!(
            std::process::Command::new("git")
                .args(["init", "-q"])
                .arg(alone.path())
                .status()
                .expect("git runs")
                .success()
        );
        assert_eq!(
            fetch_receipts(alone.path()).expect("no origin is a plain answer"),
            Receipts::NoOrigin
        );
    }

    fn git_ok(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn an_origin_that_cannot_be_asked_is_reported_and_not_an_error() {
        let repo = tempfile::tempdir().expect("a repository");
        git_ok(repo.path(), &["init", "-q"]);
        git_ok(
            repo.path(),
            &[
                "remote",
                "add",
                "origin",
                "/nowhere/that/exists/plotplot.git",
            ],
        );
        match fetch_receipts(repo.path()).expect("an unreachable origin is a plain answer") {
            Receipts::Unreachable(why) => assert!(!why.is_empty()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn receipts_on_the_remote_are_fetched_once_and_then_already_here() {
        let origin = tempfile::tempdir().expect("an origin");
        git_ok(origin.path(), &["init", "-q", "--bare"]);
        let source = tempfile::tempdir().expect("a source repository");
        git_ok(source.path(), &["init", "-q"]);
        git_ok(
            source.path(),
            &["config", "user.email", "fit@plotplot.invalid"],
        );
        git_ok(source.path(), &["config", "user.name", "plotplot fit"]);
        git_ok(
            source.path(),
            &["commit", "-q", "--allow-empty", "-m", "sealed"],
        );
        git_ok(
            source.path(),
            &[
                "notes",
                "--ref",
                RECEIPTS_REF,
                "add",
                "-m",
                "a receipt",
                "HEAD",
            ],
        );
        let origin_url = origin.path().to_str().expect("utf-8");
        git_ok(source.path(), &["remote", "add", "origin", origin_url]);
        git_ok(
            source.path(),
            &["push", "-q", "origin", "HEAD:refs/heads/main"],
        );
        git_ok(source.path(), &["push", "-q", "origin", RECEIPTS_REF]);

        let planted = tempfile::tempdir().expect("a planted clone");
        git_ok(planted.path(), &["init", "-q"]);
        git_ok(planted.path(), &["remote", "add", "origin", origin_url]);
        assert_eq!(
            fetch_receipts(planted.path()).expect("the ref is fetched"),
            Receipts::Fetched
        );
        assert!(
            local_ref(planted.path(), RECEIPTS_REF)
                .expect("git runs")
                .is_some()
        );
        assert_eq!(
            fetch_receipts(planted.path()).expect("the ref is asked for again"),
            Receipts::AlreadyHere
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
            entry("core.hooksPath", ".githooks"),
        ];
        assert!(missing(&desired, &current).is_empty());
    }

    #[test]
    fn missing_keeps_the_order_it_was_asked_in() {
        let wanted = vec![entry("a", "1"), entry("b", "2"), entry("c", "3")];
        let current = vec![entry("b", "2")];
        assert_eq!(
            missing(&wanted, &current),
            vec![entry("a", "1"), entry("c", "3")]
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
