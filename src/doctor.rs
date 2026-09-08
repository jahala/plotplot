//! `plotplot doctor`, static mode: nine questions about a planted repository.
//!
//! A hook that silently never fires is the worst failure a boundary can have, so "is it
//! planted" has to be a command rather than a belief. This is the half of that command that
//! needs no harness running: it reads what is on disk, regenerates what the stem would write
//! today, and compares. Live mode, which drives a real session and proves each hook fires,
//! is a later node and is not here.
//!
//! Every check produces one [`Finding`], whether it passed or not, so the table is the same
//! nine lines every time and a check can never go quiet. Nothing here repairs anything:
//! `doctor` reports, `init` writes.

use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::bed::{Bed, GitHook};
use crate::bundle::{self, BundleInput, FileTree};
use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::lock::{self, Lock};
use crate::manifest::one_line;
use crate::plant::{garden_block, gitconfig, githooks};
use crate::{layout, manifest};

/// What `doctor` asks, in the order it prints the answers. The nine names are the table's
/// first column and the only names a caller has to match on.
pub const CHECKS: [&str; 9] = [
    "bundles",
    "stem binary",
    "hook entries",
    "core.hooksPath",
    "git hooks",
    "garden block",
    "receipts refspec",
    "judges",
    "codex trust",
];

/// One question, its answer, and what the answer rests on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub check: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// Where Codex records which projects it trusts, under the planter's home (§5).
pub const CODEX_CONFIG: &str = ".codex/config.toml";

/// What Codex calls a project it will load hooks for.
const TRUSTED: &str = "trusted";

/// What `doctor` says about a Codex project that has not been trusted yet. The stem cannot
/// grant trust, so this is an honest report and never a failure of the stem.
pub const AWAITING_TRUST: &str = "awaiting project trust";

fn ok(check: &'static str, detail: impl Into<String>) -> Finding {
    Finding {
        check,
        ok: true,
        detail: detail.into(),
    }
}

fn fail(check: &'static str, detail: impl Into<String>) -> Finding {
    Finding {
        check,
        ok: false,
        detail: detail.into(),
    }
}

/// A finding from a list of problems: every problem named, or the passing detail when there
/// are none. Nothing is dropped, and a passing check still says what it looked at.
fn verdict(check: &'static str, problems: Vec<String>, passed: impl Into<String>) -> Finding {
    if problems.is_empty() {
        ok(check, passed)
    } else {
        fail(check, problems.join("; "))
    }
}

/// `plotplot doctor`: the table on `stdout`, exit 0 when every check passed and 3 otherwise.
///
/// An unplanted repository gets one line and exit 3: there is nothing to prove yet, and
/// nine failures would say that nine times.
pub fn run(root: &Path, home: &Path, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    if let Some(absent) = unplanted(root) {
        let _ = writeln!(
            stdout,
            "not planted: {} is not there, so there is nothing to prove",
            absent.display()
        );
        return 3;
    }

    let findings = match run_static(root, home) {
        Ok(findings) => findings,
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return 1;
        }
    };

    if let Err(error) = write!(stdout, "{}", render(&findings)) {
        let _ = writeln!(stderr, "stdout: {error}");
        return 1;
    }
    if findings.iter().all(|finding| finding.ok) {
        0
    } else {
        3
    }
}

/// The file whose absence says this repository was never planted, or `None` when both the
/// lock and the cached beds are there.
fn unplanted(root: &Path) -> Option<PathBuf> {
    [layout::garden_lock(root), layout::beds_dir(root)]
        .into_iter()
        .find(|path| !path.exists())
}

/// Every check, in `CHECKS`' order.
///
/// The repository root and the home directory arrive as arguments; the one thing this asks
/// the environment for is [`bundle::running_stem`], the binary every bundle's `bin/plotplot`
/// has to be a copy of.
///
/// # Errors
///
/// Whatever reading the repository refuses: a lock the contracts reject, a manifest the stem
/// cannot plant, a git command that fails, a file that exists and cannot be read. A check
/// that simply did not pass is a [`Finding`], not an error.
pub fn run_static(root: &Path, home: &Path) -> Result<Vec<Finding>> {
    let stem = bundle::running_stem()?;

    let lock = lock::read_lock(root)?.ok_or_else(|| Error::Io {
        path: layout::garden_lock(root),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "doctor reads the season and the pinned judges from garden.lock",
        ),
    })?;
    let beds = manifest::load_beds(root)?;
    let skills = bundle::read_skills(root, &beds)?;
    let input = BundleInput {
        stem_version: crate::VERSION,
        season: &lock.season,
        beds: &beds,
        skills: &skills,
    };

    let mut generated = Vec::with_capacity(Harness::ALL.len());
    for harness in Harness::ALL {
        generated.push((harness, bundle::generate(harness, &input)?));
    }

    let configured = gitconfig::read(root)?;

    Ok(vec![
        bundles(root, &generated)?,
        stem_binary(root, &stem)?,
        hook_entries(root)?,
        hooks_path(&configured),
        git_hooks(root, &beds)?,
        block(root, &lock, &beds)?,
        receipts_refspec(root, &configured),
        judges(root, &lock)?,
        codex_trust(root, home)?,
    ])
}

/// The table `doctor` prints: check, verdict, detail, in three aligned columns.
pub fn render(findings: &[Finding]) -> String {
    let width = findings
        .iter()
        .map(|finding| finding.check.len())
        .max()
        .unwrap_or_default();

    let mut table = String::new();
    for finding in findings {
        let verdict = if finding.ok { "ok" } else { "fail" };
        let _ = writeln!(
            table,
            "{:width$}  {verdict:4}  {}",
            finding.check, finding.detail
        );
    }
    table
}

// ------------------------------------------------------------------ the nine checks

/// Every bundle is on disk and carries the bytes the stem would generate today.
fn bundles(root: &Path, generated: &[(Harness, FileTree)]) -> Result<Finding> {
    let mut problems = Vec::new();
    for (harness, tree) in generated {
        let directory = layout::bundle_dir(root, *harness);
        if !directory.is_dir() {
            problems.push(format!("{harness}: no bundle directory"));
            continue;
        }
        for difference in bundle::diff(tree, &directory)? {
            problems.push(format!("{harness}: {difference}"));
        }
    }
    Ok(verdict(
        CHECKS[0],
        problems,
        format!(
            "all {} carry the bytes the stem generates today",
            generated.len()
        ),
    ))
}

/// Every bundle's `bin/plotplot` is a copy of the binary now running, so a hook entry calls
/// the stem that generated it rather than one left behind by an older season.
fn stem_binary(root: &Path, stem: &Path) -> Result<Finding> {
    let running = file_digest(stem)?.ok_or_else(|| Error::Io {
        path: stem.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the running stem binary is no longer at the path it was started from",
        ),
    })?;

    let mut problems = Vec::new();
    for harness in Harness::ALL {
        let path = layout::bundle_dir(root, harness).join(bundle::STEM_BINARY);
        match file_digest(&path)? {
            None => problems.push(format!("{harness}: no {}", bundle::STEM_BINARY)),
            Some(found) if found != running => problems.push(format!(
                "{harness}: {} is not the running stem",
                bundle::STEM_BINARY
            )),
            Some(_) => {}
        }
    }
    Ok(verdict(
        CHECKS[1],
        problems,
        format!(
            "{} is the running stem in all {}",
            bundle::STEM_BINARY,
            Harness::ALL.len()
        ),
    ))
}

/// Every hook entry in every bundle calls the stem's dispatcher and nothing else: the same
/// command [`bundle::hook_command`] writes, with no argument added and none taken away.
fn hook_entries(root: &Path) -> Result<Finding> {
    let mut problems = Vec::new();
    let mut entries = 0usize;

    for harness in Harness::ALL {
        let path = layout::bundle_dir(root, harness).join(bundle::hooks_path(harness));
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                problems.push(format!("{harness}: no {}", bundle::hooks_path(harness)));
                continue;
            }
            Err(source) => return Err(Error::Io { path, source }),
        };
        let document: Value = match serde_json::from_str(&text) {
            Ok(document) => document,
            Err(error) => {
                problems.push(format!("{harness}: {}", one_line(&error.to_string())));
                continue;
            }
        };
        let Some(registered) = document.get("hooks").and_then(Value::as_object) else {
            problems.push(format!("{harness}: the hook file registers no events"));
            continue;
        };
        let before = entries;
        for (event, groups) in registered {
            let wanted = bundle::hook_command(harness, event);
            for command in commands(groups) {
                entries += 1;
                if command != wanted {
                    problems.push(format!("{harness} {event}: runs {command}"));
                }
            }
        }
        // Every entry naming the dispatcher is vacuously true of a bundle with no entries,
        // and a bundle that registers nothing is a boundary that never fires.
        if entries == before {
            problems.push(format!("{harness}: registers no hook entry at all"));
        }
    }

    Ok(verdict(
        CHECKS[2],
        problems,
        format!("{entries} entries, every one the stem's dispatcher"),
    ))
}

/// The commands one event's groups register, in the order the file spells them. A shape the
/// vendors do not use yields nothing, and the entry count then differs from the bundle the
/// stem generates, which the bundles check has already refused.
fn commands(groups: &Value) -> Vec<String> {
    groups
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .filter_map(|entry| entry.get("command").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

/// `core.hooksPath` points at `.githooks`, which is what makes those four files the law
/// rather than four files nobody runs.
fn hooks_path(configured: &[(String, String)]) -> Finding {
    let held: Vec<&str> = values(configured, gitconfig::HOOKS_PATH_KEY);
    if held.contains(&layout::GITHOOKS_DIR) {
        ok(CHECKS[3], layout::GITHOOKS_DIR)
    } else if held.is_empty() {
        fail(
            CHECKS[3],
            format!(
                "unset, so git runs its own hooks and not {}",
                layout::GITHOOKS_DIR
            ),
        )
    } else {
        fail(
            CHECKS[3],
            format!("{}, not {}", held.join(", "), layout::GITHOOKS_DIR),
        )
    }
}

/// All four hook files are there, runnable, and still what the stem renders for these beds.
fn git_hooks(root: &Path, beds: &[Bed]) -> Result<Finding> {
    let mut problems = Vec::new();
    for hook in GitHook::ALL {
        let name = hook.file_name();
        let path = layout::git_hook(root, hook);
        let found = match fs::read_to_string(&path) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                problems.push(format!("{name}: not there"));
                continue;
            }
            Err(source) => return Err(Error::Io { path, source }),
        };
        if found != githooks::render(hook, beds) {
            problems.push(format!("{name}: not what the stem renders for these beds"));
        }
        if !is_executable(&path)? {
            problems.push(format!("{name}: not executable, so git will skip it"));
        }
    }
    Ok(verdict(
        CHECKS[4],
        problems,
        format!("all {} present, executable and current", GitHook::ALL.len()),
    ))
}

/// `AGENTS.md` carries the garden block the stem renders for this season and these beds.
fn block(root: &Path, lock: &Lock, beds: &[Bed]) -> Result<Finding> {
    let path = layout::agents_md(root);
    let found = match fs::read_to_string(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(fail(
                CHECKS[5],
                format!("{} is not there", layout::AGENTS_MD),
            ));
        }
        Err(source) => return Err(Error::Io { path, source }),
    };

    let planted = found
        .find(garden_block::BEGIN)
        .zip(found.find(garden_block::END))
        .filter(|(begin, end)| begin <= end)
        .and_then(|(begin, end)| found.get(begin..end + garden_block::END.len()));

    Ok(match planted {
        None => fail(
            CHECKS[5],
            format!("{} carries no plotplot block", layout::AGENTS_MD),
        ),
        Some(planted) if planted == garden_block::render(&lock.season, beds) => ok(
            CHECKS[5],
            format!(
                "{} carries the block for season {}",
                layout::AGENTS_MD,
                lock.season
            ),
        ),
        Some(_) => fail(
            CHECKS[5],
            format!(
                "the block in {} is not the one the stem renders for season {}",
                layout::AGENTS_MD,
                lock.season
            ),
        ),
    })
}

/// Both refspecs carry the receipts ref, without which a receipt never leaves the machine
/// that wrote it.
fn receipts_refspec(root: &Path, configured: &[(String, String)]) -> Finding {
    let wanted = gitconfig::desired(root);
    let problems: Vec<String> = gitconfig::missing(&wanted, configured)
        .into_iter()
        .filter(|(key, _)| key == gitconfig::FETCH_KEY || key == gitconfig::PUSH_KEY)
        .map(|(key, _)| format!("{key} does not carry {}", gitconfig::RECEIPTS_REF))
        .collect();

    verdict(
        CHECKS[6],
        problems,
        format!(
            "{} and {} carry {}",
            gitconfig::FETCH_KEY,
            gitconfig::PUSH_KEY,
            gitconfig::RECEIPTS_REF
        ),
    )
}

/// Every judge the lock pins for this platform is in `.plotplot/bin/` with the bytes the
/// lock names, so the version that judges is the version the lock says judged.
fn judges(root: &Path, lock: &Lock) -> Result<Finding> {
    let platform = lock::platform();
    let mut problems = Vec::new();
    let mut checked = Vec::new();

    for (name, judge) in &lock.judges {
        let binary = layout::judge_binary(root, name);
        let present = binary.exists();

        match judge.platforms.get(platform) {
            Some(artifact) if is_archive(&artifact.url) => {
                checked.push(format!("{name} {}", judge.version));
                if !present {
                    problems.push(format!("{name}: not in {}", layout::PLOTPLOT_DIR));
                    continue;
                }
                // The bytes on disk came out of the archive, so what is compared is the
                // digest of the archive the lock pins, recorded beside the binary when it
                // was fetched.
                let recorded = layout::judge_binary(root, &format!("{name}.sha256"));
                match fs::read_to_string(&recorded) {
                    Ok(found) if found.trim() == artifact.sha256 => {}
                    Ok(_) => problems.push(format!(
                        "{name}: fetched from other bytes than the lock pins"
                    )),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        problems.push(format!("{name}: no record of the bytes it came from"));
                    }
                    Err(source) => {
                        return Err(Error::Io {
                            path: recorded,
                            source,
                        });
                    }
                }
            }
            Some(artifact) => {
                checked.push(format!("{name} {}", judge.version));
                match file_digest(&binary)? {
                    None => problems.push(format!("{name}: not in {}", layout::PLOTPLOT_DIR)),
                    Some(found) if found != artifact.sha256 => {
                        problems.push(format!("{name}: is not the bytes the lock pins"));
                    }
                    Some(_) => {}
                }
            }
            None if judge.npm.is_some() => {
                checked.push(format!("{name} {} (npm)", judge.version));
                if !present {
                    problems.push(format!("{name}: not in {}", layout::PLOTPLOT_DIR));
                }
            }
            None => checked.push(format!(
                "{name} {} (nothing pinned for {platform})",
                judge.version
            )),
        }
    }

    Ok(verdict(
        CHECKS[7],
        problems,
        if checked.is_empty() {
            "the lock pins no judges".to_owned()
        } else {
            checked.join(", ")
        },
    ))
}

/// Whether an artifact is an archive the binary was unpacked from, rather than the binary
/// itself. The lock names the bytes it fetched, which for an archive are not the bytes that
/// end up in `.plotplot/bin/`.
fn is_archive(url: &str) -> bool {
    [".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".tar.zst", ".zip"]
        .iter()
        .any(|extension| url.ends_with(extension))
}

/// Codex loads a project's hooks only once the project is trusted, and the stem cannot grant
/// that trust. So an untrusted project is reported, not failed; only a config file Codex
/// itself could not read is a failure.
fn codex_trust(root: &Path, home: &Path) -> Result<Finding> {
    let path = home.join(CODEX_CONFIG);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ok(CHECKS[8], AWAITING_TRUST));
        }
        Err(source) => {
            return Ok(fail(
                CHECKS[8],
                format!("{}: {}", path.display(), one_line(&source.to_string())),
            ));
        }
    };

    let config: Value = match toml::from_str(&text) {
        Ok(config) => config,
        Err(error) => {
            return Ok(fail(
                CHECKS[8],
                format!("{}: {}", path.display(), one_line(&error.to_string())),
            ));
        }
    };

    Ok(if trusts(&config, root) {
        ok(CHECKS[8], format!("{TRUSTED} in {CODEX_CONFIG}"))
    } else {
        ok(CHECKS[8], AWAITING_TRUST)
    })
}

/// Whether a parsed `config.toml` trusts `root`.
///
/// Both sides are resolved before they are compared, because Codex records whichever
/// spelling the session was opened at and two spellings can name one directory: on macOS
/// `/var` is a symlink to `/private/var`, so a repository under a temporary directory is
/// reached by two paths that are the same place.
fn trusts(config: &Value, root: &Path) -> bool {
    let Some(projects) = config.get("projects").and_then(Value::as_object) else {
        return false;
    };
    let root = resolved(root);
    projects.iter().any(|(project, entry)| {
        entry.get("trust_level").and_then(Value::as_str) == Some(TRUSTED)
            && resolved(Path::new(project)) == root
    })
}

/// A path as the filesystem resolves it, or as written when it cannot be resolved: a project
/// Codex trusted and somebody has since moved still has to compare as itself.
fn resolved(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

// ---------------------------------------------------------------------- the edge

/// The sha256 of a file as lowercase hex, or `None` when the file is not there.
///
/// # Errors
///
/// [`Error::Io`] when the file exists and cannot be read.
fn file_digest(path: &Path) -> Result<Option<String>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(Some(hex::encode(hasher.finalize())))
}

/// Every value the configuration holds for one key, in git's own order.
fn values<'a>(configured: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    configured
        .iter()
        .filter(|(held, _)| held == key)
        .map(|(_, value)| value.as_str())
        .collect()
}

/// Whether git will run this file as a hook.
///
/// # Errors
///
/// [`Error::Io`] when the file's permissions cannot be read.
#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    let mode = fs::metadata(path)
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?
        .permissions()
        .mode();
    Ok(mode & 0o111 != 0)
}

/// A platform with no executable bit gives the stem nothing to read, so `doctor` reports the
/// hook as unproven rather than claiming git will run it.
#[cfg(not(unix))]
fn is_executable(_path: &Path) -> Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings() -> Vec<Finding> {
        vec![
            ok("bundles", "three bundles"),
            fail("stem binary", "gemini: no bin/plotplot"),
            ok("receipts refspec", "both refspecs"),
        ]
    }

    /// Where a line's verdict starts: past the check name and the run of spaces after it.
    fn verdict_column(line: &str) -> usize {
        let cut = line.find("  ").expect("two spaces after the check");
        cut + line[cut..].len() - line[cut..].trim_start().len()
    }

    #[test]
    fn the_verdicts_and_the_details_each_start_in_one_column() {
        let table = render(&findings());
        let widest = "receipts refspec".len();

        for line in table.lines() {
            assert_eq!(verdict_column(line), widest + 2, "{line:?}");
            let detail = line
                .get(widest + 2..)
                .expect("a line at least as wide as its columns");
            assert_eq!(verdict_column(detail), "fail".len() + 2, "{line:?}");
        }
        assert_eq!(table.lines().count(), 3);
    }

    #[test]
    fn a_failing_check_reads_fail_and_a_passing_one_reads_ok() {
        let table = render(&findings());
        assert!(
            table.contains("bundles           ok    three bundles\n"),
            "{table}"
        );
        assert!(
            table.contains("stem binary       fail  gemini: no bin/plotplot\n"),
            "{table}"
        );
    }

    #[test]
    fn no_row_carries_a_double_space_inside_its_detail() {
        // The table is read column by column, so a detail with two spaces in it would split
        // into a column that is not there.
        for finding in findings() {
            assert!(!finding.detail.contains("  "), "{}", finding.detail);
        }
    }

    #[test]
    fn an_empty_table_renders_to_nothing() {
        assert_eq!(render(&[]), "");
    }

    #[test]
    fn a_verdict_names_every_problem_it_was_given() {
        let finding = verdict(
            CHECKS[0],
            vec![
                "claude: changed hooks/hooks.json".to_owned(),
                "codex: missing hooks.json".to_owned(),
            ],
            "all three",
        );
        assert!(!finding.ok);
        assert!(finding.detail.contains("claude"), "{}", finding.detail);
        assert!(finding.detail.contains("codex"), "{}", finding.detail);
    }

    #[test]
    fn archives_are_told_from_the_binaries_the_lock_pins_directly() {
        assert!(is_archive(
            "https://example.invalid/tilth-aarch64-apple-darwin.tar.gz"
        ));
        assert!(is_archive(
            "https://example.invalid/weeder-x86_64-pc-windows-msvc.zip"
        ));
        assert!(!is_archive(
            "https://example.invalid/weeder-aarch64-apple-darwin"
        ));
        assert!(!is_archive("https://example.invalid/weeder.exe"));
    }

    #[test]
    fn trust_is_read_from_codexs_own_shape() {
        let config: Value = toml::from_str(
            "[projects.\"/work/garden\"]\ntrust_level = \"trusted\"\n\n[projects.\"/work/other\"]\ntrust_level = \"untrusted\"\n",
        )
        .expect("a config codex could have written");
        assert!(trusts(&config, Path::new("/work/garden")));
        assert!(!trusts(&config, Path::new("/work/other")));
        assert!(!trusts(&config, Path::new("/work/unnamed")));
        assert!(!trusts(&Value::Null, Path::new("/work/garden")));
    }

    #[test]
    fn an_unplanted_repository_is_named_by_what_it_is_missing() {
        let root = tempfile::tempdir().expect("a temporary root");
        assert_eq!(
            unplanted(root.path()),
            Some(layout::garden_lock(root.path()))
        );

        std::fs::write(layout::garden_lock(root.path()), "season = \"2026.09\"\n")
            .expect("a lock file");
        assert_eq!(unplanted(root.path()), Some(layout::beds_dir(root.path())));

        std::fs::create_dir_all(layout::beds_dir(root.path())).expect("the beds directory");
        assert_eq!(unplanted(root.path()), None);
    }

    #[test]
    fn the_hooks_path_check_says_what_it_found_instead() {
        let held = vec![(gitconfig::HOOKS_PATH_KEY.to_owned(), ".husky".to_owned())];
        let finding = hooks_path(&held);
        assert!(!finding.ok);
        assert!(finding.detail.contains(".husky"), "{}", finding.detail);

        let held = vec![(
            gitconfig::HOOKS_PATH_KEY.to_owned(),
            layout::GITHOOKS_DIR.to_owned(),
        )];
        assert!(hooks_path(&held).ok);
        assert!(!hooks_path(&[]).ok);
    }

    #[test]
    fn the_commands_of_a_vendor_hook_file_are_read_in_order() {
        let groups: Value = serde_json::from_str(
            "[{\"hooks\": [{\"type\": \"command\", \"command\": \"first\"}, {\"command\": \"second\"}]}]",
        )
        .expect("a hook group");
        assert_eq!(commands(&groups), ["first", "second"]);
        assert!(commands(&Value::Null).is_empty());
        assert!(commands(&serde_json::json!([{"hooks": "not an array"}])).is_empty());
    }

    #[test]
    fn every_check_name_appears_once() {
        let mut sorted = CHECKS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), CHECKS.len());
    }
}
