# plotplot — the architecture

Status: v1, under adversarial review. `plotplot-architecture-review-2026-09.md` reverses
several decisions below (contract count, the stem's scope, the monorepo, the single runtime,
push orientation, hedge as a bed, copeca as the season gate). Read the review's section 13
for the shape that survives; this document is amended on the owner's go.

The way forward, from first principles. What exists today was consulted for evidence, not
allowed to dictate the shape. Where the target differs from today, section 9 says what moves.
Companion memos: `garden-direction-2026-09.md` (why these beds), `garden-architecture-2026-09.md`
(measured context costs). This document governs; those inform.

## 0. The verdict

plotplot is **a set of open contracts plus a thin stem**. The contracts are the umbrella: data
formats and conventions any tool can implement. The beds are independent tools that implement
them. The stem, `plotplot`, is one small CLI that wires beds into any harness, assembles a
session's briefing from repo state, runs every gate as one, and shows a human what needs
them. Nothing runs as a server. Nothing decides with an LLM. Everything is a file.

Any bed works alone, with zero plotplot installed. Any two beds work together because they
share a contract, not because they know each other. A tool nobody in the garden wrote can
join by shipping one manifest file.

## 1. What the system is

A harness layer around stochastic agents, vendor-neutral, where code decides. Ten concerns,
each owned by exactly one bed:

| Concern | Question it answers | Bed |
|---|---|---|
| Intent | What should be built, and what does done mean | tend2 |
| Context, code | How does the agent read the codebase | tilth |
| Context, brand | How does the agent read design and voice | petals |
| Context, truth | Do the docs the agent reads still match the code | graft |
| Boundary | What may the agent not do, receive, or spend | hedge |
| Execution | How is one unit of agent work run on any provider | umbel |
| Evidence | Is the produced diff honest work | weed |
| Conduction | How do many units run isolated, gated, merged only when verified | pleach |
| Communication | How do agents talk to each other with a human at the gate | pollen |
| Measurement | Did any of this help, in cost per correct answer | copeca |

Two cross-cutting functions belong to no bed and therefore to the stem: **orientation**
(assembling the minimal briefing for this session) and **attention** (what needs a human,
across every repo). Memory is not a bed; it is a section of the loop file that beds write to.

## 2. Invariants

Every design choice below follows from these seven. A proposal that breaks one is wrong.

1. **Agents produce; code decides.** Done, verified, merged, on-brand, safe, cheaper: all
   decided by deterministic code reading files. Agent prose is report material, parsed,
   never interpreted.
2. **Files are the substrate.** State lives in the repo, in plain text, diffable by a human
   and readable by an agent. No servers, databases, or accounts. A local process may render a
   page; truth stays in files.
3. **Neutral across harnesses.** Integration happens only through seams every agent CLI
   shares: the filesystem, `AGENTS.md` and `SKILL.md`, lifecycle hooks, MCP over stdio, git,
   the shell. Nothing depends on one vendor's feature.
4. **Composable and standalone.** Each bed is a Unix tool: argv in, files and JSON out, exit
   codes as verdicts. It has full value alone. Composition is through contracts, never
   through shared process state or imports across beds.
5. **Zero-tax presence.** A bed costs an agent nothing in context unless it is speaking, and
   it speaks only on evidence. Capabilities are CLIs behind skills; channels are MCP with
   few tools; boundaries and evidence are hooks.
6. **Measured or silent.** A bed ships with the copeca scenario that justifies it. A bed
   that cannot be measured does not get to speak at session start.
7. **One idea, one home.** Each contract is defined once, in the contracts package, and
   consumed everywhere. No bed re-defines a findings shape, a loop shape, or a receipt.

## 3. The layers

```
 4  surfaces     documents rendered from files: loop pages · the walk · brand book · scoreboard
                 (the harness's own UI, Conductor or any other, is where sessions run)
 ──────────────────────────────────────────────────────────────────────────────────────
 3  stem         plotplot: init · doctor · orient · check · run · walk · version · mcp
                 wires beds into harnesses; resolves manifests; assembles; aggregates
 ──────────────────────────────────────────────────────────────────────────────────────
 2  beds         tend2 · tilth · petals · graft · hedge · umbel · weed · pleach · pollen · copeca
                 each one concern, each standalone, each speaks the contracts
 ──────────────────────────────────────────────────────────────────────────────────────
 1  contracts    manifest · loop · plan · unit · findings · briefing · receipt · message · rules · brand · artifact
                 JSON Schema, versioned, fixtures, conformance test — the umbrella itself
 ──────────────────────────────────────────────────────────────────────────────────────
 0  soil         git · filesystem · shell · AGENTS.md / SKILL.md · harness hooks · MCP stdio
                 not ours; the seams every harness already has
```

Dependencies point downward only. A bed depends on contracts and soil, never on another bed
or on the stem. The stem depends on contracts and soil, and discovers beds at runtime through
manifests. Surfaces depend on files.

## 4. The contracts

The contracts are what make separate tools one garden. Eleven, all JSON Schema, all in one
package, all MIT:

| Contract | Owns | Shape in one line |
|---|---|---|
| **manifest** | how a bed plugs in | `garden.json`: name, kind, version, install per platform, faces (cli, skill, hooks, mcp), check command, orient command, brand layer path, declared context cost |
| **loop** | intent and done | tend2 `FORMAT.md`: goal, checks with method and stamp, tried, needs, children; state derived from the boxes |
| **plan** | conduction | pleach's DAG: nodes, work, needs, accept gates, policy |
| **unit** | one run of agent work | umbel's signature: `invoke(task, opts) → {status, output, evidence, telemetry}` |
| **findings** | every gate's output | `[{tool, rule, severity: block or warn or info, file, line, message, action}]` plus exit code 0, 2, or 3 |
| **briefing** | orientation fragments | `{tool, priority, tokens, fires_on, text}`; the stem assembles under a budget |
| **receipt** | what a session did | harness, model, cost, tools used, gates passed, diff hash, principal; written at SessionEnd, committed if the team wants a shared view |
| **message** | agent to agent | pollen's envelope: from, to, body, gate state |
| **rules** | boundary policy | hedge's file: rule id, scope, action, reason text |
| **brand** | design and voice | petals' `.brand/` tree: umbrella plus product layers |
| **artifact** | a measurement | copeca's signed `.copeca` run |

Three of these are new and do the unifying work. **findings** means petals, weed, graft, tend2
lint and hedge all emit one shape, so `plotplot check` is concatenation and any CI can read
any bed. **briefing** means orientation is assembled, not hardcoded: each bed offers a
fragment with the condition under which it fires, and the stem takes the highest-priority
fragments that fit a token budget. **manifest** means the stem never hardcodes a bed; a
tool from outside the garden joins by shipping `garden.json` and emitting findings.

Rules of the contracts package: additive changes only within a season (section 8); every
schema has fixtures; `plotplot conform` runs a bed's faces against the fixtures and is part
of every bed's CI. A contract change and all its in-repo consumers land in one commit.

## 5. A bed

Four kinds, by how the agent meets them:

| Kind | Meets the agent through | Context cost | Examples |
|---|---|---|---|
| capability | CLI, with a `SKILL.md` for on-demand instructions | one description line | tilth, petals, copeca, graft |
| channel | MCP over stdio, at most three tools and one help topic | tool names only where the harness defers; schemas otherwise | umbel, pollen, tend2's pens |
| gate | CLI emitting findings; exit code is the verdict | zero | weed, petals check, graft check, tend2 lint |
| hook | harness lifecycle hook; speaks only to deny, refuse, or brief | zero when silent | hedge, weed at Stop, orient |

A bed may be several kinds: weed is a gate and a hook; tend2 is a capability, a channel, and a
gate. Every bed, whatever its kind, ships:

- a CLI as its primary face, usable with nothing else installed;
- a `garden.json` manifest;
- a `SKILL.md` whose description is one sentence and whose body carries everything the bed
  used to put in an instruction block;
- a `check` command emitting findings, if it has anything to judge;
- an `orient` command emitting a briefing fragment, if it has anything worth saying at
  session start, with its firing condition declared;
- its copeca scenario, in the repo, runnable;
- a `.brand/products/<bed>/` layer and the garden footer on its page.

The standalone rule is a test, not a promise: a bed's CI installs it on a clean machine with
no other garden tool and runs its full suite.

## 6. The stem

`plotplot` is a small CLI, a skill, and, where a harness still loads every schema, one
gateway MCP server. Its responsibilities are exactly the cross-cutting ones:

| Command | Does |
|---|---|
| `plotplot init` | detects the repo and which agent CLIs are installed; installs the chosen beds through their manifests; writes the garden block into `AGENTS.md`; writes hooks, skills and MCP entries in each harness's native form; records the season in `.plotplot/garden.lock` |
| `plotplot doctor` | proves every hook actually fires, every binary resolves, every version matches the lock; prints the per-harness capability table honestly |
| `plotplot orient` | the SessionStart and PreCompact hook: collects briefing fragments from beds whose firing condition holds, assembles under a budget, emits the briefing; silent on a fresh repo |
| `plotplot check` | runs every gate the manifests declare, concatenates findings, one exit code; pleach's default smoke command |
| `plotplot run <plan or loop>` | conducts through pleach with umbel as the runner; worker worktrees inherit the repo's hooks by construction |
| `plotplot walk` | the attention surface: every repo's loops, receipts and pollen knocks in one rendered page, what needs you first |
| `plotplot version` | the season, and each bed's pinned version |
| `plotplot mcp` | the gateway: `garden_help`, `garden_describe`, `garden_call`; only for harnesses that do not defer schemas |

Per-harness adapters live in the stem and nowhere else. Each knows one harness's config
locations and hook vocabulary: Claude Code (`settings.json` hooks, the plugin format, `.mcp.json`,
`.claude/skills/`), Codex (`config.toml`, its hook events, `AGENTS.md`), Gemini CLI
(`settings.json`, extensions, `BeforeTool`/`AfterTool`), OpenCode (`opencode.json`). Adding a
harness is adding one adapter file. tilth's installer already covers 20 hosts for MCP
config; that knowledge moves into the stem and tilth stops carrying it.

The garden block the stem writes into `AGENTS.md`, in full:

```
## plotplot garden

This repo is tended with plotplot (season 2026.09). Planted: tend2, tilth, petals, hedge, weed.
Orient: `plotplot orient` (runs at session start). Loops: `tend2 next docs/tend2`.
Gates: `plotplot check` must pass before a commit. Help: `plotplot help <bed>`.
Hard limits are enforced by hedge at the tool boundary; a denial comes with its reason.
```

What the stem is not: not an orchestrator (pleach conducts), not a runtime (umbel runs), not a
package manager (it delegates to cargo, npm, pip, or curl as each manifest says), not a
server, not a daemon, not a marketplace. If a feature would make it any of those, it belongs
in a bed or nowhere.

## 7. Runtime: how it works together

One session, on any harness, with the garden planted:

| Moment | Hook | What runs | Contract |
|---|---|---|---|
| session starts | SessionStart | `plotplot orient`: tend2's routing, graft's stale count, weed's open findings, hedge's mode, petals only if UI files are in scope | briefing |
| a prompt arrives | UserPromptSubmit | tilth's scout fragment, only when its gate is confident | briefing |
| a tool is about to run | PreToolUse | hedge: allow or deny with reason; weed on `git commit`: staged findings | rules, findings |
| a tool has run | PostToolUse | hedge inbound: quarantine instruction-like text in results, where the harness allows suppression | rules |
| context is about to compact | PreCompact | `plotplot orient` again | briefing |
| the agent says done | Stop | weed on the working tree; if a loop is active, tend2 verify on its code checks; refuse the stop with findings | findings, loop |
| session ends | SessionEnd | receipt written; the loop's Tried section gains what was tried | receipt, loop |

A fleet, from one command: `plotplot run plan.json`. pleach isolates each node in a worktree,
umbel spawns the worker on whatever provider the plan names, the worktree carries the repo's
hooks so hedge and weed guard every worker exactly as they guard a human's session, pollen
carries peer questions with the human at the gate, gates run in order (markers, `plotplot
check --strict`, audit by a different provider), and only a verified `node/<id>` branch
publishes. tend2 is the ledger that closes the loop.

Measurement wraps any of it: a copeca scenario runs the same tasks with a bed present and
absent and reports cost per correct answer with a confidence interval. The season's release
is gated on the whole-garden scenario.

Attention is one page: `plotplot walk` reads every repo's `docs/tend2/`, `.plotplot/receipts/`
and pollen's held knocks and renders what needs a human, oldest first, as a document you
open from disk.

## 8. Repo, language, release

**Two kinds of repos.** One monorepo, `plotplot`, holds the contracts, the stem, the umbrella
brand and site, and every bed that is glue-shaped and TypeScript: tend2, pleach, umbel,
pollen, weed, hedge, graft. Satellites hold the beds that are products in their own right
with their own communities and toolchains: tilth (Rust, 339 stars), copeca (Python, PyPI),
and petals (a skill; it may live in either). Satellites implement the contracts and ship a
manifest; they never import from the monorepo.

Why a monorepo for the glue: a contract change and all its consumers land atomically, one CI
proves the loop end to end on every commit, one version number means one season, and a solo
maintainer carries one release instead of eight. Why satellites stay out: atomic change
across Rust and Python is impossible anyway, so the contract boundary is honest there, and
their users should not clone a garden to get a code reader.

**One runtime for the monorepo:** TypeScript on Node 20 or later, ESM, zero runtime
dependencies wherever possible, `node --test`, one `tsc` build. Bun is dropped where it is
used today; Node is what every hook can rely on being present. If hedge's per-tool-call
startup proves costly, hedge alone may become a compiled binary later; the manifest hides
that from everyone.

**Publishing:** each package under `@plotplot/*` on npm, plus `plotplot` itself; `npx plotplot
init` is the whole install story. Satellites publish where they already do.

**Seasons.** The garden versions as `plotplot 2026.09`: a lockfile of bed versions proven
together by the end-to-end run and the whole-garden copeca scenario. Contracts are additive
within a season; a breaking contract change is a new season. Users pin a season.

## 9. The refactor map

| Today | Target |
|---|---|
| tilth carries a 480-token instruction block and installs into 20 hosts | instruction block becomes tilth's `SKILL.md` body; install logic moves to the stem's adapters; tilth ships `garden.json` and keeps its own installer only as a fallback |
| umbel: 12 MCP tools, Bun | 6 tools plus help, Node; moves into the monorepo; its `invoke` signature becomes the `unit` contract |
| pleach: Bun, `accept.smoke` free-form | Node; moves into the monorepo; default smoke is `plotplot check --strict`; its plan schema becomes the `plan` contract; RED/GREEN gains `weed bite` at the end |
| tend2: Node, its own plugin with 8 skills and a SessionStart hook | moves into the monorepo; the plugin is generated by the stem; SessionStart becomes a briefing fragment; lint emits findings |
| petals: bash `check.sh` with its own output | emits findings; vendored copies replaced by the stem installing the skill |
| walkie-clawkie: 5 tools | pollen: 3 tools, Node, in the monorepo; envelope becomes the `message` contract |
| mull: dormant, its own MCP server and knowledge store | if it survives its review, a batch writer of the loop's Tried section, no server, no store |
| copeca: Python satellite | unchanged, plus `garden.json` and the whole-garden scenario |
| plotplot-ai: landing page and canonical brand in their own repo | `brand/` and `site/` in the monorepo; the site adds the loop story and the walk |
| hedge, weed, graft: not built | built in the monorepo against the contracts from the start |
| eight READMEs, eight installs | one `npx plotplot init`; beds still installable alone |

## 10. Order of work, with kill criteria

1. **Contracts package**: manifest, findings, briefing, receipt as new schemas; loop, plan,
   unit, message, rules, brand, artifact lifted from where they live. Fixtures and
   `conform`. Kill: if two existing beds cannot emit the findings shape without losing
   information they need, the shape is wrong; fix it before anything else.
2. **Stem, first spark**: `init` for Claude Code and one more harness, `doctor`, `check`,
   `orient` with tend2 and weed fragments, the garden block. Kill: if driving two harnesses
   from one manifest needs harness-specific fields in bed manifests, the abstraction is
   wrong.
3. **Monorepo move** of tend2, pleach, umbel, pollen onto one runtime; the end-to-end proof
   as CI.
4. **hedge and weed** built against the contracts.
5. **The whole-garden copeca scenario**: tasks with the garden planted versus not, cost per
   correct. Kill: if planting the garden does not lower cost per correct on hard tasks at
   the pre-registered bar, the season does not ship and the memo says why.
6. **Season 2026.09**, then the site tells the loop story with real output.

## 11. Decisions for the owner

- Monorepo plus satellites, as in section 8, or everything separate with contracts only.
- Node as the single runtime for the monorepo, dropping Bun.
- The `plotplot` npm name (free as of 2026-09-05) and the `@plotplot/*` scope.
- Whether receipts commit to the repo by default or stay local until a team exists.
- Whether petals lives in the monorepo or stays a satellite.
- When to lock this document into `CLAUDE.md` as the governing decision.
