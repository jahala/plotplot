//! Putting a generated bundle where the harness that owns it will find it, at project scope.
//!
//! Three vendors, three different answers, and only one of them is a command. Claude Code
//! installs from a marketplace, so the stem writes one inside `.plotplot/` and asks
//! `claude` to install out of it. Gemini CLI has no project-scope install at all, so the
//! stem writes the hooks and the MCP servers into the project's own `.gemini/settings.json`
//! and says so. Codex loads a project's own `.codex/` only once the project is trusted, so
//! the stem writes the files and says that too; granting trust is the planter's.
//!
//! What every function here shares: it writes only where its vendor looks, it merges rather
//! than replaces where a planter's file already exists, it touches nothing at user scope,
//! and it returns what it changed so `init` can print one line per change and nothing on a
//! second run. The one exception is not the stem's doing and is named where it happens:
//! `claude plugin marketplace add` records the marketplace in the planter's own settings,
//! because that is where Claude Code keeps its marketplace list.
//!
//! A bundle's hook entries call the stem through the vendor's variable for the installed
//! bundle's directory (`${extensionPath}`, `${CLAUDE_PLUGIN_ROOT}`). A project-scope
//! configuration file is not an installed bundle, so nothing expands those; the entries are
//! retargeted here, at the one place that knows which file they are going into.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value};

use crate::bundle::{self, FileTree};
use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::init::write_if_changed;
use crate::layout;

/// The marketplace Claude Code installs the bundle out of, and the name `init` asks for.
pub const MARKETPLACE: &str = "plotplot-local";

/// Where the marketplace lives inside a planted repository. Machine-written, so under
/// `.plotplot/` like everything else the stem writes for itself (the footprint rule).
pub const MARKETPLACE_DIR: &str = "marketplace";

/// Gemini CLI's own variable for the project root, the one thing a project-scope hook
/// command can lean on. Verified against the CLI's bundled hooks reference (0.46.0).
pub const GEMINI_PROJECT_DIR: &str = "$GEMINI_PROJECT_DIR";

/// What one harness's install did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub harness: Harness,
    /// Every file or setting whose bytes this install changed, repository-relative where it
    /// is a file in the repository. Empty when the harness was already installed.
    pub changed: Vec<String>,
    /// What a planter has to know and the stem cannot do for them.
    pub notes: Vec<String>,
}

// ------------------------------------------------------------------------ the paths

/// `.plotplot/marketplace/`, the local marketplace Claude Code installs out of.
pub fn marketplace_dir(root: &Path) -> PathBuf {
    layout::plotplot_dir(root).join(MARKETPLACE_DIR)
}

/// The marketplace document Claude Code reads, inside that directory.
pub fn marketplace_manifest(root: &Path) -> PathBuf {
    marketplace_dir(root).join(".claude-plugin/marketplace.json")
}

/// The plugin directory inside the marketplace.
///
/// Claude Code 2.1.265 refuses a marketplace entry whose path climbs out of the marketplace
/// directory, so the bundle cannot be named where it is generated and is mirrored here
/// instead. The mirror is written from the same bytes every time, so it stays a copy and
/// never a second source.
pub fn marketplace_plugin(root: &Path) -> PathBuf {
    marketplace_dir(root)
        .join("plugins")
        .join(bundle::BUNDLE_NAME)
}

/// Gemini CLI's project settings, which carry its hooks and its MCP servers.
pub const GEMINI_SETTINGS: &str = ".gemini/settings.json";

/// Codex's project configuration, which carries its MCP servers.
pub const CODEX_CONFIG: &str = ".codex/config.toml";

/// Codex's project hook file, beside its project configuration.
pub const CODEX_HOOKS: &str = ".codex/hooks.json";

/// Claude Code's project settings, written by `claude plugin install --scope project`.
pub const CLAUDE_SETTINGS: &str = ".claude/settings.json";

/// The project file each harness reads at project scope, or `None` for Claude Code, whose
/// project file is written by `claude` itself and never by the stem.
pub fn project_config(harness: Harness) -> Option<&'static str> {
    match harness {
        Harness::Claude => None,
        Harness::Gemini => Some(GEMINI_SETTINGS),
        Harness::Codex => Some(CODEX_CONFIG),
    }
}

// ------------------------------------------------------------------ the three installs

/// Install one harness's bundle at project scope.
///
/// # Errors
///
/// Whatever the harness's own install refuses; see each function below.
pub fn install(root: &Path, harness: Harness, tree: &FileTree) -> Result<Installed> {
    match harness {
        Harness::Claude => claude(root, tree),
        Harness::Gemini => gemini(root, tree),
        Harness::Codex => codex(root, tree),
    }
}

/// Claude Code: a local marketplace under `.plotplot/`, then the vendor's own two commands.
///
/// The commands are idempotent (2.1.265 answers "already on disk" and "already installed"),
/// so they run on every `init`; what decides whether anything changed is the project's own
/// `.claude/settings.json`, which is where project scope actually lands.
///
/// # Errors
///
/// [`Error::Install`] naming the exact command and its stderr when `claude` is not on PATH
/// or refuses, and [`Error::Io`] when a file under `.plotplot/marketplace/` cannot be
/// written.
pub fn claude(root: &Path, tree: &FileTree) -> Result<Installed> {
    let mut changed = Vec::new();

    if write_if_changed(
        &marketplace_dir(root).join(".claude-plugin/marketplace.json"),
        marketplace_json().as_bytes(),
    )? {
        changed.push(relative(
            root,
            &marketplace_dir(root).join(".claude-plugin/marketplace.json"),
        ));
    }
    let plugin = marketplace_plugin(root);
    for path in mirror(tree, &plugin)? {
        changed.push(relative(root, &path));
    }
    // The bundle's `bin/plotplot` is put there by the install step rather than generated, so
    // it is not in the tree and is mirrored from the bundle on disk.
    let stem = layout::bundle_dir(root, Harness::Claude).join(bundle::STEM_BINARY);
    let mirrored = plugin.join(bundle::STEM_BINARY);
    if copy_if_changed(&stem, &mirrored)? {
        changed.push(relative(root, &mirrored));
    }

    let settings = root.join(CLAUDE_SETTINGS);
    let before = std::fs::read(&settings).ok();

    let marketplace_path = format!("./{}/{MARKETPLACE_DIR}", layout::PLOTPLOT_DIR);
    run_claude(root, &["plugin", "marketplace", "add", &marketplace_path])?;
    let plugin_id = format!("{}@{MARKETPLACE}", bundle::BUNDLE_NAME);
    run_claude(
        root,
        &["plugin", "install", &plugin_id, "--scope", "project"],
    )?;

    if std::fs::read(&settings).ok() != before {
        changed.push(relative(root, &settings));
    }

    Ok(Installed {
        harness: Harness::Claude,
        changed,
        notes: vec![format!(
            "claude records the {MARKETPLACE} marketplace in the planter's own settings; the \
             stem writes nothing at user scope"
        )],
    })
}

/// Gemini CLI: no project-scope install exists, so the project's own settings carry the
/// hooks and the MCP servers, and only those two keys.
///
/// # Errors
///
/// [`Error::Install`] when the settings file is not a JSON object or the bundle carries no
/// hook file, [`Error::Json`] when the settings file is not JSON at all, [`Error::Io`] when
/// it cannot be read or written.
pub fn gemini(root: &Path, tree: &FileTree) -> Result<Installed> {
    let path = root.join(GEMINI_SETTINGS);
    let existing = read_if_present(&path)?;
    let bundle_root = format!("{GEMINI_PROJECT_DIR}/{}", relative_bundle(Harness::Gemini));

    let hooks = retargeted_hooks(Harness::Gemini, tree, &bundle_root)?;
    let servers = retargeted_servers(Harness::Gemini, tree, &bundle_root)?;
    let merged = merge_settings(Harness::Gemini, existing.as_deref(), hooks, servers)?;

    let mut changed = Vec::new();
    if write_if_changed(&path, merged.as_bytes())? {
        changed.push(relative(root, &path));
    }

    Ok(Installed {
        harness: Harness::Gemini,
        changed,
        notes: vec![format!(
            "gemini has no project-scope install, so its hooks and servers went into {}",
            relative(root, &path)
        )],
    })
}

/// Codex CLI: the project's own `.codex/`, which Codex reads once the project is trusted.
///
/// # Errors
///
/// [`Error::Install`] when the bundle carries no hook file or the project's `config.toml` is
/// not a TOML table, [`Error::Toml`] when it is not TOML at all, [`Error::Io`] when a file
/// cannot be read or written.
pub fn codex(root: &Path, tree: &FileTree) -> Result<Installed> {
    let bundle_root = absolute(&layout::bundle_dir(root, Harness::Codex));

    let events = retargeted_hooks(Harness::Codex, tree, &bundle_root)?;
    let hooks_file = serde_json::json!({ "hooks": Value::Object(events) });
    let hooks_path = root.join(CODEX_HOOKS);
    let mut changed = Vec::new();
    if write_if_changed(&hooks_path, document(&hooks_file).as_bytes())? {
        changed.push(relative(root, &hooks_path));
    }

    let config_path = root.join(CODEX_CONFIG);
    let servers = retargeted_servers(Harness::Codex, tree, &bundle_root)?;
    let existing = read_if_present(&config_path)?;
    if let Some(merged) = merge_codex_config(existing.as_deref(), &servers, &config_path)?
        && write_if_changed(&config_path, merged.as_bytes())?
    {
        changed.push(relative(root, &config_path));
    }

    Ok(Installed {
        harness: Harness::Codex,
        changed,
        notes: vec![format!(
            "codex loads {CODEX_HOOKS} and {CODEX_CONFIG} only once the project is trusted: \
             add [projects.\"{}\"] trust_level = \"trusted\" to the planter's codex config",
            absolute(root)
        )],
    })
}

// ------------------------------------------------------------------ the pure renderers

/// The marketplace document Claude Code reads.
///
/// `source` is a marketplace-relative path string: 2.1.265 takes that form for a local
/// plugin and refuses both the object forms its schema documents for remote sources and any
/// path that climbs out of the marketplace directory.
pub fn marketplace_json() -> String {
    let marketplace = serde_json::json!({
        "name": MARKETPLACE,
        "owner": { "name": "the plotplot garden" },
        "plugins": [
            {
                "name": bundle::BUNDLE_NAME,
                "description": "the plotplot garden, planted in this repository",
                "source": format!("./plugins/{}", bundle::BUNDLE_NAME),
            }
        ]
    });
    document(&marketplace)
}

/// The bundle's events, one entry per event, with every dispatcher call retargeted at
/// `bundle_root`.
///
/// What comes back is the inner map: the vendors' hook file wraps it in a `hooks` key, and
/// so does a settings file, so the wrapping belongs to whoever is writing the file.
///
/// # Errors
///
/// [`Error::Install`] when the bundle carries no hook file or one the stem did not generate.
pub fn retargeted_hooks(
    harness: Harness,
    tree: &FileTree,
    bundle_root: &str,
) -> Result<Map<String, Value>> {
    let path = bundle::hooks_path(harness);
    let bytes = tree.get(path).ok_or_else(|| Error::Install {
        harness,
        problem: format!("the generated bundle carries no {path}"),
    })?;
    let mut document: Value = serde_json::from_slice(bytes).map_err(|source| Error::Json {
        path: Some(PathBuf::from(path)),
        source,
    })?;
    retarget(&mut document, harness.bundle_root_variable(), bundle_root);
    match document.get_mut("hooks") {
        Some(Value::Object(events)) => Ok(std::mem::take(events)),
        _ => Err(Error::Install {
            harness,
            problem: format!("the generated {path} carries no hooks object"),
        }),
    }
}

/// The bundle's MCP servers with every relative argument retargeted at `bundle_root`, or an
/// empty map when no channel bed brought a server.
///
/// Claude and Codex keep them in `.mcp.json`; Gemini keeps them inside its extension
/// manifest (§5), so this reads whichever file the harness's bundle has.
///
/// # Errors
///
/// [`Error::Install`] when the file the harness keeps its servers in is not a JSON object.
pub fn retargeted_servers(
    harness: Harness,
    tree: &FileTree,
    bundle_root: &str,
) -> Result<Map<String, Value>> {
    let (path, key) = match harness {
        Harness::Claude | Harness::Codex => (".mcp.json", "mcpServers"),
        Harness::Gemini => ("gemini-extension.json", "mcpServers"),
    };
    let Some(bytes) = tree.get(path) else {
        return Ok(Map::new());
    };
    let document: Value = serde_json::from_slice(bytes).map_err(|source| Error::Json {
        path: Some(PathBuf::from(path)),
        source,
    })?;
    let mut servers = match document.get(key) {
        None => return Ok(Map::new()),
        Some(Value::Object(servers)) => Value::Object(servers.clone()),
        Some(_) => {
            return Err(Error::Install {
                harness,
                problem: format!("the generated {path} has a {key} that is not an object"),
            });
        }
    };
    retarget(&mut servers, harness.bundle_root_variable(), bundle_root);
    match servers {
        Value::Object(servers) => Ok(servers),
        // `retarget` replaces inside strings and never changes a value's shape.
        other => Err(Error::Install {
            harness,
            problem: format!("the generated {path}'s {key} became {other}"),
        }),
    }
}

/// Every occurrence of `from` inside every string in `value`, replaced by `to`.
///
/// The bundles put the vendor's root variable inside command strings and inside MCP
/// arguments, so this walks the whole document rather than knowing where they are.
fn retarget(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(text) => {
            if text.contains(from) {
                *text = text.replace(from, to);
            }
        }
        Value::Array(items) => {
            for item in items {
                retarget(item, from, to);
            }
        }
        Value::Object(fields) => {
            for field in fields.values_mut() {
                retarget(field, from, to);
            }
        }
        _ => {}
    }
}

/// A harness's project settings with `hooks` and `mcpServers` set and every other key of the
/// planter's carried through.
///
/// An empty server map leaves `mcpServers` alone rather than writing an empty object: the
/// stem says nothing about a channel the garden does not have.
///
/// # Errors
///
/// [`Error::Install`] when the existing settings are not a JSON object, [`Error::Json`] when
/// they are not JSON.
pub fn merge_settings(
    harness: Harness,
    existing: Option<&str>,
    hooks: Map<String, Value>,
    servers: Map<String, Value>,
) -> Result<String> {
    let mut settings = match existing {
        None => Map::new(),
        Some(text) if text.trim().is_empty() => Map::new(),
        Some(text) => {
            let value: Value = serde_json::from_str(text).map_err(|source| Error::Json {
                path: project_config(harness).map(PathBuf::from),
                source,
            })?;
            match value {
                Value::Object(fields) => fields,
                _ => {
                    return Err(Error::Install {
                        harness,
                        problem: format!(
                            "{} is not a JSON object, so the stem cannot merge into it",
                            project_config(harness).unwrap_or("its project settings")
                        ),
                    });
                }
            }
        }
    };

    settings.insert("hooks".to_owned(), Value::Object(hooks));
    if !servers.is_empty() {
        settings.insert("mcpServers".to_owned(), Value::Object(servers));
    }
    Ok(document(&Value::Object(settings)))
}

/// A Codex project `config.toml` with `[mcp_servers]` set and every other key carried
/// through, or `None` when there is nothing to write: no servers and no file already there.
///
/// # Errors
///
/// [`Error::Toml`] naming the file when it is not TOML or the merged table cannot be
/// written back.
pub fn merge_codex_config(
    existing: Option<&str>,
    servers: &Map<String, Value>,
    path: &Path,
) -> Result<Option<String>> {
    if servers.is_empty() && existing.is_none() {
        return Ok(None);
    }

    let refuse = |message: String| Error::Toml {
        path: path.to_path_buf(),
        message,
    };

    let mut table: toml::Table = match existing {
        None => toml::Table::new(),
        Some(text) => text.parse().map_err(|error: toml::de::Error| {
            refuse(crate::manifest::one_line(&error.to_string()))
        })?,
    };

    if servers.is_empty() {
        return Ok(None);
    }
    let servers = toml::Value::try_from(Value::Object(servers.clone()))
        .map_err(|error| refuse(crate::manifest::one_line(&error.to_string())))?;
    table.insert("mcp_servers".to_owned(), servers);

    let text = toml::to_string_pretty(&table)
        .map_err(|error| refuse(crate::manifest::one_line(&error.to_string())))?;
    Ok(Some(text))
}

/// A JSON file the way the bundles write one: two-space indent, one trailing newline.
///
/// `to_string_pretty` only fails on a value serde cannot render, and every value here came
/// out of serde in the first place; the fallback is the same document without the indent
/// rather than a panic in a face that is writing a planter's configuration.
fn document(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    text.push('\n');
    text
}

// ------------------------------------------------------------------------- the edge

/// Run one `claude` command in `root`, reporting the exact command and its stderr.
fn run_claude(root: &Path, args: &[&str]) -> Result<()> {
    let line = format!("claude {}", args.join(" "));
    let refuse = |problem: String| Error::Install {
        harness: Harness::Claude,
        problem,
    };

    let output = Command::new("claude").args(args).current_dir(root).output();
    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(refuse(format!(
                "`{line}` could not run: claude is not on PATH"
            )));
        }
        Err(error) => return Err(refuse(format!("`{line}` could not run: {error}"))),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let said = if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        };
        return Err(refuse(format!(
            "`{line}` exited {}: {}",
            match output.status.code() {
                Some(code) => code.to_string(),
                None => "on a signal".to_owned(),
            },
            crate::manifest::one_line(&said)
        )));
    }
    Ok(())
}

/// Write every file of `tree` under `into`, skipping the ones already holding those bytes,
/// and return the paths whose bytes changed.
fn mirror(tree: &FileTree, into: &Path) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();
    for (relative, bytes) in &tree.0 {
        let path = into.join(relative);
        if write_if_changed(&path, bytes)? {
            changed.push(path);
        }
    }
    Ok(changed)
}

/// Copy `from` to `to` when their bytes differ, keeping `to` runnable.
fn copy_if_changed(from: &Path, to: &Path) -> Result<bool> {
    let bytes = std::fs::read(from).map_err(|source| Error::Io {
        path: from.to_path_buf(),
        source,
    })?;
    if !write_if_changed(to, &bytes)? {
        return Ok(false);
    }
    bundle::make_executable(to)?;
    Ok(true)
}

/// A file's contents, or `None` when it is not there.
fn read_if_present(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// A path as `init` prints it: relative to the repository when it is inside one.
pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// One bundle's directory, relative to the repository root, with forward slashes: what a
/// project-scope configuration file has to spell.
fn relative_bundle(harness: Harness) -> String {
    format!("{}/bundles/{}", layout::PLOTPLOT_DIR, harness.name())
}

/// A path a vendor can resolve from anywhere. The repository root is already absolute when
/// `main` reads it from the current directory; a relative one is answered as it is rather
/// than guessed at, and the vendor resolves it against its own working directory.
fn absolute(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

// ---------------------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::bed::{Bed, GitHook, HookEntry};
    use crate::bundle::{BundleInput, SkillFile};

    fn weeder() -> Bed {
        let mut hooks: BTreeMap<Harness, Vec<HookEntry>> = BTreeMap::new();
        hooks.insert(
            Harness::Gemini,
            vec![
                HookEntry {
                    event: "BeforeTool".to_owned(),
                    matcher: Some("run_shell_command".to_owned()),
                },
                HookEntry {
                    event: "AfterAgent".to_owned(),
                    matcher: None,
                },
            ],
        );
        hooks.insert(
            Harness::Codex,
            vec![
                HookEntry {
                    event: "PreToolUse".to_owned(),
                    matcher: None,
                },
                HookEntry {
                    event: "Stop".to_owned(),
                    matcher: None,
                },
            ],
        );
        Bed {
            name: "weeder".to_owned(),
            version: "0.1.0".to_owned(),
            binary: Some("weeder".to_owned()),
            skill: None,
            hooks,
            git_hooks: vec![GitHook::PreCommit],
            mcp: None,
            check: Some("weeder check --format sarif".to_owned()),
        }
    }

    fn tree(harness: Harness) -> FileTree {
        let beds = [weeder()];
        let skills: [SkillFile; 0] = [];
        let input = BundleInput {
            stem_version: "0.1.0",
            season: "2026.09",
            beds: &beds,
            skills: &skills,
        };
        bundle::generate(harness, &input).expect("the bundle generates")
    }

    #[test]
    fn the_marketplace_names_the_bundle_as_a_path_inside_itself() {
        let document: Value = serde_json::from_str(&marketplace_json()).expect("JSON");
        assert_eq!(document["name"], MARKETPLACE);
        assert_eq!(document["plugins"][0]["name"], bundle::BUNDLE_NAME);
        let source = document["plugins"][0]["source"]
            .as_str()
            .expect("a source string");
        assert!(source.starts_with("./"), "{source}");
        assert!(
            !source.contains(".."),
            "a climbing path is refused: {source}"
        );
    }

    #[test]
    fn geminis_hook_commands_resolve_through_geminis_own_project_variable() {
        let tree = tree(Harness::Gemini);
        let events = retargeted_hooks(
            Harness::Gemini,
            &tree,
            &format!("{GEMINI_PROJECT_DIR}/{}", relative_bundle(Harness::Gemini)),
        )
        .expect("the hooks retarget");
        assert!(!events.is_empty());
        for (event, groups) in &events {
            let command = groups[0]["hooks"][0]["command"]
                .as_str()
                .expect("a command string");
            assert!(command.contains(GEMINI_PROJECT_DIR), "{event}: {command}");
            assert!(!command.contains("${extensionPath}"), "{event}: {command}");
            assert!(
                command.ends_with(&format!("hook gemini {event}")),
                "{command}"
            );
        }
    }

    #[test]
    fn codexs_hook_commands_resolve_through_the_bundles_own_directory() {
        let tree = tree(Harness::Codex);
        let events = retargeted_hooks(Harness::Codex, &tree, "/work/repo/.plotplot/bundles/codex")
            .expect("the hooks retarget");
        for (event, groups) in &events {
            let command = groups[0]["hooks"][0]["command"]
                .as_str()
                .expect("a command string");
            assert!(
                command.starts_with("\"/work/repo/.plotplot/bundles/codex/bin/plotplot\""),
                "{event}: {command}"
            );
            assert!(!command.contains("${CLAUDE_PLUGIN_ROOT}"), "{command}");
        }
    }

    #[test]
    fn merging_settings_keeps_every_other_key_and_writes_the_same_bytes_twice() {
        let tree = tree(Harness::Gemini);
        let hooks = retargeted_hooks(Harness::Gemini, &tree, "$X").expect("hooks");
        let servers = retargeted_servers(Harness::Gemini, &tree, "$X").expect("servers");

        let mine = "{ \"model\": { \"name\": \"gemini-3-pro\" }, \"hooks\": { \"Old\": [] } }";
        let once = merge_settings(Harness::Gemini, Some(mine), hooks.clone(), servers.clone())
            .expect("a merge");
        let twice = merge_settings(Harness::Gemini, Some(&once), hooks, servers).expect("a merge");
        assert_eq!(once, twice, "the merge is a fixed point");

        let settings: Value = serde_json::from_str(&once).expect("JSON");
        assert_eq!(settings["model"]["name"], "gemini-3-pro");
        assert!(
            settings["hooks"]["Old"].is_null(),
            "the stem owns the hooks key"
        );
        assert!(settings["hooks"]["BeforeTool"].is_array());
    }

    #[test]
    fn settings_that_are_not_an_object_are_refused_and_name_the_harness() {
        let error = merge_settings(Harness::Gemini, Some("[1, 2]"), Map::new(), Map::new())
            .expect_err("an array is not settings");
        assert!(
            matches!(&error, Error::Install { harness, .. } if *harness == Harness::Gemini),
            "{error:?}"
        );
        assert!(error.to_string().starts_with("gemini: "), "{error}");
    }

    #[test]
    fn a_bundle_with_no_servers_leaves_the_codex_config_alone() {
        let path = Path::new("/work/repo/.codex/config.toml");
        assert_eq!(
            merge_codex_config(None, &Map::new(), path).expect("no work"),
            None
        );
        assert_eq!(
            merge_codex_config(Some("model = \"gpt\"\n"), &Map::new(), path).expect("no work"),
            None,
            "a planter's config is not rewritten to say nothing"
        );
    }

    #[test]
    fn merging_servers_into_a_codex_config_keeps_the_planters_own_keys() {
        let path = Path::new("/work/repo/.codex/config.toml");
        let mut servers = Map::new();
        servers.insert(
            "tilth".to_owned(),
            serde_json::json!({ "command": "tilth", "args": ["mcp"] }),
        );
        let once = merge_codex_config(Some("model = \"gpt-5.5\"\n"), &servers, path)
            .expect("a merge")
            .expect("something to write");
        let twice = merge_codex_config(Some(&once), &servers, path)
            .expect("a merge")
            .expect("something to write");
        assert_eq!(once, twice, "the merge is a fixed point");

        let table: toml::Table = once.parse().expect("the merged config is TOML");
        assert_eq!(table["model"].as_str(), Some("gpt-5.5"));
        assert_eq!(
            table["mcp_servers"]["tilth"]["command"].as_str(),
            Some("tilth")
        );
    }

    #[test]
    fn a_codex_config_that_is_not_toml_is_refused_and_names_the_file() {
        let path = Path::new("/work/repo/.codex/config.toml");
        let mut servers = Map::new();
        servers.insert(
            "tilth".to_owned(),
            serde_json::json!({ "command": "tilth" }),
        );
        let error =
            merge_codex_config(Some("this is [ not toml"), &servers, path).expect_err("not TOML");
        assert!(
            error
                .to_string()
                .starts_with(&format!("{}: ", path.display())),
            "{error}"
        );
    }

    #[test]
    fn each_harness_project_file_is_named_once() {
        assert_eq!(project_config(Harness::Claude), None);
        assert_eq!(
            project_config(Harness::Gemini),
            Some(".gemini/settings.json")
        );
        assert_eq!(project_config(Harness::Codex), Some(".codex/config.toml"));
    }

    #[test]
    fn the_marketplace_plugin_sits_inside_the_marketplace_directory() {
        let root = Path::new("/work/repo");
        assert!(marketplace_plugin(root).starts_with(marketplace_dir(root)));
        assert!(marketplace_dir(root).starts_with(layout::plotplot_dir(root)));
    }
}
