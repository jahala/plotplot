//! `garden.json`, the contracts' bed manifest.
//!
//! The schema is the contracts' product, embedded here and enforced before serde sees the
//! document, so a manifest the contracts reject never becomes a [`Bed`]. What the stem does
//! beyond the schema is refuse what it cannot honestly plant: a harness it does not know, an
//! event a harness does not fire, a git hook it does not install.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bed::{Bed, GitHook, HookEntry, McpServer};
use crate::error::{Error, Result};
use crate::harness::Harness;
use crate::layout;

/// The contracts' manifest schema, v1.2.0, embedded so the binary carries its own contract.
pub const MANIFEST_SCHEMA: &str = include_str!("../contracts/manifest.schema.json");

/// How a refusal by the contracts opens, as opposed to one by the stem. The two are
/// different failures: the first says the manifest is not a manifest, the second says the
/// stem cannot plant it.
pub const SCHEMA_REFUSED: &str = "contracts/manifest.schema.json refused it";

/// The serde model of `garden.json`. Fields the stem does not read are kept in `extra`
/// rather than dropped, so a manifest written against a later contracts version survives a
/// round trip through the stem.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Manifest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub kind: Vec<String>,
    pub version: String,
    pub install: Install,
    pub faces: Faces,
    #[serde(default)]
    pub check: Option<String>,
    pub metric: String,
    pub context: Context,
    pub coverage: Coverage,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flagged: Vec<Flagged>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// How to obtain the bed.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Install {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInstall>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub binaries: BTreeMap<String, String>,
    /// The executable's file name inside the release artifact; defaults to `faces.cli`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_name: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// A bed fetched by cloning at a pinned commit.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitInstall {
    pub url: String,
    pub rev: String,
}

/// The bed's surfaces.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Faces {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hooks: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpFace>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// A channel bed's MCP face. `tools` is always there; the launch line is not, and a bed
/// without one is a channel the stem cannot plant yet.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct McpFace {
    pub tools: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
}

/// The bed's declared startup context cost (F4).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Context {
    /// `None` when genuinely unmeasured; `Some(0)` is a true zero.
    pub upfront_tokens: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// What the bed's judgement actually covers, so silence on an unsupported language is
/// declared rather than read as clean.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Coverage {
    pub languages: Vec<String>,
    pub kinds: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// An unknown the manifest's author named rather than guessed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Flagged {
    pub what: String,
    pub why: String,
}

/// Read a `garden.json`: the contracts' schema first, then serde.
///
/// # Errors
///
/// [`Error::Json`] when the bytes are not JSON, [`Error::Manifest`] when they are JSON the
/// schema or the serde model refuses.
pub fn parse_manifest(json: &str) -> Result<Manifest> {
    parse_manifest_at(None, json)
}

/// Read a `garden.json` that came from a file, so a JSON failure can name it.
///
/// # Errors
///
/// As [`parse_manifest`], with the path attached to a JSON failure.
pub fn parse_manifest_at(path: Option<&Path>, json: &str) -> Result<Manifest> {
    let value: Value = serde_json::from_str(json).map_err(|source| Error::Json {
        path: path.map(Path::to_path_buf),
        source,
    })?;
    let bed = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("(unnamed)")
        .to_owned();

    let validator = manifest_validator()
        .as_ref()
        .map_err(|problem| Error::Manifest {
            bed: bed.clone(),
            problem: format!("the embedded manifest schema did not compile: {problem}"),
        })?;
    if let Err(error) = validator.validate(&value) {
        let at = error.instance_path().to_string();
        let where_ = if at.is_empty() {
            String::new()
        } else {
            format!(" at {at}")
        };
        return Err(Error::Manifest {
            bed,
            problem: one_line(&format!("{SCHEMA_REFUSED}{where_}: {error}")),
        });
    }

    serde_json::from_value(value).map_err(|source| Error::Manifest {
        bed,
        problem: one_line(&format!("the stem could not read it: {source}")),
    })
}

/// Turn a manifest into the bed the rest of the stem consumes.
///
/// # Errors
///
/// [`Error::Manifest`] when the manifest declares a face the stem cannot plant: an unknown
/// harness, an event that harness does not fire, an empty matcher, a git hook the stem does
/// not install, or hooks with no program to run them.
pub fn to_bed(manifest: &Manifest) -> Result<Bed> {
    let bed = manifest.name.clone();
    let refuse = |problem: String| Error::Manifest {
        bed: bed.clone(),
        problem,
    };

    let binary = manifest
        .install
        .binary_name
        .clone()
        .or_else(|| manifest.faces.cli.clone());

    let mut hooks: BTreeMap<Harness, Vec<HookEntry>> = BTreeMap::new();
    for (name, events) in &manifest.faces.hooks {
        let harness = name.parse::<Harness>().map_err(|_| {
            refuse(format!(
                "declares hooks for unknown harness \"{name}\"; the stem knows claude, gemini and codex"
            ))
        })?;
        let mut entries = Vec::with_capacity(events.len());
        for declared in events {
            entries.push(hook_entry(harness, declared).map_err(&refuse)?);
        }
        if !entries.is_empty() {
            hooks.insert(harness, entries);
        }
    }
    if !hooks.is_empty() && binary.is_none() {
        return Err(refuse(
            "declares hooks but names neither faces.cli nor install.binary_name, so the stem has no program to call".to_owned(),
        ));
    }

    let mut git_hooks = Vec::with_capacity(manifest.faces.git.len());
    for declared in &manifest.faces.git {
        let hook = GitHook::from_file_name(declared).ok_or_else(|| {
            refuse(format!(
                "declares git hook \"{declared}\", which the stem does not install; it installs {}",
                GitHook::ALL
                    .iter()
                    .map(|hook| hook.file_name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;
        if !git_hooks.contains(&hook) {
            git_hooks.push(hook);
        }
    }

    let mcp = manifest.faces.mcp.as_ref().and_then(|face| {
        face.command.as_ref().map(|command| McpServer {
            command: command.clone(),
            args: face.args.clone(),
            env: face.env.clone(),
        })
    });

    Ok(Bed {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        binary,
        skill: manifest.faces.skill.as_ref().map(PathBuf::from),
        hooks,
        git_hooks,
        mcp,
        check: manifest.check.clone(),
    })
}

/// Every `.plotplot/beds/*/garden.json` under `root`, as beds, sorted by name.
///
/// An absent beds directory is an unplanted repository, not a failure.
///
/// # Errors
///
/// [`Error::Io`] when a cached manifest cannot be read, [`Error::Bed`] when a bed directory
/// carries no manifest, and whatever [`parse_manifest_at`] and [`to_bed`] refuse.
pub fn load_beds(root: &Path) -> Result<Vec<Bed>> {
    let dir = layout::beds_dir(root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::Io { path: dir, source }),
    };

    let mut beds = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.metadata().map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        let manifest_path = path.join(layout::GARDEN_JSON);
        let json = match std::fs::read_to_string(&manifest_path) {
            Ok(json) => json,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::Bed {
                    bed: name,
                    problem: format!("has no {}", manifest_path.display()),
                });
            }
            Err(source) => {
                return Err(Error::Io {
                    path: manifest_path,
                    source,
                });
            }
        };
        beds.push(to_bed(&parse_manifest_at(Some(&manifest_path), &json)?)?);
    }
    beds.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(beds)
}

/// `"Event"` or `"Event:Matcher"`, checked against the events the harness really fires.
fn hook_entry(harness: Harness, declared: &str) -> std::result::Result<HookEntry, String> {
    let (event, matcher) = match declared.split_once(':') {
        Some((event, matcher)) if !matcher.is_empty() => (event, Some(matcher.to_owned())),
        Some(_) => {
            return Err(format!(
                "declares \"{declared}\" on {harness}, whose matcher is empty"
            ));
        }
        None => (declared, None),
    };
    if !harness.has_event(event) {
        return Err(format!(
            "declares \"{declared}\" on {harness}, which does not fire the event {event}"
        ));
    }
    Ok(HookEntry {
        event: event.to_owned(),
        matcher,
    })
}

/// The compiled manifest schema, built once. The embedded schema is the crate's own, so a
/// compilation failure is a build defect rather than user input; it is still returned rather
/// than unwrapped.
fn manifest_validator() -> &'static std::result::Result<jsonschema::Validator, String> {
    static VALIDATOR: OnceLock<std::result::Result<jsonschema::Validator, String>> =
        OnceLock::new();
    VALIDATOR.get_or_init(|| compile(MANIFEST_SCHEMA))
}

/// Compile an embedded schema, reporting rather than panicking on a broken one.
pub(crate) fn compile(schema: &str) -> std::result::Result<jsonschema::Validator, String> {
    let value: Value = serde_json::from_str(schema).map_err(|error| error.to_string())?;
    jsonschema::Validator::new(&value).map_err(|error| error.to_string())
}

/// Errors are printed one to a line; a validator message that wraps is folded back.
pub(crate) fn one_line(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/fixtures/manifest")
    }

    fn fixture(name: &str) -> String {
        let path = fixtures().join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// The weeder fixture as a `Value`, for tests that need one field changed and everything
    /// else real.
    fn weeder_value() -> serde_json::Value {
        serde_json::from_str(&fixture("weeder.garden.json")).expect("valid fixture JSON")
    }

    fn hook_entries(bed: &Bed, harness: Harness) -> Vec<(&str, Option<&str>)> {
        bed.hooks
            .get(&harness)
            .map(|entries| {
                entries
                    .iter()
                    .map(|entry| (entry.event.as_str(), entry.matcher.as_deref()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn manifest_error(json: &str) -> (String, String) {
        match parse_manifest(json).and_then(|manifest| to_bed(&manifest)) {
            Err(Error::Manifest { bed, problem }) => (bed, problem),
            other => panic!("expected Error::Manifest, got {other:?}"),
        }
    }

    #[test]
    fn every_positive_fixture_parses_and_becomes_a_bed() {
        let mut seen = 0;
        let entries = std::fs::read_dir(fixtures()).expect("the manifest fixture directory");
        for entry in entries {
            let path = entry.expect("a readable directory entry").path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("a fixture file name")
                .to_owned();
            if !name.ends_with(".json") || name.starts_with("invalid-") {
                continue;
            }
            seen += 1;
            let json = std::fs::read_to_string(&path).expect("a readable fixture");
            let manifest = parse_manifest(&json).unwrap_or_else(|e| panic!("{name}: {e}"));
            let bed = to_bed(&manifest).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                bed.name,
                name.trim_end_matches(".garden.json"),
                "the fixture is named after its bed"
            );
            assert!(!bed.version.is_empty(), "{name}");
        }
        assert_eq!(seen, 8, "the eight beds with a manifest fixture");
    }

    #[test]
    fn every_invalid_fixture_is_refused_and_names_its_bed() {
        let mut seen = 0;
        let entries = std::fs::read_dir(fixtures()).expect("the manifest fixture directory");
        for entry in entries {
            let path = entry.expect("a readable directory entry").path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("a fixture file name")
                .to_owned();
            if !name.starts_with("invalid-") {
                continue;
            }
            seen += 1;
            let json = std::fs::read_to_string(&path).expect("a readable fixture");
            let declared = serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .and_then(|value| {
                    value
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
                .expect("every invalid fixture still declares a name");
            let (bed, problem) = manifest_error(&json);
            assert_eq!(bed, declared, "{name}");
            assert!(!problem.is_empty(), "{name}");
            assert!(!problem.contains('\n'), "{name}: {problem}");
        }
        assert_eq!(seen, 4, "the four invalid manifest fixtures");
    }

    #[test]
    fn the_weeder_fixture_becomes_the_bed_the_stem_plants() {
        let manifest = parse_manifest(&fixture("weeder.garden.json")).expect("a parsed manifest");
        let bed = to_bed(&manifest).expect("a bed");

        assert_eq!(bed.name, "weeder");
        assert_eq!(bed.version, "0.1.0");
        assert_eq!(bed.binary.as_deref(), Some("weeder"));
        assert_eq!(bed.skill.as_deref(), Some(Path::new("SKILL.md")));
        assert_eq!(
            hook_entries(&bed, Harness::Claude),
            [("PreToolUse", Some("Bash")), ("Stop", None)]
        );
        assert_eq!(
            hook_entries(&bed, Harness::Gemini),
            [
                ("BeforeTool", Some("run_shell_command")),
                ("AfterAgent", None)
            ]
        );
        assert_eq!(
            hook_entries(&bed, Harness::Codex),
            [("PreToolUse", None), ("Stop", None)]
        );
        assert_eq!(
            bed.git_hooks,
            [GitHook::PreCommit, GitHook::PrePush, GitHook::PreRebase]
        );
        assert_eq!(bed.mcp, None);
        assert_eq!(bed.check.as_deref(), Some("weeder check --format sarif"));
    }

    #[test]
    fn the_pollen_fixture_becomes_a_channel_bed_with_no_binary_and_no_hooks() {
        let manifest = parse_manifest(&fixture("pollen.garden.json")).expect("a parsed manifest");
        let bed = to_bed(&manifest).expect("a bed");

        assert_eq!(bed.name, "pollen");
        assert_eq!(bed.binary, None);
        assert!(bed.hooks.is_empty());
        assert!(bed.git_hooks.is_empty());
        assert_eq!(bed.skill, None);
        assert_eq!(bed.check, None);
        assert_eq!(
            bed.mcp,
            Some(McpServer {
                command: "node".to_owned(),
                args: vec!["pollen.mjs".to_owned()],
                env: vec!["POLLEN_ID".to_owned(), "POLLEN_ALLOW".to_owned()],
            })
        );
    }

    #[test]
    fn an_mcp_face_carrying_only_a_tool_count_cannot_be_planted_yet() {
        let manifest = parse_manifest(&fixture("tilth.garden.json")).expect("a parsed manifest");
        assert_eq!(
            manifest.faces.mcp.as_ref().map(|mcp| mcp.tools),
            Some(7),
            "the tilth fixture does declare a channel face"
        );
        let bed = to_bed(&manifest).expect("a bed");
        assert_eq!(bed.mcp, None);
    }

    #[test]
    fn a_declared_binary_name_wins_over_the_cli_name() {
        let mut value = weeder_value();
        value["install"]["binary_name"] = serde_json::json!("weeder-bin");
        assert_eq!(value["faces"]["cli"], serde_json::json!("weeder"));
        let manifest = parse_manifest(&value.to_string()).expect("a parsed manifest");
        let bed = to_bed(&manifest).expect("a bed");
        assert_eq!(bed.binary.as_deref(), Some("weeder-bin"));
    }

    #[test]
    fn a_git_install_without_a_revision_is_refused_by_the_schema() {
        let (bed, problem) = manifest_error(&fixture("invalid-git-no-rev.garden.json"));
        assert_eq!(bed, "petals");
        assert!(
            problem.starts_with(SCHEMA_REFUSED),
            "the contracts refused it, not the stem: {problem}"
        );
    }

    #[test]
    fn a_manifest_without_coverage_is_refused_by_the_schema() {
        let mut value = weeder_value();
        value
            .as_object_mut()
            .expect("the fixture is an object")
            .remove("coverage");
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(problem.starts_with(SCHEMA_REFUSED), "{problem}");
        assert!(problem.contains("coverage"), "{problem}");
    }

    #[test]
    fn hooks_for_a_harness_the_stem_does_not_know_are_refused() {
        let mut value = weeder_value();
        value["faces"]["hooks"]["cursor"] = serde_json::json!(["PreToolUse"]);
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(!problem.starts_with(SCHEMA_REFUSED), "{problem}");
        assert!(problem.contains("cursor"), "{problem}");
    }

    #[test]
    fn an_event_the_harness_does_not_fire_is_refused() {
        let mut value = weeder_value();
        value["faces"]["hooks"]["claude"] = serde_json::json!(["BeforeTool"]);
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(problem.contains("BeforeTool"), "{problem}");
        assert!(problem.contains("claude"), "{problem}");
    }

    #[test]
    fn a_git_hook_the_stem_does_not_install_is_refused_loudly() {
        let mut value = weeder_value();
        value["faces"]["git"] = serde_json::json!(["pre-commit", "reference-transaction"]);
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(!problem.starts_with(SCHEMA_REFUSED), "{problem}");
        assert!(problem.contains("reference-transaction"), "{problem}");
    }

    #[test]
    fn an_empty_matcher_is_refused() {
        let mut value = weeder_value();
        value["faces"]["hooks"]["claude"] = serde_json::json!(["PreToolUse:"]);
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(problem.contains("PreToolUse:"), "{problem}");
    }

    #[test]
    fn hooks_without_a_program_to_run_are_refused() {
        let mut value = weeder_value();
        let faces = value["faces"]
            .as_object_mut()
            .expect("the fixture's faces are an object");
        faces.remove("cli");
        let (bed, problem) = manifest_error(&value.to_string());
        assert_eq!(bed, "weeder");
        assert!(problem.contains("binary_name"), "{problem}");
    }

    #[test]
    fn json_that_is_not_json_is_a_json_error() {
        match parse_manifest("{") {
            Err(Error::Json { path: None, .. }) => {}
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn unknown_manifest_fields_are_kept_rather_than_dropped() {
        let mut value = weeder_value();
        value["future_face"] = serde_json::json!({"kind": "not yet invented"});
        let manifest = parse_manifest(&value.to_string()).expect("a parsed manifest");
        assert_eq!(
            manifest.extra.get("future_face"),
            Some(&serde_json::json!({"kind": "not yet invented"}))
        );
    }

    #[test]
    fn load_beds_reads_every_cached_manifest_sorted_by_name() {
        let root = tempfile::tempdir().expect("a temp root");
        for bed in ["weeder", "tend2"] {
            let dir = crate::layout::bed_dir(root.path(), bed);
            std::fs::create_dir_all(&dir).expect("the bed directory");
            std::fs::write(
                dir.join("garden.json"),
                fixture(&format!("{bed}.garden.json")),
            )
            .expect("the cached manifest");
        }
        let beds = load_beds(root.path()).expect("the planted beds");
        assert_eq!(
            beds.iter().map(|bed| bed.name.as_str()).collect::<Vec<_>>(),
            ["tend2", "weeder"]
        );
    }

    #[test]
    fn load_beds_on_a_root_with_no_beds_directory_is_empty_and_not_an_error() {
        let root = tempfile::tempdir().expect("a temp root");
        assert_eq!(load_beds(root.path()).expect("no beds"), Vec::new());
        std::fs::create_dir_all(crate::layout::plotplot_dir(root.path()))
            .expect("the plotplot directory");
        assert_eq!(load_beds(root.path()).expect("no beds"), Vec::new());
    }

    #[test]
    fn load_beds_names_the_bed_whose_manifest_is_broken() {
        let root = tempfile::tempdir().expect("a temp root");
        let dir = crate::layout::bed_dir(root.path(), "weeder");
        std::fs::create_dir_all(&dir).expect("the bed directory");
        std::fs::write(
            dir.join("garden.json"),
            fixture("invalid-no-metric.garden.json"),
        )
        .expect("the cached manifest");
        match load_beds(root.path()) {
            Err(Error::Manifest { bed, .. }) => assert_eq!(bed, "weeder"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn a_bed_directory_with_no_manifest_is_refused_rather_than_skipped() {
        let root = tempfile::tempdir().expect("a temp root");
        std::fs::create_dir_all(crate::layout::bed_dir(root.path(), "weeder"))
            .expect("the bed directory");
        match load_beds(root.path()) {
            Err(Error::Bed { bed, .. }) => assert_eq!(bed, "weeder"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn a_stray_file_beside_the_bed_directories_is_not_a_bed() {
        let root = tempfile::tempdir().expect("a temp root");
        let beds = crate::layout::beds_dir(root.path());
        std::fs::create_dir_all(&beds).expect("the beds directory");
        std::fs::write(beds.join(".DS_Store"), "").expect("a stray file");
        assert_eq!(load_beds(root.path()).expect("no beds"), Vec::new());
    }
}
