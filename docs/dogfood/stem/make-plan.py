#!/usr/bin/env python3
"""Emit the pleach plans that build the stem, under docs/dogfood/stem/ as pleach keeps its own.

One node per slice of docs/prompts/stem-build-2026-09.md. Run from the repository root:

    python3 docs/dogfood/stem/make-plan.py > docs/dogfood/stem/build.plan.json
    python3 docs/dogfood/stem/make-plan.py --reaudit > docs/dogfood/stem/reaudit.plan.json
    pleach validate docs/dogfood/stem/build.plan.json

The prompts are the only context a worker gets, so each names its files, its tests, what it
must not create, and the gate it must pass. Gate commands are wrapped in `bash -lc` because
pleach executes them as an argv array with no shell.
"""

import json
import sys

GATE_SHELL = "cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test"
# pleach execs gate commands as an argv array without a shell, so a shell line is wrapped.
GATES = f"bash -lc '{GATE_SHELL}'"

PREAMBLE = """You are building one slice of the stem, the `plotplot` binary, in this repository (jahala/plotplot, the umbrella of the plotplot garden). Read first, in this order, before touching code: CLAUDE.md; docs/building-the-garden.md (the law); docs/plans/stem.md (the plan); docs/tend2/stem.tend2.html (the loop, READ-ONLY for you); docs/prompts/stem-build-2026-09.md (the brief: the module contract you build against; its §4 shapes and §5 vendor facts are fixed, add to them, never rename or remove). Other slices are being built in parallel by workers who see the same brief and not your work, so the contract is what makes the slices fit.

Rules that bind you: the failing test is the spec, write it first and watch it fail; no stubs, mocks, TODOs, placeholders or fallbacks; library code never panics on input (no unwrap/expect/panic!/fallible indexing on data from files, stdin, the network or the environment); side effects at the edge; the repository root is a parameter, never current_dir() inside the library. Delete with `trash`, never `rm`. Never `git reset`. Never touch the stash. Leave your changes in the working tree; do not commit, branch or push. Scratch files go under .loop-scratch/ only. Do not edit the loop file.

The gate you must pass before you stop, run exactly like this from the repository root:

    %s

If a claim in your slice cannot be met honestly with what you have, write BLOCKED.md at the repository root naming what you tried, what stopped you and what a fix needs, instead of faking it. Your final message states: what changed (files), the gate's output, what you could not verify, and what you assumed. No time estimates, no promises about later work.
""" % GATE_SHELL

# The crate node (Cargo.toml, the shared model, `plotplot version`) ran once as stem.crate;
# its worker finished the slice but the gate could not run (see the dogfood ledger), so the
# attempt was landed by hand after passing the gate, and the node is gone from the plan.

BUNDLE = PREAMBLE + """
## Your slice: the three vendor bundles, `src/bundle/`

The crate, the shared model (harness, bed, manifest, lock, layout, error) and `plotplot version` exist in this worktree. Read src/harness.rs, src/bed.rs, src/layout.rs and src/error.rs before writing anything, and build only on their public shapes.

Create src/bundle/mod.rs, src/bundle/claude.rs, src/bundle/gemini.rs, src/bundle/codex.rs and tests/bundle.rs, and add `pub mod bundle;` to src/lib.rs. Shapes: brief §4 "bundle/" (SkillFile, BundleInput, FileTree, generate, write, Difference, diff) and brief §5 for every vendor file, byte for byte in shape. `generate` is pure: no I/O. `write` creates directories as needed and returns the paths written, sorted. `diff` compares a tree against a directory byte for byte and reports Missing, Extra (a file under the root that the tree does not have, excluding bin/) and Changed.

What each bundle contains (brief §5, first row): the vendor manifest file, the hook file, skills/<bed>/SKILL.md for every SkillFile, and MCP entries for every Bed with mcp Some (Claude and Codex in .mcp.json; Gemini in gemini-extension.json's mcpServers); when no bed has mcp, no .mcp.json is emitted and Gemini's manifest has no mcpServers key. Never emit bin/. JSON output is pretty-printed with two-space indent and a trailing newline, keys in a deterministic order.

Hook entries: one entry per event, no vendor matcher, command exactly `"<root-variable>/bin/plotplot" hook <harness> <event>` with the root variable from Harness::bundle_root_variable, timeout in the vendor's unit (brief §4 "bundle/" for the values). The set of events per harness is the union of: every event any Bed declared for that harness; the harness's before-tool event (the deny list); the events the friction profile derives kinds from (contracts/friction-profile.md, "the pinned kinds" table) that the harness has (Claude: PreToolUse, PostToolUse, PostToolUseFailure, PermissionDenied, Stop, PreCompact, SessionEnd; Gemini: BeforeTool, AfterTool, AfterAgent, PreCompress, SessionEnd; Codex: PreToolUse, PostToolUse, Stop, PreCompact); and the harness's session-end event (the receipt draft). Emit the events in the harness's events() order.

Tests you write first (tests/bundle.rs, using beds built from contracts/fixtures/manifest/*.garden.json through manifest::parse_manifest and manifest::to_bed, plus two SkillFiles):
- for each harness, generate yields exactly the expected file set (names listed per vendor in brief §5), and no bin/;
- each vendor manifest parses back as JSON with name "plotplot", the stem version, and a description naming the season and every bed;
- the Codex manifest carries skills "./skills/", hooks "./hooks.json", and mcpServers "./.mcp.json" only when a .mcp.json is emitted;
- every hook entry's command names bin/plotplot under the vendor's root variable, `hook <harness> <event>`, and nothing else; timeouts are seconds for Claude and Codex and milliseconds for Gemini;
- the weeder bed's declared events are present in every harness's hook file, and the stem's own events (deny, friction, session end) are present even with no beds at all;
- the three bundles carry the same skill set and the same MCP server set (compare the sets, not the vendor file names);
- write then diff on a temp dir is empty; changing one byte of the hook file yields exactly one Changed; deleting a skill yields Missing; an extra file yields Extra;
- generated output is deterministic: two generate calls produce identical trees.

Do NOT create or edit: src/cli.rs, src/main.rs, src/deny.rs, src/friction.rs, src/receipt.rs, src/hook.rs, src/doctor.rs, src/plant/, scripts/. Do NOT change any public shape in the modules that exist; if one blocks you, say exactly what in your final message and work around it inside your own module.
"""

FACES = PREAMBLE + """
## Your slice: the stem's own hook-time faces, `src/deny.rs`, `src/friction.rs`, `src/receipt.rs`

The crate, the shared model (harness, bed, manifest, lock, layout, error) and `plotplot version` exist in this worktree; fixture payloads for every harness and event are under tests/fixtures/payloads/. Read src/harness.rs, src/layout.rs and src/error.rs before writing anything, and build only on their public shapes.

Create src/deny.rs, src/friction.rs, src/receipt.rs, tests/faces.rs, and add the three `pub mod` lines to src/lib.rs. Shapes: brief §4 "deny.rs", "friction.rs", "receipt.rs". The friction record shape, envelope, kinds and per-kind attributes are contracts/friction-profile.md, exactly; docs/plans/friction-ledger.md §3 to §5 is the reasoning; docs/plans/receipts.md §3 and §4 give the draft's fields. For `now` in friction::emit take a parameter (a struct or closure yielding the RFC 3339 UTC string) so tests pin the time; the CLI will pass the system clock later.

deny::decide, over the before-tool event only (any other event is Allow): Deny when the tool is a shell tool (Claude Bash, Gemini run_shell_command, Codex shell) and its command carries `git commit`, `git push` or `git merge` with `--no-verify` or a `-n` flag; Deny when a file-writing tool (Claude Write/Edit/MultiEdit, Gemini write_file/replace, Codex apply_patch, the tilth_write MCP tool on any harness, or a shell redirection `>`/`>>`/`tee` in a shell command) targets garden.lock, .githooks/, .plotplot/bin/ or .plotplot/beds/ (match on the path relative to the payload's cwd, and on absolute paths under it). The reason names the rule and the fix in one sentence. Nothing else is denied.

friction::derive, per the profile's "derived from" column, with session state for reread, churn, retry, fanout and test-loop: tool.denied from Claude PermissionDenied; tool.failed from Claude PostToolUseFailure, Gemini AfterTool with an error in tool_response, Codex PostToolUse with a non-zero exit in tool_response; tool.retry when the same tool with the same input hash appears within the last five PreToolUse/BeforeTool payloads of the session; file.reread on the second and later read of one path in a session (Read, tilth_read, read_file, `cat`/`sed -n` in a shell command); search.fanout for each Grep/Glob/tilth_search/grep_search/glob/`rg` before the session's first edit; edit.churn on the second and later edit of one path (Edit, Write, MultiEdit, tilth_write, write_file, replace, apply_patch); test.loop on the second and later run of a test command in a session (`cargo test`, `npm test`, `pnpm test`, `bun test`, `pytest`, `go test`, `node --test`, `vitest`, `jest`); context.compacted from Claude PreCompact and Gemini PreCompress; session.ended from Claude/Gemini SessionEnd and Codex Stop, with usage tokens null unless the payload carries them. Paths are made repo-relative against the payload's cwd; a path outside becomes plotplot.path.kind "outside" with no plotplot.path; path.kind is one of prod, test, doc, config, outside (test: a path with test/tests/spec/__tests__ segments or .test./.spec. in the file name; doc: .md/.txt/.html under docs or at the root; config: dotfiles, *.toml, *.json, *.yaml, *.yml at the root). plotplot.harness is claude-code / gemini-cli / codex-cli; plotplot.harness.version is read from the payload when present and omitted otherwise; gen_ai.request.model is present only when the payload carries a model. friction::emit appends to .plotplot/friction/<yyyy-mm>.jsonl (the month from `now`), keeps state under .plotplot/friction/state/<session_id>.json, prunes that file at the session-end event, creates directories as needed, and returns the number of lines written; a payload without a session id writes nothing and returns Ok(0).

receipt::draft writes .plotplot/receipts/drafts/<session_id>.json with: harness name and version (null when the payload has none), models (the payload's model in a one-element list, or empty), principal "cli", sessions [session_id], toolCalls by tool name from the friction state (empty object when no state), wallSeconds null, inputTokens and outputTokens null unless the payload carries them, and friction.summary as the sha256 of the session's state file bytes when it exists, else null; fields named as docs/plans/receipts.md §3 spells them. A second draft for the same session overwrites the first.

Tests you write first (unit beside the code; tests/faces.rs for the file-level behaviour with tempfile roots):
- deny: the `--no-verify` fixture command on each harness's shell tool is Deny with a reason naming --no-verify; `git commit -n -m x` is Deny; `git commit -m "no --no-verify here"` in quotes is still Deny (the flag is present; do not parse shell quoting cleverly, match the token); `git log -n 3` is Allow (the flag is not on commit/push/merge); a Write to garden.lock is Deny; an Edit to .githooks/pre-commit is Deny; `echo x > .plotplot/bin/weeder` is Deny; a Write to src/main.rs is Allow; every non-before-tool fixture is Allow.
- friction: feed each fixture payload through derive with a fresh state and assert the kinds produced (PostToolUseFailure → tool.failed with error.type; PermissionDenied → tool.denied; PreCompact/PreCompress → context.compacted with gen_ai.conversation.compacted true; SessionEnd and Codex Stop → session.ended; the leaky fixtures produce records with no prompt text, no tool output, and no absolute path anywhere in the serialized line, asserted by searching the JSON for the fixture's leaky strings and for "/Users/"); two reads of one path produce one file.reread on the second; two edits produce one edit.churn; two `cargo test` commands produce one test.loop; the same Bash input twice within five calls produces tool.retry; every record serializes with the envelope's required keys and the per-kind required attributes of contracts/friction-profile.md (write the tables into the test as data and check every produced record against them); `time` ends in Z.
- emit: on a temp root, emitting the Claude SessionStart, PreToolUse, PostToolUse, PostToolUseFailure and SessionEnd fixtures in order writes the expected number of lines to .plotplot/friction/<month>.jsonl, the state file exists after PostToolUse and is gone after SessionEnd; emitting without a session id returns Ok(0) and creates nothing; SessionEnd emission on the fixture completes under 200 ms (measure it).
- receipt: draft on the Claude SessionEnd fixture writes the file with the documented fields and null where the payload is silent; drafting twice overwrites; with a friction state present, toolCalls carries the counts and friction.summary the state file's sha256.

Do NOT create or edit: src/cli.rs, src/main.rs, src/bundle/, src/hook.rs, src/doctor.rs, src/plant/, scripts/. Do NOT change any public shape in the modules that exist; if one blocks you, say exactly what in your final message and work around it inside your own modules.
"""

PLANT = PREAMBLE + """
## Your slice: what `init` writes, as pure renderers, `src/plant/`

The crate, the shared model (harness, bed, manifest, lock, layout, error) and `plotplot version` exist in this worktree. Read src/bed.rs, src/layout.rs, src/lock.rs and src/error.rs before writing anything, and build only on their public shapes.

Create src/plant/mod.rs, src/plant/garden_block.rs, src/plant/githooks.rs, src/plant/gitconfig.rs and tests/plant.rs, and add `pub mod plant;` to src/lib.rs. Shapes: brief §4 "plant/". These are the parts `init` will write; `init` itself is a later slice, so nothing here reads the current directory or decides what to plant: every function takes the root, the season and the beds as parameters.

garden_block::render(season, beds) -> String: the block between `<!-- plotplot:begin -->` and `<!-- plotplot:end -->`, at most ten lines between the markers, sentence case, no exclamation marks, no em dashes, product names lowercase: what is planted (each bed name and version on one line), the season, orient with `tend2 next docs/tend2`, gates with `plotplot check`, help with `plotplot doctor`, and that the hard limits are enforced at the boundary (no --no-verify, garden.lock and .githooks are the stem's). garden_block::replace_in(agents_md, block) -> String: replaces an existing marked block in place and leaves every other byte untouched; appends the block after one blank line when no markers exist; is idempotent (applying the same block twice yields the same bytes as once); when the file has a begin marker without an end marker, returns Error::Bed? No: this function is total, it returns a Result with an Error::Manifest-free variant; add `Error::Agents { problem: String }` to src/error.rs for a malformed marker pair (additive, allowed).

githooks::render(hook, beds) -> String: a POSIX sh script with a shebang, `set -eu`, resolving its own repository root from `git rev-parse --show-toplevel`; for pre-commit, pre-push and pre-rebase it runs `.plotplot/bin/<bed binary> guard <hook-name>` for every bed whose git_hooks include that hook, forwarding stdin and arguments (pre-push receives refs on stdin) and exiting with the first non-zero status; post-commit runs `.plotplot/bin/plotplot receipt seal` when that binary exists, and the beds that declared post-commit after it; a hook no bed declared still renders a valid script that exits 0. Each script's first comment line says it is written by plotplot and not to edit it by hand.

gitconfig::desired(root) -> Vec<(String, String)>: core.hooksPath ".githooks"; remote.origin.fetch "+refs/notes/plotplot/receipts:refs/notes/plotplot/receipts"; remote.origin.push "refs/notes/plotplot/receipts:refs/notes/plotplot/receipts". gitconfig::read(root) -> Result<Vec<(String, String)>>: the current values of those keys via `git config --get-all`, absent keys omitted, Error::Git when git fails. gitconfig::missing(desired, current) -> Vec<(String, String)>: pure. gitconfig::apply(root, entries) -> Result<()>: `git config --add` for multi-valued refspecs, `git config` for hooksPath, never duplicating an entry already present.

Tests you write first (tests/plant.rs with tempfile git repositories initialised by running `git init`; unit tests beside the code for the pure functions):
- render's block has the two markers, at most ten lines between them, names every bed with its version, the season, the three commands, and contains no exclamation mark and no em dash (U+2014);
- replace_in on the repository's own AGENTS.md content (read it as a fixture string) appends the block and leaves the tend2 block byte-identical; a second application is a no-op; a changed block replaces only the marked region; a begin marker without an end is Error::Agents;
- githooks: each of the four renders, starts with #!/bin/sh, is different per hook, names every declaring bed's binary under .plotplot/bin/, and shellcheck-free in the sense that `sh -n` accepts it (run `sh -n` on the rendered text in the test);
- gitconfig: on a fresh temp repository with a remote named origin, read returns nothing for the three keys; apply(desired) then read returns all three; apply twice leaves single entries (assert `git config --get-all` counts); missing is empty afterwards; read on a directory that is not a repository is Error::Git.

Do NOT create or edit: src/cli.rs, src/main.rs, src/bundle/, src/deny.rs, src/friction.rs, src/receipt.rs, src/hook.rs, src/doctor.rs, scripts/. Do NOT change any existing public shape beyond adding the Error::Agents variant.
"""

HOOK = PREAMBLE + """
## Your slice: the dispatcher `src/hook.rs` and the CLI faces `hook`, `friction emit`, `receipt draft`

This worktree merges the crate (src/cli.rs with `version`; src/harness.rs, bed.rs, manifest.rs, lock.rs, layout.rs, error.rs) with the faces slice (src/deny.rs, src/friction.rs, src/receipt.rs). Read all of them before writing anything, and build only on their public shapes. If the merge left conflict markers anywhere, resolve them keeping every side's work before doing anything else.

Create src/hook.rs and tests/hook.rs, add `pub mod hook;` to src/lib.rs, and add three faces to src/cli.rs: `plotplot hook <harness> <event>`, `plotplot friction emit --harness <harness>`, `plotplot receipt draft --harness <harness>`; each reads the payload from stdin. Shapes: brief §4 "hook.rs" and "cli.rs". The handler for each face lives in its module (hook::run, friction::run, receipt::run taking the root, the parsed arguments, stdin as a &str, and the stdout/stderr writers) and cli.rs holds only the argument types and the match.

The contract with beds (brief §4 "hook.rs"; ruled by the umbrella 2026-09-08): `<bed> hook <harness>` reads the vendor's payload on stdin, writes the vendor's answer on stdout, exits 0 to allow, 2 to deny, 3 when it cannot judge. dispatch(root, harness, event, payload_json): (1) parse the payload; (2) run deny::decide first; a Deny renders the vendor answer on stdout, the reason on stderr, code 2, and no bed is called; (3) load the beds from .plotplot/beds via manifest::load_beds, keep those with a HookEntry for this harness and event whose matcher, when present, equals the payload's tool_name, and resolve each to .plotplot/bin/<binary>; a bed whose binary is absent is a BedResult with code 3 and a stderr naming the missing path; (4) fan_out in parallel, one thread per bed, each writing the payload to the child's stdin and collecting code, stdout, stderr, with the timeout (a child still running at the timeout is killed and reported as code 3); (5) merge per the rule: any 2 → 2, the first denial's stdout on stdout and its stderr first on stderr, the other denials' stderr after; else any 3 → 3 with each reason on stderr; else 0 with empty stdout; (6) on the harness's session-end event, receipt::draft first, because the draft reads the session's friction state and (7) prunes it: friction::emit with the system clock, always, after the merge, never changing the code, logging an error to stderr and continuing. The timeout for beds is 4 s on a stop or session-end event and 8 s otherwise. Exit code 3 for a Cannot is honoured as is.

Tests you write first (tests/hook.rs, assert_cmd against the built binary, fixture beds as executable sh scripts written into a temp root's .plotplot/bin/ by the test, with matching manifests under .plotplot/beds/<name>/garden.json built from contracts/fixtures/manifest/weeder.garden.json with the name and faces.cli changed):
- with no beds and no .plotplot, `plotplot hook claude PreToolUse` on the Claude PreToolUse fixture exits 0 with empty stdout and appends one line to .plotplot/friction/<month>.jsonl only if a kind derives (it does not for a plain PreToolUse; assert the file is absent or unchanged), and the invocation from spawn to exit takes under 50 ms measured over ten runs (report the median in the assertion message; the gate is the median);
- the `--no-verify` fixture exits 2 with the Claude deny JSON on stdout and the reason on stderr, and no bed script ran (the scripts write a marker file when they run);
- a bed script that exits 2 with a JSON answer on stdout makes the dispatcher exit 2 with that stdout, and a second denying bed's stderr follows the first's;
- a bed script that exits 3 with no denials makes the dispatcher exit 3 with the reason on stderr;
- a bed declared on `PreToolUse:Bash` runs for a Bash payload and does not run for a Read payload (marker files);
- a bed declared on Stop only does not run on PreToolUse;
- a bed whose binary is missing yields exit 3 naming the path;
- a bed script that sleeps past the timeout yields exit 3 and the dispatcher returns within the timeout plus one second;
- two beds run concurrently: two scripts that each sleep 300 ms complete in under 500 ms total;
- `plotplot hook claude SessionEnd` on the SessionEnd fixture writes the receipt draft and the session.ended friction line and prunes the state file;
- `plotplot friction emit --harness gemini` on the Gemini AfterTool leaky fixture writes a line containing no leaky string; `plotplot receipt draft --harness codex` on the Codex Stop fixture writes the draft;
- an unknown harness or event is a usage error with exit 2 and a message naming the valid values (clap's exit code for a bad value is 2; keep it).

Do NOT create or edit: src/bundle/, src/doctor.rs, src/plant/, scripts/. Do NOT change any existing public shape; add to cli.rs only the three faces, each as one enum variant and one match arm, keeping `version` intact.
"""

DOCTOR = PREAMBLE + """
## Your slice: `plotplot bundle build`, `plotplot doctor` (static mode), `src/doctor.rs`

This worktree merges the crate (src/cli.rs with `version`; src/harness.rs, bed.rs, manifest.rs, lock.rs, layout.rs, error.rs) with the bundle slice (src/bundle/) and the plant slice (src/plant/). Read all of them before writing anything, and build only on their public shapes. If the merge left conflict markers anywhere, resolve them keeping every side's work before doing anything else.

Create src/doctor.rs and tests/doctor.rs, add `pub mod doctor;` to src/lib.rs, and add two faces to src/cli.rs: `plotplot bundle build [claude|gemini|codex]` (all three when omitted) and `plotplot doctor`. Shapes: brief §4 "doctor.rs" and "cli.rs". Handlers live in their modules (bundle::run and doctor::run taking the root, the home directory, the parsed arguments and the writers); cli.rs holds only the argument types and the match.

`bundle build`: reads the beds from .plotplot/beds via manifest::load_beds, reads each bed's SKILL.md from .plotplot/beds/<name>/<skill path> when the bed declares a skill (a declared skill whose file is absent is Error::Bed naming the path), reads the season from garden.lock (Error when absent: a bundle needs a season) and the stem version from the crate, calls bundle::generate and bundle::write for each requested harness under .plotplot/bundles/<harness>/, then copies the running binary (std::env::current_exe, resolved in the CLI layer and passed in) to <bundle>/bin/plotplot with the executable bit, and prints one line per file written. A bed with mcp None but faces.mcp present in its manifest is named on stderr as a channel the stem cannot plant yet (brief §4 "manifest.rs"). Running it twice writes identical bytes.

`doctor` static mode, one Finding per check, printed as an aligned table (check, ok/fail, detail) and exit 0 when all ok else 3: (1) for each harness, the bundle directory exists and bundle::diff against a fresh generate is empty; (2) each bundle's bin/plotplot exists and its sha256 equals the running binary's; (3) every hook entry in every bundle's hook file names bin/plotplot under the vendor's root variable with `hook <harness> <event>` and nothing else; (4) core.hooksPath is .githooks (plant::gitconfig::read); (5) each of the four .githooks files exists, is executable, and equals plant::githooks::render for the current beds; (6) the garden block in AGENTS.md equals plant::garden_block::render for the current season and beds; (7) the receipts refspec is present in both fetch and push (plant::gitconfig); (8) every judge in garden.lock that has a platforms entry for this platform is present under .plotplot/bin/ with a sha256 equal to the lock's artifact sha256 when the bin is the artifact itself, or, when the artifact is an archive, the bin exists and .plotplot/bin/<judge>.sha256 (written by the lock face later) equals the lock's value; an npm judge is checked for presence of .plotplot/bin/<judge>; (9) Codex trust: <home>/.codex/config.toml has [projects."<absolute root>"] trust_level = "trusted"; when absent the finding is ok with the detail "awaiting project trust" (the stem cannot grant trust; doctor reports it honestly), when the file is unparsable the finding fails. When garden.lock or .plotplot/beds is absent, doctor prints one line saying the repository is not planted and exits 3.

Tests you write first (tests/doctor.rs with tempfile roots; build a planted fixture in the test: `git init`, a remote origin, contracts/fixtures/garden.lock copied to garden.lock, two manifests from contracts/fixtures/manifest/ under .plotplot/beds/<name>/garden.json with a SKILL.md beside each, fixture judge files under .plotplot/bin/ with .sha256 companions matching the lock, .githooks written from plant::githooks::render, AGENTS.md carrying the rendered block, gitconfig applied):
- `bundle build` writes the three bundles, each with bin/plotplot executable, and a second run changes no bytes (hash the directory before and after);
- `bundle build gemini` writes only the Gemini bundle;
- `bundle build` without garden.lock fails with exit 1 and names garden.lock;
- `doctor` on the fully planted fixture exits 0 with every finding ok, and Codex reports "awaiting project trust" with a temp HOME that has no .codex/config.toml;
- with a temp HOME whose .codex/config.toml trusts the root, the Codex finding says trusted;
- changing one byte in a bundle's hook file makes finding (1) fail and exit 3; deleting .githooks/pre-push fails (5); removing the executable bit fails (5); editing the garden block fails (6); unsetting core.hooksPath fails (4); a judge with a wrong .sha256 fails (8); replacing bin/plotplot with another file fails (2);
- `doctor` on an unplanted temp dir exits 3 with the not-planted line.

Do NOT create or edit: src/deny.rs, src/friction.rs, src/receipt.rs, src/hook.rs, scripts/. Do NOT change any existing public shape; add to cli.rs only the two faces, keeping `version` intact.
"""

INTEGRATE = PREAMBLE + """
## Your slice: integration, the fit evidence, and the loop's two checks

This worktree merges every slice of the stem built so far: the crate, src/bundle/, src/deny.rs, src/friction.rs, src/receipt.rs, src/plant/, src/hook.rs (faces hook, friction emit, receipt draft) and src/doctor.rs (faces bundle build, doctor). src/cli.rs and src/lib.rs were edited by two branches and the merge may have left conflict markers: resolve them first, keeping every face and every `pub mod` line, then run the gate. Then read the whole crate once as a reviewer would: duplicated helpers between slices (two path-normalisers, two sha256 helpers, two JSON pretty-printers) are folded into one home each, with tests kept; a public shape that drifted from docs/prompts/stem-build-2026-09.md §4 is brought back or the brief's deviation is named in your final message.

One gap between two slices to close first, test first: contracts/friction-profile.md derives `tool.denied` from "our own PreToolUse decision", but friction::derive sees only the payload, so a refusal by the stem's own deny list (src/deny.rs, called in src/hook.rs) writes no friction record while Claude's PermissionDenied does. Add a way for the dispatcher to record its own denial (for example `friction::emit_denied(root, payload, rule: &str, now)` producing one `tool.denied` record with `plotplot.rule` naming the deny-list rule and the path attributes the profile requires), call it from the dispatcher on a deny-list refusal, and assert in tests/hook.rs that `plotplot hook claude PreToolUse` on the `--no-verify` fixture appends one `tool.denied` line to the journal and that the line carries the envelope and the profile's per-kind attributes.

Then write scripts/fit/stem.sh, the evidence the loop names, composing scripts/fit/lib.sh where it helps. `scripts/fit/stem.sh <check>` builds the crate in release mode once (cargo build --release) and runs one check; exit 0 on pass, non-zero with a reason on failure; every check runs in a fresh temp directory with HOME and CODEX_HOME pointed at a temp home so nothing on the machine's user scope is touched, and cleans up with trash, never rm. Implement exactly these two checks; do not add a subcommand for a check you cannot close honestly, and do not write one that exits 0 without running the binary:

- `bundles`: plants a fixture repository (git init, garden.lock from contracts/fixtures/garden.lock, the weeder and tilth manifests from contracts/fixtures/manifest/ under .plotplot/beds/ with a SKILL.md beside each), runs `plotplot bundle build`, then installs each bundle through its vendor's own mechanism against the temp home and asserts the install succeeded and the installed bundle carries the same hooks, skills and MCP entries as the generated one: Claude via a local marketplace directory (write <tmp>/marketplace/.claude-plugin/marketplace.json listing the plotplot bundle as a local plugin, `claude plugin marketplace add <dir>`, `claude plugin install plotplot@<marketplace> --scope project`, then read what Claude wrote under the temp home and the project); Gemini via `gemini extensions install <bundle path>` (user scope under the temp home, the only scope Gemini has) and `gemini extensions validate`; Codex via `codex plugin marketplace add <dir>` and `codex plugin add plotplot` against CODEX_HOME. Where a vendor command needs a login or network and fails for that reason rather than because of the bundle, the check fails and says which vendor and why; never skip silently. Finally asserts the three generated bundles' hook event sets, skill sets and MCP sets agree with each other (jq over the files).
- `hook-faces`: builds the same fixture, runs `plotplot bundle build`, and asserts from the three hook files that every event the friction profile derives a kind from and that the harness has (brief §4 "bundle/" lists them) and the harness's session-end event carry a dispatcher entry; then runs `plotplot hook <harness> <event>` on the fixture payloads for Claude PreToolUse, Gemini BeforeTool and Codex PreToolUse and on each session-end event, asserting exit 0, the friction line for session end, the receipt draft, and a median wall time under 50 ms over ten runs per harness (use a millisecond clock; python3 or date +%s%N where available; print the medians).

Run both checks and make them pass. Then run the verifier for the two loop checks this evidence closes, from the repository root, and paste its output into your final message:

    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 5
    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 8

(check 5 is the bundles line and check 8 the hook-faces line, counting the `- [ ]` lines of the loop's Tests section from 1; confirm by reading the file, and if the numbering differs say so and use the right ones). If the verifier refuses, say exactly what it printed; do not edit the loop file yourself under any circumstance.

Do NOT add faces beyond those merged here (no init, check, lock, seal, sign). Do NOT delete tests. If shortening a file by more than half is part of folding duplicates, name the file and the reason in your final message.
"""


LOCK = PREAMBLE + """
## Your slice: the lockfile wrapper, `plotplot lock verify`, `src/fetch.rs`, `src/lock.rs` (fetching half)

This worktree carries the integrated stem: crate, bundles, faces, plant, hook dispatcher, doctor, and scripts/fit/stem.sh with the bundles and hook-faces checks. Read src/lock.rs, src/layout.rs, src/manifest.rs, src/doctor.rs, src/cli.rs and scripts/fit/stem.sh before writing anything.

Add src/fetch.rs with `pub trait Fetch { fn fetch(&self, url: &str) -> Result<Vec<u8>>; }`, `pub struct Https;` implementing it with ureq over rustls (a non-2xx status or a transport error is Error::Fetch { url, problem }; add that variant and `Error::Checksum { judge, expected, actual }` and `Error::Archive { path, problem }` to src/error.rs, additively), and a test-only `FromMap` implementation behind #[cfg(test)] or in tests/ that serves bytes from a map (this injects the network edge; it never replaces the unit under test, which is the verification and placement logic). Add the `tar` and `flate2` crates for .tar.gz; a .zip artifact is Error::Archive naming zip as unsupported on this platform (no Windows target yet). Then extend src/lock.rs with:

```rust
pub enum JudgeState { Verified, Fetched { manifest: bool }, Missing(String), Mismatch { expected: String, actual: String } }
pub struct JudgeReport { pub name: String, pub version: String, pub state: JudgeState }
pub fn verify(root: &Path, lock: &Lock, platform: &str, fetch: &dyn Fetch) -> Result<Vec<JudgeReport>>;
```

For each judge, in name order: a platforms judge resolves platforms[platform] (absent → Missing naming the platform); when .plotplot/bin/<name> exists and .plotplot/bin/<name>.sha256 equals the lock's sha256 → Verified without any fetch; when the companion file exists and differs → Mismatch, and nothing is fetched or replaced (the lock is the truth; the caller decides); otherwise fetch the url, compute sha256 over the bytes, on mismatch return Mismatch and write nothing; on match extract the .tar.gz into .plotplot/beds/<name>/artifact/, copy the executable named by the manifest's install.binary_name (default faces.cli) from the artifact root to .plotplot/bin/<binary> (the same file name, which is what src/hook.rs and src/plant/githooks.rs call; when the artifact carries no garden.json the file name falls back to the judge's lock key) with mode 0o755, write .plotplot/bin/<binary>.sha256 with the artifact's digest, and make src/doctor.rs's judges check resolve each judge's file name the same way (through the cached manifest's Bed.binary, else the lock key) instead of the lock key alone, with a test where binary_name differs from the bed name, copy the artifact's garden.json to .plotplot/beds/<name>/garden.json when the artifact carries one, and report Fetched { manifest: true|false } (the checksum contract held either way; an artifact without a manifest is reported as `fetched, no manifest in artifact` on the verify line, and it is `init`, later, that refuses to plant a bed without a cached manifest, never `lock verify`). An npm judge: run `npm install --prefix .plotplot/npm/<name> <npm>@<version>` (Error::Bed when npm exits non-zero, with its stderr), then place .plotplot/bin/<name> as a symlink to the package's bin and read garden.json from the installed package root; a git judge: `git clone <url> .plotplot/beds/<name>/artifact` then `git -C … checkout <rev>` (a symbolic rev cannot occur; the schema refuses it), garden.json from the clone. `plotplot lock verify` prints one line per judge (`<name> <version> verified|fetched|missing: …|mismatch: …`) and exits 0 when every judge is Verified or Fetched, 3 when any is Mismatch or Missing, 1 on other errors; nothing is ever used from PATH.

Tests you write first (unit with FromMap; tests/lock.rs with assert_cmd against a lock whose url the binary cannot fetch, proving the exit codes and messages, and a mismatch case using a pre-placed .sha256 companion): a fixture .tar.gz built in the test (tar + flate2) with a small executable, garden.json (from contracts/fixtures/manifest/weeder.garden.json) and SKILL.md; verify with FromMap serving those bytes under the fixture URL and the right sha256 → Fetched, the bin exists and is executable, the companion carries the digest, garden.json is cached; a second verify → Verified with no fetch (FromMap counts calls); a wrong sha256 in the lock → Mismatch and no file under .plotplot/bin/; a lock naming a platform the judge lacks → Missing; a tarball without garden.json → Fetched { manifest: false } and no cached manifest. Then add the fit subcommand `lock` to scripts/fit/stem.sh against a real release: tilth v0.10.1's aarch64-apple-darwin asset at https://github.com/jahala/tilth/releases/download/v0.10.1/tilth-aarch64-apple-darwin.tar.gz with sha256 38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55 (verified 2026-09-08 from the downloaded bytes; the tarball holds only the `tilth` executable), or the platform's asset when the fit runs elsewhere (scripts/fit/lib.sh's fit_platform names it; a platform with no asset in the lock makes the check say so and fail): in a fixture repository write a garden.lock naming tilth with that url and digest, run `plotplot lock verify` and assert exit 0, the bin under .plotplot/bin/tilth executable and `.plotplot/bin/tilth --version` printing 0.10.1, the .sha256 companion; then write a lock with one digit of the digest changed into a second fixture and assert `plotplot lock verify` exits 3 naming the mismatch with nothing under .plotplot/bin/; then assert `git check-ignore .plotplot/bin/tilth` succeeds in the fixture after copying this repository's .gitignore, and that `git ls-files` in this repository lists no file under .plotplot/. Then run the verifier for the lock check and paste its output into your final message:

    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 4


Do NOT change existing public shapes; add only. Do NOT edit the loop file.
"""

CHECK = PREAMBLE + """
## Your slice: `plotplot check`, `src/check.rs`, `src/sarif.rs`

This worktree carries the integrated stem: crate, bundles, faces, plant, hook dispatcher, doctor, fit script, and the lock face. Read src/bed.rs, src/manifest.rs, src/layout.rs, src/cli.rs and contracts/vendor/sarif-schema-2.1.0.json before writing anything.

Add src/sarif.rs: `pub fn validate(json: &str) -> Result<serde_json::Value>` enforcing contracts/vendor/sarif-schema-2.1.0.json (embedded with include_str!) and returning the parsed log; `pub fn merge(logs: &[serde_json::Value]) -> serde_json::Value` producing one SARIF 2.1.0 log with `version` "2.1.0", the schema `$schema` URL from contracts/pins.json, and every input log's runs concatenated in order; `pub fn block_count(log: &Value) -> usize` counting results whose level is "error"; add `Error::Sarif { gate: String, problem: String }` additively. Add src/check.rs: `pub struct Gate { pub bed: String, pub program: PathBuf, pub args: Vec<String> }` built from every Bed with check Some by replacing the command's first word with .plotplot/bin/<binary> (never PATH) and splitting the rest on whitespace (a check command needing shell quoting is Error::Bed naming the bed); `pub fn run(root, gates, strict, timeout) -> Vec<GateOutcome>` running each gate in the repository root with stdout captured, `GateOutcome::Sarif(Value) | CouldNotRun { bed, problem }` where a missing binary, a non-zero exit without a valid SARIF log, or invalid SARIF is CouldNotRun; `--strict` appends `--strict` only to gates whose bed manifest declares `check_strict: true` (read it from the manifest's extra fields; when the umbrella adds the field to the schema the adapter follows; name this in your final message) and never otherwise. `plotplot check [--strict] [--format sarif|table]` prints the merged log on stdout (sarif, the default when stdout is not a terminal) or a table (one row per gate: bed, results, blocks, could-not-run), and exits 2 when any run has a block-level result, 3 when any gate could not run (fail closed; both → 3), 0 otherwise. A planted repository with no gates at all prints a log with zero runs and exits 0.

Tests you write first (tests/check.rs with a temp planted root, fixture gate scripts under .plotplot/bin/ that print SARIF from contracts/fixtures/sarif/weeder-check.sarif.json or a modified copy, and manifests under .plotplot/beds/ with matching check commands): two gates → one log with two runs that validates against the SARIF schema; a gate with one error-level result → exit 2; a gate that exits 1 without SARIF → exit 3 with the bed named on stderr; a missing binary → exit 3; both a block and a could-not-run → 3; --strict appends the flag only to the bed with check_strict; --format table prints the rows; the merged log's tool names match each bed's own SARIF tool name; validate refuses a log with a missing `runs` key.

Do NOT change existing public shapes; add only. Do NOT edit the loop file. For the fit subcommand `check` in scripts/fit/stem.sh: weeder has no release yet, so the fixture repository takes the weeder binary found on the build machine (`command -v weeder`; the check fails, saying so, when there is none), places it under the fixture's .plotplot/bin/ with a .sha256 companion and a lock entry that matches it, and the script prints one line saying the judge was pre-placed from the build machine because weeder has no release. Then run `plotplot check` in the fixture on a change that weeder blocks (a deleted test file) and assert exit 2 with one SARIF log carrying weeder's run, on a clean tree assert exit 0 with a log that validates against contracts/vendor/sarif-schema-2.1.0.json (contracts/test/lib/validate-sarif.mjs does that), and with the judge removed assert exit 3. Run the verifier for the check line and paste its output: `tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 3 --force --runner "bash {evidence} check"`.
"""


INIT = PREAMBLE + """
## Your slice: `plotplot init`, `src/init.rs`, `src/install.rs`, and the fit `init` check

This worktree carries the integrated stem with the lock and check faces. Read src/cli.rs, src/lock.rs, src/bundle/mod.rs, src/plant/, src/doctor.rs and scripts/fit/stem.sh before writing anything; init composes them and adds nothing they already do.

`plotplot init [--harness claude,gemini,codex] [--beds a,b] [--profile minimal|full] [--lock <path>]`, per docs/plans/stem.md §5, idempotent, never touching a user-scope file and never installing anything globally: (1) detect harness CLIs on PATH (claude, gemini, codex) and any project configs present (.claude/settings.json, .gemini/settings.json, .codex/); --harness limits the set; (2) the profile: minimal (default, jahala/plotplot issue 17) plants weeder with its hooks, the garden block, the lock, the git hooks and the PR check workflow; full plants every judge in the lock; --beds overrides the bed list; (3) garden.lock: when absent, copy the template given by --lock (required when absent; the umbrella's template path is named in your final message as an open question if none is agreed) filtered to the chosen beds; when present, verify it (lock::verify with fetch::Https), refusing to continue on a Mismatch (exit 3); (4) bundle build for each chosen harness; (5) install each bundle at project scope, in src/install.rs, one function per harness, each returning what it wrote: Claude writes `.plotplot/marketplace/.claude-plugin/marketplace.json` listing the bundle as a local plugin and runs `claude plugin marketplace add .plotplot/marketplace` then `claude plugin install plotplot@plotplot-local --scope project`, reporting the exact command and its stderr on failure; Gemini has no project scope, so init writes the hooks and mcpServers into the project's .gemini/settings.json (merging into an existing file, touching only the `hooks` and `mcpServers` keys, byte-identical on a second run) and says so; Codex writes the project's .codex/hooks.json from the bundle's hooks.json and the MCP servers into .codex/config.toml under [mcp_servers], and says that hooks load only once the project is trusted; (6) .githooks/* from plant::githooks with mode 0o755, core.hooksPath and the receipts refspecs from plant::gitconfig; (7) the garden block in AGENTS.md from plant::garden_block, and the planted repository's own garden.json beside garden.lock, kind ["repository"], name from the directory or the git remote, written only when absent and validated against contracts/manifest.schema.json's repository case (contracts v1.3.0; fixture contracts/fixtures/manifest/repository.garden.json). manifest::to_bed must refuse a repository manifest with Error::Manifest and load_beds must skip one under .plotplot/beds/, add both with tests; (8) the PR check workflow .github/workflows/plotplot-check.yml running `plotplot check --strict` on pull_request with hosted runners only (no self-hosted runner ever on a public bed, jahala/plotplot issue 7); (9) print one line per file or setting changed, and `nothing to do` when a second run changes nothing. Add `Error::Install { harness, problem }` additively.

Then add the fit subcommand `init` to scripts/fit/stem.sh: build a fixture repository, run `plotplot init --harness gemini,codex --lock <a lock built in the fixture from a locally served artifact only if lock::verify can fetch it; otherwise pre-place the judge and its .sha256 so verify passes without a fetch>`, assert AGENTS.md carries the block, .gemini/settings.json and .codex/hooks.json exist with the dispatcher entries, core.hooksPath is .githooks, the four hooks are executable, the receipts refspecs are set, and that a second run prints `nothing to do` and leaves every file byte-identical (hash the tree before and after). Claude's install is asserted the same way when `claude` is on PATH, against a temp HOME.

Tests you write first (tests/init.rs with temp roots, a temp HOME, and the fixture judge pre-placed): the numbered steps above each have one assertion; idempotence is asserted by a tree hash; --harness gemini writes only Gemini's config; a Mismatch in the lock aborts before any bundle is written; a missing --lock with no garden.lock is a usage error naming the flag.

Then run the verifier for the init check and paste its output into your final message:

    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 1

Do NOT change existing public shapes; add only. Do NOT edit the loop file.
"""


HARDEN = PREAMBLE + """
## Your slice: three follow-ups from the umbrella's review of the first wave (pull request 20)

The whole first wave is in this worktree. Read src/deny.rs, src/hook.rs, src/friction.rs and src/harness.rs before writing anything.

(1) The deny list fails open when a writing tool's payload lacks its input or its cwd: decide() answers Allow for a Write/Edit/MultiEdit/write_file/replace/apply_patch/tilth_write call whose tool_input is absent or names no path, and guarded_target() skips an absolute target when the payload has no cwd. A hard limit fails closed. Test first, then: a writing tool call whose target cannot be determined (no tool_input, or an input with no path in it) is Deny with the reason `plotplot: cannot judge the target of a write: the payload names no path; a boundary that cannot see the target refuses it`; a writing tool call with an absolute target and no cwd is Deny with `plotplot: cannot judge the target of a write: the payload has no working directory to judge <path> against`; a shell command with a redirection whose target cannot be judged the same way is Deny likewise. A read, a search, a shell command without a redirection, and every non-before-tool event stay Allow. The dispatcher already records a deny-list refusal as a `tool.denied` friction record with `plotplot.rule`; give these refusals their own rule name (`write-target-unjudged`) and assert in tests/hook.rs that `plotplot hook claude PreToolUse` on a Write payload without tool_input exits 2 with the Claude deny JSON and appends one `tool.denied` line carrying that rule. Add fixture payloads for the two cases under tests/fixtures/payloads/claude/.

(2) Swallowed failures in src/friction.rs and src/harness.rs are acceptable only where the drop is counted, never silent. Run `weeder check --strict --format table` on the worktree and read every S2 finding in those two files, then go through every `let _ =`, `.ok()`, `unwrap_or_default()` and `if let Err(_)` in src/friction.rs, src/harness.rs and src/hook.rs's trail(): each one either propagates the error to a caller that can act, or is counted where the profile lets it be seen (a line on stderr naming what was dropped and why, from the dispatcher's trail; never a silent default). Write a test for each drop that now surfaces. Do not weaken the rule that the friction and receipt faces never change the dispatcher's exit code.

(3) Nothing else: no new faces, no changes to public shapes beyond what (1) and (2) need. State in your final message which S2 findings remain in those files, if any, and why each is a counted drop.
"""

RECEIPT = PREAMBLE + """
## Your slice: `plotplot receipt seal`, `verify` and `show`, the v0 receipt in git notes (`src/receipt.rs`, the git plumbing)

The stem is whole in this worktree: read src/receipt.rs (the draft face), src/friction.rs, src/plant/gitconfig.rs, src/plant/githooks.rs (post-commit already calls `.plotplot/bin/plotplot receipt seal`), src/cli.rs and scripts/fit/stem.sh before writing anything. The plan is docs/plans/receipts.md §1 to §5 and §7; the loop is docs/tend2/receipts.tend2.html (read-only), whose seal and verify checks name `scripts/fit/receipts.sh seal` and `scripts/fit/receipts.sh verify` as evidence. The predicate schema in contracts has not landed; write the statement to the plan's §3 shape exactly, keys in that order, and say so in your final message.

`plotplot receipt seal [--commit <rev>]` (default HEAD): builds an in-toto Statement v1 (`_type` `https://in-toto.io/Statement/v1`; `subject` with the repository's canonical URL from `remote.origin.url` normalised to `github.com/<owner>/<repo>` when it is a GitHub URL and otherwise the URL as given, and a digest set of `gitCommit` and `gitTree` from `git rev-parse <rev>` and `<rev>^{tree}`; `predicateType` `https://plotplot.ai/receipt/v1`; the predicate per §3: producer plotplot with crate::VERSION, createdAt from the system clock in UTC ending in Z, harness and models and principal and sessions from the drafts under .plotplot/receipts/drafts/ (every draft present is folded in; with none the fields are null and the sessions list empty), conductor null, changed from `git diff-tree --no-commit-id --name-only -r <rev>`, verification with weeder's SARIF digest and counts only when `.plotplot/bin/weeder` exists and `weeder check --format sarif --base <rev>^` runs (else null, never a guess), tend2 stamps as the `@<id>` tokens the diff adds to docs/tend2/*.tend2.html lines that also gain `[x]`, pleach null, commands from the drafts' tool counts as the plan spells, cost from the drafts, friction summary as the sha256 over the drafts' friction summary digests in order). The note carries the statement JSON followed by one line `sha256: <hex of the statement bytes>`, attached with `git notes --ref refs/notes/plotplot/receipts add -f -F <file> <commit>`; a second seal of the same commit whose statement bytes equal the note's changes nothing and says `unchanged`; a second seal that would differ replaces the note and says so. Drafts folded into a sealed receipt are moved under .plotplot/receipts/sealed/<commit>/ so they are not folded twice.

`plotplot receipt verify (<rev> | --range <a>..<b>) [--require-signed]`: for each commit, reads the note, re-derives gitCommit and gitTree from git, checks both against the subject, recomputes the statement's sha256 against the trailing line, and prints one line per commit (`<short sha> receipt ok` or the reason); exit 0 when every commit in the range carries a valid receipt, 3 when any is missing or invalid, 1 on a git failure; `--require-signed` exits 3 with `unsigned (v0)` on every v0 note, since signing is not this slice. `plotplot receipt show <rev>` prints the predicate pretty-printed.

Add `Error::Receipt { commit: String, problem: String }` additively. Add `scripts/fit/receipts.sh` with subcommands `seal` and `verify` composing scripts/fit/lib.sh and the helpers in scripts/fit/stem.sh where they help (source it or copy the two you need; do not fork the whole file): `seal` builds a fixture repository with two commits and a draft under .plotplot/receipts/drafts/, seals HEAD, reads the note back with `git notes --ref refs/notes/plotplot/receipts show HEAD`, recomputes `git rev-parse HEAD` and `HEAD^{tree}` independently in the script and asserts they equal the subject's digests, asserts the trailing sha256 equals `shasum -a 256` of the statement bytes, seals again and asserts the note is byte-identical and the stem said `unchanged`; `verify` seals one of two commits, asserts `verify --range` over both exits 3 naming the unsealed commit, seals the other, asserts exit 0, then runs `plotplot init` on the fixture (with a pre-placed judge as scripts/fit/stem.sh's init check does; read it), clones the fixture with `git clone` and asserts the clone's `git fetch origin` brings `refs/notes/plotplot/receipts` because init set the fetch refspec.

Tests you write first (unit beside the code with a temp git repository; tests/receipt.rs with assert_cmd): the statement's keys in §3's order; the subject digests equal git's; the trailing sha256; seal twice is unchanged; verify passes, verify on a commit without a note exits 3, verify on a tampered note (one byte of the predicate changed) exits 3 naming the digest mismatch; `--range` with a gap; show prints the predicate; `--require-signed` refuses a v0 note; the canonical URL normalisation for https and ssh GitHub remotes and a non-GitHub remote left as given. Then run the verifier for the two receipts checks and paste its output: `tend2 verify docs/tend2/receipts.tend2.html --repo-root . --check 3 --force --runner "bash {evidence} seal"` and the same with `--check 4` and `verify`; if the numbering differs, read the file and use the right ones; do not edit either loop file.

Do NOT change existing public shapes; add only. Do NOT implement `sign`.
"""

DOCTOR_LIVE = PREAMBLE + """
## Your slice: `plotplot doctor --live` through umbel, Claude Code proven, the others reported honestly

The stem is whole in this worktree: read src/doctor.rs, src/hook.rs, src/friction.rs, src/install.rs, src/cli.rs and scripts/fit/stem.sh (its init and hook-faces checks) before writing anything. The plan is docs/plans/stem.md §6 (live mode). umbel is on PATH (`umbel --help`, `umbel spawn --help`; its MCP help says: claude workers take hook config inline through `--settings`, Codex through `<cwd>/.codex/hooks.json`, Gemini through `<cwd>/.gemini/settings.json`; `umbel wait` returns 126 when a worker is blocked on input and 125 when it died). On this machine tonight: Claude Code starts a session; Codex sessions fail at their prompt with a 404 from the provider on the account's default model (a bare `codex exec` answers, a real session does not); Gemini is unauthenticated. `doctor --live` must prove Claude Code and report the other two as unavailable with the reason it observed, never as a failure of the stem and never by skipping silently.

`plotplot doctor --live [--harness claude,gemini,codex] [--timeout <s>]`, after the static findings: for each chosen harness that is planted here (its bundle installed at project scope, as `doctor`'s static findings already know), spawn one real session through umbel in the repository root (`umbel spawn --provider <harness> --cwd <root>` with umbel's unattended flag, then `umbel send` with a scripted prompt, then `umbel wait`, then `umbel kill`; read `umbel spawn --help` for the exact flags and name the session `plotplot-doctor-<harness>-<pid>` so a conductor can tell whose it is), where the prompt asks the worker to run one shell command that the deny list refuses (`git commit --no-verify -m probe`, which never commits) and then to stop; the dispatcher's own hooks are what fire. Proof comes from files the dispatcher writes, never from the worker's words: the friction journal under .plotplot/friction/ gains, for the session id the harness reported, a `tool.denied` record with `plotplot.rule` `deny.skipped-verification` (the before-tool hook fired and refused) and a `session.ended` record (the session-end hook fired), and the receipt draft for that session exists (the draft face fired). One finding per event class proved: before-tool, session-end, receipt draft, each `ok` with the record that proved it, or `fail` with what was missing. A harness whose session cannot start or whose worker is blocked at a prompt within the timeout (umbel's 126 or 125, a 404 in the pane) is reported as `unavailable: <what umbel or the pane said>`, and the doctor says so in its table; the exit code is 3 only when a harness that did start failed a proof. The Codex trust finding stays as it is. Implement the umbel driving as a small seam (`src/doctor/live.rs`, or a `Driver` trait in src/doctor.rs with the real umbel implementation and a test double that plays back recorded umbel outputs; the double is for the parsing and the verdict logic, never for the proof itself).

Then add the fit subcommand `doctor-live` to scripts/fit/stem.sh: plant the init fixture (with the pre-placed judge and Claude installed as the init check does), run `plotplot doctor --live --harness claude,gemini,codex` with a temp HOME that still lets Claude Code authenticate (copy the build machine's `~/.claude/.credentials.json` and `~/.claude.json` into the temp home when they exist; say so on stdout), and assert from the doctor's output that Claude's three proofs are ok, that Gemini and Codex are reported unavailable with a reason, and that the exit code is 0; then assert directly from .plotplot/friction/ that the `tool.denied` and `session.ended` records exist for Claude's session. Print the session ids and the records. Then run the verifier for the loop's doctor-live line and paste its output: `tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 9 --force --runner "bash {evidence} doctor-live"` (check 9 is the `doctor --live` line, counting the `- [ ]` lines of the Tests section from 1; confirm by reading the file). Do not edit the loop file. Check 2 (Claude Code and Gemini with real binaries) cannot be closed on this machine; do not try.

Do NOT change existing public shapes; add only. Read the rule in docs/building-the-garden.md §2: kill only the sessions this doctor spawned, by their name.
"""


SMOKE_INTEGRATE = f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh bundles && bash scripts/fit/stem.sh hook-faces'"

AUDIT_INTEGRATE = (
    "bash -lc 'cargo test 2>&1 | tail -20 && bash scripts/fit/stem.sh hook-faces && bash scripts/fit/stem.sh bundles "
    "&& git diff --stat HEAD~1 -- docs/tend2/stem.tend2.html'"
)

WORKER = {"provider": "claude", "model": "claude-opus-5"}


AGENTS_BLOCK = PREAMBLE + """
## Your slice: `plotplot init` leaves tend2's block in AGENTS.md byte for byte (jahala/plotplot issue 28)

The stem is built and planted in this worktree; read src/plant/garden_block.rs, src/init.rs (write_garden_block), tests/plant.rs and scripts/fit/stem.sh (check_init and plant_init_fixture) before writing anything. The rule, ruled by the umbrella agent on 2026-09-10 and recorded on the contracts loop: tend2's init writes its own contract block into AGENTS.md between `<!-- tend2:begin -->` and `<!-- tend2:end -->`, beside the stem's garden block between `<!-- plotplot:begin -->` and `<!-- plotplot:end -->`; neither restates the other, and each init preserves the other's block byte for byte. This repository's own AGENTS.md carries tend2's block at lines 1 to 10 and the stem's at 12 to 22, and is the reference for what the two blocks look like side by side.

What to build, tests first:

1. A fixture `tests/fixtures/AGENTS.tend2.md`: tend2's block copied byte for byte from this repository's AGENTS.md (the ten lines from `<!-- tend2:begin -->` through `<!-- tend2:end -->`, with the trailing newline). Never retype it; copy it with a command (`sed -n '/tend2:begin/,/tend2:end/p' AGENTS.md`), and have a test assert the fixture equals that region of the live AGENTS.md so the two cannot drift.

2. Tests in tests/plant.rs, written first and watched failing where they can fail (some pass already because replace_in copies through what is outside its markers; a test that passes at once is still the proof, say so in your final message):
   - planting the garden block into a file that is tend2's block alone leaves tend2's bytes exactly where they were, the garden block follows after one blank line, and the bytes before the plotplot begin marker equal the fixture exactly;
   - planting into a file that carries tend2's block, then prose, then an older garden block, then more prose, replaces only the marked region and every byte outside it (tend2's block included) is identical before and after, compared as byte slices and not by length;
   - planting into a file where tend2's block sits below the garden block (garden block first, blank line, tend2's block) leaves tend2's block byte for byte and in place;
   - re-planting a different garden block (another season) in each of those layouts changes only the marked region;
   - a fifth test: the fixture equals the live AGENTS.md's tend2 region (see 1).

3. The fit evidence, scripts/fit/stem.sh, check `init`: plant_init_fixture writes the fixture repository's AGENTS.md as tend2's block (read from tests/fixtures/AGENTS.tend2.md) followed by one blank line and one line of prose; after the first init and after the second, assert with `sed -n '/<!-- tend2:begin -->/,/<!-- tend2:end -->/p' "$repo/AGENTS.md" | cmp - tests/fixtures/AGENTS.tend2.md` that tend2's block is byte for byte the fixture, and that it still starts at line 1. Add a second fixture layout inside the same check, in its own scratch repository: the garden block already present at the top with tend2's block under it, `plotplot init` run once, and tend2's block byte for byte and still after the garden block. Each assertion prints a `note` line naming what it proved. Keep every existing assertion of check_init.

4. Do not change garden_block::replace_in's behaviour unless a test above fails; if one does, fix the smallest thing and say which. Do not write tend2's block from the stem anywhere (the stem's renderer never carries another bed's text): the fixture is a test input, never a rendered output.

The gate for this slice is the crate gate followed by `bash scripts/fit/stem.sh init`, which needs `claude`, `git` and `jq` on PATH; run it exactly as the accept smoke names it.

Do NOT create or edit: src/cli.rs, src/main.rs, src/doctor.rs, src/doctor/, src/hook.rs, src/bundle/, src/install.rs, src/lock.rs, docs/tend2/. Do NOT edit this repository's own AGENTS.md.
"""

PLATFORM = PREAMBLE + """
## Your slice: the platform half of `init` and `doctor` (jahala/plotplot issue 7, the part ruled urgent on 2026-09-10)

The stem is built and planted in this worktree. Read first: src/init.rs (run, plant, workflow, origin_url, write_if_changed), src/doctor.rs (CHECKS, Finding, run, run_static, render), src/doctor/live.rs (the Driver seam with its `Ran` value and the recorded double in its tests: copy that pattern), src/cli.rs (InitArgs, DoctorArgs, run), src/plant/garden_block.rs (a marked region inside a file somebody else also writes), scripts/fit/stem.sh (check_init, plant_init_fixture, the scratch and note helpers), and jahala/plotplot issue 7 as quoted here: the repository holds the truth in files and the platform enforces it on humans; git hooks bind a human until `--no-verify`, only a ruleset binds past that.

What lands in this slice, and nothing more of issue 7 (the weekly clock, the SARIF upload, the schedule check and the push ruleset wait):

### 1. `plotplot init --github`

A new flag on InitArgs. After everything plain `init` plants, `--github` does two more things, in this order, against the repository origin names:

(a) Writes the stem's region of `.github/CODEOWNERS`: the lines between `# plotplot:begin` and `# plotplot:end`, every other byte of the file preserved (the same rule as the garden block in AGENTS.md; a file with no region gets the region appended after one blank line; a malformed marker pair is refused with Error::Agents-style precision, add a variant if you need one). The region names the owner, `@<owner>` where owner is the first path segment of the origin url (`https://github.com/<owner>/<repo>.git`, `git@github.com:<owner>/<repo>.git`, with or without `.git`), for exactly these guardrail paths, one per line, in this order: `/.githooks/`, `/garden.lock`, `/garden.json`, `/AGENTS.md`, `/CLAUDE.md`, `/weeder.toml`, `/.claude/settings.json`, `/.gemini/settings.json`, `/.codex/hooks.json`, `/.github/workflows/plotplot-check.yml`, `/.github/CODEOWNERS`, `/docs/tend2/`. These are weeder's C1 guardrail files (harness settings, git hooks, weeder.toml, the hard-limits section's files), the stem's own files, the pull request gate and the map. Written through write_if_changed, so a second run changes nothing and `.github/CODEOWNERS` appears in the changed lines only when its bytes moved.

(b) Applies one ruleset through `gh api`, named `plotplot: the default branch`: target `branch`, enforcement `active`, conditions `{"ref_name": {"include": ["~DEFAULT_BRANCH"], "exclude": []}}`, rules exactly `[{"type": "deletion"}, {"type": "non_fast_forward"}, {"type": "required_status_checks", "parameters": {"strict_required_status_checks_policy": false, "required_status_checks": [{"context": "garden"}]}}]`. No bypass actors. Linear history is deliberately absent: the law merges every pull request with a merge commit, and GitHub's linear-history rule refuses those. The required check's context is `garden`, a contract ruled on 2026-09-10: the ruleset names the check, and a renamed job silently unprotects the branch, so init::workflow's job is renamed `name: garden` while its step still runs `plotplot check --strict`, a command that may change without touching the ruleset. Make `garden` one constant that workflow(), the ruleset body and doctor's read-back all read; update the existing workflow test and check_init's assertions to the new job name (the file must still run `plotplot check --strict` as a step, and check_init keeps asserting that).
   Idempotent: list the repository's rulesets (`GET repos/<owner>/<repo>/rulesets`), and when one carries this name read it (`GET repos/<owner>/<repo>/rulesets/<id>`) and compare target, enforcement, conditions and rules to the desired ones; unchanged means no call and no changed line; different means `PUT repos/<owner>/<repo>/rulesets/<id>` with the desired body; absent means `POST repos/<owner>/<repo>/rulesets`. A changed line reads `ruleset "plotplot: the default branch" applied on <owner>/<repo>`.
   Anything gh could not apply is printed verbatim: gh's stderr and stdout, as gh wrote them, each line on stderr after one line naming the call (`gh api -X POST repos/<owner>/<repo>/rulesets could not be applied:`), and init exits FAILED (1) after printing everything else it changed; the files planted before stay planted. Without gh on PATH, or with an origin that is not on github.com, `--github` refuses before writing anything of its own with one line saying which of the two it is, exit USAGE (2), and the plain planting that ran before it stays.

The seam: a trait in a new `src/platform.rs` mirroring live.rs's Driver, e.g. `pub trait Gh { fn api(&self, method: &str, path: &str, body: Option<&str>) -> Ran; }` with `Ran {code, stdout, stderr}` (move or share live's Ran rather than duplicating it), a real `GhCli` that runs `gh api -X <METHOD> <path> --input -` with the body on stdin (no body: `gh api <path>`), and in tests a recorded double that answers canned JSON per (method, path) and records every call in order. The desired ruleset body, the comparison, the CODEOWNERS region and the read-back parsing are pure functions over strings and serde_json Values; the double never replaces them. Nothing in the library reads the environment: the search path for gh and the origin url are read in init the way they already are (search_path, origin_url) and passed in.

Never run `plotplot init --github`, any `gh api -X POST|PUT|PATCH|DELETE`, or any writing gh call against a real repository from this worktree: this worktree's origin is jahala/plotplot itself and a ruleset applied from here lands on the real repository. The fit check uses a recording stub `gh`; the real proof on jahala/plotplot is the conductor's, after landing. Read-only `gh api` calls are allowed if you need to look at a shape.

### 2. `plotplot doctor --platform`

A new flag on DoctorArgs, exclusive with nothing (it may combine with --live; run static, then platform, then live in that order when both are given). After the static table, one blank line, then a second table in the live table's shape (check, verdict, detail) with three lines, read back with read-only calls only: `GET repos/<owner>/<repo>` for `default_branch`, then `GET repos/<owner>/<repo>/rules/branches/<default_branch>`, which answers the rules in force on that branch as an array of objects with `type` and, for status checks, `parameters.required_status_checks[].context`:
   - `required check`: ok when a `required_status_checks` rule lists the context `garden`, detail naming the branch and the ruleset id it came from (`ruleset_id`); fail otherwise, detail saying what the branch requires instead (the contexts found, or `none`);
   - `force push`: ok when a `non_fast_forward` rule is in force, fail otherwise;
   - `deletion`: ok when a `deletion` rule is in force, fail otherwise.
   Without gh on PATH, or with no github.com origin, the platform table is one line, `platform  unavailable  <reason>`, and the exit is 3: the mode is skipped, never guessed (issue 7). A gh call that fails is `unavailable` with gh's first line as the detail. Exit 0 only when the static table and all three platform lines are ok; 3 otherwise; 1 on the errors doctor already answers 1 to.

### 3. Tests, written first

Unit tests beside the code with the recorded double: the desired body is byte-stable JSON with the three rules in that order and no linear-history rule; an absent ruleset is POSTed; a present and equal ruleset makes no writing call; a present and different ruleset is PUT to its id; a failing call prints gh's own words verbatim and answers FAILED with the changed lines before it; the CODEOWNERS region is idempotent and leaves foreign lines (a comment above, a `*.md @someone` line below) byte for byte; the owner is parsed from https and ssh origins with and without `.git`; a non-github origin refuses with USAGE; the read-back turns the rules array into three findings, each verdict proved both ways; the mode without gh is one unavailable line and exit 3. Integration in tests/ where the binary's face matters (the flag exists, the tables print in order).

### 4. The fit evidence: `scripts/fit/stem.sh platform`

A new check in scripts/fit/stem.sh, listed in the header comment and the case at the bottom, in the style of check_init: a scratch repository planted with `plotplot init --lock <template> --harness gemini,codex` whose origin is `https://github.com/example-owner/example-repo.git` (a url only, never fetched), and a stub `gh` executable written into a scratch bin directory put first on PATH. The stub records each call (argv on one line, stdin after it) to a journal file, and answers by path: `repos/example-owner/example-repo` with `{"default_branch":"main"}`; `repos/example-owner/example-repo/rulesets` GET with `[]` before a POST and `[{"id":41,"name":"plotplot: the default branch"}]` after one (keep a state file), POST with the created ruleset (id 41) echoing the body's rules; `repos/example-owner/example-repo/rulesets/41` GET with the body it was given; `repos/example-owner/example-repo/rules/branches/main` with the three rules in force as GitHub shapes them (`[{"type":"deletion","ruleset_id":41,...},{"type":"non_fast_forward",...},{"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"garden"}],"strict_required_status_checks_policy":false},...}]`). Then assert, each with a `note`: `plotplot init --github` exits 0 and names `.github/CODEOWNERS` and the ruleset in its changed lines; the journal shows exactly one POST whose body (parsed with jq) carries the three rule types in order, no `required_linear_history`, the context `garden`, and `~DEFAULT_BRANCH`; `.github/CODEOWNERS` carries the twelve guardrail lines naming `@example-owner` inside the markers and a pre-existing foreign line outside them untouched; a second `plotplot init --github` prints `nothing to do` and the journal gained no POST or PUT; `plotplot doctor --platform` exits 0 and its platform table reads ok on all three lines naming branch main and ruleset 41; with the stub answering a rules array lacking `non_fast_forward`, `doctor --platform` exits 3 and the force push line reads fail; with gh removed from PATH, `doctor --platform` exits 3 with the one unavailable line, and `init --github` exits 2 having planted nothing new. Finally, on this repository itself (the worktree, whose origin is jahala/plotplot) run only `"$STEM" doctor --platform` and print its platform table with a note; do not assert its verdicts, because whether the real ruleset is applied yet is the conductor's act after this lands; assert only that it exits 0 or 3 and prints the three lines or the one unavailable line.

### 5. The brief

Append a `platform.rs` entry to §4 of docs/prompts/stem-build-2026-09.md naming the trait, the desired ruleset, the CODEOWNERS region and the read-back, in the style of the entries there. No em dashes anywhere you write.

The gate for this slice is the crate gate followed by `bash scripts/fit/stem.sh init && bash scripts/fit/stem.sh platform`; run it exactly as the accept smoke names it.

Do NOT create or edit: src/hook.rs, src/bundle/, src/lock.rs, src/receipt.rs, src/friction.rs, src/deny.rs, docs/tend2/, .github/ of this repository (the CODEOWNERS you render is proved on the fixture, never written here), this repository's AGENTS.md.
"""


PLATFORM_ORDER = PREAMBLE + """
## Your slice: the ordering rule for `init --github` (jahala/plotplot issue 33)

The platform half of issue 7 is landed and planted on this repository (pull request 31): read src/platform.rs whole, src/init.rs (github, report, GithubStop, workflow), src/doctor.rs (run_modes, Modes), src/cli.rs, and scripts/fit/stem.sh (check_platform, write_recording_gh, the rules-mode file) before writing anything. What happened on its first live application, and the rule that resolves it, ruled by the umbrella agent on 2026-09-11: the ruleset on master required a check named `garden` while the workflow on master still named its job "plotplot check --strict", so no pull request could satisfy the rule until the renaming pull request itself merged; a deadlock. The rule: `init --github` applies a ruleset requiring `garden` only when the default branch's workflow already carries a job of that name, and refuses with the reason otherwise; when the job name changes, the workflow lands on the default branch first and the ruleset moves after.

What to build, tests first:

### 1. The seam reads a file from the default branch

Extend the `Gh` trait with one read of raw file contents, `fn raw(&self, path: &str) -> Ran`, which `GhCli` runs as `gh api -H "Accept: application/vnd.github.raw+json" <path>` (GitHub answers the file's bytes, not JSON; no base64 and no new dependency). The recorded double answers it from its canned map like `api`. Give `Ran` nothing new.

### 2. The workflow on the default branch

In src/platform.rs: `GET repos/<owner>/<repo>` for `default_branch` (the read `doctor --platform` already makes; share it), then `raw("repos/<owner>/<repo>/contents/.github/workflows/plotplot-check.yml?ref=<branch>")` with the branch as `branch_segment` spells it. Then a pure function `gate_jobs(workflow: &str) -> Vec<Job>` reading the jobs the workflow declares without a YAML parser: after a line that is exactly `jobs:`, every line matching `^  <key>:$` (two spaces, a key of letters, digits, `-` and `_`, a colon, nothing else) starts a job; a line `^    name: <text>$` under it (four spaces, no dash) is that job's reported name, trimmed, with surrounding single or double quotes removed; a job with no `name:` reports its key, which is what GitHub does. A top-level line with no indent ends the jobs section. Say in the doc comment why this is enough: the file is one the stem itself writes, and a hand-edited workflow that hides its job name from this reader is a workflow the doctor reports as naming no `garden` job, which is the honest reading. Then `reports(jobs: &[Job], check: &str) -> bool`.

### 3. The refusal

`apply_ruleset` takes one more argument, the gate's jobs on the default branch, or is preceded by a step in init's `github` that reads them; choose the shape that keeps `apply_ruleset` pure over what it was given. Before any listing, comparing, POST or PUT: when the default branch's workflow is absent (gh exits non-zero and its first line carries 404, or the answer is empty), refuse; when its jobs report no `garden`, refuse. The refusal is a new `Unapplied` variant, printed by `unapplied_text` as one line naming the branch and what it found, then the fix, for example: `master's .github/workflows/plotplot-check.yml names no job "garden" (jobs: check, reported as "plotplot check --strict"); land the workflow on master first, then run plotplot init --github again` and, when a ruleset called `plotplot: the default branch` already exists and requires `garden`, one more line: `ruleset <id> already requires "garden", so master takes no merge until that workflow lands`. Exit FAILED (1) as an unapplied ruleset does; the CODEOWNERS region written before it stays written and is named in the changed lines. Order inside `github`: CODEOWNERS, then the gate read, then the ruleset.

### 4. The doctor's fourth line

`doctor --platform` gains a fourth line after `deletion`, `gate job`: ok when the default branch's workflow reports `garden` (detail: `master's plotplot-check.yml reports garden`), fail when it reports other jobs (detail naming them as above) or when the file is absent on that branch (detail: `master has no .github/workflows/plotplot-check.yml`). READ_BACK grows to four; the read of the file is one more read-only call; a failed read that is not a 404 is `unavailable` for that line only, the other three still answered. Update every test and the fit assertions that count three lines.

### 5. Tests, written first

Unit tests beside the code: `gate_jobs` on the workflow `init::workflow(true)` renders finds one job `garden` reporting `garden`; on the same text with the job's `name:` line changed to `plotplot check --strict` finds one job `garden` reporting that; on a workflow with two jobs, one unnamed, reports the key for the unnamed one; on a file with no `jobs:` finds none; quotes around the name are removed; a step's `- name:` line is never a job. With the recorded double: a workflow reporting the old name makes no POST and no PUT, the refusal names the branch, the file, the jobs and the fix, and `github` still wrote CODEOWNERS and answers FAILED; a 404 on the file refuses naming the missing file; a workflow reporting `garden` proceeds to the POST as before; an existing ruleset requiring `garden` adds the deadlock line. The doctor's fourth line proved ok, fail with jobs, fail without the file, and unavailable on a non-404 read failure. Integration in tests/doctor.rs where the table's shape is asserted (four platform lines now).

### 6. The fit evidence, `scripts/fit/stem.sh platform`

The recording gh answers `GET repos/example-owner/example-repo/contents/.github/workflows/plotplot-check.yml?ref=main` (with the raw Accept header; match on the path) from a `gate-mode` file beside `rules-mode`: `garden` (the default; answer the workflow `plotplot init` itself wrote into the fixture repository, read from `$repo/.github/workflows/plotplot-check.yml`), `old` (the same text with the job's name line set to `plotplot check --strict`), `absent` (exit 1 with `gh: Not Found (HTTP 404)` on stderr). Keep every existing assertion, extending the three-line table assertions to four with `gate job` ok. Add, each with a `note`: with `gate-mode` old, in a fresh scratch repository planted the same way, `init --github` exits 1, the journal holds no POST and no PUT, stderr names `main`, the file, `check` reported as `plotplot check --strict`, and the fix, and `.github/CODEOWNERS` was still written; `doctor --platform` there exits 3 with `gate job` fail and the other three lines answered; with `gate-mode` absent, `init --github` exits 1 naming the missing file and makes no writing call. The self read of this repository at the end now asserts nothing more than before but prints four lines.

### 7. The brief

Extend the `platform.rs` entry in §4 of docs/prompts/stem-build-2026-09.md with the gate read, the refusal and the fourth line. No em dashes anywhere you write.

The gate for this slice is the crate gate followed by `bash scripts/fit/stem.sh platform`; run it exactly as the accept smoke names it. Never run `plotplot init --github` or any writing gh call against a real repository from this worktree; the fit uses the recording gh, and the live proof on jahala/plotplot is the conductor's after landing.

Do NOT create or edit: src/hook.rs, src/bundle/, src/lock.rs, src/receipt.rs, src/friction.rs, src/deny.rs, docs/tend2/, .github/ of this repository, this repository's AGENTS.md.
"""


def node(id_, prompt, needs=(), smoke=GATES, audit=None, timeout_ms=3_600_000, setup="npm ci"):
    accept = {"smoke": smoke}
    if audit:
        accept["audit"] = audit
    return {
        "id": id_,
        "worker": dict(WORKER),
        "work": {"prompt": prompt},
        "setup": setup,
        "needs": list(needs),
        "accept": accept,
        "policy": {"maxAttempts": 2, "timeoutMs": timeout_ms, "onDead": "resume", "reauditWhen": ["compacted"]},
    }


plan = {
    "goal": (
        "Build the schema-independent half of the stem, the plotplot binary at this repository's root: "
        "the crate and shared model with `version`, the three vendor bundles, the stem's own hook-time faces, "
        "the pieces init writes, the hook dispatcher, doctor's static mode, and the fit evidence for the loop's "
        "bundles and hook-faces checks. Brief: docs/prompts/stem-build-2026-09.md. Loop: docs/tend2/stem.tend2.html."
    ),
    "source": "docs/dogfood/stem/build.plan.json",
    "maxConcurrency": 2,
    "nodes": [
        node("stem.bundle", BUNDLE),
        node("stem.faces", FACES),
        node("stem.plant", PLANT),
        node("stem.hook", HOOK, needs=["stem.faces"]),
        node("stem.doctor", DOCTOR, needs=["stem.bundle", "stem.plant"]),
        node(
            "stem.integrate",
            INTEGRATE,
            needs=["stem.hook", "stem.doctor"],
            smoke=SMOKE_INTEGRATE,
            # codex answers 404 on this machine tonight (its ChatGPT account has no usable model), so the
            # audit is cast to the bring-any-model lane on a different vendor's model.
            audit={"command": AUDIT_INTEGRATE, "provider": "opencode", "model": "deepseek/deepseek-v4-pro"},
            timeout_ms=5_400_000,
        ),
    ],
}

# `--reaudit`: relay-only command nodes that settle the integration node's audit on the
# repository HEAD, where the integration work landed by hand after the first auditor's
# relay came back unparseable. pleach's auditor is a relay: the audit command itself must
# print the fenced tend-audit-result block, which is what `tend2 verify --audit-egress`
# does, one check per invocation, so one node per stamped check. A command node has no
# builder and is exempt from the hygiene gate; nothing is rebuilt, only the gates settle.
VERIFY = "tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress"
RECEIPTS_VERIFY = "tend2 verify docs/tend2/receipts.tend2.html --repo-root . --force --audit-egress"
# Every stamped check, one relay-only node each; `--reaudit` re-audits the ones named after
# it (`--reaudit init check lock`), or all of them when none is named.
ALL_CHECKS = [("init", 1), ("check", 3), ("lock", 4), ("bundles", 5), ("hook-faces", 8), ("doctor-live", 9)]
# The receipts loop's checks the stem's fit script for receipts closes, relayed the same way.
RECEIPT_CHECKS = [("seal", 3), ("verify", 4)]
_wanted = [a for a in sys.argv[sys.argv.index("--reaudit") + 1:] if not a.startswith("--")] if "--reaudit" in sys.argv else []
REAUDIT_CHECKS = [(n, i) for n, i in ALL_CHECKS if not _wanted or n in _wanted]
REAUDIT_RECEIPTS = [(n, i) for n, i in RECEIPT_CHECKS if n in _wanted]
reaudit = {
    "goal": "Settle the integration node's audit for the landed stem stack (jahala/plotplot 20): tend2 verify with audit egress on the two stamped checks, relayed by an auditor on another provider, no builder.",
    "source": "docs/dogfood/stem/reaudit.plan.json",
    "maxConcurrency": 1,
    "nodes": [
        {
            # git refuses a ref ending in .lock, so the lock check's node is named after the judges.
            "id": f"stem.audit.{'judges' if name == 'lock' else name}",
            "worker": dict(WORKER),
            "work": {"command": "bash -lc 'cargo build --release 2>&1 | tail -1'"},
            # The fit script's SARIF validator and lock reader are the contracts' node
            # modules; a fresh worktree has none, and setup is where pleach provisions.
            "setup": "npm ci",
            "needs": [],
            "accept": {
                "smoke": GATES,
                "audit": {
                    "command": f"bash -lc '{VERIFY} --check {number} --runner \"bash {{evidence}} {name}\"'",
                    # codex sessions 404 at their prompt on this account (a bare exec answers,
                    # a real session does not), so the relay runs on the bring-any-model lane.
                    "provider": "opencode",
                    "model": "deepseek/deepseek-v4-pro",
                },
            },
            "policy": {"maxAttempts": 1, "timeoutMs": 1_800_000, "onDead": "resume", "reauditWhen": ["compacted"]},
        }
        for name, number in REAUDIT_CHECKS
    ]
    + [
        {
            "id": f"receipts.audit.{name}",
            "worker": dict(WORKER),
            "work": {"command": "bash -lc 'cargo build --release 2>&1 | tail -1'"},
            "setup": "npm ci",
            "needs": [],
            "accept": {
                "smoke": GATES,
                "audit": {
                    "command": f"bash -lc '{RECEIPTS_VERIFY} --check {number} --runner \"bash {{evidence}} {name}\"'",
                    "provider": "opencode",
                    "model": "deepseek/deepseek-v4-pro",
                },
            },
            "policy": {"maxAttempts": 1, "timeoutMs": 1_800_000, "onDead": "resume", "reauditWhen": ["compacted"]},
        }
        for name, number in REAUDIT_RECEIPTS
    ],
}


# `--second`: the second run, after pull request 20 merged. At most two nodes at a time
# (the shared session window): harden and lock in parallel, then check, then init.
second = {
    "goal": (
        "The stem's second run: the review's three follow-ups, lock verify against tilth v0.10.1's real "
        "release, check with one merged SARIF log, and init with the minimal profile and the repository's "
        "own garden.json. Brief: docs/prompts/stem-build-2026-09.md. Loop: docs/tend2/stem.tend2.html."
    ),
    "source": "docs/dogfood/stem/second.plan.json",
    "maxConcurrency": 2,
    "nodes": [
        node("stem.harden", HARDEN),
        node("stem.judges", LOCK, smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh lock'"),
        node("stem.check", CHECK, needs=["stem.harden", "stem.judges"], smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh lock && bash scripts/fit/stem.sh check'"),
        node(
            "stem.init",
            INIT,
            needs=["stem.check"],
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh lock && bash scripts/fit/stem.sh check && bash scripts/fit/stem.sh init && bash scripts/fit/stem.sh bundles && bash scripts/fit/stem.sh hook-faces'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress --check 1 --runner \"bash {evidence} init\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
            timeout_ms=5_400_000,
        ),
    ],
}

# `--third`: receipts v0 and doctor --live, two root nodes in parallel, each audited by a
# relay of the verifier's own egress on the check it closes.
third = {
    "goal": (
        "The stem's third run: receipt seal, verify and show as an unsigned in-toto statement in git notes, "
        "and doctor --live through umbel proving Claude Code and reporting Codex and Gemini honestly. "
        "Brief: docs/prompts/stem-build-2026-09.md."
    ),
    "source": "docs/dogfood/stem/third.plan.json",
    "maxConcurrency": 2,
    "nodes": [
        node(
            "stem.receipt",
            RECEIPT,
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/receipts.sh seal && bash scripts/fit/receipts.sh verify'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/receipts.tend2.html --repo-root . --force --audit-egress --check 3 --runner \"bash {evidence} seal\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
        ),
        node(
            "stem.doctor-live",
            DOCTOR_LIVE,
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh doctor-live'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress --check 9 --runner \"bash {evidence} doctor-live\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
            timeout_ms=5_400_000,
        ),
    ],
}

# A planted repository's git hooks call the judges under .plotplot/bin/, which is ignored, so a
# fresh pleach worktree has none and the pre-commit hook refuses every commit until
# `plotplot lock verify` has placed them: the fourth run's setup does that before anything else.
SETUP_PLANTED = "bash -lc 'npm ci && cargo build --release && ./target/release/plotplot lock verify'"

# `--fourth`: the umbrella agent's order of 2026-09-10 after the hold, one node at a time in
# this order: the AGENTS.md preservation proof (issue 28), then the platform half of issue 7
# that the day's incident made urgent. Each audited by a relay of the verifier's egress on
# the check it closes: init's check 1 re-stamped for 28, the new check 11 for the platform.
fourth = {
    "goal": (
        "The stem's fourth run: `plotplot init` proved to leave tend2's AGENTS.md block byte for byte "
        "(jahala/plotplot 28), then `init --github` applying the default-branch ruleset and the CODEOWNERS "
        "region through gh and `doctor --platform` reading them back (the urgent half of jahala/plotplot 7)."
    ),
    "source": "docs/dogfood/stem/fourth.plan.json",
    "maxConcurrency": 1,
    "nodes": [
        node(
            "stem.agents-block",
            AGENTS_BLOCK,
            setup=SETUP_PLANTED,
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh init'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress --check 1 --runner \"bash {evidence} init\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
        ),
        node(
            "stem.platform",
            PLATFORM,
            needs=["stem.agents-block"],
            setup=SETUP_PLANTED,
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh init && bash scripts/fit/stem.sh platform'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress --check 11 --runner \"bash {evidence} platform\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
            timeout_ms=5_400_000,
        ),
    ],
}

# `--fifth`: issue 33, the ordering rule the platform half's first live application deadlocked
# on, one node, audited on the loop's check 12.
fifth = {
    "goal": (
        "The stem's fifth run: `init --github` requires only a check the default branch's workflow already "
        "reports, refuses with the reason otherwise, and `doctor --platform` reports the gate's job "
        "(jahala/plotplot 33)."
    ),
    "source": "docs/dogfood/stem/fifth.plan.json",
    "maxConcurrency": 1,
    "nodes": [
        node(
            "stem.platform-order",
            PLATFORM_ORDER,
            setup=SETUP_PLANTED,
            smoke=f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh platform'",
            audit={
                "command": "bash -lc 'tend2 verify docs/tend2/stem.tend2.html --repo-root . --force --audit-egress --check 12 --runner \"bash {evidence} platform\"'",
                "provider": "opencode",
                "model": "deepseek/deepseek-v4-pro",
            },
            timeout_ms=5_400_000,
        ),
    ],
}

if "--fifth" in sys.argv:
    print(json.dumps(fifth, indent=2, ensure_ascii=False))
elif "--fourth" in sys.argv:
    print(json.dumps(fourth, indent=2, ensure_ascii=False))
elif "--third" in sys.argv:
    print(json.dumps(third, indent=2, ensure_ascii=False))
elif "--second" in sys.argv:
    print(json.dumps(second, indent=2, ensure_ascii=False))
elif "--reaudit" in sys.argv:
    print(json.dumps(reaudit, indent=2, ensure_ascii=False))
else:
    print(json.dumps(plan, indent=2, ensure_ascii=False))
