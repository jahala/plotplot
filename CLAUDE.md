# plotplot — the umbrella (project notes for agents)

This repository is the **root of the plotplot garden**: the canonical brand, the law bed agents
follow, the contracts every bed speaks, the map of fit loops, the plans and the briefs. The stem
binary (`plotplot`) will live here too. The website plotplot.ai lives in `jahala/plotplot-ai` and
pulls the brand from here by tag, like every bed's page. Read this before changing anything.

## What plotplot is

- An umbrella for a garden of small, sharp tools for **building with AI**. Motto: **the repo is
  the harness**. Tagline: **Grow what matters.** Agents produce; code decides.
- The beds: **tilth** (code intelligence), **tend** (loops and the verifier), **petals** (brand
  intelligence for agents), **pleach** (the conductor), **umbel** (fan out agent CLIs in tmux),
  **copeca** (cost per correct), **pollen** (agent-to-agent messaging with a human at the gate),
  **weeder** (the judge of the diff). Unbuilt, planned here: the friction ledger, receipts, the
  stem.

## What lives where (decisions)

- `.brand/*.md` is **canonical**; `tokens.css` / `tokens.json` are **derived** (regenerate via
  `/petals update`; never edit tokens by hand as the source).
- `docs/building-the-garden.md` is the law. `docs/tend2/` is the umbrella map: fit checks only,
  never a bed's module checks. `docs/plans/` holds plans; `docs/prompts/` holds briefs and
  transcripts of messages sent (transcripts are never rewritten).
- `contracts/` is the umbrella's own product: manifest and lock schemas, the friction and
  identifiers profiles, fixtures, tests. Standards over schemas: SARIF 2.1.0 for findings,
  in-toto statements for receipts, an OpenTelemetry GenAI profile for friction.
- **Where work lives:** unplanned work is a GitHub issue; planned work is a loop in `docs/tend2/`
  with checks written first (plus a file in `docs/plans/` when it needs pages); the issue points
  at the loop.
- Beds pin this repository by tag (`v1.x`); a change a bed must react to moves the major.
- No scheduler in any bed. Time is external; cadence commands are idempotent CLIs.

## Brand (short version; full rules in `.brand/`)

- Palette: paper `#FAF5E9`, ink `#3A2718`, growth green `#357E2C` (primary), forest `#214A2C`,
  sunlight `#E89227` (accent), leaf `#4A9E3F` (decorative), a bloom per product, soil-night
  `#1C1610`. Sunlight, leaf, petal and muted are display only, never body text on paper.
- Type: Fraunces × Hanken Grotesk × JetBrains Mono, base 18px. Voice: calm, precise, literate, a
  little wit; product names lowercase always; sentence case; no exclamation marks; no em dashes in
  copy; never the forbidden words in `.brand/voice.md`.
- Every bed page passes `petals check` at 0 errors. Gotchas for page builders: anchors that look
  like hex parse as colours; the `var()` resolver takes the last definition, so it validates dark
  mode; spacing on the 4px scale; `@media` only at 880 and 520; shadows ink-tinted.

## Working here

- Edit `.brand/*.md` → regenerate tokens → tag when beds must follow.
- Loops: `tend2 lint docs/tend2/*.tend2.html` clean before any PR; only `tend2 verify` stamps.
- Delete with `trash`, never `rm`. Never `git reset`. Do not push tags, publish or rename
  repositories: the owner's actions.
- The umbrella agent on pollen is `cape-town`; settlements between beds are recorded in the
  loops' Tried the same day.
