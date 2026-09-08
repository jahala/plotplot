//! The Claude Code plugin.
//!
//! `.claude-plugin/plugin.json` names the plugin; `hooks/hooks.json`, `.mcp.json` and
//! `skills/` are found by their conventional paths rather than declared, so the manifest
//! carries only the plugin's identity (§5). `${CLAUDE_PLUGIN_ROOT}` is what Claude expands
//! to the installed plugin's own directory.

use super::{
    AUTHOR_URL, BundleInput, FileTree, Json, add_skills, hooks_file, hooks_path, manifest_head,
    mcp_file,
};
use crate::error::Result;
use crate::harness::Harness;

const HARNESS: Harness = Harness::Claude;

/// The plugin, as files.
pub(super) fn generate(input: &BundleInput) -> Result<FileTree> {
    let mut tree = FileTree::new();
    tree.insert(".claude-plugin/plugin.json", plugin_json(input));
    tree.insert(hooks_path(HARNESS), hooks_file(HARNESS, input.beds)?);
    if let Some(mcp) = mcp_file(HARNESS, input.beds) {
        tree.insert(".mcp.json", mcp);
    }
    add_skills(input, &mut tree)?;
    Ok(tree)
}

fn plugin_json(input: &BundleInput) -> Vec<u8> {
    let mut fields = manifest_head(input);
    fields.push((
        "author".to_owned(),
        Json::object([
            ("name", Json::string("the plotplot garden")),
            ("url", Json::string(AUTHOR_URL)),
        ]),
    ));
    Json::Object(fields).document().into_bytes()
}
