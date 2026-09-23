# The garden film

A printed (risograph) garden in section that grows from seed to bloom and returns to the soil. It is a 24-second seamless loop, and it is the family's one moving illustration (`DESIGN.md` §8). It is plain Canvas 2D in one file, with no libraries, images or network calls. Every frame is a pure function of time.

## Use it

`embed.html` is the source. Paste three things from it into a page: the `<style>`, the `<section class="pp-film">` and the `<script>`. Paste them verbatim and never edit the copy. Change `embed.html` instead, run `node build.mjs` to regenerate the standalone player `garden.html`, and paste again. `reference/landing.html` carries it just above "why a garden".

- **Placement.** The band is full width at 4:1, or 3:1 below 880 and 2:1 below 520. On a phone it shows the centre of the garden.
- **Seam.** The soil fades to nothing at the band's bottom edge. Where the next section has a band colour, give `.pp-film` a background that shades from transparent (55%) into that colour, and drop that section's top border (`layout.md`, Seams).
- **Inks.** It uses four palette inks, multiplied onto the page, with night values on soil-night (`colors.md`, The Garden Film's Inks). A theme switch redraws it at once.
- **Motion.** It pauses off screen and when the tab is hidden. Under `prefers-reduced-motion` it shows the full-bloom still (t = 15.4 s) and never plays. It measures its own frame cost and falls back to 12 fps, then to the still, rather than stall a page.
- **Cost.** A frame costs under 8 ms in Chrome at 1440 × 360 and DPR 2, and 17 to 25 ms in Firefox.

## The story, by second

| Seconds | What happens |
|---|---|
| 0–3.5 | Dawn over tilled soil, seen in section, with one seed resting under each plant. Roots go down first, then shoots arch up through the soil and straighten. |
| 3.5–11 | Seed leaves open and the row grows. The carrot's fronds fan and its roots reach their stones, the pleach arms weave along their wires, and the rule's marker steps up beside its stem. |
| 10.5–13.3 | Bloom unfolds. The umbel, the daisy, the five-petal flower and the pleach blossoms open, the anthers drop, and the open secateurs rise to the thistle. |
| 13.1–17.6 | Full bloom is held while pollen drifts from the grass to the flower. The still at 15.4 s is this moment. |
| 17–21.5 | Evening. Florets turn to seed, petals fall, the secateurs snip the thistle, stems dry and nod, and the sun lowers and turns coral. |
| 19.9–24 | Plants sink back into the soil and seeds settle. The sun sets at 22.6 s, and the frame at 24 s is the frame at 0. |

The left of the band stays open, so words or a quiet page can sit beside it.

## Credit

The dot-screen kit inside `embed.html` is third-party code used under the MIT License. The notice is in `THIRD-PARTY.md` and in the code, and it travels with every copy.
