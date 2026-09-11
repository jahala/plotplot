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

- **Every change lands by pull request.** A ruleset on `master` requires the `garden` check; a
  direct push is refused. Branch → push → `gh pr create` → `gh pr checks <n> --watch --exit-status`
  → `gh pr merge <n> --merge`, in one chain (the law: on green is a command chain).
- **Run pinned tools, never the machine's links.** `pleach` and `tend2` on PATH point into agents'
  live checkouts. Conduct with a pinned master clone (`bun <clone>/src/main.ts`), verify and lint
  with a pinned tend2 build (`node <clone>/dist/cli.js`, through a shim dir on PATH), re-pin after
  each tool landing; until the lock pins the tools (issue 39).
- **Conducting a node here:** a hand-written `docs/dogfood/<loop>/plan.json`, one phased node
  closing one check; setup builds the stem and runs `plotplot lock verify` so a fresh worktree
  carries its judges; the test requires the evidence file then runs it; smoke `weeder check
  --strict`; audit `tend2 verify … --audit-egress` relayed by a second provider. A test that is red
  by design cannot pass the green gate; land its script from the quarantine branch and say so.
- **Evidence scripts** compose `scripts/fit/lib.sh`, print one line per claim, exit 0/1/3, and
  answer `unevaluable` with the claim's name for a claim they do not carry.

- Edit `.brand/*.md` → regenerate tokens → tag when beds must follow.
- Loops: `tend2 lint docs/tend2/*.tend2.html` clean before any PR; only `tend2 verify` stamps.
- Delete with `trash`, never `rm`. Never `git reset`. Do not push tags, publish or rename
  repositories: the owner's actions.
- The umbrella agent on pollen is `cape-town`; settlements between beds are recorded in the
  loops' Tried the same day.
