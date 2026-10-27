# Typography — plotplot

## Font Families

| Role | Font Family | Weight | Style |
|------|------------|--------|-------|
| Headings | Protest Riot | 400 | normal |
| Body | Hanken Grotesk | 400, 500, 600, 700 | normal |
| Interface | Hanken Grotesk | 400, 500, 600 | normal |
| Code | JetBrains Mono | 400, 500, 700 | normal |

## Font Weights

| Weight | Value | Usage |
|--------|-------|-------|
| Regular | 400 | Body, interface |
| Medium | 500 | Labels |
| Semibold | 600 | Emphasis, buttons |
| Bold | 700 | Strong body emphasis, stat numerals |

Protest Riot has one weight and no italic. Headings set it as drawn: weight 400, normal style, letter-spacing 0, and `font-synthesis: none`, so a browser never fakes a bold or a slant. A heading's one live word stands out by colour or a bloom underline, never by weight or italic. The weights above belong to Hanken Grotesk and JetBrains Mono.

## Type Scale

Base: `html { font-size: 18px }`. Sizes are in `rem` for fluid scaling; the clamp on h1 keeps the hero balanced across viewports.

| Level | Size | Line Height | Weight | Usage |
|-------|------|------------|--------|-------|
| h1 | clamp(3.4rem, 6vw, 4.6rem) | 1.04 | 400 | Page / hero titles |
| h2 | 2.4rem | 1.12 | 400 | Section headings |
| h3 | 1.6rem | 1.2 | 400 | Subsection headings |
| h4 | 1.25rem | 1.3 | 400 | Card titles |
| lead | 1.3rem | 1.5 | 400 | Hero deck, section ledes |
| body | 1.0625rem | 1.65 | 400 | Body text |
| small | 0.9375rem | 1.5 | 400 | Captions, labels |
| stat | 3.75rem | 1.0 | 700 | Display numerals |
| code | 0.95rem | 1.6 | 400 | Code / terminal text |

### Display pairings

- **Hero title.** The first line at 80% of the h1 scale (`clamp(2.72rem, 5.12vw, 4.4rem)`), and the one live word in growth green on its own line at 1.75em of that. The live word is lowercase, so it needs this margin to read as larger: its x-height must clear the first line's cap height. At the same size it reads only as heavier. The two lines are set at `line-height: 1.04` so they do not touch. No full stop after a title set this way.
- **Allotment-plan pages.** The reference pages set the section title (h2) at `clamp(2.2rem, 3.6vw, 3rem)`, line-height 1.05; the lede at 1.2rem (1.3rem in the umbrella hero, 1.15rem on a bed page); a bed page's hero title at `clamp(3.4rem, 7.2vw, 6.4rem)`, line-height .95, its one live word in the bed's word colour. A bed whose bloom is too pale to read as a word (tend, tilth) sets the word in ink with a bloom underline: `.11em` thick, `.12em` below. These lead the scale above where they differ.
- **Full stop.** A title set as a split pairing (one line, then the live word on its own line) takes no full stop. A title that is one sentence on the line keeps its stop, as weeder's does.
- **Plot numerals.** JetBrains Mono 700, two digits (`01`), set in the bed's sign plate. On the plan they are 16px.
- **The motto rule.** One Protest Riot line (`clamp(1.7rem, 3vw, 2.5rem)`) led by the sprout mark, with the supporting line in body type beneath it.

## Spacing

The spacing scale lives in `layout.md`, with the containers, breakpoints, and density rules it belongs to. Typography owns type only.

## Font Sources

- Protest Riot: Google Fonts, a bold display face with a hand-cut edge, in one weight. Sturdy and friendly from the hero down to a plot name: the reason plotplot reads as a garden, not a dashboard. Chosen by the owner on 2026-10-27, replacing Fraunces.
- Hanken Grotesk: Google Fonts — a warm, friendly, rounded-humanist sans. Legible and precise at body sizes, with a little more personality than a neutral grotesk.
- JetBrains Mono: Google Fonts — compact, legible, familiar in developer workflows. The builder/terminal voice — used for kickers, labels, commands, and code.

## Google Fonts Import

Protest Riot comes in one style, so the URL loads only that.

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Protest+Riot&family=Hanken+Grotesk:wght@400..800&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
```

```css
@import url('https://fonts.googleapis.com/css2?family=Protest+Riot&family=Hanken+Grotesk:wght@400..800&family=JetBrains+Mono:wght@400;500;700&display=swap');

:root {
  --font-heading: 'Protest Riot', Georgia, serif;
  --font-body: 'Hanken Grotesk', system-ui, sans-serif;
  --font-interface: 'Hanken Grotesk', system-ui, sans-serif;
  --font-code: 'JetBrains Mono', monospace;
}
```
