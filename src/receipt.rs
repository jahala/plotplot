//! The receipt draft: what a session can say about itself the moment it ends.
//!
//! `docs/plans/receipts.md` §4 puts the draft at the session-end event, inside Claude Code's
//! 1.5-second budget, so this face writes counters and nothing else: no network, no git, no
//! model. The fields are §3's, and anything the payload cannot supply is `null` rather than
//! an estimate. Signing, the subject digest and the verification block belong to
//! `receipt seal` and `receipt sign`, which are not this face.
//!
//! Ordering the dispatcher owes this face: on the session-end event the draft must be written
//! before [`crate::friction::emit`], because emit prunes the session's friction state and the
//! tool counts and the summary digest come from it. When the state is already gone the draft
//! is still written, with no counts and no digest, which is the documented empty case.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::error::{Error, Result};
use crate::friction::{
    SessionState, harness_label, harness_version, session_file_name, usage_tokens,
};
use crate::harness::Payload;
use crate::layout;

/// The principal a session driven from a terminal has, in tend2's vocabulary.
pub const PRINCIPAL_CLI: &str = "cli";

/// `.plotplot/receipts/drafts/<session_id>.json`, the unsigned v0 draft.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    pub harness: DraftHarness,
    /// The models the payload named; empty when it named none.
    pub models: Vec<String>,
    pub principal: String,
    pub sessions: Vec<String>,
    pub cost: Cost,
    pub friction: Friction,
}

/// Which harness produced the session, and at which version when it says.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftHarness {
    pub name: String,
    pub version: Option<String>,
}

/// What the session cost, in the units a hook can know. Never estimated.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cost {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    /// Wall time is not derivable from a session-end payload alone, so it is always null here.
    pub wall_seconds: Option<u64>,
    pub tool_calls: BTreeMap<String, u64>,
}

/// The digest of the session's friction state, when the ledger kept one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Friction {
    pub summary: Option<Sha256Digest>,
}

/// A sha256, hex encoded, in the `{"sha256": …}` shape the predicate uses throughout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sha256Digest {
    pub sha256: String,
}

/// The draft a payload and a session's friction state make, as a value.
///
/// Pure: the caller supplies the state and the digest of the bytes it was read from, and
/// nothing here touches the disk.
pub fn build(
    payload: &Payload,
    state: Option<&SessionState>,
    summary: Option<String>,
) -> Result<Draft> {
    let session = payload.session_id.clone().ok_or_else(|| Error::Harness {
        problem: format!(
            "the {} {} payload has no session_id, and a receipt draft is named by its session",
            payload.harness, payload.event
        ),
    })?;
    let (input_tokens, output_tokens) = usage_tokens(payload);

    Ok(Draft {
        harness: DraftHarness {
            name: harness_label(payload.harness).to_owned(),
            version: harness_version(payload),
        },
        models: payload.model.clone().into_iter().collect(),
        principal: PRINCIPAL_CLI.to_owned(),
        sessions: vec![session],
        cost: Cost {
            input_tokens,
            output_tokens,
            wall_seconds: None,
            tool_calls: state
                .map(|state| state.tool_calls.clone())
                .unwrap_or_default(),
        },
        friction: Friction {
            summary: summary.map(|sha256| Sha256Digest { sha256 }),
        },
    })
}

/// Write this session's draft, replacing any draft the session already has.
///
/// Returns the path written.
pub fn draft(root: &Path, payload: &Payload) -> Result<PathBuf> {
    let session = payload
        .session_id
        .as_deref()
        .ok_or_else(|| Error::Harness {
            problem: format!(
                "the {} {} payload has no session_id, and a receipt draft is named by its session",
                payload.harness, payload.event
            ),
        })?;
    let session = session_file_name(session)?;

    let state_path = layout::friction_state(root, session);
    let (state, summary) = match fs::read(&state_path) {
        Ok(bytes) => {
            let state: SessionState =
                serde_json::from_slice(&bytes).map_err(|source| Error::Json {
                    path: Some(state_path.clone()),
                    source,
                })?;
            (Some(state), Some(hex::encode(Sha256::digest(&bytes))))
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => (None, None),
        Err(source) => {
            return Err(Error::Io {
                path: state_path,
                source,
            });
        }
    };

    let draft = build(payload, state.as_ref(), summary)?;
    let text = serde_json::to_string_pretty(&draft)
        .map_err(|source| Error::Json { path: None, source })?;

    let directory = layout::receipt_drafts_dir(root);
    fs::create_dir_all(&directory).map_err(|source| Error::Io {
        path: directory,
        source,
    })?;
    let path = layout::receipt_draft(root, session);
    fs::write(&path, format!("{text}\n")).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::{Harness, parse_payload};
    use serde_json::{Value, json};

    fn session_end(harness: Harness, extra: Value) -> Payload {
        let mut object = json!({
            "hook_event_name": harness.session_end_event(),
            "session_id": "6f3c1b2a",
            "cwd": "/work/repo",
        });
        if let (Some(object), Some(extra)) = (object.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                object.insert(key.clone(), value.clone());
            }
        }
        parse_payload(harness, &object.to_string()).expect("a parsed payload")
    }

    fn value(draft: &Draft) -> Value {
        serde_json::to_value(draft).expect("a serializable draft")
    }

    #[test]
    fn the_draft_names_the_fields_the_receipts_plan_spells() {
        let payload = session_end(Harness::Claude, json!({"model": "claude-opus-5"}));
        let draft = build(&payload, None, None).expect("a draft");
        let value = value(&draft);
        let object = value.as_object().expect("a draft is a JSON object");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "cost",
                "friction",
                "harness",
                "models",
                "principal",
                "sessions"
            ]
        );
        let mut cost: Vec<&str> = object["cost"]
            .as_object()
            .expect("cost is an object")
            .keys()
            .map(String::as_str)
            .collect();
        cost.sort_unstable();
        assert_eq!(
            cost,
            ["inputTokens", "outputTokens", "toolCalls", "wallSeconds"]
        );
        assert_eq!(object["harness"]["name"], "claude-code");
        assert_eq!(object["principal"], "cli");
        assert_eq!(object["models"], json!(["claude-opus-5"]));
        assert_eq!(object["sessions"], json!(["6f3c1b2a"]));
    }

    #[test]
    fn every_harness_names_itself_as_the_profile_does() {
        for (harness, name) in [
            (Harness::Claude, "claude-code"),
            (Harness::Gemini, "gemini-cli"),
            (Harness::Codex, "codex-cli"),
        ] {
            let draft = build(&session_end(harness, json!({})), None, None).expect("a draft");
            assert_eq!(draft.harness.name, name);
        }
    }

    #[test]
    fn a_payload_that_names_no_model_carries_no_models() {
        let draft = build(&session_end(Harness::Gemini, json!({})), None, None).expect("a draft");
        assert!(draft.models.is_empty());
        assert_eq!(value(&draft)["models"], json!([]));
    }

    #[test]
    fn what_the_payload_is_silent_about_is_null_and_never_estimated() {
        let draft = build(&session_end(Harness::Claude, json!({})), None, None).expect("a draft");
        assert_eq!(draft.harness.version, None);
        assert_eq!(draft.cost.input_tokens, None);
        assert_eq!(draft.cost.output_tokens, None);
        assert_eq!(draft.cost.wall_seconds, None);
        assert!(draft.cost.tool_calls.is_empty());
        assert_eq!(draft.friction.summary, None);

        let value = value(&draft);
        for null in ["inputTokens", "outputTokens", "wallSeconds"] {
            assert!(value["cost"][null].is_null(), "{null}");
        }
        assert!(value["harness"]["version"].is_null());
        assert!(value["friction"]["summary"].is_null());
        assert_eq!(value["cost"]["toolCalls"], json!({}));
    }

    #[test]
    fn a_payload_that_carries_its_version_and_its_usage_says_so() {
        let payload = session_end(
            Harness::Claude,
            json!({
                "harness_version": "2.1.265",
                "usage": {"input_tokens": 41_000, "output_tokens": 2_100},
            }),
        );
        let draft = build(&payload, None, None).expect("a draft");
        assert_eq!(draft.harness.version.as_deref(), Some("2.1.265"));
        assert_eq!(draft.cost.input_tokens, Some(41_000));
        assert_eq!(draft.cost.output_tokens, Some(2_100));
    }

    #[test]
    fn the_friction_state_is_where_the_tool_counts_come_from() {
        let mut state = SessionState::new();
        state.tool_calls.insert("Bash".to_owned(), 41);
        state.tool_calls.insert("Edit".to_owned(), 12);
        let draft = build(
            &session_end(Harness::Claude, json!({})),
            Some(&state),
            Some("9c1d".to_owned()),
        )
        .expect("a draft");

        assert_eq!(draft.cost.tool_calls.get("Bash"), Some(&41));
        assert_eq!(draft.cost.tool_calls.get("Edit"), Some(&12));
        assert_eq!(
            draft.friction.summary,
            Some(Sha256Digest {
                sha256: "9c1d".to_owned()
            })
        );
        assert_eq!(
            value(&draft)["friction"]["summary"],
            json!({"sha256": "9c1d"})
        );
    }

    #[test]
    fn a_payload_with_no_session_cannot_be_drafted() {
        let json = r#"{"hook_event_name":"SessionEnd","cwd":"/work/repo"}"#;
        let payload = parse_payload(Harness::Claude, json).expect("a parsed payload");
        assert!(matches!(
            build(&payload, None, None),
            Err(Error::Harness { .. })
        ));
    }

    #[test]
    fn a_draft_survives_a_round_trip_through_its_file_shape() {
        let draft = build(
            &session_end(Harness::Claude, json!({"model": "claude-opus-5"})),
            None,
            None,
        )
        .expect("a draft");
        let text = serde_json::to_string(&draft).expect("a serializable draft");
        let back: Draft = serde_json::from_str(&text).expect("a readable draft");
        assert_eq!(back, draft);
    }
}
