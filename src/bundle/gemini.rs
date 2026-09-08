//! The Gemini CLI extension.
//!
//! `gemini-extension.json` carries the extension's identity and its MCP servers inline;
//! Gemini has no `.mcp.json` (§5). The hook file sits at `hooks/hooks.json` as it does for
//! Claude, with timeouts in milliseconds, and `${extensionPath}` is what Gemini expands to
//! the installed extension's own directory.

use super::{BundleInput, FileTree, Json, add_skills, hooks_file, manifest_head, mcp_servers};
use crate::error::Result;
use crate::harness::Harness;

const HARNESS: Harness = Harness::Gemini;

/// The extension, as files.
pub(super) fn generate(input: &BundleInput) -> Result<FileTree> {
    let mut tree = FileTree::new();
    tree.insert("gemini-extension.json", extension_json(input));
    tree.insert("hooks/hooks.json", hooks_file(HARNESS, input.beds)?);
    add_skills(input, &mut tree)?;
    Ok(tree)
}

fn extension_json(input: &BundleInput) -> Vec<u8> {
    let mut fields = manifest_head(input);
    let servers = mcp_servers(HARNESS, input.beds);
    if !servers.is_empty() {
        fields.push(("mcpServers".to_owned(), Json::Object(servers)));
    }
    Json::Object(fields).document().into_bytes()
}
