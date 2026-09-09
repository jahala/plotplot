//! SARIF 2.1.0, the one shape every gate's findings take.
//!
//! The garden picked a standard rather than inventing a findings schema (the law, §4), so
//! this module owns three things and nothing else: enforcing the schema OASIS published,
//! concatenating validated logs into one, and counting what blocks. The schema and the
//! `$schema` URL both come from the contracts; neither is spelled here.
//!
//! Nothing here reads a file or spawns anything. A gate's bytes come in as a string and a
//! log goes out as a value, so the decision that follows is a decision over values.

use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::error::{Error, Result};

/// The official SARIF 2.1.0 schema as OASIS published it, vendored by the contracts and
/// embedded so the binary carries its own contract.
pub const SARIF_SCHEMA: &str = include_str!("../contracts/vendor/sarif-schema-2.1.0.json");

/// The `$schema` URL `contracts/pins.json` names for SARIF 2.1.0.
///
/// Read out of the pins by `build.rs`, so a pins file that names no URL fails the build
/// rather than a gate run: the URL a merged log carries is a fact of the contracts this
/// binary was built against, and there is no honest answer to give at run time without it.
pub const SCHEMA_URL: &str = env!("PLOTPLOT_SARIF_SCHEMA_URL");

/// The one version of the format the garden speaks.
pub const VERSION: &str = "2.1.0";

/// The level a result carries when it blocks.
pub const BLOCK_LEVEL: &str = "error";

/// The level SARIF gives a failing result whose rule declares no default (§3.27.10).
const WARNING_LEVEL: &str = "warning";

/// The level SARIF gives a result that reports something other than a failure.
const NO_LEVEL: &str = "none";

/// What SARIF calls a result that is a failure, the `kind` a result has by default.
const FAILURE_KIND: &str = "fail";

/// How a log whose tool has no name is named in a refusal.
pub const UNNAMED: &str = "(unnamed gate)";

/// Read one gate's output as SARIF 2.1.0, enforcing the contracts' vendored schema.
///
/// The gate is named from the log's own first tool where the bytes get far enough to carry
/// one; a caller that already knows whose output this is says so with [`validate_from`].
///
/// # Errors
///
/// [`Error::Sarif`] when the bytes are not JSON, or are JSON the schema refuses.
pub fn validate(json: &str) -> Result<Value> {
    let value = read(UNNAMED, json)?;
    let gate = tool_name(&value).unwrap_or(UNNAMED).to_owned();
    enforce(&gate, value)
}

/// Read the output of a named gate as SARIF 2.1.0, enforcing the contracts' vendored schema.
///
/// # Errors
///
/// [`Error::Sarif`] naming `gate` when the bytes are not JSON, or are JSON the schema
/// refuses.
pub fn validate_from(gate: &str, json: &str) -> Result<Value> {
    let value = read(gate, json)?;
    enforce(gate, value)
}

/// One SARIF 2.1.0 log carrying every input log's runs, in the order they were given.
///
/// The inputs are logs the schema already accepted, so a run is copied as the gate wrote it:
/// its tool, its rules, its invocations and its results all travel, and the merged log says
/// which gate found what because each run still names its own tool.
pub fn merge(logs: &[Value]) -> Value {
    let mut runs = Vec::new();
    for log in logs {
        if let Some(list) = log.get("runs").and_then(Value::as_array) {
            runs.extend(list.iter().cloned());
        }
    }
    json!({ "$schema": SCHEMA_URL, "version": VERSION, "runs": runs })
}

/// How many of a log's results block.
///
/// A result's level is the one it carries; a result that carries none takes the level its
/// rule declares as its default, and a failing result whose rule declares none is a warning
/// (SARIF 2.1.0 §3.27.10). Reading the level the gate printed and stopping there would let a
/// gate that leans on its own rule defaults block nothing, which is the one way this count
/// could fail open.
pub fn block_count(log: &Value) -> usize {
    runs(log)
        .map(|run| {
            results(run)
                .filter(|result| effective_level(run, result) == BLOCK_LEVEL)
                .count()
        })
        .sum()
}

/// How many results a log carries, blocking or not.
pub fn result_count(log: &Value) -> usize {
    runs(log).map(|run| results(run).count()).sum()
}

/// The name of the first tool a log names, when it names one.
pub fn tool_name(log: &Value) -> Option<&str> {
    runs(log)
        .next()
        .and_then(|run| run.pointer("/tool/driver/name"))
        .and_then(Value::as_str)
}

/// Every run of a log, and none when the log carries no run array (`runs` may be null).
fn runs(log: &Value) -> impl Iterator<Item = &Value> {
    log.get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
}

/// Every result of one run.
fn results(run: &Value) -> impl Iterator<Item = &Value> {
    run.get("results")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
}

/// The level a result really has: its own, else its rule's default, else what SARIF gives a
/// result of its kind.
fn effective_level<'a>(run: &'a Value, result: &'a Value) -> &'a str {
    if let Some(level) = result.get("level").and_then(Value::as_str) {
        return level;
    }
    if let Some(level) = rule_default_level(run, result) {
        return level;
    }
    match result.get("kind").and_then(Value::as_str) {
        Some(kind) if kind != FAILURE_KIND => NO_LEVEL,
        _ => WARNING_LEVEL,
    }
}

/// The default level of the rule a result points at, by index into the driver's rules or by
/// the rule's own id.
fn rule_default_level<'a>(run: &'a Value, result: &'a Value) -> Option<&'a str> {
    let rules = run
        .pointer("/tool/driver/rules")
        .and_then(Value::as_array)?;

    let by_index = result
        .get("ruleIndex")
        .and_then(Value::as_u64)
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| rules.get(index));
    let rule = match by_index {
        Some(rule) => rule,
        None => {
            let id = result.get("ruleId").and_then(Value::as_str)?;
            rules
                .iter()
                .find(|rule| rule.get("id").and_then(Value::as_str) == Some(id))?
        }
    };

    rule.pointer("/defaultConfiguration/level")
        .and_then(Value::as_str)
}

/// A gate's bytes as JSON, or what is wrong with them, named for the gate that printed them.
fn read(gate: &str, json: &str) -> Result<Value> {
    serde_json::from_str(json).map_err(|source| Error::Sarif {
        gate: gate.to_owned(),
        problem: format!("its output is not JSON: {source}"),
    })
}

/// The schema, over a document that is already JSON.
fn enforce(gate: &str, value: Value) -> Result<Value> {
    let validator = validator().as_ref().map_err(|problem| Error::Sarif {
        gate: gate.to_owned(),
        problem: format!("the embedded SARIF schema did not compile: {problem}"),
    })?;
    if let Err(error) = validator.validate(&value) {
        let at = error.instance_path().to_string();
        let where_ = if at.is_empty() {
            String::new()
        } else {
            format!(" at {at}")
        };
        return Err(Error::Sarif {
            gate: gate.to_owned(),
            problem: crate::manifest::one_line(&format!(
                "the SARIF 2.1.0 schema refused it{where_}: {error}"
            )),
        });
    }
    Ok(value)
}

/// The vendored schema, compiled once. A schema that will not compile is reported rather
/// than panicked on, the way the manifest schema is.
fn validator() -> &'static std::result::Result<jsonschema::Validator, String> {
    static VALIDATOR: OnceLock<std::result::Result<jsonschema::Validator, String>> =
        OnceLock::new();
    VALIDATOR.get_or_init(|| crate::manifest::compile(SARIF_SCHEMA))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture() -> Value {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contracts/fixtures/sarif/weeder-check.sarif.json");
        let text = std::fs::read_to_string(&path).expect("the contracts' SARIF fixture");
        serde_json::from_str(&text).expect("the fixture is JSON")
    }

    /// A log with one run, one tool name and the results given, each as `(ruleId, level)`
    /// where a level of `None` leaves the result without one.
    fn log_of(tool: &str, results: &[(&str, Option<&str>)]) -> Value {
        let results: Vec<Value> = results
            .iter()
            .map(|(rule, level)| {
                let mut result = json!({
                    "ruleId": rule,
                    "message": { "text": "something the gate found" }
                });
                if let Some(level) = level {
                    result["level"] = json!(level);
                }
                result
            })
            .collect();
        json!({
            "$schema": SCHEMA_URL,
            "version": VERSION,
            "runs": [{
                "tool": { "driver": {
                    "name": tool,
                    "rules": [
                        { "id": "T1", "defaultConfiguration": { "level": "error" } },
                        { "id": "T4", "defaultConfiguration": { "level": "warning" } },
                        { "id": "N1" }
                    ]
                }},
                "results": results
            }]
        })
    }

    #[test]
    fn the_contracts_own_fixture_validates() {
        let text = serde_json::to_string(&fixture()).expect("the fixture serialises");
        let log = validate(&text).expect("the contracts' own fixture is SARIF 2.1.0");
        assert_eq!(tool_name(&log), Some("weeder"));
    }

    #[test]
    fn a_log_with_no_runs_key_is_refused_and_says_which_key() {
        let error = validate("{\"version\": \"2.1.0\"}").expect_err("runs is required");
        let message = error.to_string();
        assert!(message.contains("runs"), "{message}");
        assert!(message.starts_with(UNNAMED), "{message}");
    }

    #[test]
    fn bytes_that_are_not_json_are_refused_by_the_gate_that_printed_them() {
        let error = validate_from("weeder", "weeder: fatal: not a git repository")
            .expect_err("a message is not a log");
        let message = error.to_string();
        assert!(message.starts_with("weeder: "), "{message}");
        assert!(message.contains("not JSON"), "{message}");
    }

    #[test]
    fn a_log_of_the_wrong_version_is_refused() {
        let mut log = fixture();
        log["version"] = json!("2.0.0");
        let text = serde_json::to_string(&log).expect("it serialises");
        let error = validate_from("weeder", &text).expect_err("2.1.0 is the only version");
        assert!(error.to_string().contains("weeder"), "{error}");
    }

    #[test]
    fn a_refusal_is_one_line() {
        let error = validate("{\"version\": \"2.1.0\"}").expect_err("runs is required");
        assert!(!error.to_string().contains('\n'), "{error}");
    }

    #[test]
    fn merging_nothing_is_a_log_with_no_runs() {
        let merged = merge(&[]);
        assert_eq!(merged["version"], json!(VERSION));
        assert_eq!(merged["$schema"], json!(SCHEMA_URL));
        assert_eq!(merged["runs"], json!([]));
        let text = serde_json::to_string(&merged).expect("it serialises");
        validate(&text).expect("a log with no runs is still a log");
    }

    #[test]
    fn merging_keeps_every_run_in_the_order_it_was_given() {
        let merged = merge(&[
            log_of("weeder", &[("T1", Some("error"))]),
            log_of("petals", &[]),
            log_of("tend2", &[("T4", Some("note")), ("T4", Some("note"))]),
        ]);
        let runs = merged["runs"].as_array().expect("runs");
        assert_eq!(runs.len(), 3);
        assert_eq!(
            runs.iter()
                .map(|run| run.pointer("/tool/driver/name").and_then(Value::as_str))
                .collect::<Vec<_>>(),
            [Some("weeder"), Some("petals"), Some("tend2")]
        );
        assert_eq!(result_count(&merged), 3);
        let text = serde_json::to_string(&merged).expect("it serialises");
        validate(&text).expect("a merged log is SARIF 2.1.0");
    }

    #[test]
    fn merging_two_real_gate_logs_carries_both_tools_findings() {
        let mut petals = fixture();
        petals["runs"][0]["tool"]["driver"]["name"] = json!("petals");
        let merged = merge(&[fixture(), petals]);
        let text = serde_json::to_string(&merged).expect("it serialises");
        let log = validate(&text).expect("a merged log of two real logs is SARIF 2.1.0");
        assert_eq!(result_count(&log), 4);
        assert_eq!(block_count(&log), 4);
    }

    #[test]
    fn a_log_with_a_null_runs_key_contributes_nothing() {
        let merged = merge(&[json!({"version": VERSION, "runs": Value::Null})]);
        assert_eq!(merged["runs"], json!([]));
        assert_eq!(block_count(&merged), 0);
        assert_eq!(result_count(&merged), 0);
    }

    #[test]
    fn only_error_level_results_are_blocks() {
        let log = log_of(
            "weeder",
            &[
                ("T1", Some("error")),
                ("T4", Some("warning")),
                ("N1", Some("note")),
                ("N1", Some("none")),
            ],
        );
        assert_eq!(result_count(&log), 4);
        assert_eq!(block_count(&log), 1);
    }

    #[test]
    fn a_result_with_no_level_takes_the_level_its_rule_declares() {
        let log = log_of("weeder", &[("T1", None), ("T4", None)]);
        assert_eq!(block_count(&log), 1, "T1 defaults to error, T4 to warning");
    }

    #[test]
    fn a_result_with_no_level_and_no_rule_default_is_a_warning() {
        let log = log_of("weeder", &[("N1", None)]);
        assert_eq!(block_count(&log), 0);
    }

    #[test]
    fn a_result_pointing_at_its_rule_by_index_takes_that_rules_default() {
        let mut log = log_of("weeder", &[("nonsense", None)]);
        log["runs"][0]["results"][0]["ruleIndex"] = json!(0);
        assert_eq!(
            block_count(&log),
            1,
            "index 0 is T1, whose default is error"
        );
    }

    #[test]
    fn a_result_that_is_not_a_failure_carries_no_level() {
        let mut log = log_of("weeder", &[("N1", None)]);
        log["runs"][0]["results"][0]["kind"] = json!("informational");
        assert_eq!(block_count(&log), 0);
    }

    #[test]
    fn a_run_with_no_results_counts_nothing() {
        let log = log_of("petals", &[]);
        assert_eq!(result_count(&log), 0);
        assert_eq!(block_count(&log), 0);
    }

    #[test]
    fn the_schema_url_is_the_one_the_contracts_pin() {
        let pins: Value = serde_json::from_str(include_str!("../contracts/pins.json"))
            .expect("the contracts' pins are JSON");
        assert_eq!(
            pins.pointer("/sarif_schema/url").and_then(Value::as_str),
            Some(SCHEMA_URL)
        );
        assert_eq!(
            pins.pointer("/sarif_schema/version")
                .and_then(Value::as_str),
            Some(VERSION)
        );
    }

    #[test]
    fn the_embedded_schema_is_the_file_the_contracts_pin_by_digest() {
        use sha2::{Digest, Sha256};

        let pins: Value = serde_json::from_str(include_str!("../contracts/pins.json"))
            .expect("the contracts' pins are JSON");
        let digest = hex::encode(Sha256::digest(SARIF_SCHEMA.as_bytes()));
        assert_eq!(
            pins.pointer("/sarif_schema/sha256").and_then(Value::as_str),
            Some(digest.as_str())
        );
    }

    #[test]
    fn a_log_with_no_tool_name_is_unnamed_rather_than_guessed() {
        let log = json!({"version": VERSION, "runs": [{"tool": {"driver": {}}}]});
        assert_eq!(tool_name(&log), None);
        assert_eq!(tool_name(&json!({"version": VERSION, "runs": []})), None);
    }
}
