# Architecture of the umbrella: the shape, and every decision with its reason

The core page (`AGENTS.md`) states the shape in a few lines. This page holds the long form and
the decisions, each with the alternative turned down, so nobody decides them again
differently. Written 2026-09-24 from the project notes this page replaces.

## The shape, in full

- The umbrella is the root of a garden of independent tools ("beds"), four rows by question
  (plan, read, run, judge; law §0). It owns what no bed owns: the law, the contracts, the
  brand, the fit runner, the stem and the map of fit loops.
- `contracts/` defines each contract once as a schema or a page with a fixture and a test; beds
  cite the page in their own tests, which is the seam rule (law §8).
- `scripts/fit/` verifies each bed from here against the version `garden.lock` pins, on a
  clean PATH with no other garden tool present, so a bed's independence is proven and not
  assumed.
- The stem (`src/`) is the one binary the owner installs: it plants beds into any harness, runs
  every planted gate as one SARIF log, verifies the lock, redacts, seals receipts.
- The map (`docs/tend2/`) holds only fit loops for beds and the umbrella's own loops; a bed's
  module checks live on the bed's map.

## Decisions

- **Standards over schemas of our own.** SARIF 2.1.0 for findings, in-toto Statement v1 for
  receipts and proof records, an OpenTelemetry GenAI profile for friction. Turned down: a
  findings schema and a receipt schema of our own, because every editor and CI already reads
  the standards.
- **`.brand/*.md` is canonical; `tokens.css` and `tokens.json` are derived** and regenerated,
  never edited as the source. Turned down: tokens as the source, because prose carries the
  reasons a token cannot.
- **Beds pin this repository by tag** (`v1.x`); a change a bed must react to moves the major.
  Turned down: beds reading `master`, because a brand or contract change would then land on
  every bed unannounced.
- **No scheduler in any bed.** Time is external; cadence commands are idempotent CLIs. Turned
  down: a daemon, because it is a server and the substrate is files.
- **Where work lives.** Unplanned work is an issue; planned work is a loop with checks written
  first, with a plan file when it needs pages; the issue points at the loop. Turned down: work
  tracked in issues alone, because an issue cannot be proven.
- **The proof record** (`contracts/delivery.md`, "What a gate may trust"): a tree is proven
  once and later gates read the record on an identical key. Turned down: four executions per
  node, which cost hours where evidence is slow, and a cache keyed by anything less than the
  tree, the claim's words and the tool pins.
- **The runner seam** (`contracts/runner.md`): wait reasons and exit codes are one table both
  umbel and pleach test against; a non-zero exit with a reason is a result, and a spawn that
  needs a person exits 126. Turned down: each side classifying the other's reasons by guess,
  which lost four verified trees in one night.
- **Redaction is one face** (`plotplot redact`, `contracts/redaction.md`) every tool calls
  before writing a record. Turned down: each bed's own list of patterns.
- **Every bed keeps its references** (law §3, F7): a core page within 150 lines, an
  architecture and an engineering document, cited paths that resolve. Turned down: one
  growing instruction file, which nobody reads past the first screen.
- **A proof runs once per tree** and the audit keeps its judgement; a scheduled pass hunts
  flaky evidence (issue 121). Turned down: trusting a worker's own run.

## Boundaries with the beds

Settled boundaries are in the law §6a. Proposed ones between tend, graft and stake are in
`docs/ideas/stake.md` until the owner settles them.
