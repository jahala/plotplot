# weed — product identity

A product layer over the plotplot umbrella. Deltas only; everything absent inherits the umbrella.

| Field | Value |
|---|---|
| Product | weed |
| Tagline | tests still mean what they meant. |
| Accent | #8E3B5E bramble — the weed that stops you (from the umbrella Product Accents table) |
| Commands | `weed check` · `weed scan` · `weed guard` · `weed bite` |

## Positioning

The judge of the diff. Agents produce; weed decides whether what they produced is honest
work: a test deleted or weakened, a skip added, a stub left in production code, an error
swallowed, a secret pasted in, a guardrail edited, an import crossing a boundary, a file
touched outside the agreed scope. It reads the diff and the tree with a parser, never with a
model, and answers in milliseconds with an exit code and a SARIF log.

Four faces on one core. `check` judges a diff and may block. `scan` judges the tree and never
blocks. `guard` is the law in git, at commit and push, for every agent and every human alike.
`bite` proves a test still fails without the change. It sits where the work happens: the
agent's Stop hook, pleach's gate on every node, a pre-commit hook, a check on the pull request.

Only the unambiguous shapes block. Everything else is a warning aimed at the person reading
the pull request, and every allowance a person grants is itself a visible finding.

## Mark

A single bramble cane in growth green (`#357E2C`), arcing from lower left to upper right the
way the sprout mark's stem rises, with **two thorns** and **one drupelet** in bramble
(`#8E3B5E`). The thorns say what the product does: growth that catches what tries to pass.
The drupelet says it is still a garden plant, not a fence. It is drawn in the garden's diagram
language, stroked stem and filled accent, so it sits beside pleach's weave and pollen's grains
as a sibling. The cane leans **against** the direction of reading; a bramble is something you
meet, not something you follow.

Files: `assets/weed-mark.svg` (paper) · `assets/weed-mark-night.svg` (soil-night).

## Logo usage

- Minimum size: 16px mark height (checked at 16 · 20 · 28 · 48). Below 20, drop the drupelet
  and keep the thorns; below 16, drop one thorn rather than shrink everything.
- Clear space: half the mark height on every side.
- Fills are exact — bramble `#8E3B5E` and growth green `#357E2C` on paper; on soil-night the
  cane brightens to `#84C56A` and bramble stays `#8E3B5E` as a fill while any bramble **word**
  lifts to `#B85C82` (the night file). Never recolour outside these four values.
- Never: add a third thorn, curl the cane into a hook or a checkmark, set the thorns in red
  (that is the error colour's job), outline the wordmark, or set the wordmark in anything but
  the body face.
