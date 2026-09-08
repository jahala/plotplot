//! The four git hooks a planted repository carries under `core.hooksPath`.
//!
//! These are the boundary: the place where a claim stops being a claim because git refuses
//! the commit. So the scripts are POSIX `sh` with no interpreter to install, they resolve
//! their own repository root rather than trusting the directory git happened to run them in,
//! they call judges only through `.plotplot/bin/` so the version that judged is the version
//! the lock names, and they exit on the first refusal rather than collecting opinions.
//!
//! A hook no bed declared is still written, so `core.hooksPath` holds four files and
//! `doctor` has four files to check.

use crate::bed::{Bed, GitHook};
use crate::layout;

/// The stem's own binary, which `post-commit` calls to seal the session's receipt.
const STEM_BINARY: &str = env!("CARGO_PKG_NAME");

/// The judges' directory as a shell script spells it: relative to the repository root, with
/// forward slashes. The same directory as [`crate::layout::bin_dir`].
fn bin_dir() -> String {
    format!("{}/bin", layout::PLOTPLOT_DIR)
}

/// The `sh` script for one git hook, given every planted bed.
///
/// Only the beds that declared this hook and carry a binary appear, in the order they are
/// given. `post-commit` seals the receipt first and runs those beds after it; the other
/// three hooks run the beds alone.
pub fn render(hook: GitHook, beds: &[Bed]) -> String {
    let name = hook.file_name();
    let guards: Vec<&str> = beds
        .iter()
        .filter(|bed| bed.git_hooks.contains(&hook))
        .filter_map(|bed| bed.binary.as_deref())
        .collect();

    let mut script = String::new();
    script.push_str("#!/bin/sh\n");
    script.push_str(&format!(
        "# Written by plotplot: the {name} hook of a planted garden. Do not edit by hand;\n"
    ));
    script.push_str("# `plotplot init` rewrites this file from the beds' manifests.\n");
    script.push_str("set -eu\n\n");
    script.push_str("root=$(git rev-parse --show-toplevel)\n");
    script.push_str("cd \"$root\"\n\n");

    if hook == GitHook::PostCommit {
        script
            .push_str("# The receipt is the stem's own, and it is sealed before any bed speaks.\n");
        script.push_str(&format!(
            "if [ -x \"$root/{bin}/{STEM_BINARY}\" ]; then\n",
            bin = bin_dir()
        ));
        script.push_str(&format!(
            "\t\"$root/{bin}/{STEM_BINARY}\" receipt seal || exit $?\n",
            bin = bin_dir()
        ));
        script.push_str("fi\n\n");
    }

    if guards.is_empty() {
        script.push_str(&format!("# No planted bed declared {name}.\n\n"));
    } else {
        if hook == GitHook::PrePush {
            script.push_str(
                "# git sends the refs being pushed on stdin, and every guard reads the same bytes.\n",
            );
            script.push_str("refs=$(cat; printf x)\n");
            script.push_str("refs=${refs%x}\n\n");
        }
        script.push_str(&format!(
            "# The first guard that refuses ends the {name}, with its own status.\n"
        ));
        for binary in guards {
            let call = format!(
                "\"$root/{bin}/{binary}\" guard {name} \"$@\"",
                bin = bin_dir()
            );
            if hook == GitHook::PrePush {
                script.push_str(&format!("printf '%s' \"$refs\" | {call} || exit $?\n"));
            } else {
                script.push_str(&format!("{call} || exit $?\n"));
            }
        }
        script.push('\n');
    }

    script.push_str("exit 0\n");
    script
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn bed(name: &str, binary: Option<&str>, git_hooks: &[GitHook]) -> Bed {
        Bed {
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            binary: binary.map(str::to_owned),
            skill: None,
            hooks: BTreeMap::new(),
            git_hooks: git_hooks.to_vec(),
            mcp: None,
            check: None,
        }
    }

    fn beds() -> Vec<Bed> {
        vec![
            bed(
                "weeder",
                Some("weeder"),
                &[GitHook::PreCommit, GitHook::PrePush, GitHook::PreRebase],
            ),
            bed("tend2", Some("tend2"), &[GitHook::PostCommit]),
            bed("tilth", Some("tilth"), &[]),
        ]
    }

    #[test]
    fn the_scripts_bin_directory_is_the_layouts_bin_directory() {
        assert_eq!(bin_dir(), layout::bin_dir(Path::new("")).to_string_lossy());
    }

    #[test]
    fn every_script_opens_with_the_shebang_and_a_comment_claiming_the_file() {
        for hook in GitHook::ALL {
            let script = render(hook, &beds());
            let mut lines = script.lines();
            assert_eq!(lines.next(), Some("#!/bin/sh"));
            let claim = lines.next().unwrap_or_default();
            assert!(claim.starts_with("# Written by plotplot"), "{claim}");
            assert!(claim.contains(hook.file_name()), "{claim}");
            assert!(claim.contains("Do not edit by hand"), "{claim}");
        }
    }

    #[test]
    fn every_script_refuses_a_silent_failure_and_finds_its_own_root() {
        for hook in GitHook::ALL {
            let script = render(hook, &beds());
            assert!(script.contains("\nset -eu\n"), "{script}");
            assert!(
                script.contains("root=$(git rev-parse --show-toplevel)\n"),
                "{script}"
            );
            assert!(script.ends_with("exit 0\n"), "{script}");
        }
    }

    #[test]
    fn a_hook_calls_the_beds_that_declared_it_and_no_others() {
        let script = render(GitHook::PreCommit, &beds());
        assert!(
            script.contains("\"$root/.plotplot/bin/weeder\" guard pre-commit \"$@\" || exit $?\n"),
            "{script}"
        );
        assert!(!script.contains("tend2"), "{script}");
        assert!(!script.contains("tilth"), "{script}");
    }

    #[test]
    fn the_beds_keep_the_order_they_were_given() {
        let beds = vec![
            bed("weeder", Some("weeder"), &[GitHook::PreCommit]),
            bed("apple", Some("apple"), &[GitHook::PreCommit]),
        ];
        let script = render(GitHook::PreCommit, &beds);
        let weeder = script.find("bin/weeder").expect("the weeder call");
        let apple = script.find("bin/apple").expect("the apple call");
        assert!(weeder < apple, "{script}");
    }

    #[test]
    fn pre_push_buffers_the_refs_so_every_guard_reads_them() {
        let beds = vec![
            bed("weeder", Some("weeder"), &[GitHook::PrePush]),
            bed("tend2", Some("tend2"), &[GitHook::PrePush]),
        ];
        let script = render(GitHook::PrePush, &beds);
        assert!(script.contains("refs=$(cat; printf x)\n"), "{script}");
        assert!(script.contains("refs=${refs%x}\n"), "{script}");
        assert_eq!(
            script.matches("printf '%s' \"$refs\" |").count(),
            2,
            "{script}"
        );
    }

    #[test]
    fn the_hooks_with_no_stdin_of_their_own_do_not_read_any() {
        for hook in [GitHook::PreCommit, GitHook::PreRebase, GitHook::PostCommit] {
            let script = render(hook, &beds());
            assert!(!script.contains("$(cat"), "{}: {script}", hook.file_name());
        }
    }

    #[test]
    fn post_commit_seals_the_receipt_first_and_only_when_the_stem_is_fetched() {
        let script = render(GitHook::PostCommit, &beds());
        assert!(
            script.contains("if [ -x \"$root/.plotplot/bin/plotplot\" ]; then\n"),
            "{script}"
        );
        let seal = script
            .find("\"$root/.plotplot/bin/plotplot\" receipt seal || exit $?")
            .expect("the seal");
        let guard = script
            .find("\"$root/.plotplot/bin/tend2\" guard post-commit")
            .expect("the tend2 guard");
        assert!(seal < guard, "{script}");
    }

    #[test]
    fn post_commit_still_seals_when_no_bed_declared_it() {
        let script = render(GitHook::PostCommit, &[]);
        assert!(script.contains("receipt seal"), "{script}");
        assert!(
            script.contains("# No planted bed declared post-commit.\n"),
            "{script}"
        );
    }

    #[test]
    fn a_bed_without_a_binary_cannot_be_called_and_is_not() {
        let beds = vec![bed("petals", None, &[GitHook::PreCommit])];
        let script = render(GitHook::PreCommit, &beds);
        assert!(!script.contains("petals"), "{script}");
        assert!(
            script.contains("# No planted bed declared pre-commit.\n"),
            "{script}"
        );
    }

    #[test]
    fn no_two_hooks_render_the_same_script_even_with_no_beds() {
        let mut scripts: Vec<String> = GitHook::ALL
            .into_iter()
            .map(|hook| render(hook, &[]))
            .collect();
        scripts.sort();
        scripts.dedup();
        assert_eq!(scripts.len(), 4);
    }
}
