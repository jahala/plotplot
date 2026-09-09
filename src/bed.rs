//! What the stem needs to know about a planted bed.
//!
//! This is the stem's internal view of a bed, produced once by [`crate::manifest`] from the
//! bed's `garden.json` and consumed by everything else. No other module reads a manifest, so
//! a change to the contracts lands in one place.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::harness::Harness;

/// A bed as the stem plants it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bed {
    pub name: String,
    pub version: String,
    /// The executable's file name inside the artifact and under `.plotplot/bin/`. `None` for
    /// a bed with neither `faces.cli` nor `install.binary_name`, which registers no hooks.
    pub binary: Option<String>,
    /// `SKILL.md`, relative to the bed's artifact root.
    pub skill: Option<PathBuf>,
    pub hooks: BTreeMap<Harness, Vec<HookEntry>>,
    pub git_hooks: Vec<GitHook>,
    /// Channel beds only, and only those that declared how to launch their server.
    pub mcp: Option<McpServer>,
    /// The gate command, SARIF 2.1.0 on stdout.
    pub check: Option<String>,
}

/// One hook registration: `"PreToolUse:Bash"` is event `PreToolUse` with matcher `Bash`.
///
/// The matcher is the stem's, not the vendor's: the dispatcher registers one entry per event
/// and applies the matcher itself, so a tool call never fires the dispatcher twice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookEntry {
    pub event: String,
    pub matcher: Option<String>,
}

/// The git hooks the stem installs into `core.hooksPath`, in the order git runs them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitHook {
    PreCommit,
    PrePush,
    PreRebase,
    PostCommit,
}

impl GitHook {
    /// Every git hook the stem installs.
    pub const ALL: [GitHook; 4] = [
        GitHook::PreCommit,
        GitHook::PrePush,
        GitHook::PreRebase,
        GitHook::PostCommit,
    ];

    /// The file name git looks for under `core.hooksPath`.
    pub fn file_name(self) -> &'static str {
        match self {
            GitHook::PreCommit => "pre-commit",
            GitHook::PrePush => "pre-push",
            GitHook::PreRebase => "pre-rebase",
            GitHook::PostCommit => "post-commit",
        }
    }

    /// The hook git spells this way, or `None` for a hook the stem does not install. The
    /// manifest schema also accepts `reference-transaction`, which lands here as `None` so
    /// the gap is refused loudly rather than dropped.
    pub fn from_file_name(name: &str) -> Option<GitHook> {
        GitHook::ALL
            .into_iter()
            .find(|hook| hook.file_name() == name)
    }
}

/// How the stem launches a channel bed's MCP server in a generated bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    /// The environment variable *names* the server reads; the stem writes `${NAME}` into the
    /// bundle and the planter's environment fills them.
    pub env: Vec<String>,
}

/// The beds that declared `event` on `harness`, in the order they were given.
pub fn beds_registered_for<'a>(beds: &'a [Bed], harness: Harness, event: &str) -> Vec<&'a Bed> {
    beds.iter()
        .filter(|bed| {
            bed.hooks
                .get(&harness)
                .is_some_and(|entries| entries.iter().any(|entry| entry.event == event))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bed(name: &str, hooks: &[(Harness, &[&str])]) -> Bed {
        Bed {
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            binary: Some(name.to_owned()),
            skill: None,
            hooks: hooks
                .iter()
                .map(|(harness, entries)| {
                    let entries = entries
                        .iter()
                        .map(|entry| match entry.split_once(':') {
                            Some((event, matcher)) => HookEntry {
                                event: event.to_owned(),
                                matcher: Some(matcher.to_owned()),
                            },
                            None => HookEntry {
                                event: (*entry).to_owned(),
                                matcher: None,
                            },
                        })
                        .collect();
                    (*harness, entries)
                })
                .collect(),
            git_hooks: Vec::new(),
            mcp: None,
            check: None,
        }
    }

    #[test]
    fn git_hook_file_names_are_gits_own() {
        assert_eq!(GitHook::PreCommit.file_name(), "pre-commit");
        assert_eq!(GitHook::PrePush.file_name(), "pre-push");
        assert_eq!(GitHook::PreRebase.file_name(), "pre-rebase");
        assert_eq!(GitHook::PostCommit.file_name(), "post-commit");
    }

    #[test]
    fn git_hooks_round_trip_through_their_file_names() {
        for hook in GitHook::ALL {
            assert_eq!(GitHook::from_file_name(hook.file_name()), Some(hook));
        }
        assert_eq!(GitHook::ALL.len(), 4);
        assert_eq!(GitHook::from_file_name("reference-transaction"), None);
        assert_eq!(GitHook::from_file_name("pre-receive"), None);
        assert_eq!(GitHook::from_file_name("PreCommit"), None);
    }

    #[test]
    fn git_hooks_sort_in_the_order_they_run() {
        let mut hooks = vec![
            GitHook::PostCommit,
            GitHook::PreRebase,
            GitHook::PreCommit,
            GitHook::PrePush,
        ];
        hooks.sort();
        assert_eq!(
            hooks,
            vec![
                GitHook::PreCommit,
                GitHook::PrePush,
                GitHook::PreRebase,
                GitHook::PostCommit
            ]
        );
    }

    #[test]
    fn beds_registered_for_selects_the_harness_and_the_event() {
        let beds = vec![
            bed("weeder", &[(Harness::Claude, &["PreToolUse:Bash", "Stop"])]),
            bed("tend2", &[(Harness::Claude, &["SessionStart"])]),
            bed(
                "petals",
                &[
                    (Harness::Claude, &["PreToolUse:Write"]),
                    (Harness::Gemini, &["BeforeTool:write_file"]),
                ],
            ),
        ];

        let names = |harness, event| {
            beds_registered_for(&beds, harness, event)
                .into_iter()
                .map(|bed| bed.name.as_str())
                .collect::<Vec<_>>()
        };

        assert_eq!(names(Harness::Claude, "PreToolUse"), ["weeder", "petals"]);
        assert_eq!(names(Harness::Claude, "Stop"), ["weeder"]);
        assert_eq!(names(Harness::Claude, "SessionStart"), ["tend2"]);
        assert_eq!(names(Harness::Gemini, "BeforeTool"), ["petals"]);
        assert!(names(Harness::Gemini, "Stop").is_empty());
        assert!(names(Harness::Codex, "PreToolUse").is_empty());
        assert!(names(Harness::Claude, "PostToolUse").is_empty());
    }

    #[test]
    fn beds_registered_for_keeps_the_order_it_was_given() {
        let beds = vec![
            bed("weeder", &[(Harness::Claude, &["Stop"])]),
            bed("apple", &[(Harness::Claude, &["Stop"])]),
        ];
        assert_eq!(
            beds_registered_for(&beds, Harness::Claude, "Stop")
                .into_iter()
                .map(|bed| bed.name.as_str())
                .collect::<Vec<_>>(),
            ["weeder", "apple"]
        );
    }

    #[test]
    fn a_bed_with_no_hooks_is_never_registered() {
        let beds = vec![bed("pollen", &[])];
        for harness in Harness::ALL {
            for event in harness.events() {
                assert!(beds_registered_for(&beds, harness, event).is_empty());
            }
        }
    }
}
