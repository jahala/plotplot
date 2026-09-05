# pollen conversion — plan

Status: plan, revised 2026-09-05 after the brand layer was authored. Companion to
`docs/prompts/pollen-conversion-2026-09.md`, which is now correct on the brand and on the
umbrella path. Two things in it are still wrong, and one thing about the *state of the
machine* will break Part 5. Both are in section 2.

## 1. What was verified

Brand layer, checked on disk rather than taken on trust — every figure quoted in the brief
reproduces exactly:

| Pair | Measured | Class |
|---|---|---|
| Pollen `#C8B330` on paper | 1.94 | decorative — never a word |
| Ink on Pollen | 6.71 | reading |
| Pollen-ink `#7E6A08` on paper | 4.88 | reading |
| Night-pollen `#D9C44A` on soil-night | 10.20 | reading |
| Juniper on paper (was missing) | 3.88 | labels |

`palette_contrast.py` reports **0 failing pairs**. `npm run check` on `public/index.html` is
**PASS, 0 errors, 0 warnings** — the baseline any later edit must hold. The product layer has
all five files including both marks; `.brand/components.md` and `CLAUDE.md` now carry copeca
and pollen.

Also confirmed: walkie-clawkie is 18 commits, `walkie.mjs` 575 lines, 14 tests, six tracked
files; rename scope is 138 `walkie`, 51 `WALKIE`, 7 `clawkie` across four files; `umbel`,
`codex`, `gemini` and `trash` are installed; `pollen` on npm is squatted at 0.0.5 and
`@plotplot/pollen` is free.

## 2. What will break, in order of severity

**The brand work is uncommitted, on a branch Part 5 does not build from.** It is nine
modified or untracked paths in `cape-town-v1`, on `jahala/plotplot-landing-page`, which sits
**11 behind and 1 ahead** of `origin/master`. `origin/master` has no pollen product layer and
no `--pp-pollen` token. Part 5 says:

```
git worktree add ~/conductor/workspaces/plotplot-ai/pollen-bed -b jahala/pollen-bed origin/master
```

That worktree would contain none of it. The agent would then add a bed card referring to a
custom property that is not defined there, and copy a footer garden row "verbatim from
`.brand/components.md`" from a file that still lists neither pollen nor copeca — and run
`npm run check` against the wrong tokens. `/petals init --from` reads the working tree, so the
*pollen repo* would get the right brand while the *umbrella* half diverged.

Fix before Phase 5: commit the brand work, then branch `pollen-bed` from that commit rather
than from `origin/master`. Whether the landing-page branch is first brought up to date with
its eleven-commit lag is the owner's call, but authoring on a stale base is worth knowing.

**The prompt still says to branch `pollen` off `main`.** Local `main` in the walkie-clawkie
worktree is `443d239`; `origin/main` is `97c21c1`. Between them are four content-free merge
commits and a tracked empty `.gitkeep` that `origin/main` deliberately dropped. Branch off
`origin/main`, or the bed starts life carrying junk into a repo about to be renamed and
published.

**Minor:** `~/conductor/workspaces/walkie-clawkie/main` is a symlink to `…/yokohama`. It
resolves, so it does no harm, but it is one working tree, not two.

*Resolved since the first draft:* the stale `~/conductor/repos/plotplot-ai` read paths, the
accent measurement gap, and the footer-row ordering problem — `components.md` now carries the
seven-bed row, so Phase 3 can copy it verbatim.

## 3. The sequencing change worth making

The direction memo adopts pollen only if a real multi-agent run finds a message umbel cannot
carry. The brief runs that proof last — after the rename, the page, the release standard and
the umbrella edits. If the run concludes umbel would have sufficed, all of it is wasted.

**Run the kill criterion first, against the build as it stands.** The proof is about
capability, not naming: `WALKIE_ID` demonstrates a peer exchange exactly as `POLLEN_ID` would.
The cross-harness claim moves with it, being the other result that can invalidate a README
sentence the landing page will repeat.

## 4. Phases

Each phase ends on a gate. A failed gate stops the run and goes in the report.

**Phase 0 — prove it.** Kill criterion via umbel: two workers in separate worktrees, distinct
ids, a task needing a peer exchange the supervisor channel cannot express; a third unlisted
worker knocks and is held until allowed. Then one non-Claude host — codex or gemini —
configured, one message each way, exact config recorded. Re-check whether MCP channels are
still a research preview.
*Gate: a named message umbel cannot carry, and one non-Claude host proven. If umbel would have
sufficed, stop and report it — that is a finding, not a failure.*
**Outcome — the criterion was retired, not tested.** The owner adopted pollen regardless, so
the peer-exchange run was never made; the cross-harness half was proven against codex. Recorded
in the pollen repo's `docs/proof-2026-09.md` and here, so neither the memo nor this plan claims
a gate that never ran. pollen is live on plotplot.ai on that decision, not on this evidence.

**Phase 1 — rename.** Branch `pollen` off **`origin/main`**. Both files, five tools, five env
vars, `/tmp/pollen`, `.mcp.json` key, `[pollen]` watch prefix. Every user-facing string reread
in the garden voice; errors as what happened, why it matters, next action. README rewritten as
a product page.
*Gate: `node --test` green with nothing mocked; zero hits for `walkie`, `clawkie`, `WALKIE`.*

**Phase 2 — fetch the brand.** Not author it. `/petals init --from
~/conductor/workspaces/plotplot-ai/cape-town-v1` in the pollen repo; the fetch brings
`.brand/` including `products/pollen/`. Rename `petalsrc.example` to `.petalsrc` at the root.
Do not edit the fetched `.brand/`. Anything the layer does not answer is flagged in the
handover, never invented.
*Gate: layer present and unmodified; `.petalsrc` in place.*

**Phase 3 — landing page.** `index.html` at the repo root, no build step. Hero with **carry
what matters.** and a real terminal exchange; the stack section against umbel and pleach; the
trust gate explained; the seven-bed footer copied from `components.md`.
*The trap: `#C8B330` is fill-only on paper at 1.9:1. Any word, kicker or `[pollen]` prefix set
in it on cream is off-brand — use `--pp-pollen-ink` `#7E6A08`.*
*Gate: `check.sh index.html` → 0 errors; content shows without JS; reduced motion honoured.*

**Phase 4 — release standard.** LICENSE (MIT, flagged), CI on Node 18/20/current, dependabot
for Actions, `VERSION` reported by `initialize` plus `--version`, conduct/contributing/security
files. Scan history for secrets and personal paths and report what was found.
*Gate: CI runs the same suite; history scan reported.*

**Phase 5 — umbrella, page half only.** The `.brand/` half is done. What remains is
`public/index.html` — the `--pp-pollen` var, a bed card replacing the first placeholder,
the footer garden row, and the heading that still reads "six tools, one garden" — plus the
`README.md` garden table. Re-run the copeca grep and add anything it finds.
*Prerequisite: the brand commit from section 2. Gate: `npm run check` → 0 errors, matching the
recorded PASS.*

**Phase 6 — handover.** `docs/proof-2026-09.md` and `docs/handover-2026-09.md` in the pollen
repo, plus the final report: branches and commits, corrected inventory, owner decisions, the
exact garden-row line each sibling page needs, and what could not be verified.

## 5. Open risks

**codex and gemini are installed; their auth state is unverified.** If neither is
authenticated the cross-harness claim cannot be proven, and the README sentence must be
softened rather than left standing.

**Journal state under `/tmp`** is an owner decision, not a change to make: cleared on reboot,
never rotated. State the trade-off only.

## 6. Decisions left for the owner

Accent, tagline and mark are **settled** and must not be reopened. Done since: the repo rename
to `jahala/pollen` and Pages from `main:/` — `jahala.github.io/pollen/` serves the product page,
and the umbrella's bed card reads **live**. Remaining: MIT licence · description, topics,
homepage · tag `v0.1.0` · delete the two merged remote branches · npm (`@plotplot/pollen` is
free; curl install works today, so this is a choice) · whether journal state stays under
`/tmp`.
