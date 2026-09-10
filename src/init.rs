//! `plotplot init`: plant the garden in this repository, and change nothing on the way back.
//!
//! `init` composes what the other faces already do and adds nothing they do: the lock
//! resolves the judges, `bundle` generates the three vendor bundles, `plant` renders the
//! garden block, the git hooks and the git configuration, `install` puts each bundle where
//! its vendor looks. What lives here is the order, the choosing, and the one rule the whole
//! face turns on: every write compares the bytes first, so a second run writes nothing and
//! says `nothing to do`.
//!
//! Two things it never does. It never installs anything globally: a judge goes into
//! `.plotplot/bin/`, a bundle into `.plotplot/bundles/`, and the vendors are asked for
//! project scope. And it never edits a user-scope settings file; where a vendor's own
//! install command records something at user scope, the note says so and the stem still
//! wrote nothing there.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::bundle::{self, BundleInput, FileTree};
use crate::cli::{InitArgs, Profile};
use crate::error::{Error, Result};
use crate::fetch::Https;
use crate::harness::Harness;
use crate::lock::{self, JudgeState, Lock};
use crate::plant::{garden_block, gitconfig, githooks};
use crate::platform::{self, Gh, GhCli, Unapplied, Unreachable};
use crate::{install, layout, manifest};

/// What the minimal profile plants: the judge of the diff, and nothing else
/// (jahala/plotplot issue 17). Everything else the minimal profile writes, it writes for
/// weeder: its hooks, the git hooks it declared, the garden block that names it.
pub const MINIMAL_BEDS: [&str; 1] = ["weeder"];

/// Where the pull request gate lives.
pub const WORKFLOW: &str = ".github/workflows/plotplot-check.yml";

/// What `init` prints when a run changed nothing.
pub const NOTHING: &str = "nothing to do";

/// Exit codes. Zero is a planted repository; the other three are the three ways this face
/// stops, and each is a different thing for a planter to do about it.
pub const PLANTED: i32 = 0;
/// The command itself was wrong: a flag the run needed was not given, or `--github` was asked
/// of a repository it cannot reach (no `gh`, or no origin on github.com).
pub const USAGE: i32 = 2;
/// The lock could not be resolved, so nothing was planted. Fail closed.
pub const REFUSED: i32 = 3;
/// Something the run needed could not be read or written.
pub const FAILED: i32 = 1;

/// Which harnesses this machine and this repository already know about.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Detected {
    /// The harnesses whose CLI is on the search path.
    pub on_path: Vec<Harness>,
    /// The harnesses this repository already carries a project configuration for.
    pub configured: Vec<Harness>,
}

/// What one run of `init` did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Planted {
    /// One entry per file or setting whose bytes changed, in the order they were written.
    pub changed: Vec<String>,
    /// What the planter has to know: what a vendor cannot do at project scope, what waits
    /// on their trust, what a vendor recorded outside the repository.
    pub notes: Vec<String>,
}

// ------------------------------------------------------------------------- detection

/// The CLI each harness is called by on a search path.
fn cli_name(harness: Harness) -> &'static str {
    // The three vendors' commands are their own names; this says so once rather than
    // leaving `Harness::name` to mean two things.
    harness.name()
}

/// What this repository already carries for a harness: a project settings file for Claude
/// and Gemini, a project directory for Codex.
fn project_marker(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude => install::CLAUDE_SETTINGS,
        Harness::Gemini => install::GEMINI_SETTINGS,
        Harness::Codex => ".codex",
    }
}

/// Which harnesses are on `search_path`, and which this repository already configures.
///
/// The search path arrives as an argument rather than being read here: this is a fact of
/// the environment the process was started in, and the library is told it.
pub fn detect(root: &Path, search_path: Option<&OsStr>) -> Detected {
    let mut on_path = Vec::new();
    let mut configured = Vec::new();
    for harness in Harness::ALL {
        if find_on_path(cli_name(harness), search_path).is_some() {
            on_path.push(harness);
        }
        if root.join(project_marker(harness)).exists() {
            configured.push(harness);
        }
    }
    Detected {
        on_path,
        configured,
    }
}

/// The first directory of `search_path` holding an executable called `name`.
pub(crate) fn find_on_path(name: &str, search_path: Option<&OsStr>) -> Option<PathBuf> {
    let search_path = search_path?;
    std::env::split_paths(search_path)
        .map(|directory| directory.join(name))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// The harnesses this run plants.
///
/// A `--harness` list is an explicit ask and is taken as given: Gemini and Codex are planted
/// by writing their project files, which the stem does itself, so a planter may plant a
/// harness before installing it. With no list, what was detected is what is planted.
pub fn chosen(requested: Option<&[Harness]>, detected: &Detected) -> Vec<Harness> {
    if let Some(requested) = requested {
        return Harness::ALL
            .into_iter()
            .filter(|harness| requested.contains(harness))
            .collect();
    }
    Harness::ALL
        .into_iter()
        .filter(|harness| {
            detected.on_path.contains(harness) || detected.configured.contains(harness)
        })
        .collect()
}

// ----------------------------------------------------------------------- the bed list

/// The beds this run plants, out of the judges a lock pins.
///
/// `--beds` overrides the profile. The minimal profile keeps the beds [`MINIMAL_BEDS`]
/// names; the full profile keeps every judge the lock pins.
///
/// # Errors
///
/// [`Error::Bed`] naming a bed the lock does not pin, because a bed the lock cannot resolve
/// is a bed nothing can verify, and planting it would be a claim about bytes nobody has.
pub fn chosen_beds(
    profile: Profile,
    requested: Option<&[String]>,
    judges: &BTreeMap<String, lock::Judge>,
) -> Result<Vec<String>> {
    let wanted: Vec<String> = match (requested, profile) {
        (Some(requested), _) => requested.to_vec(),
        (None, Profile::Minimal) => MINIMAL_BEDS.iter().map(|bed| (*bed).to_owned()).collect(),
        (None, Profile::Full) => return Ok(judges.keys().cloned().collect()),
    };

    let mut beds = Vec::with_capacity(wanted.len());
    for bed in wanted {
        if !judges.contains_key(&bed) {
            return Err(Error::Bed {
                bed: bed.clone(),
                problem: format!(
                    "is not pinned by the lock, which pins {}",
                    judges.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            });
        }
        if !beds.contains(&bed) {
            beds.push(bed);
        }
    }
    Ok(beds)
}

/// A lock template with every judge but `beds` taken out, as TOML.
///
/// The template is read as a table rather than as a [`Lock`], so a key the stem does not
/// know survives into the lock a planter ends up carrying. What comes out is put back
/// through [`lock::parse_lock`], so a filtered lock the contracts would refuse never
/// reaches disk.
///
/// # Errors
///
/// [`Error::Toml`] naming the template when it is not TOML or has no judges table,
/// [`Error::Bed`] when a chosen bed is not in it, and whatever [`lock::parse_lock`] refuses
/// of the result.
pub fn filter_lock(template: &str, template_path: &Path, beds: &[String]) -> Result<String> {
    let refuse = |message: String| Error::Toml {
        path: template_path.to_path_buf(),
        message,
    };

    let mut table: toml::Table = template
        .parse()
        .map_err(|error: toml::de::Error| refuse(manifest::one_line(&error.to_string())))?;

    let judges = match table.get("judges") {
        Some(toml::Value::Table(judges)) => judges.clone(),
        _ => return Err(refuse("it pins no judges table".to_owned())),
    };

    let mut kept = toml::map::Map::new();
    for bed in beds {
        let judge = judges.get(bed).ok_or_else(|| Error::Bed {
            bed: bed.clone(),
            problem: format!("is not pinned by {}", template_path.display()),
        })?;
        kept.insert(bed.clone(), judge.clone());
    }
    table.insert("judges".to_owned(), toml::Value::Table(kept));

    let text = toml::to_string_pretty(&table)
        .map_err(|error| refuse(manifest::one_line(&error.to_string())))?;
    lock::parse_lock(&text)?;
    Ok(text)
}

// -------------------------------------------------------- the repository's own manifest

/// The name a planted repository's own manifest takes: the directory it sits in, or, when
/// that is not a name the contracts accept, the last segment of its `origin` remote.
///
/// # Errors
///
/// [`Error::Manifest`] when neither yields a name matching the schema's pattern, naming
/// both of the things it looked at, because guessing a name here would put a manifest the
/// contracts refuse into somebody's repository.
pub fn repository_name(root: &Path, remote: Option<&str>) -> Result<String> {
    // The remote names the repository; the directory is whatever the clone was called. A
    // Conductor workspace, a worktree or a scratch clone carries the same repository under
    // another directory name, and the manifest must say which repository it is.
    let from_remote = remote
        .map(|url| {
            url.trim_end_matches('/')
                .trim_end_matches(".git")
                .rsplit(['/', ':'])
                .next()
                .unwrap_or_default()
                .to_owned()
        })
        .unwrap_or_default();
    if let Some(name) = as_manifest_name(&from_remote) {
        return Ok(name);
    }
    let directory = root.file_name().and_then(OsStr::to_str).unwrap_or_default();
    if let Some(name) = as_manifest_name(directory) {
        return Ok(name);
    }
    Err(Error::Manifest {
        bed: directory.to_owned(),
        problem: format!(
            "neither the directory name \"{directory}\" nor the origin remote \
             \"{from_remote}\" is a name the manifest schema accepts; write garden.json by \
             hand with a name matching ^[a-z][a-z0-9-]*$"
        ),
    })
}

/// `candidate` as a manifest name, or `None` when nothing usable is left.
///
/// Letters and digits survive lowercased, everything else becomes one hyphen, and the name
/// has to start with a letter. This is the schema's own pattern and nothing looser.
fn as_manifest_name(candidate: &str) -> Option<String> {
    let mut name = String::with_capacity(candidate.len());
    for character in candidate.chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character.to_ascii_lowercase());
        } else if !name.ends_with('-') && !name.is_empty() {
            name.push('-');
        }
    }
    let name = name.trim_end_matches('-').to_owned();
    if name.starts_with(|c: char| c.is_ascii_lowercase()) {
        Some(name)
    } else {
        None
    }
}

/// The planted repository's own `garden.json`: a name and the one kind that says this is not
/// a bed (contracts v1.3.0).
pub fn repository_manifest(name: &str) -> String {
    format!(
        "{{\n  \"name\": \"{name}\",\n  \"kind\": [\n    \"{}\"\n  ]\n}}\n",
        manifest::REPOSITORY_KIND
    )
}

/// The pull request gate `init` writes.
///
/// Hosted runners only: a runner registered on a public repository lets a pull request from
/// a fork run its own code on that machine (jahala/plotplot issue 7).
pub fn workflow(own_stem: bool) -> String {
    // The umbrella is the stem's own repository, and its gate builds the stem from the very
    // checkout it judges; every other repository takes the stem from the umbrella, which
    // needs the umbrella reachable from a hosted runner: public, or a release to name here.
    let plant = if own_stem {
        [
            "      - name: plant the stem",
            "        # This repository is the stem's own, so the gate builds it from the checkout.",
            "        run: cargo install --locked --path .",
        ]
        .join("\n")
    } else {
        [
            "      - name: plant the stem".to_owned(),
            "        # plotplot has no release yet, so this builds it from the umbrella's default".to_owned(),
            "        # branch, which a hosted runner reaches only while the umbrella is public. Name".to_owned(),
            "        # a tag here once a season is tagged.".to_owned(),
            format!(
                "        run: cargo install --locked --git {} plotplot",
                env!("CARGO_PKG_REPOSITORY")
            ),
        ]
        .join("\n")
    };
    format!(
        "\
# Written by plotplot init: the gate every pull request passes.
#
# Runs on GitHub's hosted runners and nothing else. A runner of your own, registered on a
# public repository, lets a pull request from a fork run its own code on that machine
# (jahala/plotplot issue 7).

name: plotplot check

on:
  pull_request:

permissions:
  contents: read

jobs:
  # The ruleset plotplot applies on the default branch requires a check called {check}.
  # Rename this job and the branch is silently unprotected; change the steps freely.
  {check}:
    name: {check}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          # A gate that judges a diff needs the base of the pull request, not one commit.
          fetch-depth: 0

{plant}

      - name: resolve the pinned judges
        run: plotplot lock verify

      - name: plotplot check --strict
        run: plotplot check --strict
",
        check = platform::REQUIRED_CHECK,
    )
}

/// Whether the repository at `root` is the stem's own: a `Cargo.toml` at the root whose
/// package is `plotplot`.
///
/// # Errors
///
/// [`Error::Io`] when a `Cargo.toml` is there and cannot be read; a manifest that is not
/// TOML, or names another package, is simply not the stem's.
pub fn is_stems_own(root: &Path) -> Result<bool> {
    let path = root.join("Cargo.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => return Err(Error::Io { path, source }),
    };
    let Ok(manifest) = text.parse::<toml::Table>() else {
        return Ok(false);
    };
    Ok(manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        == Some(env!("CARGO_PKG_NAME")))
}

// --------------------------------------------------------------------------- the face

/// `plotplot init`: plant, and print one line per change.
///
/// The search path arrives as an argument for the reason the repository root does: nothing
/// in the library reads the environment. `main` reads both and passes them down.
pub fn run(
    root: &Path,
    args: &InitArgs,
    search_path: Option<&OsStr>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let detected = detect(root, search_path);
    let harnesses = chosen(args.harness.as_deref(), &detected);
    let _ = writeln!(stderr, "{}", detection_line(&detected, &harnesses));

    let existing = match lock::read_lock(root) {
        Ok(existing) => existing,
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return FAILED;
        }
    };
    if existing.is_none() && args.lock.is_none() {
        let _ = writeln!(
            stderr,
            "this repository pins no {}, so init needs --lock <path> naming the template to \
             copy it from",
            layout::GARDEN_LOCK
        );
        return USAGE;
    }

    match plant(root, args, &harnesses, existing) {
        Ok(mut planted) => {
            let stop = if args.github {
                let gh = find_on_path(platform::GH, search_path).map(|program| GhCli { program });
                github(
                    root,
                    origin_url(root).as_deref(),
                    gh.as_ref().map(|gh| gh as &dyn Gh),
                    &mut planted,
                )
            } else {
                None
            };
            report(&planted, stop.as_ref(), stdout, stderr)
        }
        Err(Refusal::Unresolved(reasons)) => {
            for reason in &reasons {
                let _ = writeln!(stderr, "{reason}");
            }
            let _ = writeln!(
                stderr,
                "nothing was planted: the stem runs only judges the lock can account for"
            );
            REFUSED
        }
        Err(Refusal::Failed(error)) => {
            let _ = writeln!(stderr, "{error}");
            FAILED
        }
    }
}

/// What `init` prints after planting, and the exit code it answers.
///
/// The notes first, then one line per change on stdout, or `nothing to do` when a finished
/// run changed nothing. When `--github` stopped short, what it could not do follows on
/// stderr, after everything the run did change.
pub fn report(
    planted: &Planted,
    stop: Option<&GithubStop>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    for note in &planted.notes {
        let _ = writeln!(stderr, "{note}");
    }
    let lines = match (planted.changed.is_empty(), stop) {
        (true, None) => Some(NOTHING.to_owned()),
        (true, Some(_)) => None,
        (false, _) => Some(planted.changed.join("\n")),
    };
    if let Some(lines) = lines
        && let Err(error) = writeln!(stdout, "{lines}")
    {
        let _ = writeln!(stderr, "stdout: {error}");
        return FAILED;
    }
    match stop {
        None => PLANTED,
        Some(GithubStop::Unreachable(why)) => {
            let _ = writeln!(stderr, "--github wrote nothing: {why}");
            USAGE
        }
        Some(GithubStop::Failed(error)) => {
            let _ = writeln!(stderr, "{error}");
            FAILED
        }
        Some(GithubStop::Unapplied(unapplied)) => {
            let _ = write!(stderr, "{}", platform::unapplied_text(unapplied));
            FAILED
        }
    }
}

/// How `--github` stopped short. Each is a different thing for a planter to do about it.
#[derive(Debug)]
pub enum GithubStop {
    /// It could not start: no `gh` on the search path, or no origin on github.com. Nothing of
    /// its own was written. [`USAGE`].
    Unreachable(Unreachable),
    /// Its region of `.github/CODEOWNERS` could not be written. [`FAILED`].
    Failed(Error),
    /// gh did not apply the ruleset, and said why. [`FAILED`].
    Unapplied(Unapplied),
}

/// `--github`, after the plain planting: the stem's region of `.github/CODEOWNERS`, then the
/// ruleset on the default branch, in that order. Each change is a line in `planted`.
///
/// The origin and the `gh` to call arrive as arguments: `run` reads both from around the
/// process, and a test hands in a recorded `gh`.
pub fn github(
    root: &Path,
    origin: Option<&str>,
    gh: Option<&dyn Gh>,
    planted: &mut Planted,
) -> Option<GithubStop> {
    let (gh, repository) = match platform::reach(gh, origin) {
        Ok(reached) => reached,
        Err(why) => return Some(GithubStop::Unreachable(why)),
    };
    match write_codeowners(root, &repository.owner) {
        Ok(true) => planted.changed.push(layout::CODEOWNERS.to_owned()),
        Ok(false) => {}
        Err(error) => return Some(GithubStop::Failed(error)),
    }
    match platform::apply_ruleset(gh, &repository) {
        Ok(Some(line)) => planted.changed.push(line),
        Ok(None) => {}
        Err(unapplied) => return Some(GithubStop::Unapplied(unapplied)),
    }
    None
}

/// The stem's region of `.github/CODEOWNERS` for `owner`, and every other line of that file
/// left alone. Whether the bytes moved.
fn write_codeowners(root: &Path, owner: &str) -> Result<bool> {
    let path = layout::codeowners(root);
    let existing = match std::fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(Error::Io { path, source }),
    };
    let planted = platform::codeowners_in(&existing, &platform::codeowners_region(owner))?;
    write_if_changed(&path, planted.as_bytes())
}

/// The two ways planting stops, which are two different exit codes.
enum Refusal {
    /// The lock named a judge this machine could not account for. Nothing was planted.
    Unresolved(Vec<String>),
    /// Something could not be read or written.
    Failed(Error),
}

impl From<Error> for Refusal {
    fn from(error: Error) -> Refusal {
        Refusal::Failed(error)
    }
}

/// Plant, in the order §5 of `docs/plans/stem.md` gives.
///
/// Nothing after the lock runs until the lock resolves, so a repository whose judges cannot
/// be accounted for keeps whatever it had.
fn plant(
    root: &Path,
    args: &InitArgs,
    harnesses: &[Harness],
    existing: Option<Lock>,
) -> std::result::Result<Planted, Refusal> {
    let mut planted = Planted::default();

    // 3. The lock: written from the template when there is none, then always resolved,
    //    because the resolving is what puts the beds' manifests where everything else reads
    //    them from.
    let lock = match existing {
        Some(lock) => lock,
        None => write_lock(root, args, &mut planted)?,
    };
    resolve(root, &lock, &mut planted)?;

    // 4. The bundles, from the beds the resolved lock placed.
    let beds = manifest::load_beds(root)?;
    let skills = bundle::read_skills(root, &beds)?;
    let input = BundleInput {
        stem_version: crate::VERSION,
        season: &lock.season,
        beds: &beds,
        skills: &skills,
    };
    for name in manifest::unplantable_channels(root)? {
        planted.notes.push(format!(
            "{name}: declares an MCP face with no launch line, so it is a channel the stem \
             cannot plant yet"
        ));
    }

    let stem = bundle::running_stem()?;
    for harness in harnesses {
        let tree = bundle::generate(*harness, &input)?;
        write_bundle(root, *harness, &tree, &stem, &mut planted)?;

        // 5. The install, in the vendor's own mechanism or the vendor's own project file.
        let installed = install::install(root, *harness, &tree)?;
        planted.changed.extend(installed.changed);
        planted.notes.extend(installed.notes);
    }

    // 6. The git hooks, and the configuration that makes git run them.
    write_git_hooks(root, &beds, &mut planted)?;
    write_git_config(root, &mut planted)?;

    // 7. The garden block, and the repository's own manifest.
    write_garden_block(root, &lock, &beds, &mut planted)?;
    write_own_manifest(root, &mut planted)?;

    // 8. The gate every pull request passes.
    let workflow_path = root.join(WORKFLOW);
    if write_if_changed(&workflow_path, workflow(is_stems_own(root)?).as_bytes())? {
        planted.changed.push(WORKFLOW.to_owned());
    }

    Ok(planted)
}

/// Copy the template the planter named, filtered to the chosen beds.
fn write_lock(
    root: &Path,
    args: &InitArgs,
    planted: &mut Planted,
) -> std::result::Result<Lock, Refusal> {
    let Some(template_path) = &args.lock else {
        // `run` refuses this before planting starts; this says the same where the value is
        // needed, rather than unwrapping an option somebody else checked.
        return Err(Refusal::Failed(Error::Io {
            path: PathBuf::from(layout::GARDEN_LOCK),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "init needs --lock <path> to write a garden.lock from",
            ),
        }));
    };
    let template = std::fs::read_to_string(template_path).map_err(|source| Error::Io {
        path: template_path.clone(),
        source,
    })?;
    let judges = lock::parse_lock_at(template_path, &template)?.judges;
    let beds = chosen_beds(args.profile, args.beds.as_deref(), &judges)?;
    let text = filter_lock(&template, template_path, &beds)?;

    let path = layout::garden_lock(root);
    if write_if_changed(&path, text.as_bytes())? {
        planted.changed.push(layout::GARDEN_LOCK.to_owned());
    }
    Ok(lock::parse_lock(&text)?)
}

/// Resolve every judge the lock pins, and refuse to plant when any cannot be accounted for.
fn resolve(root: &Path, lock: &Lock, planted: &mut Planted) -> std::result::Result<(), Refusal> {
    let reports = lock::verify(root, lock, lock::platform(), &Https)?;
    let unresolved: Vec<String> = reports
        .iter()
        .filter(|report| {
            !matches!(
                report.state,
                JudgeState::Verified | JudgeState::Fetched { .. }
            )
        })
        .map(lock::line)
        .collect();
    if !unresolved.is_empty() {
        return Err(Refusal::Unresolved(unresolved));
    }
    for report in &reports {
        if matches!(report.state, JudgeState::Fetched { .. }) {
            planted.changed.push(lock::line(report));
        }
    }
    Ok(())
}

/// One harness's bundle on disk, with the running stem in its `bin/`.
fn write_bundle(
    root: &Path,
    harness: Harness,
    tree: &FileTree,
    stem: &Path,
    planted: &mut Planted,
) -> Result<()> {
    let bundle = layout::bundle_dir(root, harness);
    for (relative, bytes) in &tree.0 {
        let path = bundle.join(relative);
        if write_if_changed(&path, bytes)? {
            planted.changed.push(install::relative(root, &path));
        }
    }
    let placed = bundle.join(bundle::STEM_BINARY);
    if copy_if_changed(stem, &placed)? {
        planted.changed.push(install::relative(root, &placed));
    }
    Ok(())
}

/// The four hook files under `core.hooksPath`, each runnable.
fn write_git_hooks(root: &Path, beds: &[crate::bed::Bed], planted: &mut Planted) -> Result<()> {
    for hook in crate::bed::GitHook::ALL {
        let path = layout::git_hook(root, hook);
        let script = githooks::render(hook, beds);
        let written = write_if_changed(&path, script.as_bytes())?;
        // The bit is set every run, not only when the bytes changed: a hook nothing can run
        // is a boundary that silently never fires, and that is worth one syscall.
        bundle::make_executable(&path)?;
        if written {
            planted.changed.push(install::relative(root, &path));
        }
    }
    Ok(())
}

/// `core.hooksPath`, and the receipts ref by name when origin has it.
fn write_git_config(root: &Path, planted: &mut Planted) -> Result<()> {
    let desired = gitconfig::desired(root);
    let current = gitconfig::read(root)?;
    let missing = gitconfig::missing(&desired, &current);
    if !missing.is_empty() {
        gitconfig::apply(root, &missing)?;
        for (key, value) in missing {
            planted.changed.push(format!("{key} {value}"));
        }
    }
    // The receipts ref travels when asked, never by a refspec: bring it now when origin has
    // it, so a fresh clone carries the receipts its history was sealed with.
    match gitconfig::fetch_receipts(root)? {
        gitconfig::Receipts::Fetched => planted
            .changed
            .push(format!("{} fetched from origin", gitconfig::RECEIPTS_REF)),
        gitconfig::Receipts::Unreachable(why) => planted.notes.push(format!(
            "origin could not be asked for {}, so no receipts were fetched: {why}",
            gitconfig::RECEIPTS_REF
        )),
        gitconfig::Receipts::AlreadyHere
        | gitconfig::Receipts::NoneOnRemote
        | gitconfig::Receipts::NoOrigin => {}
    }
    Ok(())
}

/// The garden block in `AGENTS.md`, and every other byte of that file left alone.
fn write_garden_block(
    root: &Path,
    lock: &Lock,
    beds: &[crate::bed::Bed],
    planted: &mut Planted,
) -> Result<()> {
    let path = layout::agents_md(root);
    let existing = match std::fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(Error::Io { path, source }),
    };
    let block = garden_block::render(&lock.season, beds);
    let planted_text = garden_block::replace_in(&existing, &block)?;
    if write_if_changed(&path, planted_text.as_bytes())? {
        planted.changed.push(layout::AGENTS_MD.to_owned());
    }
    Ok(())
}

/// The planted repository's own `garden.json`, written only when it has none.
fn write_own_manifest(root: &Path, planted: &mut Planted) -> Result<()> {
    let path = root.join(layout::GARDEN_JSON);
    if path.exists() {
        return Ok(());
    }
    let name = repository_name(root, origin_url(root).as_deref())?;
    let document = repository_manifest(&name);
    manifest::validate_repository(&document)?;
    if write_if_changed(&path, document.as_bytes())? {
        planted.changed.push(layout::GARDEN_JSON.to_owned());
    }
    Ok(())
}

/// The url of `origin`, or `None` when the repository has no such remote. A repository with
/// no remote is not a failure; it is a repository that has not been pushed anywhere.
pub(crate) fn origin_url(root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8(output.stdout).ok()?;
    let url = url.trim();
    if url.is_empty() {
        None
    } else {
        Some(url.to_owned())
    }
}

/// What `init` says it found before it plants anything.
fn detection_line(detected: &Detected, harnesses: &[Harness]) -> String {
    let names = |list: &[Harness]| {
        if list.is_empty() {
            "none".to_owned()
        } else {
            list.iter()
                .map(|harness| harness.name())
                .collect::<Vec<_>>()
                .join(", ")
        }
    };
    format!(
        "on PATH: {}; already configured here: {}; planting: {}",
        names(&detected.on_path),
        names(&detected.configured),
        names(harnesses)
    )
}

// ------------------------------------------------------------------------- the writes

/// Write `bytes` to `path` when they are not already there, and say whether anything
/// changed.
///
/// This is the whole of idempotence: every file `init` and `install` write goes through it,
/// so a second run touches nothing and a tree hashed before and after a second run is the
/// same tree.
///
/// # Errors
///
/// [`Error::Io`] naming the file or its directory when it cannot be read, created or
/// written.
pub(crate) fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    match std::fs::read(path) {
        Ok(existing) if existing == bytes => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(path, bytes).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(true)
}

/// Copy `from` to `to` when their bytes differ, and leave `to` runnable either way.
///
/// # Errors
///
/// [`Error::Io`] naming the file that could not be read, written or made executable.
pub(crate) fn copy_if_changed(from: &Path, to: &Path) -> Result<bool> {
    let bytes = std::fs::read(from).map_err(|source| Error::Io {
        path: from.to_path_buf(),
        source,
    })?;
    let changed = write_if_changed(to, &bytes)?;
    bundle::make_executable(to)?;
    Ok(changed)
}

// ---------------------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn judges(names: &[&str]) -> BTreeMap<String, lock::Judge> {
        names
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    lock::Judge {
                        version: "0.1.0".to_owned(),
                        npm: None,
                        platforms: BTreeMap::new(),
                        git: None,
                        extra: BTreeMap::new(),
                    },
                )
            })
            .collect()
    }

    const TEMPLATE: &str = "\
season = \"2026.09\"

[judges.tend2]
version = \"1.0.0\"
npm = \"@plotplot/tend2\"

[judges.weeder]
version = \"0.1.0\"

[judges.weeder.platforms.\"aarch64-apple-darwin\"]
url = \"https://example.invalid/weeder.tar.gz\"
sha256 = \"11a1cdbb1f2a4d2ff53f3f0d2ae0ff9d1a2c4f81ec4d5ba9b30f5a4c9e17d2b6\"
";

    #[test]
    fn the_minimal_profile_is_weeder_alone() {
        let beds = chosen_beds(
            Profile::Minimal,
            None,
            &judges(&["tend2", "tilth", "weeder"]),
        )
        .expect("a bed list");
        assert_eq!(beds, vec!["weeder".to_owned()]);
    }

    #[test]
    fn the_full_profile_is_every_judge_the_lock_pins() {
        let beds = chosen_beds(Profile::Full, None, &judges(&["tend2", "tilth", "weeder"]))
            .expect("a bed list");
        assert_eq!(beds, vec!["tend2", "tilth", "weeder"]);
    }

    #[test]
    fn the_beds_list_overrides_either_profile() {
        for profile in [Profile::Minimal, Profile::Full] {
            let requested = ["tilth".to_owned()];
            let beds = chosen_beds(profile, Some(&requested), &judges(&["tilth", "weeder"]))
                .expect("a bed list");
            assert_eq!(beds, vec!["tilth".to_owned()], "{profile:?}");
        }
    }

    #[test]
    fn a_bed_the_lock_does_not_pin_is_refused_and_names_what_is_pinned() {
        let error = chosen_beds(
            Profile::Minimal,
            Some(&["petals".to_owned()]),
            &judges(&["tilth", "weeder"]),
        )
        .expect_err("petals is not pinned");
        let message = error.to_string();
        assert!(message.starts_with("petals: "), "{message}");
        assert!(message.contains("tilth, weeder"), "{message}");
    }

    #[test]
    fn the_minimal_profile_refuses_a_lock_that_pins_no_weeder() {
        let error = chosen_beds(Profile::Minimal, None, &judges(&["tilth"]))
            .expect_err("the minimal profile plants weeder");
        assert!(error.to_string().starts_with("weeder: "), "{error}");
    }

    #[test]
    fn filtering_keeps_the_chosen_judges_and_the_season() {
        let filtered = filter_lock(TEMPLATE, Path::new("template.lock"), &["weeder".to_owned()])
            .expect("a filtered lock");
        let lock = lock::parse_lock(&filtered).expect("the filtered lock is a lock");
        assert_eq!(lock.season, "2026.09");
        assert_eq!(lock.judges.keys().collect::<Vec<_>>(), vec!["weeder"]);
        assert_eq!(
            lock.judges["weeder"].platforms["aarch64-apple-darwin"].sha256,
            "11a1cdbb1f2a4d2ff53f3f0d2ae0ff9d1a2c4f81ec4d5ba9b30f5a4c9e17d2b6"
        );
    }

    #[test]
    fn filtering_the_same_template_twice_gives_the_same_bytes() {
        let once = filter_lock(TEMPLATE, Path::new("t.lock"), &["weeder".to_owned()])
            .expect("a filtered lock");
        let twice = filter_lock(&once, Path::new("t.lock"), &["weeder".to_owned()])
            .expect("a filtered lock");
        assert_eq!(once, twice);
    }

    #[test]
    fn filtering_to_nothing_is_refused_by_the_contracts() {
        let error = filter_lock(TEMPLATE, Path::new("t.lock"), &[])
            .expect_err("a lock pins at least one judge");
        assert!(error.to_string().contains(lock::SCHEMA_REFUSED), "{error}");
    }

    #[test]
    fn a_template_that_is_not_a_lock_is_refused_and_names_the_template() {
        let error = filter_lock("season = 3", Path::new("t.lock"), &["weeder".to_owned()])
            .expect_err("no judges table");
        assert!(error.to_string().starts_with("t.lock: "), "{error}");
    }

    #[test]
    fn the_harness_flag_is_taken_as_given_and_stays_in_one_order() {
        let detected = Detected::default();
        assert_eq!(
            chosen(Some(&[Harness::Codex, Harness::Claude]), &detected),
            vec![Harness::Claude, Harness::Codex]
        );
    }

    #[test]
    fn with_no_flag_what_was_detected_is_what_is_planted() {
        let detected = Detected {
            on_path: vec![Harness::Codex],
            configured: vec![Harness::Gemini],
        };
        assert_eq!(
            chosen(None, &detected),
            vec![Harness::Gemini, Harness::Codex]
        );
        assert!(chosen(None, &Detected::default()).is_empty());
    }

    #[test]
    fn the_repository_takes_its_remotes_name_over_its_directorys() {
        assert_eq!(
            repository_name(
                Path::new("/home/someone/work/a-clone-called-otherwise"),
                Some("https://github.com/jahala/plotplot.git")
            )
            .expect("a name"),
            "plotplot",
            "a workspace clone is the repository its remote names"
        );
        assert_eq!(
            repository_name(
                Path::new("/work/plotplot"),
                Some("git@github.com:jahala/weeder.git")
            )
            .expect("a name"),
            "weeder"
        );
    }

    #[test]
    fn without_a_remote_the_repository_takes_its_directorys_name() {
        assert_eq!(
            repository_name(Path::new("/work/My Repo"), None).expect("a name"),
            "my-repo"
        );
        assert_eq!(
            repository_name(Path::new("/work/plotplot"), None).expect("a name"),
            "plotplot"
        );
    }

    #[test]
    fn a_remote_that_is_not_a_manifest_name_falls_to_the_directory() {
        assert_eq!(
            repository_name(
                Path::new("/work/plotplot"),
                Some("https://example.invalid/2026.git")
            )
            .expect("a name"),
            "plotplot"
        );
    }

    #[test]
    fn a_directory_name_that_is_not_a_manifest_name_falls_to_the_remote() {
        assert_eq!(
            repository_name(
                Path::new("/work/2026"),
                Some("https://github.com/jahala/plotplot.git")
            )
            .expect("a name"),
            "plotplot"
        );
        assert_eq!(
            repository_name(
                Path::new("/work/2026"),
                Some("git@github.com:jahala/weeder.git")
            )
            .expect("a name"),
            "weeder"
        );
    }

    #[test]
    fn a_repository_neither_source_can_name_is_refused_rather_than_guessed() {
        let error = repository_name(Path::new("/work/2026"), None)
            .expect_err("no name the contracts accept");
        assert!(error.to_string().contains("2026"), "{error}");
        assert!(error.to_string().contains("garden.json"), "{error}");
    }

    #[test]
    fn the_repositorys_own_manifest_is_what_the_contracts_call_a_repository() {
        let document = repository_manifest("plotplot");
        manifest::validate_repository(&document).expect("the contracts accept it");
        let value: serde_json::Value = serde_json::from_str(&document).expect("JSON");
        assert_eq!(value["name"], "plotplot");
        assert_eq!(value["kind"], serde_json::json!(["repository"]));
    }

    #[test]
    fn the_repositorys_own_manifest_is_never_a_bed() {
        let document = repository_manifest("plotplot");
        let error = manifest::parse_manifest(&document).expect_err("not a bed");
        assert!(
            error.to_string().contains(manifest::REPOSITORY_NOT_A_BED),
            "{error}"
        );
    }

    #[test]
    fn the_workflow_runs_the_strict_check_on_a_pull_request_and_on_a_hosted_runner() {
        let workflow = workflow(false);
        assert!(workflow.contains("on:\n  pull_request:\n"), "{workflow}");
        assert!(workflow.contains("runs-on: ubuntu-latest"), "{workflow}");
        assert!(
            workflow.contains("run: plotplot check --strict"),
            "{workflow}"
        );
        assert!(!workflow.contains("self-hosted"), "{workflow}");
        assert_eq!(
            workflow.matches("runs-on:").count(),
            1,
            "one job, one runner: {workflow}"
        );
    }

    #[test]
    fn the_workflows_job_is_named_the_check_the_ruleset_requires() {
        for text in [workflow(false), workflow(true)] {
            assert!(
                text.contains(&format!(
                    "\n  {check}:\n    name: {check}\n",
                    check = platform::REQUIRED_CHECK
                )),
                "{text}"
            );
            // The command the job runs may change without touching the ruleset; it is a step.
            assert!(
                text.contains(
                    "      - name: plotplot check --strict\n        run: plotplot check --strict\n"
                ),
                "{text}"
            );
            assert!(
                !text.contains("    name: plotplot check --strict\n"),
                "{text}"
            );
        }
    }

    #[test]
    fn the_stems_own_repository_builds_the_stem_from_its_checkout() {
        let own = workflow(true);
        assert!(own.contains("cargo install --locked --path ."), "{own}");
        assert!(!own.contains("--git"), "{own}");
        let other = workflow(false);
        assert!(
            other.contains(
                "cargo install --locked --git https://github.com/jahala/plotplot plotplot"
            ),
            "{other}"
        );
        assert!(other.contains("while the umbrella is public"), "{other}");
        for text in [&own, &other] {
            assert!(text.contains("runs-on: ubuntu-latest"), "{text}");
            assert!(!text.contains("self-hosted"), "{text}");
            assert!(text.contains("plotplot check --strict"), "{text}");
            // The step's lines sit where YAML wants them: every `run:` under a step is
            // indented eight spaces, and none of the step's lines lost its indentation.
            for line in text.lines().filter(|line| line.contains("cargo install")) {
                assert!(line.starts_with("        run: "), "{line:?}");
            }
            assert!(!text.contains("\nrun:"), "{text}");
            assert!(!text.contains("\n# This repository"), "{text}");
        }
    }

    #[test]
    fn a_repository_is_the_stems_own_when_its_cargo_package_is_plotplot() {
        let root = tempfile::tempdir().expect("a temp root");
        assert!(!is_stems_own(root.path()).expect("no Cargo.toml is a plain answer"));
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"weeder\"\nversion = \"0.1.0\"\n",
        )
        .expect("a manifest");
        assert!(!is_stems_own(root.path()).expect("another package is a plain answer"));
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"plotplot\"\nversion = \"0.1.0\"\n",
        )
        .expect("the stem's manifest");
        assert!(is_stems_own(root.path()).expect("the stem's own"));
        std::fs::write(root.path().join("Cargo.toml"), "not = [toml").expect("garbage");
        assert!(!is_stems_own(root.path()).expect("garbage is not the stem's"));
    }

    #[test]
    fn writing_the_same_bytes_twice_changes_nothing_the_second_time() {
        let tmp = tempfile::tempdir().expect("a temp directory");
        let path = tmp.path().join("deep/inside/file.txt");
        assert!(write_if_changed(&path, b"one").expect("a write"));
        assert!(!write_if_changed(&path, b"one").expect("a write"));
        assert!(write_if_changed(&path, b"two").expect("a write"));
        assert_eq!(std::fs::read(&path).expect("the file"), b"two");
    }

    #[test]
    fn a_harness_is_detected_by_an_executable_on_the_given_path() {
        let tmp = tempfile::tempdir().expect("a temp directory");
        let bin = tmp.path().join("bin");
        std::fs::create_dir_all(&bin).expect("a bin directory");
        let gemini = bin.join("gemini");
        std::fs::write(&gemini, "#!/bin/sh\n").expect("a file");
        bundle::make_executable(&gemini).expect("the bit");
        // A file with no executable bit is not a CLI on the path.
        std::fs::write(bin.join("codex"), "#!/bin/sh\n").expect("a file");

        let root = tmp.path().join("repo");
        std::fs::create_dir_all(root.join(".codex")).expect("a project directory");

        let detected = detect(&root, Some(bin.as_os_str()));
        assert_eq!(detected.on_path, vec![Harness::Gemini]);
        assert_eq!(detected.configured, vec![Harness::Codex]);
        assert_eq!(
            chosen(None, &detected),
            vec![Harness::Gemini, Harness::Codex]
        );
    }

    #[test]
    fn with_no_search_path_nothing_is_found_on_it() {
        let tmp = tempfile::tempdir().expect("a temp directory");
        let detected = detect(tmp.path(), None);
        assert!(detected.on_path.is_empty());
    }

    // ------------------------------------------------------------------ --github

    use crate::platform::recorded::Recorded;
    use crate::platform::{Ran, Unreachable};

    const GITHUB: &str = "https://github.com/example-owner/example-repo.git";
    const LIST: &str = "repos/example-owner/example-repo/rulesets";

    /// What `report` wrote on each stream, and the code it answered.
    fn reported(planted: &Planted, stop: Option<&GithubStop>) -> (i32, String, String) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = report(planted, stop, &mut stdout, &mut stderr);
        (
            code,
            String::from_utf8(stdout).expect("utf-8"),
            String::from_utf8(stderr).expect("utf-8"),
        )
    }

    #[test]
    fn github_writes_the_region_then_applies_the_ruleset_and_names_both() {
        let root = tempfile::tempdir().expect("a temp root");
        let gh = Recorded::default()
            .answer("GET", LIST, "[]")
            .answer("POST", LIST, r#"{"id":41}"#);
        let mut planted = Planted::default();

        let stop = github(root.path(), Some(GITHUB), Some(&gh), &mut planted);
        assert!(stop.is_none(), "{stop:?}");
        assert_eq!(
            planted.changed,
            [
                layout::CODEOWNERS.to_owned(),
                "ruleset \"plotplot: the default branch\" applied on example-owner/example-repo"
                    .to_owned(),
            ]
        );
        let codeowners =
            std::fs::read_to_string(layout::codeowners(root.path())).expect("CODEOWNERS");
        assert!(
            codeowners.contains("/garden.lock @example-owner\n"),
            "{codeowners}"
        );

        // The second run: the region is already there and the ruleset already says this.
        let gh = Recorded::default()
            .answer(
                "GET",
                LIST,
                r#"[{"id":41,"name":"plotplot: the default branch"}]"#,
            )
            .answer(
                "GET",
                "repos/example-owner/example-repo/rulesets/41",
                &crate::platform::ruleset_body(),
            );
        let mut again = Planted::default();
        assert!(github(root.path(), Some(GITHUB), Some(&gh), &mut again).is_none());
        assert!(again.changed.is_empty(), "{:?}", again.changed);
        assert!(gh.writes().is_empty(), "{:?}", gh.calls());
        assert_eq!(
            reported(&again, None),
            (PLANTED, "nothing to do\n".to_owned(), String::new())
        );
    }

    #[test]
    fn a_failing_call_prints_ghs_own_words_after_the_changed_lines_and_answers_failed() {
        let root = tempfile::tempdir().expect("a temp root");
        let gh = Recorded::default().answer("GET", LIST, "[]").ran(
            "POST",
            LIST,
            Ran {
                code: 1,
                stdout:
                    "{\"message\":\"Resource not accessible by integration\",\"status\":\"403\"}"
                        .to_owned(),
                stderr: "gh: Resource not accessible by integration (HTTP 403)\n".to_owned(),
            },
        );
        let mut planted = Planted {
            changed: vec!["AGENTS.md".to_owned()],
            notes: vec!["a note".to_owned()],
        };

        let stop = github(root.path(), Some(GITHUB), Some(&gh), &mut planted);
        assert!(matches!(stop, Some(GithubStop::Unapplied(_))), "{stop:?}");
        // What was planted before the refusal stays planted.
        assert!(layout::codeowners(root.path()).is_file());

        let (code, stdout, stderr) = reported(&planted, stop.as_ref());
        assert_eq!(code, FAILED);
        assert_eq!(stdout, "AGENTS.md\n.github/CODEOWNERS\n");
        assert_eq!(
            stderr,
            "a note\n\
             gh api -X POST repos/example-owner/example-repo/rulesets could not be applied:\n\
             gh: Resource not accessible by integration (HTTP 403)\n\
             {\"message\":\"Resource not accessible by integration\",\"status\":\"403\"}\n"
        );
    }

    #[test]
    fn without_gh_github_refuses_with_usage_and_writes_nothing_of_its_own() {
        let root = tempfile::tempdir().expect("a temp root");
        let mut planted = Planted {
            changed: vec!["garden.lock".to_owned()],
            notes: Vec::new(),
        };
        let stop = github(root.path(), Some(GITHUB), None, &mut planted);
        assert!(
            matches!(stop, Some(GithubStop::Unreachable(Unreachable::NoGh))),
            "{stop:?}"
        );
        assert!(!layout::codeowners(root.path()).exists());

        let (code, stdout, stderr) = reported(&planted, stop.as_ref());
        assert_eq!(code, USAGE);
        assert_eq!(
            stdout, "garden.lock\n",
            "the plain planting is still reported"
        );
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.contains("gh is not on PATH"), "{stderr}");
    }

    #[test]
    fn an_origin_off_github_refuses_with_usage_and_asks_gh_nothing() {
        let root = tempfile::tempdir().expect("a temp root");
        let gh = Recorded::default();
        let mut planted = Planted::default();
        let stop = github(
            root.path(),
            Some("https://example.invalid/jahala/fixture.git"),
            Some(&gh),
            &mut planted,
        );
        assert!(
            matches!(
                stop,
                Some(GithubStop::Unreachable(Unreachable::NotGithub { .. }))
            ),
            "{stop:?}"
        );
        assert!(gh.calls().is_empty(), "{:?}", gh.calls());
        assert!(!layout::codeowners(root.path()).exists());

        let (code, stdout, stderr) = reported(&planted, stop.as_ref());
        assert_eq!(code, USAGE);
        assert!(stdout.is_empty(), "{stdout}");
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.contains("example.invalid"), "{stderr}");
        assert!(stderr.contains("github.com"), "{stderr}");
    }

    #[test]
    fn a_codeowners_with_broken_markers_is_refused_before_any_call() {
        let root = tempfile::tempdir().expect("a temp root");
        let path = layout::codeowners(root.path());
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory");
        std::fs::write(&path, "# plotplot:begin\n* @someone\n").expect("a broken region");
        let gh = Recorded::default();
        let mut planted = Planted::default();

        let stop = github(root.path(), Some(GITHUB), Some(&gh), &mut planted);
        assert!(
            matches!(stop, Some(GithubStop::Failed(Error::Codeowners { .. }))),
            "{stop:?}"
        );
        assert!(gh.calls().is_empty(), "{:?}", gh.calls());
        assert_eq!(
            std::fs::read_to_string(&path).expect("the file"),
            "# plotplot:begin\n* @someone\n"
        );
        let (code, _, stderr) = reported(&planted, stop.as_ref());
        assert_eq!(code, FAILED);
        assert!(stderr.starts_with(layout::CODEOWNERS), "{stderr}");
    }
}
