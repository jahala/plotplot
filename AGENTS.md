# plotplot, the umbrella: the core page

The repo is the harness. This repository is the root of the plotplot garden: the law every bed
follows (`docs/building-the-garden.md`), the contracts every bed speaks (`contracts/`), the
canonical brand (`.brand/`), the fit map (`docs/tend2/`), the plans, the briefs, and the stem
binary `plotplot`. The website is `jahala/plotplot-ai` and pulls the brand from here by tag.
Long forms: `docs/architecture.md` (decisions with reasons) and `docs/engineering.md` (how
work is done and proven here). Read the law before changing anything.

## What it is for, and what it is not

For: an owner who directs agents and has to be able to trust what comes out; a worker agent on
any vendor; a tool author bringing a bed (the three persona pages on the map, pull request 137).
Not: a framework, a scheduler, a server, or a home for any bed's module checks. The umbrella
verifies beds from outside; it never builds them.

## Shape

- `contracts/` is the product: schemas (`manifest.schema.json`, `lock.schema.json`,
  `proof.schema.json`), pages (`delivery.md`, `runner.md`, `redaction.md`), pins (`pins.json`),
  fixtures and tests (`contracts/test/*.test.sh`, run by `scripts/contracts-test.sh`).
- `scripts/fit/<bed>.sh` proves a bed against the fit contract from here; `scripts/fit/lib.sh`
  is the shared runner.
- `src/` and `build.rs` are the stem, a Rust binary: `plotplot check`, `lock verify`, `redact`,
  `receipt`, `doctor`, `init`.
- `docs/tend2/` is the map: one loop per bed's fit, the contracts loop, the stem, receipts and
  friction, three persona pages and one opportunity page. State is derived; only the verifier
  writes a pass.
- Machine-written state lives under the .plotplot directory; human-authored records under
  `docs/` and root config. `garden.lock` and `.githooks/` are the stem's to write.

## Glossary

- **bed**: one independent tool in the garden, with its own repository, manifest and map.
- **stem**: the `plotplot` binary that plants beds in any harness and proves they are planted.
- **law**: `docs/building-the-garden.md`, the invariants and the fit contract every bed meets.
- **fit contract**: the checks F1 to F7 the umbrella runs against a bed's pinned release.
- **seam**: a contract two beds cite in their own tests, so a change is seen by both first.
- **loop**: a tend2 page, a bet with checks; **stamp**: a pass the verifier wrote.
- **proof record**: the verifier's record of a pass on a tree, reused by later gates.
- **receipt**: the per-commit provenance statement the stem seals as a git note.

## Invariants (the law §1, numbered there; the ones this repository enforces itself)

1. Agents produce; code decides. Done, verified, merged, on-brand and safe are decided by code
   that reads files, never by an agent's prose.
2. Files are the substrate. State in the repo, plain text, diffable. No server, no database.
3. One idea, one home. A contract is defined once and consumed everywhere; the instruction
   file points and never restates.
4. Independence of slices. Whatever judges differs from the builder in mechanism; a second
   opinion comes from another provider in a shape code can refuse.
5. Everything through the gate. Every change lands by pull request on green; the `garden`
   check is required on `master`; a direct push is refused.
6. Measured or silent. A claim carries its number and the command that prints it, or it is
   not made.

## Hard limits

- Delete with `trash`, never `rm`. Never `git reset`. Never force-push; never push tags,
  publish or rename repositories: the owner's actions.
- Never a secret in chat, a log, a file or a command line; read it by name at the moment of
  use. Fixture rows that look like credentials are base64 at rest.
- This repository is public: no vendor or competitor names, in prose, issues or comments.
- No em dashes anywhere. Plain, complete sentences.

## What done means

A loop's check is done when `tend2 verify` stamps it against its evidence script; a hand
`[x]` renders as claimed. A landing is done when its pull request merged on green with
`npm test` run first on any change under `contracts/` (it is not in CI). A bed's version is
proven when its fit script holds every claim against `garden.lock`. Records (Tried lines,
plans, briefs) are the only things written by hand; work is conducted through tend2, pleach
and umbel by the recipes in `docs/engineering.md`.

<!-- tend2:begin -->
## tend2 — this project plans on loops

- Orient first: `tend2 next docs/tend2` (or the `loop_next` MCP tool) — next up, running now, needs-you, gone stale.
- Nothing gets built that is not a loop first: shape the goal and its checks on the map before any code.
- Only `tend2 verify` writes a pass. Never hand-flip a checkbox — a naked [x] renders claimed, not proven.
- Record decisions, scope-outs and dead ends in the loop's `## Tried` — append-only memory for whoever comes next.
- The map holds bets, not maybes: ideas attached to a loop park in its `## Tried`; free-standing ideas park in `docs/ideas/`; shaping is the only transition onto the map.
- Full craft lives in the tend2 plugin skills (next, shape, run, verify, discover, change).
<!-- tend2:end -->

<!-- plotplot:begin -->
## plotplot: what is planted here

Planted: weeder 0.2.1.
Season: 2026.09.

- Orient first: `tend2 next docs/tend2` says what is next, what is running, and what went stale.
- Gates: `plotplot check` runs every planted gate and returns one SARIF log.
- Help: `plotplot doctor` says whether the law is live, and what is missing when it is not.
- The hard limits are enforced at the boundary: `--no-verify` is refused, and `garden.lock` and `.githooks/` are the stem's to write.
<!-- plotplot:end -->
