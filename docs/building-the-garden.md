# Building the garden — the brief every bed agent reads first

You are building one bed of plotplot, or the stem that plants them. This page is the shared
law for all of that work. It is self-contained; the loops in `docs/tend2/` are the work
orders; the documents it cites are the reasoning. Read this page, then your bed's loop, then
your bed's own map. Do not start from the code.

## 0. The motto and the shape

**The repo is the harness.** A tended repository carries its own intent with proof, its own
law, its own structure, a findings channel, and a yardstick, as files and git hooks. Every
agent on every vendor is a worker against that. plotplot is the set of open contracts that
define a tended repository plus the independent tools that implement them.

Read, in this order: `docs/plotplot-thesis.md` (why), `docs/tended-repository.md` (the theory
and the vital choices), `docs/plotplot-architecture-review-2026-09.md` §13 (the shape that
survived review), `docs/garden-2031.md` §5 (each tool's what, how, why). The v1 architecture
document is superseded where the review says so.

Four rows, eight beds, one stem:

| Row | Beds | Question |
|---|---|---|
| plan | tend2 | what should be built, and what does done mean |
| read | tilth, petals | how the agent reads code, design, and voice |
| run | umbel, pleach, pollen | how one unit runs, how many run gated, how workers talk |
| judge | weed, copeca | is the diff honest, did any of it help |

The stem, `plotplot`, plants all of it in any harness and proves it is planted. Two support
repos carry what no bed owns: `contracts` (schemas, fixtures, the fit runner) and `proof`
(the end-to-end run on pinned versions, the generated bundles).

## 1. Invariants

A change that breaks one of these is wrong even if every test passes.

1. **Agents produce; code decides.** Done, verified, merged, on-brand, safe: decided by
   deterministic code reading files. Where a judgment is unavoidable it comes from a second
   model on a different provider, in a shape code can refuse.
2. **Files are the substrate.** State in the repo, plain text, diffable. No server, database,
   or account. A local process may render a page.
3. **Neutral across harnesses.** Integrate only through seams every agent CLI shares: the
   filesystem, `AGENTS.md` and `SKILL.md`, lifecycle hooks, MCP over stdio, git, the shell.
4. **Standalone and composable.** Every bed has full value alone with nothing else installed.
   Composition is through contracts, never imports across beds or shared process state.
5. **Zero-tax presence.** A bed costs an agent nothing in context unless it is speaking, and
   it speaks only on evidence. Capabilities are CLIs behind a skill; channels are MCP with
   at most three tools; gates and boundaries are hooks and git hooks.
6. **Measured or silent.** Every bed declares one falsifiable metric and ships the command
   that measures it. A bed that cannot be measured does not get to speak at session start.
7. **One idea, one home.** Each contract is defined once and consumed everywhere.
8. **Independence of slices.** Anything that judges differs from the builder in mechanism,
   information source, and party. A model review sharing the builder's context is not a gate.
9. **Everything through the gate.** Repairs, harness changes, and your own work land only
   through verified checks. There is no privileged path.

## 2. How work flows, and what done means

- **Two maps.** The umbrella map in this repo (`docs/tend2/`) holds one loop per bed carrying
  the **fit checks**: does this bed belong to the garden. Your bed's own repo holds its own
  `docs/tend2/` map carrying the **module checks**: does the bed work. Parents carry
  integration, children carry modules. Never duplicate a module check into the umbrella map.
- **Shape before code.** Nothing is built that is not a loop first. If your task has no check
  a verifier can close, shape one, then build.
- **Only the verifier writes pass.** `tend2 verify` runs the evidence and stamps it. A
  hand-flipped box is a claim and renders as one. Your work order forbids touching the loop
  file; you produce evidence, the verifier produces stamps.
- **Tried is a required output.** Every work order ends with dated `## Tried` lines in the
  loop you worked: what you tried, what you scoped out, what you assumed, what failed. A
  handback without them is incomplete.
- **The failing test is the spec.** Write it first. If you cannot make it fail, you do not yet
  understand the change.
- **No stubs, mocks, TODOs, or fallbacks in committed code.** weed will refuse them; do not
  make it.
- **Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash. Do not
  push, publish, rename repositories, or enable services.** Those are the owner's actions.

## 3. What every bed ships: the fit contract

The umbrella map verifies these from this repo against the version pinned in `garden.lock`,
using `scripts/fit/<bed>.sh <check>`. Your bed must make each one pass.

| Id | Check | Applies to |
|---|---|---|
| F1 | `garden.json` at the bed's root validates against the manifest schema: name, kind, version, install per platform, faces, check command, metric command, declared context cost | every bed |
| F2 | the bed's check face emits **SARIF 2.1.0** that validates against the SARIF schema | every bed with a gate |
| F3 | on a clean machine with no other garden tool, the bed installs and its own suite passes | every bed |
| F4 | measured startup context cost is at or under the cost declared in the manifest | every bed with a skill or MCP face |
| F5 | the declared metric command runs and its latest result is committed in the bed's repo | every bed |
| F6 | `.brand/products/<bed>/` exists, the page passes the petals check with zero errors, and the garden footer lists every bed | every bed with a page |

Beyond the fit contract, every bed ships: a CLI as its primary face; a `SKILL.md` whose
description is one sentence and whose body carries everything an instruction block used to;
its own `docs/tend2/` map; the garden footer with the plotplot band on its page.

Rulings on the fit contract (2026-09-05, from tilth's questions; they bind every bed):

- **F6 and `.brand/`.** Umbrella-derived files (`.brand/*.md`, `tokens.css`, `tokens.json`)
  may stay pulled from the umbrella and ignored by git. The product layer
  `.brand/products/<bed>/` is the bed's own identity and is committed in the bed's repository;
  that is the path F6 verifies. The accent claim lives only in the umbrella's `colors.md`
  Product Accents table.
- **F5 and results.** Raw benchmark or calibration output may stay ignored. What must be
  committed is one small summary: the metric's latest value, its confidence interval, the
  date, the bed version measured, and a pointer to the signed artifact. Pages and READMEs may
  keep citing the version they measured; the summary is what must be current.
- **Skill location.** A bed chooses where its `SKILL.md` lives and declares the path in
  `garden.json`; the stem installs it into each harness's discovery directory.
- **Split crates and releases.** A bed that publishes a library crate other beds depend on
  publishes that crate before its binary; the release workflow enforces the order. Publishing
  is the owner's action.

## 4. Standards to use, never reinvent

| Need | Use | Not |
|---|---|---|
| findings from any gate | SARIF 2.1.0 | a garden findings schema |
| provenance of a change | in-toto Statement v1, signed with Sigstore | a receipt schema |
| agent telemetry, friction events | OpenTelemetry GenAI semantic conventions | a log format |
| schemas | JSON Schema draft 2020-12 | prose |
| instructions and procedures | `AGENTS.md`, `SKILL.md` | instruction blocks in server responses |
| channels | MCP over stdio, three tools and one help topic at most | a gateway |
| law at commit and push | git hooks via `core.hooksPath`, branch protection | a shell regex list |
| the filesystem fence | the harness's sandbox, seatbelt, bubblewrap | a deny list of commands |
| pinned judges | a lockfile with version and per-platform checksum, bytes fetched on first use | binaries in git, global installs |

## 5. Mandates per bed

One paragraph each; the loop carries the checks.

- **tend2** stays the center of proof. Add the verifier's identity and version to the stamp,
  additively. Make Tried a machine-checked required section of every work order. Build the
  friction ledger: hook events in OpenTelemetry shape, reduced per path, surfaced as hot spots
  in `next`. Keep the SessionStart hook as tend2's own experiment; do not build a briefing
  assembler.
- **tilth** becomes the parsing substrate as well as the reader. Move the instruction block
  into `SKILL.md`. Expose the tree-sitter core as a library crate other crates in the
  workspace depend on. Retire the host installer in favour of the stem's bundles once they
  exist; keep it as a fallback until then. Its metric stays cost per correct.
- **weed** is the judge of the diff, in its own repository, Rust, depending on a `tilth-core`
  library crate as a git dependency until that crate is published. Static rules to SARIF
  first; `guard` via git hooks second; `bite` third, once pleach commits the test phase
  separately. Only unambiguous rules block. Its first distribution is a GitHub Action posting
  SARIF to code scanning. The full brief is `docs/prompts/weed-build-2026-09.md`, amended by
  this page where they differ: SARIF, not a findings schema; guard replaces hedge.
- **pleach** commits the test phase separately from the implementation, adopts `weed check
  --strict` as its default smoke, and requires a Tried entry in every node's handback. Its
  plan schema stays the plan contract.
- **umbel** cuts to six tools plus help, adds a headless structured-output adapter alongside
  tmux, and keeps its unit signature as the unit contract.
- **pollen** is walkie-clawkie renamed and folded to three tools. The brief is
  `docs/prompts/pollen-conversion-2026-09.md`.
- **petals** cuts its skill description to one sentence and emits SARIF from its check.
- **copeca** adds `garden.json`, then `copeca init` to scaffold any repo's own eval corpus from
  its history.
- **the stem** is three generated bundles, `doctor`, `check`, `init`, and the lockfile
  wrapper. Nothing else. No `run`, no `walk`, no gateway, no briefing assembler.
- **mull** is not a bed. If its review (`docs/prompts/mull-relook-2026-09.md`) finds it lives,
  it lives as a skill that writes a loop's Tried from a session transcript.

## 6. Anti-goals

Not an app. Not a proxy. Not an MCP gateway. Not model-extracted memory. Not an
orchestration UI. Not a monorepo. Not a single runtime for everything. Not a push briefing
without a measured win. Not a rule that blocks on ambiguity. Not a scheduler in any bed:
time is deliberately external, cadence commands are idempotent CLIs, and scheduler
configuration lives in the repo as workflow files or a local cron layer. Each was argued and
cut in the review or settled on the channel; reopening one needs new evidence in a Tried line.

## 6a. Boundaries settled between beds (2026-09-05, on the channel)

- **weed and tend2.** `weed check` judges a diff and may block. `weed scan` reads the tree,
  never blocks, and carries the repo-state hygiene rules: cited path or symbol missing, dead
  exports, TODO age, dependency lag. tend2 keeps the hygiene loop pattern and doc-evidence
  staleness. `tend2 gate` (are the touched claims still proven) and weed (is the diff honest)
  are different questions; both emit SARIF, `plotplot check` concatenates them.
- **tend2 and copeca.** tend2 owns eval checks as claims: cases and thresholds in the loop,
  run by the verifier. copeca owns corpora, regression harnesses, and did-it-help measurement
  over time, plus a one-way exporter into tend2's case-file shape. tend2 never depends on
  copeca.
- **friction.** Federated: contracts pins an OpenTelemetry GenAI profile (event names,
  attributes, one required path attribute, a fixture journal); each bed emits its own
  domain's events; tend2 owns the reducer and the hot spots in `next`. No dashboard, no
  exporter.
- **tend2 and pleach.** The Tried-handback requirement is FORMAT's grammar, travels in the
  work order tend2 emits, is refused at close by tend2's verifier, and is validated in shape
  by pleach's run loop through an additive plan-contract v1.3 field pleach owns.

## 7. Handback

Your final message and your loop's Tried carry the same facts: what changed, test output
before and after, the fit checks you believe now pass and the command that proves each, the
metric result, spend if any, what you could not verify, and the decisions waiting for the
owner. No time estimates. No "phase two". No promise about work not done.

## 8. Order, and the kill criteria that travel with it

1. contracts: manifest and lock schemas, SARIF fixtures, the fit runner. Kill: if two existing
   beds cannot express their findings in SARIF without loss, stop and report.
2. tend2 to npm; petals and pleach public. The loop must be installable.
3. weed in its own repository, on `tilth-core` once tilth lands the split (weed proposes the
   API and the branch, tilth lands it). Kill: over two percent false blocks over the pooled
   merged commits of the garden repos, up to 200 each, or reaching the bar requires disabling
   the test-deletion, skip, or stub rules.
4. Stamps carry the verifier version; Tried required in pleach work orders; the one-page
   tended-repository spec published.
5. Friction ledger in tend2. Kill: a hot spot it names is not confirmed by the next
   simplification loop.
6. The stem as bundles plus lockfile wrapper, `doctor` proving hooks fire on two harnesses.
7. `copeca init`.
8. The proof repo runs the loop end to end on pinned versions, and the site tells it with real
   output.

## 9. Decisions reserved for the owner

The name and accent of weed; MIT for weed and pollen; the `plotplot` and `@plotplot/*` npm
names; whether receipts commit by default; whether petals lives with the umbrella or alone;
when this page and the thesis lock into `CLAUDE.md`.
