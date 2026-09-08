//! The three vendor bundles, generated from one input.
//!
//! One garden, three dialects. [`generate`] is pure: beds and skill files in, a
//! [`FileTree`] out, the same bytes every time. [`write`] is the edge that puts that tree on
//! disk, and [`diff`] reads a directory back and says how it differs, which is how `doctor`
//! knows a bundle is still the one the stem would generate today.
//!
//! Every file name, every key and every hook shape here is a vendor fact from
//! `docs/prompts/stem-build-2026-09.md` §5, verified on 2026-09-08 against the installed
//! binaries. When a vendor changes one, that table changes first and this module follows.
//!
//! What a bundle carries: the vendor's manifest, one hook file, one `SKILL.md` per skill,
//! and the MCP servers of every channel bed that declared how to launch itself. Never
//! `bin/`: the running stem binary and each bed's artifact are copied there by the install
//! step, so `generate` cannot know their bytes and never claims to.

mod claude;
mod codex;
mod gemini;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bed::Bed;
use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::lock::read_lock;
use crate::{layout, manifest};

/// The name every vendor manifest carries, and the name of the installed bundle.
pub const BUNDLE_NAME: &str = "plotplot";

/// Where the install step puts the running stem binary inside a bundle, and the only thing
/// a hook entry ever calls.
pub const STEM_BINARY: &str = "bin/plotplot";

/// The bundle directory `generate` never writes into and `diff` never reports as extra: the
/// install step fills it with the stem's own binary and each bed's artifact.
pub const BINARY_DIR: &str = "bin";

/// The stem's home, as the vendors' manifests name its author.
const AUTHOR_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// A bed's `SKILL.md`, already read, on its way into `skills/<bed>/SKILL.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillFile {
    pub bed: String,
    pub content: String,
}

/// Everything the three bundles are generated from, and nothing else.
#[derive(Clone, Copy, Debug)]
pub struct BundleInput<'a> {
    pub stem_version: &'a str,
    pub season: &'a str,
    pub beds: &'a [Bed],
    pub skills: &'a [SkillFile],
}

/// Relative path to bytes. A `BTreeMap` so the tree, and everything written from it, is in
/// one order on every machine and every run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileTree(pub BTreeMap<PathBuf, Vec<u8>>);

impl FileTree {
    /// An empty tree.
    pub fn new() -> FileTree {
        FileTree(BTreeMap::new())
    }

    /// Put one file in the tree, replacing whatever was at that path.
    pub fn insert(&mut self, path: impl Into<PathBuf>, bytes: Vec<u8>) {
        self.0.insert(path.into(), bytes);
    }

    /// One file's bytes, or `None` when the tree does not carry that path.
    pub fn get(&self, path: impl AsRef<Path>) -> Option<&[u8]> {
        self.0.get(path.as_ref()).map(Vec::as_slice)
    }

    /// Every path in the tree, sorted.
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.0.keys()
    }
}

/// Generate one harness's bundle. Pure: no file is read and none is written.
///
/// # Errors
///
/// [`Error::Bed`] when a bed declares a hook event this harness does not fire, or when two
/// skill files claim the same bed, or when a skill's bed name is not a single path
/// component. Nothing is dropped silently.
pub fn generate(harness: Harness, input: &BundleInput) -> Result<FileTree> {
    match harness {
        Harness::Claude => claude::generate(input),
        Harness::Gemini => gemini::generate(input),
        Harness::Codex => codex::generate(input),
    }
}

/// Write a tree under `root`, creating the directories it needs, and return the paths
/// written, sorted.
///
/// Files under `root` that the tree does not carry are left alone; [`diff`] is what reports
/// them.
///
/// # Errors
///
/// [`Error::Io`] naming the directory or the file that could not be created or written.
pub fn write(tree: &FileTree, root: &Path) -> Result<Vec<PathBuf>> {
    let mut written = Vec::with_capacity(tree.0.len());
    for (relative, bytes) in &tree.0 {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&path, bytes).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        written.push(path);
    }
    written.sort();
    Ok(written)
}

/// How a directory differs from the tree the stem would generate for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Difference {
    /// The tree has this file and the directory does not.
    Missing(PathBuf),
    /// The directory has this file and the tree does not.
    Extra(PathBuf),
    /// Both have it, with different bytes.
    Changed(PathBuf),
}

impl Difference {
    /// The path this difference is about, relative to the bundle's root.
    pub fn path(&self) -> &Path {
        match self {
            Difference::Missing(path) | Difference::Extra(path) | Difference::Changed(path) => path,
        }
    }

    /// Missing before extra before changed, so a rendered list reads in one order.
    fn rank(&self) -> u8 {
        match self {
            Difference::Missing(_) => 0,
            Difference::Extra(_) => 1,
            Difference::Changed(_) => 2,
        }
    }
}

impl fmt::Display for Difference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = match self {
            Difference::Missing(_) => "missing",
            Difference::Extra(_) => "extra",
            Difference::Changed(_) => "changed",
        };
        write!(f, "{word} {}", self.path().display())
    }
}

/// Compare a generated tree against a directory, byte for byte.
///
/// A file the tree carries and the directory does not is [`Difference::Missing`]; a file
/// under `root` the tree does not carry is [`Difference::Extra`], except anything under
/// `bin/`, which the install step owns; a file both have with different bytes is
/// [`Difference::Changed`]. An absent `root` is every path missing, not a failure. The list
/// is sorted by path.
///
/// # Errors
///
/// [`Error::Io`] naming the file or directory that could not be read.
pub fn diff(tree: &FileTree, root: &Path) -> Result<Vec<Difference>> {
    let mut differences = Vec::new();
    for (relative, bytes) in &tree.0 {
        let path = root.join(relative);
        match fs::read(&path) {
            Ok(found) if found == *bytes => {}
            Ok(_) => differences.push(Difference::Changed(relative.clone())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                differences.push(Difference::Missing(relative.clone()));
            }
            Err(source) => return Err(Error::Io { path, source }),
        }
    }
    for relative in files_under(root)? {
        if !tree.0.contains_key(&relative) {
            differences.push(Difference::Extra(relative));
        }
    }
    differences.sort_by(|left, right| {
        left.path()
            .cmp(right.path())
            .then_with(|| left.rank().cmp(&right.rank()))
    });
    Ok(differences)
}

/// Every file under `root`, relative to it, skipping the install step's `bin/`. An absent
/// directory has no files.
fn files_under(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, Path::new(""), &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(directory: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(Error::Io {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let child = relative.join(entry.file_name());
        if child == Path::new(BINARY_DIR) {
            continue;
        }
        let kind = entry.file_type().map_err(|source| Error::Io {
            path: entry.path(),
            source,
        })?;
        if kind.is_dir() {
            collect_files(&entry.path(), &child, files)?;
        } else {
            files.push(child);
        }
    }
    Ok(())
}

/// `plotplot bundle build`: regenerate the bundles for `harnesses` from the planted beds.
///
/// The season comes from `garden.lock`, the beds and their skills from `.plotplot/beds/`,
/// and the stem's version from the crate; each bundle is written under
/// `.plotplot/bundles/<harness>/` with the running binary copied into its `bin/plotplot`.
/// One line per file written goes to `stdout`; a channel bed the stem cannot plant is named
/// on `stderr`.
///
/// Nothing is written until every input has been read, so a repository with no season keeps
/// the bundles it already had.
pub fn run(
    root: &Path,
    stem_binary: &Path,
    harnesses: &[Harness],
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> i32 {
    let built = match build(root, stem_binary, harnesses) {
        Ok(built) => built,
        Err(error) => {
            let _ = writeln!(stderr, "{error}");
            return 1;
        }
    };

    for bed in &built.unplantable {
        let _ = writeln!(
            stderr,
            "{bed}: declares an MCP face with no launch line, so it is a channel the stem cannot plant yet"
        );
    }
    for path in &built.written {
        let shown = path.strip_prefix(root).unwrap_or(path);
        if let Err(error) = writeln!(stdout, "{}", shown.display()) {
            let _ = writeln!(stderr, "stdout: {error}");
            return 1;
        }
    }
    0
}

/// What one `bundle build` did: the files it wrote, and the channels it had to leave out.
struct Built {
    written: Vec<PathBuf>,
    unplantable: Vec<String>,
}

/// Read every input, generate every tree, and only then write: reading and generating are
/// what can refuse, and a refusal for one harness must not leave a bundle for another
/// harness half planted beside it.
fn build(root: &Path, stem_binary: &Path, harnesses: &[Harness]) -> Result<Built> {
    let beds = manifest::load_beds(root)?;
    let unplantable = manifest::unplantable_channels(root)?;
    let season = season(root)?;
    let skills = read_skills(root, &beds)?;

    let input = BundleInput {
        stem_version: crate::VERSION,
        season: &season,
        beds: &beds,
        skills: &skills,
    };

    let mut trees = Vec::with_capacity(harnesses.len());
    for harness in harnesses {
        trees.push((*harness, generate(*harness, &input)?));
    }

    let mut written = Vec::new();
    for (harness, tree) in &trees {
        let bundle = layout::bundle_dir(root, *harness);
        written.extend(write(tree, &bundle)?);
        written.push(install_stem(stem_binary, &bundle)?);
    }
    Ok(Built {
        written,
        unplantable,
    })
}

/// The season the lock pins. A bundle names the season it was grown in, so a repository
/// without a lock has nothing to generate from.
///
/// # Errors
///
/// [`Error::Io`] naming `garden.lock` when there is none, and whatever
/// [`crate::lock::read_lock`] refuses.
fn season(root: &Path) -> Result<String> {
    match read_lock(root)? {
        Some(lock) => Ok(lock.season),
        None => Err(Error::Io {
            path: layout::garden_lock(root),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "a bundle names the season it was grown in, and this repository pins none",
            ),
        }),
    }
}

/// Each bed's `SKILL.md`, read from the path its manifest declares inside its cached
/// artifact, in the beds' own order.
///
/// `doctor` reads them too, to regenerate a bundle and compare it against the one on disk,
/// so where a bed's skill file lives is spelled once.
///
/// # Errors
///
/// [`Error::Bed`] naming the path when a bed declares a skill that is not there, and
/// [`Error::Io`] when the file exists and cannot be read.
pub fn read_skills(root: &Path, beds: &[Bed]) -> Result<Vec<SkillFile>> {
    let mut skills = Vec::new();
    for bed in beds {
        let Some(relative) = &bed.skill else {
            continue;
        };
        let path = layout::bed_dir(root, &bed.name).join(relative);
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::Bed {
                    bed: bed.name.clone(),
                    problem: format!("declares a skill at {}, which is not there", path.display()),
                });
            }
            Err(source) => return Err(Error::Io { path, source }),
        };
        skills.push(SkillFile {
            bed: bed.name.clone(),
            content,
        });
    }
    Ok(skills)
}

/// The binary now running: what `bundle build` copies into every bundle's `bin/plotplot`,
/// and what `doctor` compares those copies against.
///
/// This is one of the two things the library asks the environment for, and it is asked here
/// because `bin/plotplot` is a bundle's own idea. Both faces call it at their own edge and
/// pass the answer down.
///
/// # Errors
///
/// [`Error::Io`] when the operating system will not say where the running binary is.
pub fn running_stem() -> Result<PathBuf> {
    std::env::current_exe().map_err(|source| Error::Io {
        path: PathBuf::from(env!("CARGO_PKG_NAME")),
        source,
    })
}

/// Copy the running stem into a bundle's `bin/plotplot` and make it runnable, so every hook
/// entry in that bundle calls the binary that generated it.
///
/// # Errors
///
/// [`Error::Io`] naming the file that could not be created, copied or made executable.
fn install_stem(stem_binary: &Path, bundle: &Path) -> Result<PathBuf> {
    let path = bundle.join(STEM_BINARY);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(stem_binary, &path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    make_executable(&path)?;
    Ok(path)
}

/// Give a file the bit that lets a harness run it.
///
/// # Errors
///
/// [`Error::Io`] naming the file whose permissions could not be read or set.
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// On a platform with no executable bit the stem says it cannot make the binary runnable,
/// rather than writing a bundle whose hooks will never fire.
#[cfg(not(unix))]
fn make_executable(path: &Path) -> Result<()> {
    Err(Error::Io {
        path: path.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "this platform has no executable bit, so the stem cannot make a bundle's binary runnable",
        ),
    })
}

/// The events every friction kind derives from, per `contracts/friction-profile.md`'s
/// pinned kinds table, mapped onto each vendor's own event names (§5). The stem registers
/// these whether or not a bed asked for them: the emitter is the stem's own face.
const CLAUDE_FRICTION_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionDenied",
    "Stop",
    "PreCompact",
    "SessionEnd",
];
const GEMINI_FRICTION_EVENTS: &[&str] = &[
    "BeforeTool",
    "AfterTool",
    "AfterAgent",
    "PreCompress",
    "SessionEnd",
];
const CODEX_FRICTION_EVENTS: &[&str] = &["PreToolUse", "PostToolUse", "Stop", "PreCompact"];

fn friction_events(harness: Harness) -> &'static [&'static str] {
    match harness {
        Harness::Claude => CLAUDE_FRICTION_EVENTS,
        Harness::Gemini => GEMINI_FRICTION_EVENTS,
        Harness::Codex => CODEX_FRICTION_EVENTS,
    }
}

/// Every event this harness's hook file registers, in the harness's own order: the union of
/// what the beds declared, the before-tool event the deny list needs, the events the
/// friction kinds derive from, and the session-end event the receipt draft needs.
///
/// # Errors
///
/// [`Error::Bed`] when a bed declares an event this harness does not fire.
fn events_for(harness: Harness, beds: &[Bed]) -> Result<Vec<&'static str>> {
    let mut wanted: BTreeSet<&str> = BTreeSet::new();
    wanted.insert(harness.before_tool_event());
    wanted.insert(harness.session_end_event());
    wanted.extend(friction_events(harness));
    for bed in beds {
        let Some(entries) = bed.hooks.get(&harness) else {
            continue;
        };
        for entry in entries {
            if !harness.has_event(&entry.event) {
                return Err(Error::Bed {
                    bed: bed.name.clone(),
                    problem: format!(
                        "declares the {harness} hook event \"{}\", which {harness} does not fire",
                        entry.event
                    ),
                });
            }
            wanted.insert(entry.event.as_str());
        }
    }
    Ok(harness
        .events()
        .iter()
        .copied()
        .filter(|event| wanted.contains(event))
        .collect())
}

/// Where a vendor keeps its hook file inside a bundle (§5): Claude and Gemini under
/// `hooks/`, Codex at the plugin's root.
pub fn hooks_path(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude | Harness::Gemini => "hooks/hooks.json",
        Harness::Codex => "hooks.json",
    }
}

/// The one command every hook entry runs: the stem's dispatcher, under the vendor's own
/// variable for the installed bundle's directory.
///
/// `doctor` reads the entries a bundle carries and compares each against this, so the shape
/// a hook entry may take is written once.
pub fn hook_command(harness: Harness, event: &str) -> String {
    format!(
        "\"{}/{STEM_BINARY}\" hook {} {event}",
        harness.bundle_root_variable(),
        harness.name()
    )
}

/// A hook's budget in the vendor's own unit: ten seconds for an event about a tool call,
/// one second for the event that ends the session (Claude gives a session end and a stop
/// 1.5 s between them, §5), five seconds for everything else.
fn timeout(harness: Harness, event: &str) -> u64 {
    let seconds = if event == harness.session_end_event() {
        1
    } else if is_tool_event(event) {
        10
    } else {
        5
    };
    match harness {
        Harness::Gemini => seconds * 1000,
        Harness::Claude | Harness::Codex => seconds,
    }
}

/// Whether an event is about one tool call, on any of the three vendors: every `…Tool…`
/// event, plus Claude's permission events, which are a tool call's decision.
fn is_tool_event(event: &str) -> bool {
    event.contains("Tool") || event.starts_with("Permission")
}

/// The vendors' shared hook file: one entry per event, no vendor matcher, one dispatcher
/// call. The dispatcher applies each bed's own matcher, so a tool call never fires it twice.
fn hooks_file(harness: Harness, beds: &[Bed]) -> Result<Vec<u8>> {
    let events = events_for(harness, beds)?;
    let registered = events
        .into_iter()
        .map(|event| {
            let command = Json::object([
                ("type", Json::string("command")),
                ("command", Json::string(hook_command(harness, event))),
                ("timeout", Json::Number(timeout(harness, event))),
            ]);
            let group = Json::object([("hooks", Json::Array(vec![command]))]);
            (event.to_owned(), Json::Array(vec![group]))
        })
        .collect();
    Ok(Json::object([("hooks", Json::Object(registered))])
        .document()
        .into_bytes())
}

/// Every channel bed that declared how to launch its server, as MCP entries, sorted by bed
/// name. A bed whose manifest carries only a tool count has no `mcp` and is not here.
fn mcp_servers(harness: Harness, beds: &[Bed]) -> Vec<(String, Json)> {
    let mut servers: BTreeMap<String, Json> = BTreeMap::new();
    for bed in beds {
        let Some(server) = &bed.mcp else {
            continue;
        };
        let args = server
            .args
            .iter()
            .map(|arg| Json::string(mcp_argument(harness, &bed.name, arg)))
            .collect();
        let env = server
            .env
            .iter()
            .map(|name| (name.clone(), Json::string(format!("${{{name}}}"))))
            .collect();
        servers.insert(
            bed.name.clone(),
            Json::object([
                ("command", Json::string(server.command.clone())),
                ("args", Json::Array(args)),
                ("env", Json::Object(env)),
            ]),
        );
    }
    servers.into_iter().collect()
}

/// The `.mcp.json` Claude and Codex read, or `None` when no bed brought a server: an empty
/// file would claim a channel the garden does not have.
fn mcp_file(harness: Harness, beds: &[Bed]) -> Option<Vec<u8>> {
    let servers = mcp_servers(harness, beds);
    if servers.is_empty() {
        return None;
    }
    Some(
        Json::object([("mcpServers", Json::Object(servers))])
            .document()
            .into_bytes(),
    )
}

/// One argument of an MCP launch line. A path relative to the bed's artifact root is
/// resolved inside the installed bundle, where the install step lays that artifact out; a
/// flag, an absolute path and a bare subcommand are passed as the bed declared them.
fn mcp_argument(harness: Harness, bed: &str, argument: &str) -> String {
    if is_relative_path(argument) {
        format!(
            "{}/{BINARY_DIR}/{bed}/{argument}",
            harness.bundle_root_variable()
        )
    } else {
        argument.to_owned()
    }
}

/// Whether an argument is a path the vendor must resolve: not a flag, not absolute, and
/// either spelling a directory or carrying a file extension.
fn is_relative_path(argument: &str) -> bool {
    if argument.starts_with('-') || Path::new(argument).is_absolute() {
        return false;
    }
    argument.contains('/') || Path::new(argument).extension().is_some()
}

/// What a vendor shows about the bundle: the season, and the beds it plants.
fn description(input: &BundleInput) -> String {
    let names: Vec<&str> = input.beds.iter().map(|bed| bed.name.as_str()).collect();
    if names.is_empty() {
        format!(
            "the plotplot garden, season {}, with no beds planted",
            input.season
        )
    } else {
        format!(
            "the plotplot garden, season {}, planting {}",
            input.season,
            names.join(", ")
        )
    }
}

/// The first three fields of every vendor manifest, in one order.
fn manifest_head(input: &BundleInput) -> Vec<(String, Json)> {
    vec![
        ("name".to_owned(), Json::string(BUNDLE_NAME)),
        ("version".to_owned(), Json::string(input.stem_version)),
        ("description".to_owned(), Json::string(description(input))),
    ]
}

/// Put every skill file under `skills/<bed>/SKILL.md`, where all three vendors look.
///
/// # Errors
///
/// [`Error::Bed`] when a bed name is not one path component, or when two skill files claim
/// the same bed and one would silently replace the other.
fn add_skills(input: &BundleInput, tree: &mut FileTree) -> Result<()> {
    for skill in input.skills {
        if skill.bed.is_empty()
            || skill.bed.contains('/')
            || skill.bed.contains('\\')
            || skill.bed == "."
            || skill.bed == ".."
        {
            return Err(Error::Bed {
                bed: skill.bed.clone(),
                problem: "is not a name a skill directory can be called".to_owned(),
            });
        }
        let path = PathBuf::from(format!("skills/{}/SKILL.md", skill.bed));
        if tree.0.contains_key(&path) {
            return Err(Error::Bed {
                bed: skill.bed.clone(),
                problem: "was given two skill files, and one would replace the other".to_owned(),
            });
        }
        tree.insert(path, skill.content.clone().into_bytes());
    }
    Ok(())
}

/// A JSON document in the order the stem chooses, rather than the order a map sorts into.
///
/// The vendors read these files; a diff of two generated bundles should show what changed
/// in the garden, not what a hash map felt like today. Only the shapes the bundles need
/// exist here.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Json {
    String(String),
    Number(u64),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn string(value: impl Into<String>) -> Json {
        Json::String(value.into())
    }

    /// An object from fields given in the order they should be written.
    fn object<const N: usize>(fields: [(&str, Json); N]) -> Json {
        Json::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )
    }

    /// The file a vendor reads: pretty-printed with two spaces, one trailing newline.
    fn document(&self) -> String {
        let mut text = String::new();
        self.render(0, &mut text);
        text.push('\n');
        text
    }

    fn render(&self, depth: usize, out: &mut String) {
        match self {
            Json::String(value) => {
                out.push_str(&serde_json::Value::String(value.clone()).to_string());
            }
            Json::Number(value) => out.push_str(&value.to_string()),
            Json::Array(items) if items.is_empty() => out.push_str("[]"),
            Json::Array(items) => {
                out.push_str("[\n");
                for (index, item) in items.iter().enumerate() {
                    indent(depth + 1, out);
                    item.render(depth + 1, out);
                    if index + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(depth, out);
                out.push(']');
            }
            Json::Object(fields) if fields.is_empty() => out.push_str("{}"),
            Json::Object(fields) => {
                out.push_str("{\n");
                for (index, (key, value)) in fields.iter().enumerate() {
                    indent(depth + 1, out);
                    out.push_str(&serde_json::Value::String(key.clone()).to_string());
                    out.push_str(": ");
                    value.render(depth + 1, out);
                    if index + 1 < fields.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(depth, out);
                out.push('}');
            }
        }
    }
}

fn indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bed(name: &str) -> Bed {
        Bed {
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            binary: Some(name.to_owned()),
            skill: None,
            hooks: BTreeMap::new(),
            git_hooks: Vec::new(),
            mcp: None,
            check: None,
        }
    }

    fn input<'a>(beds: &'a [Bed], skills: &'a [SkillFile]) -> BundleInput<'a> {
        BundleInput {
            stem_version: "0.1.0",
            season: "2026.09",
            beds,
            skills,
        }
    }

    #[test]
    fn the_printer_writes_what_serde_json_writes() {
        // Alphabetical keys, so serde_json's own sorted order and the stem's chosen order
        // are the same document and the comparison is about formatting alone.
        let ours = Json::object([
            (
                "array",
                Json::Array(vec![Json::Number(1), Json::string("two")]),
            ),
            ("empty_array", Json::Array(Vec::new())),
            ("empty_object", Json::Object(Vec::new())),
            (
                "nested",
                Json::object([(
                    "deep",
                    Json::Array(vec![Json::object([("quoted", Json::string("a \"b\" c"))])]),
                )]),
            ),
            ("string", Json::string("plain")),
        ]);
        let theirs: serde_json::Value = serde_json::json!({
            "array": [1, "two"],
            "empty_array": [],
            "empty_object": {},
            "nested": {"deep": [{"quoted": "a \"b\" c"}]},
            "string": "plain",
        });
        let expected = format!(
            "{}\n",
            serde_json::to_string_pretty(&theirs).expect("serde_json prints it")
        );
        assert_eq!(ours.document(), expected);
    }

    #[test]
    fn a_document_ends_in_exactly_one_newline() {
        let document = Json::object([("a", Json::string("b"))]).document();
        assert_eq!(document, "{\n  \"a\": \"b\"\n}\n");
    }

    #[test]
    fn every_friction_event_is_one_its_harness_fires() {
        for harness in Harness::ALL {
            for event in friction_events(harness) {
                assert!(harness.has_event(event), "{harness}: {event}");
            }
            assert!(
                friction_events(harness).contains(&harness.session_end_event()),
                "{harness}: the session end is where the totals are"
            );
        }
    }

    #[test]
    fn the_stem_registers_its_own_events_on_every_harness() {
        for harness in Harness::ALL {
            let events = events_for(harness, &[]).expect("the stem's own events");
            assert!(events.contains(&harness.before_tool_event()), "{harness}");
            assert!(events.contains(&harness.session_end_event()), "{harness}");
            for event in friction_events(harness) {
                assert!(events.contains(event), "{harness}: {event}");
            }
            // The harness's own order, with no repeats.
            let mut sorted: Vec<usize> = events
                .iter()
                .filter_map(|event| harness.events().iter().position(|known| known == event))
                .collect();
            assert_eq!(sorted.len(), events.len(), "{harness}");
            let given = sorted.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted, given, "{harness}");
        }
    }

    #[test]
    fn a_beds_event_is_registered_once_however_many_beds_declare_it() {
        let mut first = bed("weeder");
        first.hooks.insert(
            Harness::Claude,
            vec![crate::bed::HookEntry {
                event: "Stop".to_owned(),
                matcher: None,
            }],
        );
        let mut second = bed("tend2");
        second.hooks.insert(
            Harness::Claude,
            vec![crate::bed::HookEntry {
                event: "Stop".to_owned(),
                matcher: Some("Bash".to_owned()),
            }],
        );
        let events = events_for(Harness::Claude, &[first, second]).expect("the events");
        assert_eq!(events.iter().filter(|event| **event == "Stop").count(), 1);
    }

    #[test]
    fn the_timeout_is_ten_for_a_tool_five_for_a_stop_and_one_for_the_session_end() {
        assert_eq!(timeout(Harness::Claude, "PreToolUse"), 10);
        assert_eq!(timeout(Harness::Claude, "PermissionDenied"), 10);
        assert_eq!(timeout(Harness::Claude, "Stop"), 5);
        assert_eq!(timeout(Harness::Claude, "PreCompact"), 5);
        assert_eq!(timeout(Harness::Claude, "SessionEnd"), 1);
        assert_eq!(timeout(Harness::Codex, "Stop"), 1);
        assert_eq!(timeout(Harness::Gemini, "BeforeTool"), 10_000);
        assert_eq!(timeout(Harness::Gemini, "PreCompress"), 5_000);
        assert_eq!(timeout(Harness::Gemini, "SessionEnd"), 1_000);
    }

    #[test]
    fn every_registered_event_has_a_budget_in_the_vendors_unit() {
        for harness in Harness::ALL {
            for event in harness.events() {
                let budget = timeout(harness, event);
                let seconds = match harness {
                    Harness::Gemini => budget / 1000,
                    Harness::Claude | Harness::Codex => budget,
                };
                assert!([1, 5, 10].contains(&seconds), "{harness} {event}: {budget}");
                if harness == Harness::Gemini {
                    assert_eq!(budget % 1000, 0, "{harness} {event}");
                }
            }
        }
    }

    #[test]
    fn the_command_is_the_dispatcher_under_the_vendors_root_variable() {
        assert_eq!(
            hook_command(Harness::Claude, "PreToolUse"),
            "\"${CLAUDE_PLUGIN_ROOT}/bin/plotplot\" hook claude PreToolUse"
        );
        assert_eq!(
            hook_command(Harness::Gemini, "AfterAgent"),
            "\"${extensionPath}/bin/plotplot\" hook gemini AfterAgent"
        );
        assert_eq!(
            hook_command(Harness::Codex, "Stop"),
            "\"${CLAUDE_PLUGIN_ROOT}/bin/plotplot\" hook codex Stop"
        );
    }

    #[test]
    fn only_a_path_shaped_argument_is_resolved_inside_the_bundle() {
        assert!(is_relative_path("pollen.mjs"));
        assert!(is_relative_path("dist/server.js"));
        assert!(!is_relative_path("mcp"));
        assert!(!is_relative_path("--stdio"));
        assert!(!is_relative_path("/usr/local/bin/pollen"));
        assert_eq!(
            mcp_argument(Harness::Gemini, "pollen", "pollen.mjs"),
            "${extensionPath}/bin/pollen/pollen.mjs"
        );
        assert_eq!(mcp_argument(Harness::Gemini, "tend2", "mcp"), "mcp");
    }

    #[test]
    fn a_tool_event_is_one_about_a_tool_call() {
        for event in [
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
            "BeforeTool",
            "AfterTool",
            "BeforeToolSelection",
            "PermissionDenied",
            "PermissionRequest",
        ] {
            assert!(is_tool_event(event), "{event}");
        }
        for event in [
            "Stop",
            "SessionEnd",
            "PreCompact",
            "AfterAgent",
            "PreCompress",
        ] {
            assert!(!is_tool_event(event), "{event}");
        }
    }

    #[test]
    fn the_description_names_the_season_and_every_bed() {
        let beds = [bed("weeder"), bed("tend2")];
        let planted = description(&input(&beds, &[]));
        assert_eq!(
            planted,
            "the plotplot garden, season 2026.09, planting weeder, tend2"
        );
        assert_eq!(
            description(&input(&[], &[])),
            "the plotplot garden, season 2026.09, with no beds planted"
        );
    }

    #[test]
    fn two_skill_files_for_one_bed_are_refused_rather_than_one_replacing_the_other() {
        let skills = [
            SkillFile {
                bed: "weeder".to_owned(),
                content: "first".to_owned(),
            },
            SkillFile {
                bed: "weeder".to_owned(),
                content: "second".to_owned(),
            },
        ];
        let mut tree = FileTree::new();
        let error = add_skills(&input(&[], &skills), &mut tree).expect_err("two files, one bed");
        assert!(error.to_string().contains("weeder"), "{error}");
    }

    #[test]
    fn a_skill_cannot_be_written_outside_the_skills_directory() {
        for name in ["..", ".", "", "a/b", "a\\b"] {
            let skills = [SkillFile {
                bed: name.to_owned(),
                content: "content".to_owned(),
            }];
            let mut tree = FileTree::new();
            assert!(
                add_skills(&input(&[], &skills), &mut tree).is_err(),
                "{name:?}"
            );
        }
    }

    #[test]
    fn a_bed_with_only_a_tool_count_brings_no_mcp_entry() {
        let beds = [bed("tilth")];
        assert!(mcp_servers(Harness::Claude, &beds).is_empty());
        assert_eq!(mcp_file(Harness::Claude, &beds), None);
    }
}
