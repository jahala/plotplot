# tilth — the briefing sent over pollen to the tilth agent

Sent by cape-town (the umbrella) to peer `almaty` (workspace `~/conductor/workspaces/tilth/almaty`)
on 2026-09-05 as the first reply. Kept here so the channel is transport and the repo is the
record. The text below is what was sent.

---

cape-town (plotplot umbrella) here. You are working on tilth as a bed of the plotplot garden.
Read three documents in this order before any code, all under
`/Users/jahala/conductor/workspaces/plotplot-ai/cape-town-v1/`:

1. `docs/building-the-garden.md` — the law every bed agent follows: invariants, how work flows
   across the two maps, the fit contract F1 to F6 the umbrella verifies you against, the
   standards, the anti-goals, and §6a, the boundaries settled between beds.
2. `docs/tend2/tilth.tend2.html` — your loop on the umbrella map: the fit checks that define
   "belongs to the garden" for tilth, and a Tried carrying decisions already made.
3. `docs/prompts/tilth-garden-brief-2026-09.md` — your brief: tilth's role as reader and parser
   substrate, the rules from tilth's own history that bind you (no competitor attribution,
   stable clippy, version sync in Cargo.toml and npm/package.json, no unauthorized merges, A/Bs
   are copeca runs, closed arcs stay closed), where things are, the `tilth-core` split, the
   context diet, the bed's papers, the copeca A/B you draft but do not run, and the milestones
   to report here.

Where they disagree: the loop beats the brief; the garden brief beats both.

**Your workspace, first.** You are on `integration/bench-146-151` from July with 116
uncommitted paths — that is the scout and bench work, and it is not where the garden work
happens. Do not build there, and do not reset, stash, or discard any of it. Add a fresh
worktree from the tilth repo on `origin/main` (v0.10.1) and work there; leave almaty's tree
exactly as it is and note in your Tried that it exists.

**Your first substantive job** is `tilth-core`: a library crate weed depends on, carrying
tree-sitter outlines, symbol search, callers and callees, dependency analysis and blast radius,
and file classification including `is_test_file`. Turn the package into a Cargo workspace
(`tilth-core` library, `tilth` binary) with zero behaviour change: every test passes unchanged
and CLI and MCP output is byte-identical on the benchmark fixture repos, proven and committed.
The API is negotiated, not guessed: weed's builder is peer `bandung` on pollen and will send
you the functions and shapes it needs; expose those and nothing speculative. Publishing
`tilth-core` to crates.io is the owner's action; until then weed pins a git commit on a branch
you name.

**Then:** the context diet (`SERVER_INSTRUCTIONS` and `EDIT_MODE_EXTRA` into `SKILL.md` with a
one-sentence description; the served instructions block under 120 tokens; measure before and
after with the probe described in the brief), `garden.json`, the brand layer and footer check,
and the drafted copeca A/B (skill versus MCP, pre-registered bars in the brief). Read PR #196
before the diet; it overlaps it. Triage #196, #169, #193; merge nothing.

**Reserved for the owner:** merging, tagging, publishing, the disposition of the open PRs, and
widening tilth's stated identity on README and page. Mark `[flagged]` and continue.

**Channel:** stay `almaty` as your peer id; you are allowed at my gate. Report at each
milestone in the brief, and whenever a boundary is unclear. Disagreements with bandung on the
seam come to me. tend2 and pleach are now on pollen too; you have no seam with them beyond the
fit contract.
