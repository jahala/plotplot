#!/usr/bin/env python3
"""Emit docs/plans/stem-build.plan.json, the pleach plan that builds the stem.

One node per slice of docs/prompts/stem-build-2026-09.md. Run from the repository root:

    python3 scripts/stem-plan.py > docs/plans/stem-build.plan.json && pleach validate docs/plans/stem-build.plan.json

The prompts are the only context a worker gets, so each names its files, its tests, what it
must not create, and the gate it must pass. Gate commands are wrapped in `bash -lc` because
pleach executes them as an argv array with no shell.
"""

import json

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
- render's block has the two markers, at most ten lines between them, names every bed with its version, the season, the three commands, and contains no "!" and no "—";
- replace_in on the repository's own AGENTS.md content (read it as a fixture string) appends the block and leaves the tend2 block byte-identical; a second application is a no-op; a changed block replaces only the marked region; a begin marker without an end is Error::Agents;
- githooks: each of the four renders, starts with #!/bin/sh, is different per hook, names every declaring bed's binary under .plotplot/bin/, and shellcheck-free in the sense that `sh -n` accepts it (run `sh -n` on the rendered text in the test);
- gitconfig: on a fresh temp repository with a remote named origin, read returns nothing for the three keys; apply(desired) then read returns all three; apply twice leaves single entries (assert `git config --get-all` counts); missing is empty afterwards; read on a directory that is not a repository is Error::Git.

Do NOT create or edit: src/cli.rs, src/main.rs, src/bundle/, src/deny.rs, src/friction.rs, src/receipt.rs, src/hook.rs, src/doctor.rs, scripts/. Do NOT change any existing public shape beyond adding the Error::Agents variant.
"""

HOOK = PREAMBLE + """
## Your slice: the dispatcher `src/hook.rs` and the CLI faces `hook`, `friction emit`, `receipt draft`

This worktree merges the crate (src/cli.rs with `version`; src/harness.rs, bed.rs, manifest.rs, lock.rs, layout.rs, error.rs) with the faces slice (src/deny.rs, src/friction.rs, src/receipt.rs). Read all of them before writing anything, and build only on their public shapes. If the merge left conflict markers anywhere, resolve them keeping every side's work before doing anything else.

Create src/hook.rs and tests/hook.rs, add `pub mod hook;` to src/lib.rs, and add three faces to src/cli.rs: `plotplot hook <harness> <event>`, `plotplot friction emit --harness <harness>`, `plotplot receipt draft --harness <harness>`; each reads the payload from stdin. Shapes: brief §4 "hook.rs" and "cli.rs". The handler for each face lives in its module (hook::run, friction::run, receipt::run taking the root, the parsed arguments, stdin as a &str, and the stdout/stderr writers) and cli.rs holds only the argument types and the match.

The contract with beds (brief §4 "hook.rs"; ruled by the umbrella 2026-09-08): `<bed> hook <harness>` reads the vendor's payload on stdin, writes the vendor's answer on stdout, exits 0 to allow, 2 to deny, 3 when it cannot judge. dispatch(root, harness, event, payload_json): (1) parse the payload; (2) run deny::decide first; a Deny renders the vendor answer on stdout, the reason on stderr, code 2, and no bed is called; (3) load the beds from .plotplot/beds via manifest::load_beds, keep those with a HookEntry for this harness and event whose matcher, when present, equals the payload's tool_name, and resolve each to .plotplot/bin/<binary>; a bed whose binary is absent is a BedResult with code 3 and a stderr naming the missing path; (4) fan_out in parallel, one thread per bed, each writing the payload to the child's stdin and collecting code, stdout, stderr, with the timeout (a child still running at the timeout is killed and reported as code 3); (5) merge per the rule: any 2 → 2, the first denial's stdout on stdout and its stderr first on stderr, the other denials' stderr after; else any 3 → 3 with each reason on stderr; else 0 with empty stdout; (6) friction::emit with the system clock, always, after the merge, never changing the code, logging an error to stderr and continuing; (7) on the harness's session-end event, receipt::draft the same way. The timeout for beds is 4 s on a stop or session-end event and 8 s otherwise. Exit code 3 for a Cannot is honoured as is.

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

For each judge, in name order: a platforms judge resolves platforms[platform] (absent → Missing naming the platform); when .plotplot/bin/<name> exists and .plotplot/bin/<name>.sha256 equals the lock's sha256 → Verified without any fetch; when the companion file exists and differs → Mismatch, and nothing is fetched or replaced (the lock is the truth; the caller decides); otherwise fetch the url, compute sha256 over the bytes, on mismatch return Mismatch and write nothing; on match extract the .tar.gz into .plotplot/beds/<name>/artifact/, copy the executable named by the manifest's install.binary_name (default faces.cli) from the artifact root to .plotplot/bin/<name> with mode 0o755, write .plotplot/bin/<name>.sha256 with the artifact's digest, copy the artifact's garden.json to .plotplot/beds/<name>/garden.json when the artifact carries one, and report Fetched { manifest: true|false } (the checksum contract held either way; an artifact without a manifest is reported as `fetched, no manifest in artifact` on the verify line, and it is `init`, later, that refuses to plant a bed without a cached manifest, never `lock verify`). An npm judge: run `npm install --prefix .plotplot/npm/<name> <npm>@<version>` (Error::Bed when npm exits non-zero, with its stderr), then place .plotplot/bin/<name> as a symlink to the package's bin and read garden.json from the installed package root; a git judge: `git clone <url> .plotplot/beds/<name>/artifact` then `git -C … checkout <rev>` (a symbolic rev cannot occur; the schema refuses it), garden.json from the clone. `plotplot lock verify` prints one line per judge (`<name> <version> verified|fetched|missing: …|mismatch: …`) and exits 0 when every judge is Verified or Fetched, 3 when any is Mismatch or Missing, 1 on other errors; nothing is ever used from PATH.

Tests you write first (unit with FromMap; tests/lock.rs with assert_cmd against a lock whose url the binary cannot fetch, proving the exit codes and messages, and a mismatch case using a pre-placed .sha256 companion): a fixture .tar.gz built in the test (tar + flate2) with a small executable, garden.json (from contracts/fixtures/manifest/weeder.garden.json) and SKILL.md; verify with FromMap serving those bytes under the fixture URL and the right sha256 → Fetched, the bin exists and is executable, the companion carries the digest, garden.json is cached; a second verify → Verified with no fetch (FromMap counts calls); a wrong sha256 in the lock → Mismatch and no file under .plotplot/bin/; a lock naming a platform the judge lacks → Missing; a tarball without garden.json → Fetched { manifest: false } and no cached manifest. Then add the fit subcommand `lock` to scripts/fit/stem.sh against a real release: tilth v0.10.1's aarch64-apple-darwin asset at https://github.com/jahala/tilth/releases/download/v0.10.1/tilth-aarch64-apple-darwin.tar.gz with sha256 38c36e471f61d5a7101d9e0f9dfd401939c5a363e0f4a96384b86f43a662fa55 (verified 2026-09-08 from the downloaded bytes; the tarball holds only the `tilth` executable), or the platform's asset when the fit runs elsewhere (scripts/fit/lib.sh's fit_platform names it; a platform with no asset in the lock makes the check say so and fail): in a fixture repository write a garden.lock naming tilth with that url and digest, run `plotplot lock verify` and assert exit 0, the bin under .plotplot/bin/tilth executable and `.plotplot/bin/tilth --version` printing 0.10.1, the .sha256 companion; then write a lock with one digit of the digest changed into a second fixture and assert `plotplot lock verify` exits 3 naming the mismatch with nothing under .plotplot/bin/; then assert `git check-ignore .plotplot/bin/tilth` succeeds in the fixture after copying this repository's .gitignore, and that `git ls-files` in this repository lists no file under .plotplot/. Then run the verifier for the lock check and paste its output into your final message:

    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 4


Do NOT change existing public shapes; add only. Do NOT edit the loop file.
"""

CHECK = PREAMBLE + """
## Your slice: `plotplot check`, `src/check.rs`, `src/sarif.rs`

This worktree carries the integrated stem: crate, bundles, faces, plant, hook dispatcher, doctor, fit script, and the lock face. Read src/bed.rs, src/manifest.rs, src/layout.rs, src/cli.rs and contracts/vendor/sarif-schema-2.1.0.json before writing anything.

Add src/sarif.rs: `pub fn validate(json: &str) -> Result<serde_json::Value>` enforcing contracts/vendor/sarif-schema-2.1.0.json (embedded with include_str!) and returning the parsed log; `pub fn merge(logs: &[serde_json::Value]) -> serde_json::Value` producing one SARIF 2.1.0 log with `version` "2.1.0", the schema `$schema` URL from contracts/pins.json, and every input log's runs concatenated in order; `pub fn block_count(log: &Value) -> usize` counting results whose level is "error"; add `Error::Sarif { gate: String, problem: String }` additively. Add src/check.rs: `pub struct Gate { pub bed: String, pub program: PathBuf, pub args: Vec<String> }` built from every Bed with check Some by replacing the command's first word with .plotplot/bin/<binary> (never PATH) and splitting the rest on whitespace (a check command needing shell quoting is Error::Bed naming the bed); `pub fn run(root, gates, strict, timeout) -> Vec<GateOutcome>` running each gate in the repository root with stdout captured, `GateOutcome::Sarif(Value) | CouldNotRun { bed, problem }` where a missing binary, a non-zero exit without a valid SARIF log, or invalid SARIF is CouldNotRun; `--strict` appends `--strict` only to gates whose bed manifest declares `check_strict: true` (read it from the manifest's extra fields; when the umbrella adds the field to the schema the adapter follows; name this in your final message) and never otherwise. `plotplot check [--strict] [--format sarif|table]` prints the merged log on stdout (sarif, the default when stdout is not a terminal) or a table (one row per gate: bed, results, blocks, could-not-run), and exits 2 when any run has a block-level result, 3 when any gate could not run (fail closed; both → 3), 0 otherwise. A planted repository with no gates at all prints a log with zero runs and exits 0.

Tests you write first (tests/check.rs with a temp planted root, fixture gate scripts under .plotplot/bin/ that print SARIF from contracts/fixtures/sarif/weeder-check.sarif.json or a modified copy, and manifests under .plotplot/beds/ with matching check commands): two gates → one log with two runs that validates against the SARIF schema; a gate with one error-level result → exit 2; a gate that exits 1 without SARIF → exit 3 with the bed named on stderr; a missing binary → exit 3; both a block and a could-not-run → 3; --strict appends the flag only to the bed with check_strict; --format table prints the rows; the merged log's tool names match each bed's own SARIF tool name; validate refuses a log with a missing `runs` key.

Do NOT change existing public shapes; add only. Do NOT edit the loop file. Do NOT add the fit `check` subcommand unless a real gate binary (weeder from a real release under .plotplot/bin) can run in the fixture; a fixture-script gate is a test, not fit evidence.
"""


INIT = PREAMBLE + """
## Your slice: `plotplot init`, `src/init.rs`, `src/install.rs`, and the fit `init` check

This worktree carries the integrated stem with the lock and check faces. Read src/cli.rs, src/lock.rs, src/bundle/mod.rs, src/plant/, src/doctor.rs and scripts/fit/stem.sh before writing anything; init composes them and adds nothing they already do.

`plotplot init [--harness claude,gemini,codex] [--beds a,b] [--profile minimal|full] [--lock <path>]`, per docs/plans/stem.md §5, idempotent, never touching a user-scope file and never installing anything globally: (1) detect harness CLIs on PATH (claude, gemini, codex) and any project configs present (.claude/settings.json, .gemini/settings.json, .codex/); --harness limits the set; (2) the profile: minimal (default, jahala/plotplot issue 17) plants weeder with its hooks, the garden block, the lock, the git hooks and the PR check workflow; full plants every judge in the lock; --beds overrides the bed list; (3) garden.lock: when absent, copy the template given by --lock (required when absent; the umbrella's template path is named in your final message as an open question if none is agreed) filtered to the chosen beds; when present, verify it (lock::verify with fetch::Https), refusing to continue on a Mismatch (exit 3); (4) bundle build for each chosen harness; (5) install each bundle at project scope, in src/install.rs, one function per harness, each returning what it wrote: Claude writes `.plotplot/marketplace/.claude-plugin/marketplace.json` listing the bundle as a local plugin and runs `claude plugin marketplace add .plotplot/marketplace` then `claude plugin install plotplot@plotplot-local --scope project`, reporting the exact command and its stderr on failure; Gemini has no project scope, so init writes the hooks and mcpServers into the project's .gemini/settings.json (merging into an existing file, touching only the `hooks` and `mcpServers` keys, byte-identical on a second run) and says so; Codex writes the project's .codex/hooks.json from the bundle's hooks.json and the MCP servers into .codex/config.toml under [mcp_servers], and says that hooks load only once the project is trusted; (6) .githooks/* from plant::githooks with mode 0o755, core.hooksPath and the receipts refspecs from plant::gitconfig; (7) the garden block in AGENTS.md from plant::garden_block, and the planted repository's own garden.json beside garden.lock, kind ["repository"], name from the directory or the git remote, written only when absent and validated against contracts/manifest.schema.json's repository case (contracts v1.3.0; fixture contracts/fixtures/manifest/repository.garden.json) — manifest::to_bed must refuse a repository manifest with Error::Manifest and load_beds must skip one under .plotplot/beds/, add both with tests; (8) the PR check workflow .github/workflows/plotplot-check.yml running `plotplot check --strict` on pull_request with hosted runners only (no self-hosted runner ever on a public bed, jahala/plotplot issue 7); (9) print one line per file or setting changed, and `nothing to do` when a second run changes nothing. Add `Error::Install { harness, problem }` additively.

Then add the fit subcommand `init` to scripts/fit/stem.sh: build a fixture repository, run `plotplot init --harness gemini,codex --lock <a lock built in the fixture from a locally served artifact only if lock::verify can fetch it; otherwise pre-place the judge and its .sha256 so verify passes without a fetch>`, assert AGENTS.md carries the block, .gemini/settings.json and .codex/hooks.json exist with the dispatcher entries, core.hooksPath is .githooks, the four hooks are executable, the receipts refspecs are set, and that a second run prints `nothing to do` and leaves every file byte-identical (hash the tree before and after). Claude's install is asserted the same way when `claude` is on PATH, against a temp HOME.

Tests you write first (tests/init.rs with temp roots, a temp HOME, and the fixture judge pre-placed): the numbered steps above each have one assertion; idempotence is asserted by a tree hash; --harness gemini writes only Gemini's config; a Mismatch in the lock aborts before any bundle is written; a missing --lock with no garden.lock is a usage error naming the flag.

Then run the verifier for the init check and paste its output into your final message:

    tend2 verify docs/tend2/stem.tend2.html --repo-root . --check 1

Do NOT change existing public shapes; add only. Do NOT edit the loop file.
"""

SMOKE_INTEGRATE = f"bash -lc '{GATE_SHELL} && bash scripts/fit/stem.sh bundles && bash scripts/fit/stem.sh hook-faces'"

AUDIT_INTEGRATE = (
    "bash -lc 'cargo test 2>&1 | tail -20 && bash scripts/fit/stem.sh hook-faces && bash scripts/fit/stem.sh bundles "
    "&& git diff --stat HEAD~1 -- docs/tend2/stem.tend2.html'"
)

WORKER = {"provider": "claude", "model": "claude-opus-5"}


def node(id_, prompt, needs=(), smoke=GATES, audit=None, timeout_ms=3_600_000):
    accept = {"smoke": smoke}
    if audit:
        accept["audit"] = audit
    return {
        "id": id_,
        "worker": dict(WORKER),
        "work": {"prompt": prompt},
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
    "source": "docs/plans/stem-build.plan.json",
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
            audit={"command": AUDIT_INTEGRATE, "provider": "codex"},
            timeout_ms=5_400_000,
        ),
    ],
}

print(json.dumps(plan, indent=2, ensure_ascii=False))
