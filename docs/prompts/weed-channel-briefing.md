# weed — the briefing sent over pollen to the builder

Note: weed was renamed weeder on 2026-09-06; this is the message as sent.

Sent by cape-town (the umbrella) to peer `bandung` (workspace `~/conductor/workspaces/weed/bandung`)
on 2026-09-05 as the first reply. Kept here so the channel is transport and the repo is the
record. The text below is what was sent.

---

cape-town (plotplot umbrella) here. You are building weed, the judge of the diff, a bed of the
plotplot garden. Read three documents in this order before any code, all under
`/Users/jahala/conductor/workspaces/plotplot-ai/cape-town-v1/`:

1. `docs/building-the-garden.md` — the law every bed agent follows: invariants, how work flows
   across the two maps, the fit contract F1 to F6 the umbrella verifies you against, the
   standards to use (SARIF 2.1.0 for findings, never a schema of ours), the anti-goals, and
   §6a, the boundaries settled with tend2 today.
2. `docs/tend2/weed.tend2.html` — your loop on the umbrella map. Its checks are what "belongs
   to the garden" means for weed; its Tried carries every decision already made so you do not
   re-argue them.
3. `docs/prompts/weed-build-2026-09.md` — the build brief: what weed is (three faces on one
   core: `check` on the diff and may block, `scan` on the tree and never blocks, `guard` as
   git-hook law, plus `bite`), the rule catalogue with fixtures per language, calibration with
   ship and kill bars, milestones to report here.

Where they disagree: the loop beats the build brief; the garden brief beats both.

**What.** A static binary that reads what an agent produced and refuses dishonest growth —
deleted or weakened tests, skips, stubs, swallowed errors, secrets, guardrail edits,
dependency-direction violations — in milliseconds, zero tokens, every finding as SARIF. Faces:
pre-commit and pre-push via git hooks, the harness Stop hook, pleach's smoke gate, a GitHub
Action posting to code scanning.

**Who.** Maintainers drowning in agent-authored pull requests who need a deterministic gate
before reading; teams running agents on several harnesses who want one law; the garden's own
fleets through pleach.

**Where.** weed lives in its own repository, the one the builder's workspace holds, not inside
the tilth Cargo workspace. Rust, static binary, own release pipeline copied from tilth's
(multi-platform binaries, crates.io, npm wrapper). It depends on a `tilth-core` library crate
for tree-sitter outlines, symbol search, test-file classification and dependency analysis, as
a git dependency on the tilth repo until that crate is published. tilth-core does not exist
yet: the first milestone is the extraction proposal in a branch of
`/Users/jahala/CascadeProjects/tilth` (modules named in the brief's "Where it lives" section),
tilth's own suite still green, reported here before building on it. If rules must start
before that lands, all parsing is isolated behind one internal seam so the swap is mechanical,
recorded in Tried.

**Tech constraints.** Rust; zero runtime tokens; SARIF 2.1.0 validating against the official
schema; exit codes 0/2/3 with fail-closed 3; only unambiguous rules block (T1, T3, S1, X1, C1,
G1), everything else warns and warnings are for the human at the PR, not the agent; fixtures
in TypeScript/JavaScript, Python, Rust, Go; nothing mocked; failing fixture before every rule.

**Acceptance.** The loop's checks plus the brief's Acceptance section. Calibration over 200
merged commits in five garden repos with under 2 percent block-level false positives per repo
is the ship bar; needing to disable T1, T2, T3 or S1 to get there is the kill bar, reported
rather than softened.

**First steps, in order.** Read the three docs; `tend2 init docs/tend2` in the repo and shape
module loops there before code (the umbrella map holds fit, the bed's map holds modules);
tilth-core extraction proposal; T1, T3, S1, X1, C1, G1 to SARIF; the rest of the catalogue;
calibration.

**Reserved for the owner.** The name weed, its accent, MIT, anything on GitHub (push, publish,
Pages). Marked `[flagged]`, work continues.

**Channel.** The builder stays `bandung` as its peer id and is allowed at cape-town's gate.
Reports at each milestone in the brief's §8 and whenever a boundary is unclear. tend2 is the
first consumer for `scan`; until it appears on pollen, anything for it routes through
cape-town.
