//! The Codex CLI plugin.
//!
//! `.codex-plugin/plugin.json` declares where the rest of the bundle is, rather than
//! leaving it to convention: `./skills/`, `./hooks.json` and `./.mcp.json` (§5). The hook
//! file sits at the plugin's root, not under `hooks/`, and Codex expands the same
//! `${CLAUDE_PLUGIN_ROOT}` Claude does. Codex has no `SessionEnd`, so its `Stop` carries the
//! session's end as well as the turn's.

use super::{
    BundleInput, FileTree, Json, add_skills, hooks_file, hooks_path, manifest_head, mcp_file,
};
use crate::error::Result;
use crate::harness::Harness;

const HARNESS: Harness = Harness::Codex;

/// The plugin, as files.
pub(super) fn generate(input: &BundleInput) -> Result<FileTree> {
    let mut tree = FileTree::new();
    let mcp = mcp_file(HARNESS, input.beds);
    tree.insert(
        ".codex-plugin/plugin.json",
        plugin_json(input, mcp.is_some()),
    );
    tree.insert(hooks_path(HARNESS), hooks_file(HARNESS, input.beds)?);
    if let Some(mcp) = mcp {
        tree.insert(".mcp.json", mcp);
    }
    add_skills(input, &mut tree)?;
    Ok(tree)
}

/// Codex reads the paths this manifest names, so it names only paths the bundle has: the
/// skills directory when there are skills, the hook file always, and `.mcp.json` only when
/// a bed brought a server.
fn plugin_json(input: &BundleInput, has_mcp: bool) -> Vec<u8> {
    let mut fields = manifest_head(input);
    if !input.skills.is_empty() {
        fields.push(("skills".to_owned(), Json::string("./skills/")));
    }
    fields.push(("hooks".to_owned(), Json::string("./hooks.json")));
    if has_mcp {
        fields.push(("mcpServers".to_owned(), Json::string("./.mcp.json")));
    }
    Json::Object(fields).document().into_bytes()
}
