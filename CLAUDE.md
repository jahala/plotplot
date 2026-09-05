# plotplot.ai — project notes for agents

This repo is the **umbrella landing page** for plotplot and the **canonical brand**
for the garden. Read this before changing the page or the brand.

## What plotplot is

- An umbrella/org for a garden of small, sharp tools for **building with AI** — not a
  single product, and **not** the older "Company Coherence Platform / strategy-OS"
  framing (that was superseded; if you find it in old assets, it's stale).
- Tagline: **Grow what matters.** Audience: everyone building with AI.
- The beds: **tilth** (code intelligence, live), **tend** (feature mapping +
  narration), **petals** (brand intelligence for agents), **pleach** (the conductor —
  gates agent work and ships only what's verified), **umbel** (fan out agent CLIs in
  tmux — renamed from rctrl), **copeca** (cost-per-correct benchmarking), **pollen**
  (agent-to-agent messaging, with a human at the trust gate — adopted from
  walkie-clawkie; see `docs/pollen-conversion-plan-2026-09.md`).

## Architecture / tech (decisions)

- **Static single page**, no build step. `public/index.html` is the whole site;
  `public/plotplot-mark.svg` is the favicon (a deploy copy of the canonical
  `.brand/assets/plotplot-mark.svg`).
- Brand **tokens are inlined** in the page `:root` (mirrors `.brand/tokens.css`) so the
  page is self-contained and `/petals check` can resolve `var()` contrast pairs. Fonts
  load from the Google Fonts CDN.
- **Cloudflare Workers Static Assets** (`wrangler.jsonc`, `assets → ./public`,
  `account_id` set). Manual deploy: `npm run deploy`.
- **Auto-deploy:** `.github/workflows/deploy.yml` runs `cloudflare/wrangler-action`
  (`wrangler deploy`) on push to `main`/`master`. Needs repo secret
  `CLOUDFLARE_API_TOKEN`.
- **Domains:** plotplot.ai + plotplot.io (apex + www) are `custom_domain` routes in
  `wrangler.jsonc`, **live and attached** — both zones are Active on Cloudflare and
  serve this Worker. (Gotcha: adding a route onto an *inactive* zone fails the whole
  deploy — only attach once a zone is Active.)

## Brand = petals (decisions)

- The brand is authored in **petals' structure**; `.brand/*.md` is **canonical** and
  `tokens.css` / `tokens.json` are **derived** (regenerate via `/petals update`; never
  treat the tokens as the source).
- petals is vendored at `petals/`. It does **not invent** brand values — unknowns are
  **flagged**, never guessed.
- **Every page must pass `/petals check`** (`npm run check` →
  `bash petals/scripts/check.sh public/index.html`). 0 errors is the gate; we ship 0/0.
- Personality dial = **playful** ("playful garden, not boring old garden"). DESIGN.md
  §7–§8 allow a mascot + characterful botanicals (the sprout mark, the inchworm).
  Playful in craft, never hype in claims.

## Visual system (short version; full rules in `.brand/`)

- **Palette** (measured — `scripts/palette_contrast.py`): paper `#FAF5E9`, ink
  `#3A2718`, growth green `#357E2C` (primary), forest `#214A2C`, sunlight `#E89227`
  (accent/fills), leaf `#4A9E3F` (decorative), a bloom per product, soil-night
  `#1C1610`.
  - **Legibility rule:** sunlight, leaf, petal, and muted are display/decorative only
    — never body text on paper.
- **Type:** Fraunces (display) × Hanken Grotesk (body) × JetBrains Mono (code/labels),
  base 18px.
- **Voice:** calm · precise · literate · a little wit. Product names are **lowercase
  always**. Sentence case. No exclamation marks. Never the forbidden hype words (see
  `.brand/voice.md`).
- **Motion:** one idea — things *unfold* (`ease-petal`, no bounce/confetti). Honor
  `prefers-reduced-motion`.

## Gotchas (learned the hard way)

- **`check.sh` reads anchors as hex:** `href="#beds"` parses as the color `#bed`. Avoid
  anchors that look like 3/6-digit hex (we use `#garden`).
- **`check.sh`'s `var()` contrast resolver takes the LAST `--var` definition** — i.e.
  the `[data-theme="dark"]` values — so it validates *dark-mode* contrast. Keep both
  light and dark legible.
- Keep `padding` / `margin` / `gap` on the **4px scale** (use the `--pp-space-*` vars to
  sidestep the check), radius from the scale, `@media` only at **880 / 520**, and
  shadows **ink-tinted** — never pure black, even in dark mode (use a warm near-black,
  not `rgba(0,0,0)`).
- The **inchworm** is a JS `requestAnimationFrame` two-anchor gait (one foot planted
  while the other loops; the body bends, it does not stretch). Hidden under
  `prefers-reduced-motion`; no-JS shows a static worm.
- Reveal/motion is **progressive enhancement** (gated on a `.js` class) so content
  shows without JS.

## Open decisions (resolve before / at launch)

- None — page, brand, CI auto-deploy, and both custom domains (plotplot.ai +
  plotplot.io, apex + www) are live.

## Working here

- Edit `.brand/*.md` (canonical) → regenerate tokens → update `public/index.html` to use
  the vars → `npm run check` → preview.
- Reports/plans go in `docs/`; scripts go in `scripts/`.
