//! The three vendor bundles, generated from one input, written to disk and diffed back.
//!
//! The beds come from the contracts' own manifest fixtures through
//! `manifest::parse_manifest` and `manifest::to_bed`, so what the bundles carry is what the
//! contracts describe. Every file name, every key and every hook command asserted here is a
//! vendor fact from `docs/prompts/stem-build-2026-09.md` §5.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use plotplot::bed::{Bed, HookEntry};
use plotplot::bundle::{BundleInput, Difference, FileTree, SkillFile, diff, generate, write};
use plotplot::harness::Harness;
use plotplot::manifest::{parse_manifest, to_bed};

const SEASON: &str = "2026.09";
const WEEDER_SKILL: &str = "---\nname: weeder\n---\n\nweeder judges the diff.\n";
const TEND2_SKILL: &str = "---\nname: tend2\n---\n\ntend2 keeps the loops.\n";

/// One bed, read from the contracts' fixture corpus exactly as the stem reads a planted
/// bed's `garden.json`.
fn fixture_bed(name: &str) -> Bed {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts/fixtures/manifest")
        .join(format!("{name}.garden.json"));
    let json =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let manifest =
        parse_manifest(&json).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    to_bed(&manifest).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// weeder brings hooks and git hooks, tend2 brings a launchable MCP server, pollen brings an
/// MCP server and no binary at all.
fn beds() -> Vec<Bed> {
    vec![
        fixture_bed("weeder"),
        fixture_bed("tend2"),
        fixture_bed("pollen"),
    ]
}

fn skills() -> Vec<SkillFile> {
    vec![
        SkillFile {
            bed: "weeder".to_owned(),
            content: WEEDER_SKILL.to_owned(),
        },
        SkillFile {
            bed: "tend2".to_owned(),
            content: TEND2_SKILL.to_owned(),
        },
    ]
}

fn tree(harness: Harness) -> FileTree {
    let beds = beds();
    let skills = skills();
    let input = BundleInput {
        stem_version: plotplot::VERSION,
        season: SEASON,
        beds: &beds,
        skills: &skills,
    };
    generate(harness, &input).unwrap_or_else(|error| panic!("{harness}: {error}"))
}

/// A bundle for a garden with nothing planted in it: only the stem's own entries.
fn bare_tree(harness: Harness) -> FileTree {
    let input = BundleInput {
        stem_version: plotplot::VERSION,
        season: SEASON,
        beds: &[],
        skills: &[],
    };
    generate(harness, &input).unwrap_or_else(|error| panic!("{harness}: {error}"))
}

fn paths(tree: &FileTree) -> Vec<String> {
    tree.0
        .keys()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect()
}

fn text(tree: &FileTree, path: &str) -> String {
    let bytes = tree
        .0
        .get(Path::new(path))
        .unwrap_or_else(|| panic!("{path} is not in the tree: {:?}", paths(tree)));
    String::from_utf8(bytes.clone()).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn json(tree: &FileTree, path: &str) -> Value {
    serde_json::from_str(&text(tree, path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The file each vendor puts its manifest in (§5).
fn manifest_path(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude => ".claude-plugin/plugin.json",
        Harness::Gemini => "gemini-extension.json",
        Harness::Codex => ".codex-plugin/plugin.json",
    }
}

/// Where each vendor keeps its hook file (§5).
fn hooks_path(harness: Harness) -> &'static str {
    match harness {
        Harness::Claude | Harness::Gemini => "hooks/hooks.json",
        Harness::Codex => "hooks.json",
    }
}

/// The events a hook file registers, in the order the file spells them. Read from the text
/// rather than the parsed document, because parsing sorts the keys.
fn event_order(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            // An event key is the second level of the document: `    "PreToolUse": [`.
            let rest = line.strip_prefix("    \"")?;
            let (event, tail) = rest.split_once('"')?;
            if tail == ": [" {
                Some(event.to_owned())
            } else {
                None
            }
        })
        .collect()
}

/// One hook file as event → the single command entry the stem registers for it.
fn hook_entries(harness: Harness) -> BTreeMap<String, Value> {
    let document = json(&tree(harness), hooks_path(harness));
    let hooks = document
        .get("hooks")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{harness}: the hook file has no hooks object"));
    hooks
        .iter()
        .map(|(event, groups)| {
            let groups = groups
                .as_array()
                .unwrap_or_else(|| panic!("{harness} {event}: not an array"));
            assert_eq!(groups.len(), 1, "{harness} {event}: one group per event");
            let group = groups[0]
                .as_object()
                .unwrap_or_else(|| panic!("{harness} {event}: the group is not an object"));
            assert!(
                !group.contains_key("matcher"),
                "{harness} {event}: the stem registers no vendor matcher"
            );
            let entries = group
                .get("hooks")
                .and_then(Value::as_array)
                .unwrap_or_else(|| panic!("{harness} {event}: the group has no hooks array"));
            assert_eq!(entries.len(), 1, "{harness} {event}: one command per event");
            (event.clone(), entries[0].clone())
        })
        .collect()
}

/// Every occurrence of a vendor's root variable replaced by one marker, so the three
/// bundles can be compared harness names aside (§4).
fn normalize(value: &Value, harness: Harness) -> Value {
    match value {
        Value::String(text) => {
            Value::String(text.replace(harness.bundle_root_variable(), "${BUNDLE_ROOT}"))
        }
        Value::Array(items) => {
            Value::Array(items.iter().map(|item| normalize(item, harness)).collect())
        }
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), normalize(item, harness)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The MCP servers a bundle declares, wherever that vendor keeps them.
fn mcp_servers(harness: Harness) -> Value {
    let tree = tree(harness);
    let servers = match harness {
        Harness::Claude | Harness::Codex => json(&tree, ".mcp.json")
            .get("mcpServers")
            .cloned()
            .unwrap_or_else(|| panic!("{harness}: .mcp.json has no mcpServers")),
        Harness::Gemini => json(&tree, manifest_path(harness))
            .get("mcpServers")
            .cloned()
            .unwrap_or_else(|| panic!("{harness}: the extension manifest has no mcpServers")),
    };
    normalize(&servers, harness)
}

#[test]
fn each_harness_yields_exactly_its_vendor_file_set() {
    let expected: [(Harness, Vec<&str>); 3] = [
        (
            Harness::Claude,
            vec![
                ".claude-plugin/plugin.json",
                ".mcp.json",
                "hooks/hooks.json",
                "skills/tend2/SKILL.md",
                "skills/weeder/SKILL.md",
            ],
        ),
        (
            Harness::Gemini,
            vec![
                "gemini-extension.json",
                "hooks/hooks.json",
                "skills/tend2/SKILL.md",
                "skills/weeder/SKILL.md",
            ],
        ),
        (
            Harness::Codex,
            vec![
                ".codex-plugin/plugin.json",
                ".mcp.json",
                "hooks.json",
                "skills/tend2/SKILL.md",
                "skills/weeder/SKILL.md",
            ],
        ),
    ];
    for (harness, files) in expected {
        let mut want: Vec<String> = files.into_iter().map(str::to_owned).collect();
        want.sort();
        assert_eq!(paths(&tree(harness)), want, "{harness}");
    }
}

#[test]
fn no_bundle_ever_carries_the_binary_directory() {
    for harness in Harness::ALL {
        for path in paths(&tree(harness)) {
            assert!(!path.starts_with("bin/"), "{harness}: {path}");
        }
        for path in paths(&bare_tree(harness)) {
            assert!(!path.starts_with("bin/"), "{harness}: {path}");
        }
    }
}

#[test]
fn every_vendor_manifest_names_the_stem_the_season_and_every_bed() {
    for harness in Harness::ALL {
        let manifest = json(&tree(harness), manifest_path(harness));
        assert_eq!(
            manifest.get("name").and_then(Value::as_str),
            Some("plotplot"),
            "{harness}"
        );
        assert_eq!(
            manifest.get("version").and_then(Value::as_str),
            Some(plotplot::VERSION),
            "{harness}"
        );
        let description = manifest
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{harness}: no description"));
        assert!(description.contains(SEASON), "{harness}: {description}");
        for bed in beds() {
            assert!(
                description.contains(&bed.name),
                "{harness}: {description} does not name {}",
                bed.name
            );
        }
    }
}

#[test]
fn the_codex_manifest_points_at_its_skills_hooks_and_mcp_servers() {
    let manifest = json(&tree(Harness::Codex), manifest_path(Harness::Codex));
    assert_eq!(
        manifest.get("skills").and_then(Value::as_str),
        Some("./skills/")
    );
    assert_eq!(
        manifest.get("hooks").and_then(Value::as_str),
        Some("./hooks.json")
    );
    assert_eq!(
        manifest.get("mcpServers").and_then(Value::as_str),
        Some("./.mcp.json")
    );
}

#[test]
fn the_codex_manifest_names_no_mcp_servers_when_no_bundle_carries_them() {
    let manifest = json(&bare_tree(Harness::Codex), manifest_path(Harness::Codex));
    assert_eq!(manifest.get("mcpServers"), None);
    assert_eq!(
        manifest.get("hooks").and_then(Value::as_str),
        Some("./hooks.json")
    );
}

#[test]
fn every_hook_entry_is_one_dispatcher_call_and_nothing_else() {
    for harness in Harness::ALL {
        for (event, entry) in hook_entries(harness) {
            assert_eq!(
                entry.get("type").and_then(Value::as_str),
                Some("command"),
                "{harness} {event}"
            );
            assert_eq!(
                entry.get("command").and_then(Value::as_str),
                Some(
                    format!(
                        "\"{}/bin/plotplot\" hook {} {event}",
                        harness.bundle_root_variable(),
                        harness.name()
                    )
                    .as_str()
                ),
                "{harness} {event}"
            );
            let keys: BTreeSet<&str> = entry
                .as_object()
                .unwrap_or_else(|| panic!("{harness} {event}: not an object"))
                .keys()
                .map(String::as_str)
                .collect();
            assert_eq!(
                keys,
                BTreeSet::from(["type", "command", "timeout"]),
                "{harness} {event}"
            );
        }
    }
}

#[test]
fn timeouts_are_seconds_for_claude_and_codex_and_milliseconds_for_gemini() {
    let timeout = |harness: Harness, event: &str| -> u64 {
        hook_entries(harness)
            .get(event)
            .and_then(|entry| entry.get("timeout"))
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("{harness} {event}: no timeout"))
    };

    // ten seconds for a tool event, five for a stop, one for the session-end event.
    assert_eq!(timeout(Harness::Claude, "PreToolUse"), 10);
    assert_eq!(timeout(Harness::Claude, "PostToolUse"), 10);
    assert_eq!(timeout(Harness::Claude, "Stop"), 5);
    assert_eq!(timeout(Harness::Claude, "SessionEnd"), 1);

    assert_eq!(timeout(Harness::Codex, "PreToolUse"), 10);
    assert_eq!(timeout(Harness::Codex, "PreCompact"), 5);
    // Codex has no SessionEnd, so its Stop is the session-end event.
    assert_eq!(timeout(Harness::Codex, "Stop"), 1);

    assert_eq!(timeout(Harness::Gemini, "BeforeTool"), 10_000);
    assert_eq!(timeout(Harness::Gemini, "AfterTool"), 10_000);
    assert_eq!(timeout(Harness::Gemini, "AfterAgent"), 5_000);
    assert_eq!(timeout(Harness::Gemini, "SessionEnd"), 1_000);
}

#[test]
fn every_harness_registers_the_events_weeder_declared() {
    let declared: [(Harness, [&str; 2]); 3] = [
        (Harness::Claude, ["PreToolUse", "Stop"]),
        (Harness::Gemini, ["BeforeTool", "AfterAgent"]),
        (Harness::Codex, ["PreToolUse", "Stop"]),
    ];
    for (harness, events) in declared {
        let registered = hook_entries(harness);
        for event in events {
            assert!(
                registered.contains_key(event),
                "{harness}: {event} is not registered; {:?}",
                registered.keys().collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn the_stems_own_events_are_registered_with_no_beds_at_all() {
    // The deny list's event, every event a friction kind derives from, and the session-end
    // event, in the harness's own order.
    let expected: [(Harness, Vec<&str>); 3] = [
        (
            Harness::Claude,
            vec![
                "PreToolUse",
                "PostToolUse",
                "PostToolUseFailure",
                "PermissionDenied",
                "SessionEnd",
                "Stop",
                "PreCompact",
            ],
        ),
        (
            Harness::Gemini,
            vec![
                "BeforeTool",
                "AfterTool",
                "AfterAgent",
                "SessionEnd",
                "PreCompress",
            ],
        ),
        (
            Harness::Codex,
            vec!["PreToolUse", "PostToolUse", "PreCompact", "Stop"],
        ),
    ];
    for (harness, events) in expected {
        let file = text(&bare_tree(harness), hooks_path(harness));
        assert_eq!(event_order(&file), events, "{harness}");
        for event in &events {
            assert!(harness.has_event(event), "{harness}: {event}");
        }
        assert!(
            events.contains(&harness.before_tool_event()),
            "{harness}: the deny list's event"
        );
        assert!(
            events.contains(&harness.session_end_event()),
            "{harness}: the receipt draft's event"
        );
    }
}

#[test]
fn a_beds_own_event_widens_the_set_in_the_harnesss_order() {
    let bed = Bed {
        name: "mullein".to_owned(),
        version: "0.1.0".to_owned(),
        binary: Some("mullein".to_owned()),
        skill: None,
        hooks: BTreeMap::from([(
            Harness::Claude,
            vec![HookEntry {
                event: "SessionStart".to_owned(),
                matcher: None,
            }],
        )]),
        git_hooks: Vec::new(),
        mcp: None,
        check: None,
    };
    let beds = [bed];
    let input = BundleInput {
        stem_version: plotplot::VERSION,
        season: SEASON,
        beds: &beds,
        skills: &[],
    };
    let tree = generate(Harness::Claude, &input).expect("a claude bundle");
    let order = event_order(&text(&tree, hooks_path(Harness::Claude)));
    assert_eq!(
        order,
        [
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
            "PermissionDenied",
            "SessionStart",
            "SessionEnd",
            "Stop",
            "PreCompact",
        ]
    );
}

#[test]
fn an_event_the_harness_does_not_fire_is_refused() {
    let bed = Bed {
        name: "mullein".to_owned(),
        version: "0.1.0".to_owned(),
        binary: Some("mullein".to_owned()),
        skill: None,
        hooks: BTreeMap::from([(
            Harness::Gemini,
            vec![HookEntry {
                event: "PreToolUse".to_owned(),
                matcher: None,
            }],
        )]),
        git_hooks: Vec::new(),
        mcp: None,
        check: None,
    };
    let beds = [bed];
    let input = BundleInput {
        stem_version: plotplot::VERSION,
        season: SEASON,
        beds: &beds,
        skills: &[],
    };
    let error = generate(Harness::Gemini, &input).expect_err("gemini does not fire PreToolUse");
    let message = error.to_string();
    assert!(message.contains("mullein"), "{message}");
    assert!(message.contains("PreToolUse"), "{message}");
}

#[test]
fn the_three_bundles_carry_the_same_skills() {
    let skills = |harness: Harness| -> BTreeMap<String, String> {
        let tree = tree(harness);
        tree.0
            .iter()
            .filter(|(path, _)| path.starts_with("skills"))
            .map(|(path, bytes)| {
                (
                    path.to_string_lossy().replace('\\', "/"),
                    String::from_utf8_lossy(bytes).into_owned(),
                )
            })
            .collect()
    };
    let claude = skills(Harness::Claude);
    assert_eq!(claude.len(), 2);
    assert_eq!(
        claude.get("skills/weeder/SKILL.md").map(String::as_str),
        Some(WEEDER_SKILL)
    );
    assert_eq!(
        claude.get("skills/tend2/SKILL.md").map(String::as_str),
        Some(TEND2_SKILL)
    );
    assert_eq!(claude, skills(Harness::Gemini));
    assert_eq!(claude, skills(Harness::Codex));
}

#[test]
fn the_three_bundles_carry_the_same_mcp_servers() {
    let claude = mcp_servers(Harness::Claude);
    assert_eq!(claude, mcp_servers(Harness::Gemini));
    assert_eq!(claude, mcp_servers(Harness::Codex));

    let servers = claude
        .as_object()
        .expect("mcpServers is an object")
        .keys()
        .cloned()
        .collect::<BTreeSet<String>>();
    assert_eq!(
        servers,
        BTreeSet::from(["pollen".to_owned(), "tend2".to_owned()])
    );

    // Declared as the bed declared them: a bare subcommand stays, a relative path is
    // resolved inside the bundle, and every environment variable is named, never valued.
    assert_eq!(
        claude.pointer("/tend2/command").and_then(Value::as_str),
        Some("tend2")
    );
    assert_eq!(
        claude.pointer("/tend2/args"),
        Some(&Value::Array(vec![Value::String("mcp".to_owned())]))
    );
    assert_eq!(
        claude.pointer("/pollen/command").and_then(Value::as_str),
        Some("node")
    );
    assert_eq!(
        claude.pointer("/pollen/args"),
        Some(&Value::Array(vec![Value::String(
            "${BUNDLE_ROOT}/bin/pollen/pollen.mjs".to_owned()
        )]))
    );
    assert_eq!(
        claude
            .pointer("/pollen/env/POLLEN_ID")
            .and_then(Value::as_str),
        Some("${POLLEN_ID}")
    );
    assert_eq!(
        claude
            .pointer("/pollen/env/POLLEN_ALLOW")
            .and_then(Value::as_str),
        Some("${POLLEN_ALLOW}")
    );
}

#[test]
fn a_garden_with_no_channel_bed_gets_no_mcp_file() {
    let beds = [fixture_bed("weeder")];
    let input = BundleInput {
        stem_version: plotplot::VERSION,
        season: SEASON,
        beds: &beds,
        skills: &[],
    };
    for harness in Harness::ALL {
        let tree = generate(harness, &input).unwrap_or_else(|error| panic!("{harness}: {error}"));
        assert!(
            !paths(&tree).contains(&".mcp.json".to_owned()),
            "{harness} emitted an empty .mcp.json"
        );
    }
    let gemini = generate(Harness::Gemini, &input).expect("a gemini bundle");
    let manifest = json(&gemini, manifest_path(Harness::Gemini));
    assert_eq!(manifest.get("mcpServers"), None);
}

#[test]
fn every_json_file_is_pretty_printed_with_two_spaces_and_one_trailing_newline() {
    for harness in Harness::ALL {
        let tree = tree(harness);
        for path in paths(&tree) {
            if !path.ends_with(".json") {
                continue;
            }
            let file = text(&tree, &path);
            assert!(
                file.ends_with('\n'),
                "{harness} {path}: no trailing newline"
            );
            assert!(
                !file.ends_with("\n\n"),
                "{harness} {path}: more than one trailing newline"
            );
            assert!(!file.contains('\t'), "{harness} {path}: a tab");
            serde_json::from_str::<Value>(&file)
                .unwrap_or_else(|error| panic!("{harness} {path}: {error}"));
            for line in file.lines() {
                let indent = line.len() - line.trim_start().len();
                assert!(indent % 2 == 0, "{harness} {path}: {line:?}");
            }
        }
    }
}

#[test]
fn generating_twice_produces_identical_trees() {
    for harness in Harness::ALL {
        assert_eq!(tree(harness), tree(harness), "{harness}");
    }
}

#[test]
fn writing_a_tree_reports_every_path_sorted_and_diffs_clean() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let root = directory.path();
    let tree = tree(Harness::Claude);

    let written = write(&tree, root).expect("the bundle is written");
    let expected: Vec<PathBuf> = paths(&tree)
        .into_iter()
        .map(|path| root.join(path))
        .collect();
    assert_eq!(written, expected);
    let mut sorted = written.clone();
    sorted.sort();
    assert_eq!(written, sorted);
    for path in &written {
        assert!(path.is_file(), "{}", path.display());
    }
    assert_eq!(diff(&tree, root).expect("a diff"), Vec::new());
}

#[test]
fn one_changed_byte_in_the_hook_file_is_one_difference() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let root = directory.path();
    let tree = tree(Harness::Claude);
    write(&tree, root).expect("the bundle is written");

    let hooks = root.join(hooks_path(Harness::Claude));
    let mut bytes = fs::read(&hooks).expect("the hook file");
    let last = bytes.len() - 1;
    bytes[last] = b' ';
    fs::write(&hooks, &bytes).expect("the hook file is rewritten");

    assert_eq!(
        diff(&tree, root).expect("a diff"),
        vec![Difference::Changed(PathBuf::from(hooks_path(
            Harness::Claude
        )))]
    );
}

#[test]
fn a_deleted_skill_is_missing_and_a_stray_file_is_extra() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let root = directory.path();
    let tree = tree(Harness::Claude);
    write(&tree, root).expect("the bundle is written");

    fs::remove_file(root.join("skills/weeder/SKILL.md")).expect("the skill is removed");
    assert_eq!(
        diff(&tree, root).expect("a diff"),
        vec![Difference::Missing(PathBuf::from("skills/weeder/SKILL.md"))]
    );

    fs::write(root.join("stray.json"), b"{}\n").expect("a stray file");
    assert_eq!(
        diff(&tree, root).expect("a diff"),
        vec![
            Difference::Missing(PathBuf::from("skills/weeder/SKILL.md")),
            Difference::Extra(PathBuf::from("stray.json")),
        ]
    );
}

#[test]
fn the_installed_binary_directory_is_never_extra() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let root = directory.path();
    let tree = tree(Harness::Codex);
    write(&tree, root).expect("the bundle is written");

    fs::create_dir_all(root.join("bin/pollen")).expect("the binary directory");
    fs::write(root.join("bin/plotplot"), b"a copy of the stem").expect("the stem binary");
    fs::write(root.join("bin/pollen/pollen.mjs"), b"a copy of pollen").expect("pollen");

    assert_eq!(diff(&tree, root).expect("a diff"), Vec::new());
}

#[test]
fn a_bundle_that_was_never_written_is_all_missing() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let root = directory.path().join("bundles/gemini");
    let tree = tree(Harness::Gemini);
    let expected: Vec<Difference> = paths(&tree)
        .into_iter()
        .map(|path| Difference::Missing(PathBuf::from(path)))
        .collect();
    assert_eq!(diff(&tree, &root).expect("a diff"), expected);
}
