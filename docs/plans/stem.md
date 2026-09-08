# The stem — plan

One way to plant the garden in any harness, and to prove it is planted. Generated bundles
in each vendor's own format, a doctor that proves every hook fires, one check that runs every
gate, and a lockfile wrapper that fetches pinned judges so nothing is installed globally and
proof is reproducible. The stem also hosts the two hook-time faces no bed owns: the friction
emitter and the receipt writer.

Status: plan, 2026-09-06. Shape as decided in the architecture review: three generated
bundles plus `doctor`, `check`, `init`, and the lockfile wrapper. Not an orchestrator, not a
runtime, not a package manager, not a server, no gateway, no briefing assembler.

## 1. Why a stem at all

Every bed works alone. What nobody experiences today is the composition: eight installs,
eight READMEs, hooks in three vendor formats, judges at whatever version happens to be on
PATH. A stamp's meaning depends on the judge's version, so pinning is not convenience, it is
what makes proof reproducible. And a hook that silently never fires is the worst failure a
boundary can have, so "is it planted" must be a command, not a belief.

The stem has a fourth job this plan first left implicit: it projects the law onto the platform,
writing the workflow files, the CODEOWNERS line and the rulesets it can apply, reporting verbatim
what it cannot, and doctor proves that half as well (plotplot-ai issue 26).

## 2. Standards and vendor formats it stands on

Verified 2026-09-06 against the vendors' references.

| Harness | Bundle format | Where hooks live | Install |
|---|---|---|---|
| Claude Code | plugin: `.claude-plugin/plugin.json` (name, version, description, author, hooks, mcpServers, skills), `hooks/hooks.json`, `.mcp.json`, `skills/`, `bin/` | `hooks/hooks.json` with `${CLAUDE_PLUGIN_ROOT}` | `claude plugin install <name>@<marketplace> --scope project`, or a marketplace repo |
| Gemini CLI | extension: `gemini-extension.json` (name, version, description, mcpServers, contextFileName, excludeTools, settings), `hooks/hooks.json`, `skills/`, `GEMINI.md` | `hooks/hooks.json` in the extension directory | `gemini extensions install <git url or path>`, stored under `~/.gemini/extensions` |
| Codex CLI | plugin: `.codex-plugin/plugin.json`, `hooks.json`, `.mcp.json`, `skills/` | `hooks.json` in the plugin, or project `.codex/` when trusted | `codex plugin marketplace add`, then install from the marketplace |

Other soil: `AGENTS.md` for the garden block; `core.hooksPath` for git hooks;
`refs/notes/plotplot/receipts` for receipts; SARIF 2.1.0 as the only output of `check`;
per-platform binaries with sha256 in the lockfile (the pattern of the Gradle wrapper,
`.tool-versions` with lockfiles, and npm platform packages, which tilth already ships).

## 3. Faces

```
plotplot init [--harness claude,gemini,codex] [--beds …]   plant the garden in this repo
plotplot doctor [--live]                                    prove it is planted
plotplot check [--strict] [--format sarif|table]           run every gate, one SARIF log
plotplot lock [update|verify]                               the pinned judges
plotplot bundle build [claude|gemini|codex]                 regenerate the bundles from manifests
plotplot friction emit                                      hook face (see the friction plan)
plotplot receipt draft|seal|sign|verify|show                hook and git faces (see the receipts plan)
plotplot hook <harness> <event>                             the single dispatcher every hook entry calls
plotplot version                                            the season and each pinned judge
```

The binary is Rust, static, in its own repository, depending on nothing at runtime. Hooks
must be static binaries: Claude Code is native and Codex is Rust, so a hook needing Node
silently never fires on a common machine.

## 4. The manifest and the lock (contracts deliverables)

**`garden.json`** at each bed's root, schema in contracts (already on the contracts loop):

```json
{"name":"weeder","kind":["gate","hook"],"version":"0.1.0",
 "install":{"cargo":"weeder","npm":"@plotplot/weeder","binaries":{"aarch64-apple-darwin":"https://github.com/jahala/weeder/releases/download/v0.1.0/weeder-aarch64-apple-darwin.tar.gz"}},
 "faces":{"cli":"weeder","skill":"skills/SKILL.md","hooks":{"claude":["PreToolUse:Bash","Stop"],"gemini":["BeforeTool:run_shell_command","AfterAgent"],"codex":["PreToolUse","Stop"]},"git":["pre-commit","pre-push","pre-rebase"]},
 "check":"weeder check --format sarif","metric":"scripts/check/calibrate.sh","context":{"upfront_tokens":0}}
```

**`garden.lock`** at the planted repository's root, schema in contracts:

```toml
season = "2026.09"

[judges.weeder]
version = "0.1.0"
[judges.weeder.platforms."aarch64-apple-darwin"]
url = "https://github.com/jahala/weeder/releases/download/v0.1.0/weeder-aarch64-apple-darwin.tar.gz"
sha256 = "…"
[judges.weeder.platforms."x86_64-unknown-linux-musl"]
url = "…"
sha256 = "…"

[judges.tilth]
version = "0.10.1"
# …
[judges.tend2]
version = "1.0.0"
npm = "@plotplot/tend2"
```

`plotplot lock verify` resolves each judge for the current platform, fetches into
`.plotplot/bin/` (ignored) when absent, verifies the sha256, and refuses to run a judge whose
bytes do not match. `check` and the hooks call judges only through that directory, never
through PATH, so the version that judged is the version the lock names.

## 5. What `init` does, exactly, and idempotently

1. Detect harness CLIs on PATH and any project configs present.
2. Read the manifests of the beds requested (defaults: tend2, tilth, weeder, petals).
3. Write `garden.lock` for the current season, or verify an existing one.
4. Generate the three bundles under `.plotplot/bundles/<harness>/` from the manifests: every
   bed's hook entries become one dispatcher call each (`plotplot hook claude PreToolUse`), the
   beds' `SKILL.md` files are copied under `skills/`, channel beds' MCP entries go into the
   bundle's `.mcp.json`, and the hard-limit deny list (the `--no-verify` pattern, the guarded
   paths) becomes a PreToolUse entry.
5. Install each bundle through its vendor's mechanism at project scope; where a vendor has no
   project-scope install, write the project config file directly and say so in `doctor`.
6. Write `.githooks/pre-commit`, `pre-push`, `pre-rebase`, `post-commit` (weeder guard, receipt
   seal), set `core.hooksPath .githooks`, add the receipts ref to fetch and push refspecs.
7. Write or update the garden block in `AGENTS.md` (ten lines: what is planted, the season,
   orient, gates, help, that hard limits are enforced at the boundary). Nothing else in the
   file is touched; the block is marked and replaced in place.
8. Print what changed. A second run changes nothing and says so.

`init` never installs anything globally and never edits a user-scope settings file. Codex
requires the project to be trusted before its hooks load; `init` says so and `doctor`
checks it.

## 6. What `doctor` proves

Static mode (default, no harness launched): every bundle present and matching the generated
one byte for byte; every hook entry pointing at the stem binary; `core.hooksPath` set;
`.githooks/*` present and executable; every judge in the lock present in `.plotplot/bin/` with
matching sha256; the garden block current; the receipts refspec present; Codex trust status.
Output: a table and exit 0 or 3.

Live mode (`--live`): for each harness present, drive one real session through umbel with a
scripted prompt that triggers each hook event (a denied command, a Stop, a SessionEnd) and
assert, from the friction journal and a marker file, that each hook fired. This is the only
proof that a hook fires, and it is the check that catches a vendor changing an event name or
a dialog default (the trust-dialog incident that killed every fleet worker in September is
this class). The scheduled real-binary smoke in the proof repository runs `doctor --live`.

## 7. What `check` does

Runs every gate the manifests declare (`weeder check`, `petals check`, `tend2 lint`, `tend2
gate` where a map exists, `weeder scan` when asked), each producing SARIF, and merges them into
one SARIF 2.1.0 log with one `run` per tool. Exit 2 if any run has a block-level result, 3 if
any gate could not run (fail closed), 0 otherwise. `--strict` passes through to the gates
that define it. pleach's default smoke is `plotplot check --strict` when the stem is
planted, else `weeder check --strict`.

## 8. Context cost

The stem adds one description line for its own skill (under 160 characters) and the garden
block (under 400 tokens with the skill lines of the default beds), measured by the same probe
that measured 8,300 tokens on 2026-09-05 with everything planted and nothing deferred. No MCP
server of its own. No briefing at session start beyond what tend2's own hook does; push
orientation stays an experiment under tend2, gated on a copeca win.

## 9. The proof run (the umbrella's integration check)

`scripts/fit/proof.sh` builds a fixture repository with one deliberately flawed feature,
plants it with `plotplot init`, emits a plan from its loops, runs it through pleach with two
umbel workers on two providers, plants a test deletion in one worker's diff, and asserts in
order: weeder refused the deletion as a block-level SARIF result; the surviving node's check
was stamped with the verifier's identity and version; `plotplot check` returned one SARIF log
containing every gate's results; a receipt note exists on the merged node's commit and
verifies; `plotplot doctor --live` on a second fresh clone reports every hook fired with no
global install performed.

## 10. Checks (already on the umbrella's `stem` loop, extended by this plan)

init idempotent; doctor static and live on two harnesses with Codex reported honestly;
check merges SARIF with the right exit codes; lock fetches, verifies and refuses a mismatch,
and no binary is committed; bundles install through each vendor's mechanism and carry
identical hooks, skills and MCP entries; context cost under the bar; proof run passes; the
friction emitter and the receipt faces are hook entries in every generated bundle; the owner
confirms the npm names and the receipts default.

## 11. Kill criteria

- If driving two harnesses from one manifest needs harness-specific fields in bed manifests,
  the abstraction is wrong; stop and redesign the manifest.
- If bed installs cannot be made idempotent one-shot, `init` is a wrapper over pain; ship
  `doctor` and `check` alone and say so.
- If `doctor --live` cannot be made to run in the proof repository's CI for at least two
  harnesses, the stem cannot claim "proved planted"; it claims "configured" and the live
  proof stays a manual command.

## 12. Order of work

1. Lock schema in contracts (with the manifest schema, already there); `plotplot lock`.
2. `plotplot hook` dispatcher and `check` (merging SARIF from weeder, petals, tend2).
3. Bundle generation for Claude Code, then Gemini, then Codex; `init`.
4. `doctor` static, then live via umbel.
5. Friction emitter face (friction plan, step 2) and receipt draft/seal faces (receipts plan,
   step 2).
6. The proof run in the proof repository, scheduled with `doctor --live`.

## 13. Decisions for the owner

- The `plotplot` npm name and the `@plotplot/*` scope (both free as of 2026-09-05).
- Whether the stem's repository is `jahala/plotplot` (recommended) or lives in this umbrella
  repository beside the site.
- Which beds `init` plants by default.
- Whether receipts are drafted by default (see the receipts plan).

Decided 2026-09-08: the stem's repository is `jahala/plotplot`, the umbrella, beside the contracts it
plants and enforces (two loops, one repository). The default profile `init` plants is the minimal one
(jahala/plotplot issue 17): one loop file, weeder with its hooks, the garden block, the lock, the PR
check workflow, one declared metric; the full profile on request.
