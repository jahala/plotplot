# weed — the landing page briefing

Note: weed was renamed weeder on 2026-09-06; this is the message as sent.

Sent by cape-town (the umbrella) to peer `bandung` on 2026-09-06, when the owner authorised
weed's landing page (the build brief had deferred it). Kept here so the channel is transport
and the repo is the record.

---

cape-town. The owner authorises weed's landing page now. It is one static file, `index.html`
at your repository root, served by GitHub Pages from `main:/` at `jahala.github.io/weed/`,
exactly as tilth, umbel, copeca and pollen publish theirs. No build step, no framework, fonts
from the Google Fonts CDN, tokens inlined. Read these before writing a line:

1. **The umbrella page**, `/Users/jahala/conductor/workspaces/plotplot-ai/cape-town-v1/public/index.html`,
   live at plotplot.ai. It is the reference for the head (title, meta, fonts, the theme
   script), the `:root` token block and its `[data-theme="dark"]` twin, the header with mark
   and wordmark and theme toggle, the hero with a kicker, an h1, a lede and two buttons beside
   a soil-night terminal pane, the band sections, and the garden footer.
2. **pleach's product page**, `/Users/jahala/conductor/repos/pleach/index.html`, about 50 KB:
   the structure a bed page follows, sections hero · why · loop · gates · proof · fit ·
   start, and how a product accent is used with conviction as the single product voice.
3. **The garden footer spec**, `.brand/components.md` in the umbrella, section "Footer — the
   garden footer", with a reference implementation you copy verbatim, changing only the
   identity column, the `.is-current` pill and the repo links. A product page adds the
   **plotplot band** above the columns; the garden row must list every bed with its bloom dot.
4. **The brand rules**: `.brand/DESIGN.md` (principles, type, spacing, motion), `.brand/colors.md`
   (the palette and the measured contrast table), `.brand/voice.md` (forbidden words,
   sentence case, lowercase product names, no exclamation marks), and your own canonical
   layer in the weed repo at `.brand/products/weed/` (identity, colours, voice, the shears
   mark in paper and night files).

**Your tokens.** Inline the umbrella's `:root` block as the umbrella page has it, and add:

```css
:root{ --pp-weed:#8E3B5E; --pp-weed-word:#8E3B5E; }
[data-theme="dark"]{ --pp-weed-word:#B85C82; }
```

Rules for bramble, measured and written in your colours.md: on paper it reads as a word
(6.57:1), so kickers, labels and rule ids may be set in `--pp-weed` directly. On soil-night
it is display-only (2.51:1): fills, dots, the mark, never text; any bramble word on dark uses
`--pp-weed-word`, which lifts to `#B85C82` there. Never set bramble-night text next to petal
text on dark. Status colours stay the umbrella's: block-level results in error `#BC4126`,
warnings in caution `#B0741C`, clean in healthy `#46913C`; bramble marks weed, never
severity, and the thorns are never red.

**The mark.** Use the canonical shears SVG from `.brand/products/weed/assets/weed-mark.svg`
inline in the header beside a lowercase `weed` wordmark in Fraunces, and its night file
under the dark theme; the plotplot band in the footer keeps the umbrella's sprout mark,
which the reference implementation carries. Minimum 20px; below that the wordmark alone.

**What the page says.** Kicker `the judge`; h1 the tagline, `tests still mean what they
meant.`; a lede that says what weed refuses and that no model is in the loop. The terminal
pane shows real output: one `weed check` run with a block-level T1 result in the table
format, and one `weed guard install`. Then sections in pleach's order: why (the failure mode:
an agent that deletes a test to get green, and why a model review cannot be the gate); what
it judges (the rule catalogue, block versus warn, only unambiguous shapes block); where it
runs (pre-commit and pre-push through guard, the Stop hook, pleach's gate, the GitHub
Action posting SARIF); the numbers (calibration on 635 pinned commits, 20 blocked, 4 false
positives, 0.63 percent; recall at or above 95 percent per rule per language; the blind
re-grade at 19 and 20 of 20; dated, with the old-class number named as the report names it);
fit (weed alone, weed with tend2 and pleach, weed beside aislop and anti-slop, and the honest
scope: it says nothing about whether the code is good); start (install by cargo, npm wrapper,
or the release binary, then `weed guard install`).

**The footer's garden row**, all eight, with `.is-current` on yours:

```html
<a href="https://jahala.github.io/tilth/"  style="--bloom:var(--pp-tilth)"><span class="gf-dot"></span>tilth</a>
<a href="https://jahala.github.io/tend/"   style="--bloom:var(--pp-tend)"><span class="gf-dot"></span>tend</a>
<a href="https://jahala.github.io/petals/" style="--bloom:var(--pp-petals)"><span class="gf-dot"></span>petals</a>
<a href="https://jahala.github.io/pleach/" style="--bloom:var(--pp-pleach)"><span class="gf-dot"></span>pleach</a>
<a href="https://jahala.github.io/umbel/"  style="--bloom:var(--pp-umbel)"><span class="gf-dot"></span>umbel</a>
<a href="https://jahala.github.io/copeca/" style="--bloom:var(--pp-copeca)"><span class="gf-dot"></span>copeca</a>
<a href="https://jahala.github.io/pollen/" style="--bloom:var(--pp-pollen)"><span class="gf-dot"></span>pollen</a>
<a href="https://jahala.github.io/weed/"   style="--bloom:var(--pp-weed)" class="is-current"><span class="gf-dot"></span>weed</a>
```

**Traps the brand check will catch**, from the umbrella's CLAUDE.md, learned the hard way:

- Anchors that look like hex parse as colours: never `href="#bed"` or `#beds`; the umbrella
  uses `#garden`.
- The `var()` contrast resolver takes the **last** definition of each variable, so it
  validates the dark theme; keep both themes legible and define the dark overrides after
  `:root`.
- Spacing only on the 4px scale, through the `--pp-space-*` variables; radius from the scale;
  `@media` only at 880 and 520; shadows ink-tinted, never pure black, in both themes.
- Motion is one idea, things unfold with `--pp-ease-petal`; honour `prefers-reduced-motion`;
  reveal is progressive enhancement gated on a `.js` class so content shows without JS.
- Voice: sentence case everywhere, product names lowercase, no exclamation marks, none of the
  forbidden words, no em dashes in the copy (the umbrella page has none now), and every
  number on the page true to your calibration report on the day you write it.

**Gate.** `bash petals/scripts/check.sh index.html` with the umbrella's `.brand` fetched
beside your product layer must report 0 errors and 0 warnings; your `scripts/check/brand.sh`
already does the fetch. Add the page to your map as its own loop with that check, a check
that the garden row lists all eight beds, and a human check for the owner's approval of the
page. Report here when it passes.
