//! Where a planted repository keeps things.
//!
//! Every path §3 of `docs/prompts/stem-build-2026-09.md` names is a function of the
//! repository root and nothing else, and no other module spells one. Human-authored files
//! stay at the root and under `docs/`; everything the stem writes for itself goes under
//! `.plotplot/`.

use std::path::{Path, PathBuf};

use crate::bed::GitHook;
use crate::harness::Harness;

/// The directory holding everything the stem writes for itself.
pub const PLOTPLOT_DIR: &str = ".plotplot";
/// The directory `core.hooksPath` points at.
pub const GITHOOKS_DIR: &str = ".githooks";
/// The lockfile's name, at the repository root and committed.
pub const GARDEN_LOCK: &str = "garden.lock";
/// The file carrying the garden block.
pub const AGENTS_MD: &str = "AGENTS.md";
/// The manifest file name inside a cached bed directory.
pub const GARDEN_JSON: &str = "garden.json";

/// `garden.lock`, the pinned judges. Committed.
pub fn garden_lock(root: &Path) -> PathBuf {
    root.join(GARDEN_LOCK)
}

/// `AGENTS.md`, which carries the garden block between its markers.
pub fn agents_md(root: &Path) -> PathBuf {
    root.join(AGENTS_MD)
}

/// `.githooks/`, what `core.hooksPath` is set to. Committed.
pub fn githooks_dir(root: &Path) -> PathBuf {
    root.join(GITHOOKS_DIR)
}

/// One git hook file under `.githooks/`.
pub fn git_hook(root: &Path, hook: GitHook) -> PathBuf {
    githooks_dir(root).join(hook.file_name())
}

/// `.plotplot/`, the machine-written tree.
pub fn plotplot_dir(root: &Path) -> PathBuf {
    root.join(PLOTPLOT_DIR)
}

/// `.plotplot/bin/`, the fetched judges. Ignored by git.
pub fn bin_dir(root: &Path) -> PathBuf {
    plotplot_dir(root).join("bin")
}

/// One fetched judge, named by its executable's file name.
pub fn judge_binary(root: &Path, binary: &str) -> PathBuf {
    bin_dir(root).join(binary)
}

/// `.plotplot/beds/`, the manifests read out of each verified artifact. Ignored by git.
pub fn beds_dir(root: &Path) -> PathBuf {
    plotplot_dir(root).join("beds")
}

/// One planted bed's cached directory.
pub fn bed_dir(root: &Path, bed: &str) -> PathBuf {
    beds_dir(root).join(bed)
}

/// One planted bed's `garden.json`.
pub fn bed_manifest(root: &Path, bed: &str) -> PathBuf {
    bed_dir(root, bed).join(GARDEN_JSON)
}

/// `.plotplot/bundles/`, the three generated vendor bundles.
pub fn bundles_dir(root: &Path) -> PathBuf {
    plotplot_dir(root).join("bundles")
}

/// One harness's generated bundle.
pub fn bundle_dir(root: &Path, harness: Harness) -> PathBuf {
    bundles_dir(root).join(harness.name())
}

/// `.plotplot/friction/`, the friction journal and its per-session state. Ignored by git.
pub fn friction_dir(root: &Path) -> PathBuf {
    plotplot_dir(root).join("friction")
}

/// One month's friction journal, `month` being `yyyy-mm`.
pub fn friction_journal(root: &Path, month: &str) -> PathBuf {
    friction_dir(root).join(format!("{month}.jsonl"))
}

/// `.plotplot/friction/state/`, the emitter's per-session counters.
pub fn friction_state_dir(root: &Path) -> PathBuf {
    friction_dir(root).join("state")
}

/// One session's friction state.
pub fn friction_state(root: &Path, session_id: &str) -> PathBuf {
    friction_state_dir(root).join(format!("{session_id}.json"))
}

/// `.plotplot/receipts/`, the receipt drafts waiting to be sealed. Ignored by git.
pub fn receipts_dir(root: &Path) -> PathBuf {
    plotplot_dir(root).join("receipts")
}

/// `.plotplot/receipts/drafts/`.
pub fn receipt_drafts_dir(root: &Path) -> PathBuf {
    receipts_dir(root).join("drafts")
}

/// One session's receipt draft.
pub fn receipt_draft(root: &Path, session_id: &str) -> PathBuf {
    receipt_drafts_dir(root).join(format!("{session_id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every path §3 of `docs/prompts/stem-build-2026-09.md` spells, as a relative path and
    /// the call that produces it. If §3 changes, this table changes with it.
    fn table(root: &Path) -> Vec<(&'static str, PathBuf)> {
        vec![
            ("garden.lock", garden_lock(root)),
            ("AGENTS.md", agents_md(root)),
            (".githooks", githooks_dir(root)),
            (".githooks/pre-commit", git_hook(root, GitHook::PreCommit)),
            (".githooks/pre-push", git_hook(root, GitHook::PrePush)),
            (".githooks/pre-rebase", git_hook(root, GitHook::PreRebase)),
            (".githooks/post-commit", git_hook(root, GitHook::PostCommit)),
            (".plotplot", plotplot_dir(root)),
            (".plotplot/bin", bin_dir(root)),
            (".plotplot/bin/weeder", judge_binary(root, "weeder")),
            (".plotplot/beds", beds_dir(root)),
            (".plotplot/beds/weeder", bed_dir(root, "weeder")),
            (
                ".plotplot/beds/weeder/garden.json",
                bed_manifest(root, "weeder"),
            ),
            (".plotplot/bundles", bundles_dir(root)),
            (
                ".plotplot/bundles/claude",
                bundle_dir(root, Harness::Claude),
            ),
            (
                ".plotplot/bundles/gemini",
                bundle_dir(root, Harness::Gemini),
            ),
            (".plotplot/bundles/codex", bundle_dir(root, Harness::Codex)),
            (".plotplot/friction", friction_dir(root)),
            (
                ".plotplot/friction/2026-09.jsonl",
                friction_journal(root, "2026-09"),
            ),
            (".plotplot/friction/state", friction_state_dir(root)),
            (
                ".plotplot/friction/state/6f3c1b2a.json",
                friction_state(root, "6f3c1b2a"),
            ),
            (".plotplot/receipts", receipts_dir(root)),
            (".plotplot/receipts/drafts", receipt_drafts_dir(root)),
            (
                ".plotplot/receipts/drafts/6f3c1b2a.json",
                receipt_draft(root, "6f3c1b2a"),
            ),
        ]
    }

    #[test]
    fn every_path_is_the_root_joined_with_its_documented_relative_path() {
        for root in [
            Path::new("/work/repo"),
            Path::new("/tmp/fixture with spaces"),
            Path::new("relative/root"),
        ] {
            for (relative, produced) in table(root) {
                assert_eq!(produced, root.join(relative), "{relative}");
                assert!(produced.starts_with(root), "{relative}");
            }
        }
    }

    #[test]
    fn the_table_covers_every_path_section_three_names() {
        assert_eq!(table(Path::new("/work/repo")).len(), 24);
    }

    #[test]
    fn the_git_hook_files_live_in_the_hooks_directory() {
        let root = Path::new("/work/repo");
        for hook in GitHook::ALL {
            assert_eq!(
                git_hook(root, hook),
                githooks_dir(root).join(hook.file_name())
            );
        }
    }

    #[test]
    fn the_bundle_directory_is_named_by_the_harness() {
        let root = Path::new("/work/repo");
        for harness in Harness::ALL {
            assert_eq!(
                bundle_dir(root, harness),
                bundles_dir(root).join(harness.name())
            );
        }
    }

    #[test]
    fn the_machine_written_paths_all_sit_under_the_plotplot_directory() {
        let root = Path::new("/work/repo");
        let plotplot = plotplot_dir(root);
        for path in [
            bin_dir(root),
            beds_dir(root),
            bundles_dir(root),
            friction_dir(root),
            friction_state_dir(root),
            receipts_dir(root),
            receipt_drafts_dir(root),
        ] {
            assert!(path.starts_with(&plotplot), "{}", path.display());
        }
    }

    #[test]
    fn a_bed_manifest_sits_in_that_beds_own_directory() {
        let root = Path::new("/work/repo");
        assert_eq!(
            bed_manifest(root, "tend2"),
            bed_dir(root, "tend2").join("garden.json")
        );
    }
}
