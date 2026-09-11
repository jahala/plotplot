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
| judge | weeder, copeca | is the diff honest, did any of it help |

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
   Independence of information is derived by code and re-derived at verify time, never
   declared by an agent: an audit packet is what code derives from the pinned corpus, the
   verbatim hunks git produces, the gate's own printed finding and its catalogue text, and
   never a word the builder wrote; an audit whose packet differs from what code re-derives,
   or whose transcript shows any input beyond the packet, is refused. A packet must also be
   fair: it carries enough of the commit that the auditor can judge whether the rule's claim
   is true of the change, not merely whether the change looks bad. And an audit asks only
   that question: whether a true finding was acceptable anyway is a human allowance, never
   an audited class (ruled 2026-09-06 after two blind runs split evenly on exactly that line).
   A worker that writes both the evidence and the artefact stamps itself; a conductor reads
   the artefact and re-runs the evidence rather than trusting either. (Learned 2026-09-06, when a "blind" audit's
   diffs turned out to be prose the same worker had written after reading the ledger.)
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
  file; you produce evidence, the verifier produces stamps. Evidence that executed zero tests
  is not evidence, and neither is evidence that only mocks the unit it claims to cover: a
  green run on an empty room stamps nothing (learned 2026-09-06 from a CI profile that
  compiled a timing test out).
- **Tried is a required output.** Every work order ends with dated `## Tried` lines in the
  loop you worked: what you tried, what you scoped out, what you assumed, what failed. A
  handback without them is incomplete.
- **Where work lives.** Unplanned work is an issue on the repository it concerns. Planned work
  is a loop on that repository's map, checks written first; the issue then points at the loop
  and closes when the loop closes or a Tried line scopes it out. A plan that needs pages is a
  file under `docs/plans/` in the same repository, linked from the loop; an issue body never
  carries the plan. Cross-bed work not yet built lives on the umbrella the same way (owner's
  rule, 2026-09-08).
- **The failing test is the spec.** Write it first. If you cannot make it fail, you do not yet
  understand the change.
- **No stubs, mocks, TODOs, or fallbacks in committed code.** weeder will refuse them; do not
  make it.
- **Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash.** Merge your
  own pull request once every gate is green (the verifier's stamps, the second-provider audit,
  weeder), with a merge commit so the narrative survives; the umbrella's own pull requests are the
  umbrella agent's to merge. "On green" is a command chain, not a habit: the merge runs only
  after the check watch has succeeded, in the same chain, and a landing script the loops call
  carries that chain (learned 2026-09-10, a hotfix merged on a red map gate because a `;` stood
  where `&&` belonged). The platform's required-check ruleset, once the stem projects it, is the
  gate no chain can bypass. Tags, publishing, renaming repositories and enabling services stay
  the owner's actions (owner's rule, 2026-09-09).
- **Never kill what you did not start.** Several conductors share one machine. Before you stop
  a session, a worktree or a process, read who owns it (umbel's `meta.json` names the working
  directory); if it is not yours, leave it and tell the umbrella on pollen. A kill costs another
  agent its node and the record of what it was doing (learned 2026-09-08, jahala/umbel 65).
- **A guardrail changes only by allowance.** The hooks, the harness settings, the lockfile and
  the hard limits are guardrail files, and weeder refuses a change to them, including the change
  the stem itself makes when it plants a repository. A person authorizes such a change with a
  `Weeder-allow` trailer on the commit, judged at commit-msg where the message first exists.
  There is no exemption for a tool's own writes: a judge that asks the planter about the
  planter's files is the planter allowing itself (learned 2026-09-10, the first planting).

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
- **weeder** is the judge of the diff, in its own repository, Rust, depending on a `tilth-core`
  library crate as a git dependency until that crate is published. Static rules to SARIF
  first; `guard` via git hooks second; `bite` third, once pleach commits the test phase
  separately. Only unambiguous rules block. Its first distribution is a GitHub Action posting
  SARIF to code scanning. The full brief is `docs/prompts/weeder-build-2026-09.md`, amended by
  this page where they differ: SARIF, not a findings schema; guard replaces hedge.
- **pleach** commits the test phase separately from the implementation, adopts `weeder check
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

- **weeder and tend2.** `weeder check` judges a diff and may block. `weeder scan` reads the tree,
  never blocks, and carries the repo-state hygiene rules: cited path or symbol missing, dead
  exports, TODO age, dependency lag. tend2 keeps the hygiene loop pattern and doc-evidence
  staleness. `tend2 gate` (are the touched claims still proven) and weeder (is the diff honest)
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

Rewritten 2026-09-11 after a week in which the beds landed features by their own maps while the
umbrella's proofs that they form one garden stayed prose: twelve of the fourteen fit scripts the
umbrella's loops name did not exist, and the seam between two tools carried eleven issues before
anyone tested it. The order below is the correction.

**The umbrella's open checks are the program.** A bed's builder is dispatched only for work that
turns an open check on the umbrella map green (a fit check or a seam check), or for a defect in
the first tier, one that loses work or kills a worker. Milestones inside a bed do not run ahead
of the umbrella's checks; `tend2 next` on the umbrella names what the garden does next.

1. **The umbrella builds its own evidence first.** The per-bed fit scripts (`scripts/fit/<bed>.sh`
   composing `scripts/fit/lib.sh`), the seam tests on the contracts loop, and the proof run
   (`scripts/fit/proof.sh`), conducted on this repository through tend2, pleach and umbel, one
   node at a time. Until these exist the map cannot say which bed belongs, and nothing else is
   dispatched except tier one.
2. **A seam gets its contract before either side moves.** One page under `contracts/` and one
   test both tools run in their own CI against the other's pinned version: the plan and delivery
   between tend2 and pleach, the worker lifecycle and usage between pleach and umbel, hooks and
   artifacts between weeder and the stem, the instruction file between tend2 and the stem, the
   channel's gate and liveness between pollen and everyone. Kill criterion per seam: if a seam
   test has not caught one real drift in a season, it is a fixture and comes out.
3. **Beds work down their fit checks, then tier one, then nothing.** Weeder's fourteen open,
   pleach's eleven, tend2's seven, in the order the umbrella's map routes them; the first-tier
   defects (a worker killed, work lost, a record gone) run beside them. Second-tier (wrong) and
   third-tier (expensive) defects wait for a green fit loop on their tool.
4. **The proof run is the wave's gate.** A wave ends when `scripts/fit/proof.sh` is green on the
   pinned season: a fixture repository planted, a plan conducted with a planted test deletion
   refused, receipts sealed, doctor live on a fresh clone. No wave ends on landings alone.
5. **Measurement follows the record.** Usage reported per node (umbel), cost per verified claim
   with cache reads priced apart (tend2), then the one question that decides the default profile:
   does a planted repository lower cost per correct answer. Withdrawn until a yardstick exists;
   revived the day one does.

Pacing: one agent runs at a time, two only when a seam test says two tools must move together,
and the window is the owner's.

Kill criteria that travel with the order: a fit script that cannot be made honest for a bed says
the bed does not fit, not that the script is wrong; a seam whose two sides cannot agree on one
page is two beds that should be one or none; a wave whose proof run stays red for two seasons
is a garden that does not work end to end, whatever its loops say.

## 9. Decisions reserved for the owner

The name and accent of weeder; MIT for weeder and pollen; the `plotplot` and `@plotplot/*` npm
names; whether receipts commit by default; whether petals lives with the umbrella or alone;
when this page and the thesis lock into `CLAUDE.md`.
