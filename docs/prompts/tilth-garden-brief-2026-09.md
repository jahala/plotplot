# tilth — garden brief

Note: weed was renamed weeder on 2026-09-06; this is the message as sent.

Brief for the agent working on tilth. Self-contained: you have not seen any prior
conversation. Read in this order before touching code:
`~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/building-the-garden.md` (the law every
bed agent follows), `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/tend2/tilth.tend2.html`
(your loop on the umbrella map: the fit checks the umbrella verifies you against), then this
brief. Where they disagree: the loop beats this brief; the garden brief beats both.

## tilth's role in the garden

tilth is the read row: structural reading of code for agents, with a measured cost-per-correct
gain. It is also becoming the **parser substrate** the garden's judge is built on. weed, the
judge of the diff, is being built in its own repository and needs a `tilth-core` library crate
for tree-sitter outlines, symbol search, test-file classification, and dependency analysis.
Your two jobs, in order: make that crate exist without changing tilth's behaviour, and bring
tilth to the garden's shape (zero context tax, a manifest, a measured metric, the brand layer).

## Rules that bind you, from tilth's own history

These come from tilth's project memory and CLAUDE.md. They are not advice.

- **Work from `origin/main`.** It is at v0.10.1. The checkout at `~/CascadeProjects/tilth` is
  stale (Cargo.toml 0.5.7 on branch `feat/glob-filter`); do not build on it. Use a fresh
  workspace on `main`.
- **No competitor attribution.** Never name or attribute external repos, authors, or products
  in tilth code, commits, PRs, tasks, or docs. tilth's token-reduction features are
  cleanroom, tilth-native work. Derivative work inspired by a contributor carries
  `Co-authored-by`; that is credit, and it is distinct from the attribution ban.
- **CI lints on stable.** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` run
  on every push and PR. Match your local clippy to stable before claiming green; a stale
  toolchain gave a false green once (#148).
- **Version bumps** land in **both** `Cargo.toml` and `npm/package.json`, via a dedicated PR
  before tagging. A `v*` tag on main triggers multi-platform binaries, crates.io, and npm.
  README and `index.html` version references track the released version; benchmark results
  keep the version they were measured on.
- **No unauthorized merges.** Your work stops at PR opened, evidence attached, owner asked.
  Contributor PRs: full-diff, hands-on review before any recommendation; single-instance
  framing issues are not worth flagging, repeat patterns are.
- **A/Bs are copeca runs.** Do not run the in-repo `benchmark/` for A/Bs; draft the scenario
  brief and the copeca agent runs it. You only draft binary, task kind, and metrics.
- **Closed arcs stay closed** unless you bring new evidence: the concept-intent footer
  (capped, built, do not ship), the scout hook (killed by a pre-registered A/B in July; the
  reactive `tilth_scout` tool is the only surviving form), the cross-encoder reranker (offline
  file-rank is a distrusted metric), the image channel, and `tilth_run` (PR #54, killed on
  real-session data; its parsers remain in the branch as a parts bin).
- **MCP instruction changes are surgical.** Haiku is sensitive to position, framing, and
  concrete examples; instruction changes are tested on the hard navigation tasks.
- **No subagents.** Execute yourself. Final message is the deliverable.
- Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash. Do not push
  to main, merge, tag, or publish. Those wait for the owner.

## Where things are (verified 2026-09-05)

| What | Where |
|---|---|
| Repository | `github.com/jahala/tilth`, `main` at v0.10.1 (releases v0.10.0 and v0.10.1 landed via `release/*` PRs #206, #207). 339 stars. |
| Crate today | a single package: `[[bin]] tilth` plus `src/lib.rs` as the public API. No Cargo workspace yet. |
| Parsing modules | `src/read/outline/` (tree-sitter outlines, 14 languages), `src/search/symbol.rs`, `src/search/callers.rs`, `src/search/callees.rs`, `src/search/deps.rs`, `src/search/blast.rs`, `src/index/`, `src/types.rs` (`is_test_file`), `src/read/generated.rs`, `src/read/binary.rs`. |
| Faces | `src/main.rs` (CLI), `src/mcp.rs` (server, `SERVER_INSTRUCTIONS`, `EDIT_MODE_EXTRA`), `src/install.rs` (writes MCP config for about twenty hosts), `src/edit.rs`, `src/session.rs`. |
| Served MCP tools on v0.10.x | seven: deps, diff, files, grok, read, search, write. Verify against `main`. |
| Context cost today | ≈ 480 tokens of server instructions plus ≈ 2,500 tokens of tool schemas, measured 2026-09-05 from source and a live session. The largest of any bed. |
| Open PRs | #196 (external, "port scout, unified listing, and shorter prompts", overlaps the diet below), #169 (owner's scout platform port), #193 (dependabot). Triage, do not merge. |
| Map | none. `tend2 init docs/tend2` is your first commit. |
| Brand | `index.html` is live at jahala.github.io/tilth. Check whether `.brand/products/tilth/` exists and whether the footer carries the plotplot band and every bed. |
| weed | its builder is peer `bandung` on pollen, workspace `~/conductor/workspaces/weed/bandung`, brief at `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/prompts/weed-build-2026-09.md`. It will send you the `tilth-core` API it needs. |

## Part 1 — `tilth-core`

Turn the package into a Cargo workspace with two members: `tilth-core` (library: parsing,
outlines, symbol index and search, callers and callees, dependency analysis and blast radius,
file classification including `is_test_file`, generated and binary detection) and `tilth`
(the binary: CLI, MCP, install, edit, session, formatting, budget). Rules for the split:

- **Zero behaviour change.** Every existing test passes unchanged; the CLI and MCP output is
  byte-identical on the fixture corpus. Prove it with a before/after diff of outputs on
  `benchmark/fixtures` repos, committed under `docs/proof-2026-09.md`.
- **The API is negotiated with weed, not guessed.** bandung will send the functions it needs
  and the shapes it wants back. Expose those as the crate's public surface, `pub` and
  documented, nothing more. If weed asks for something tilth does not have, say so on the
  channel; do not build speculative API.
- **No new dependencies in the library.** Tree-sitter grammars move with it; ripgrep
  internals move only if symbol search needs them.
- **Publishing `tilth-core` to crates.io is the owner's action.** Until then weed depends on
  it as a git dependency pinned to a commit on a branch you name in your report.

## Part 2 — Zero context tax

- Move the content of `SERVER_INSTRUCTIONS` and `EDIT_MODE_EXTRA` into `SKILL.md` as the
  body; the skill's description is one sentence under 160 characters. The served
  `instructions` block becomes a pointer under 120 tokens. `AGENTS.md` stays in sync as
  CLAUDE.md requires.
- Measure before and after with the same probe the umbrella used: spawn the server, read
  `initialize.instructions` and `tools/list`, count characters over four. Record both numbers
  in your loop's Tried and in `garden.json` as the declared context cost.
- Read PR #196 first. It claims shorter prompts and a unified listing. If it does part of this
  honestly, recommend it to the owner with your review; do not duplicate it silently and do
  not merge it.

## Part 3 — The bed's papers

- `garden.json` at the repo root: kind capability and channel; faces cli, mcp, skill; install
  per platform (cargo, npm wrapper, release binaries); metric command (the copeca
  cost-per-correct scenario); declared context cost; no check face.
- Fit contract F1, F3, F4, F5, F6 from `building-the-garden.md` §3, verified by the umbrella
  with `scripts/fit/tilth.sh`. F3 means a clean machine installs tilth alone and its suite
  passes. F6 means `.brand/products/tilth/` exists and the page passes the petals check with a
  footer listing every bed.
- `docs/tend2/` map in the repo with your module loops, shaped before code.

## Part 4 — The measurement that decides tilth's shape

The garden's rule is capabilities as CLIs behind a skill, channels as MCP. Nobody knows
whether tilth as CLI-plus-skill keeps the cost-per-correct gain it measured as an MCP server.
Draft the copeca scenario for the copeca agent to run: the navigation corpus, Sonnet, three
reps, arm A tilth as MCP with the pointer instructions, arm B tilth as CLI with the skill,
baseline built-in tools. Pre-register: CLI-plus-skill becomes the documented default only if
its cost per correct is within 5 percent of the MCP arm; otherwise MCP stays default and the
skill is the fallback. Write the brief; do not run it.

## Part 5 — The installer

`tilth install <host>` stays as a fallback. Once the stem's bundles exist it stops being the
documented primary install path. Nothing to build now; note it in your loop's Tried and do
not extend the host table.

## Coordination

- **bandung (weed):** owns the API request for `tilth-core`; you own landing it. Disagreements
  on the seam go to cape-town.
- **cape-town (umbrella):** milestones, boundary questions, anything involving a bed not on
  pollen, the fit script, the manifest schema.
- **copeca:** runs your A/B when its agent exists; until then the brief waits in your repo.
- Nobody else. tend2, pleach, umbel, petals have no seam with tilth beyond the fit contract.

## Milestones to report over pollen to cape-town

1. Fresh workspace on `main`, `tend2 init`, module loops shaped, open PRs triaged with a
   recommendation for #196.
2. `tilth-core` workspace split on a branch, suite green on stable clippy, byte-identical
   output proof, the pinned commit weed can depend on.
3. Skill written, instructions block reduced, context cost measured before and after.
4. `garden.json`, brand layer verified or authored, fit checks you believe pass and the
   command proving each.
5. The copeca A/B brief drafted.

## Reserved for the owner

Merging anything; tagging and publishing, including `tilth-core` on crates.io; the disposition
of PR #196 and #169; widening tilth's stated identity on the README and page from "code
reading" to "code reading and the parser the judge uses".

## Acceptance

- `cargo fmt --check`, `cargo clippy -- -D warnings` on stable, `cargo test` green in the
  workspace, both crates.
- Byte-identical output proof committed.
- Served instructions under 120 tokens; `SKILL.md` carries what they carried; both numbers
  recorded.
- `garden.json` present; the crate's own map present; brand layer verified.
- Nothing merged, tagged, pushed to main, or published.
- Final message: what was built, test output, the two context numbers, the pinned commit for
  weed, the PR #196 recommendation, and what you could not verify.
