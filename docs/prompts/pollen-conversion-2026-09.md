# walkie-clawkie → pollen

Brief for an agent. Self-contained: you have not seen any prior conversation. Read the
whole file before touching anything.

## The task in one paragraph

walkie-clawkie is a working, tested, one-file MCP server for peer-to-peer messaging
between AI agents, with a human-approved trust gate. It is being adopted into the plotplot
garden as the bed named **pollen**. Your job is to convert it: rename it end to end, give it
the garden's product brand layer and landing page, bring the repo to the garden's release
standard, prove the cross-harness claim, and wire the new bed into the umbrella. Some
decisions are the owner's alone; you prepare them, you do not make them.

## Posture and guardrails

- Execute the work yourself; no subagents. Your final message is the deliverable: the
  branches you created, the file list, the test output, and the list of decisions waiting
  for the owner.
- Ground truth over inference. Read the file before changing it; run the tests before and
  after; quote outputs.
- Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash. Commit on
  the branches named below; do not push, do not open PRs, do not rename the GitHub repo, do
  not enable GitHub Pages. All of those are outward-facing and wait for the owner's go.
- Never invent brand values. Where a value is not in `.brand/`, propose and flag it; petals
  forbids guessing.
- Voice, everywhere you write (README, page, docs, tool descriptions, log lines): calm,
  precise, literate, a little wit. Sentence case. Product names lowercase always. No
  exclamation marks. Never: supercharge, unlock, 10x, magic, synergy, revolutionary,
  game-changing, cutting-edge, seamless, effortless, next-gen, AI-powered. Playful is
  allowed; cute is not. "walkie-talkie", "push-to-talk", and "clawkie" all go.
- No time estimates. No TODOs. No stubs. Nothing silently dropped: what you skipped or
  could not verify goes in the final report, named.

## Where things are (verified 2026-09-05)

| What | Where |
|---|---|
| The code | `~/conductor/workspaces/walkie-clawkie/main`, branch `main`, 18 commits, last 2026-08-20. `~/conductor/repos/walkie-clawkie` is a stale checkout; ignore it. |
| Files | `walkie.mjs` (575 lines, raw MCP over stdio, no SDK, Node 18+), `walkie.test.mjs` (14 tests), `README.md`, `.mcp.json`, `.github/FUNDING.yml`, `.gitignore`. That is the whole repo. |
| Tests | `node --test` in that directory: 14 pass in about 4 seconds. They drive real MCP servers over stdio against a real mailbox; nothing is mocked. Keep it that way. |
| GitHub | `jahala/walkie-clawkie`, public, default branch `main`, description set, no topics, no homepage, **no LICENSE file** (GitHub reports license null). PRs #1, #2, #4, #5, #6 merged; issue #3 closed. Remote branches `fix/gate-survives-restart` and `fix/relay-failure-kills-server` are the already-merged PR branches, never deleted. |
| The umbrella | **working tree: `~/conductor/workspaces/plotplot-ai/cape-town-v1`** (branch `jahala/plotplot-landing-page`; `~/conductor/repos/plotplot-ai` holds only the git dir — reading from it fails). Read its `CLAUDE.md`, `README.md`, `.brand/DESIGN.md`, `.brand/voice.md`, `.brand/colors.md`, `.brand/components.md` (the garden footer spec and reference implementation), `.brand/identity.md`. |
| **pollen's brand layer** | `~/conductor/workspaces/plotplot-ai/cape-town-v1/.brand/products/pollen/` — authored and measured 2026-09-05. Use it as-is (Part 2). `~/conductor/repos/pleach/.brand/products/pleach/` shows the same shape for another bed. |
| A finished product page | `~/conductor/repos/pleach/index.html` (about 50 KB, sections: hero · why · loop · gates · proof · fit · start). Read it for structure and for how the footer and tokens are inlined. |
| Live product pages | tilth serves from `main:/`, umbel from `master:/`, both via GitHub Pages at `jahala.github.io/<tool>/`. That is the publishing pattern: `index.html` at the repo root, Pages from the default branch root. |
| Brand tooling | petals is vendored in the umbrella at `~/conductor/workspaces/plotplot-ai/cape-town-v1/petals/`; the check is `bash petals/scripts/check.sh <file>`. Read `petals/SKILL.md` sections on umbrella and product layers, `/petals init --from`, and `/petals update`. |
| Contrast script | `~/conductor/workspaces/plotplot-ai/cape-town-v1/scripts/palette_contrast.py` (already covers pollen and juniper; 0 failing pairs) — measures WCAG pairs in the shape colors.md expects. |
| Direction memo | `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/garden-direction-2026-09.md` — why pollen is being adopted and the kill criterion it must survive. |
| umbel | `~/conductor/repos/rctrl` (the repo is named umbel on GitHub). Read `docs/positioning.md`: umbel is supervisor-to-worker; pollen is peer-to-peer. The two must not be confused on the page. |

## What walkie-clawkie has today

Inventory this against the code before you rename anything; correct this list where the
code disagrees and say so.

**Five MCP tools:** `walkie_send`, `walkie_agents`, `walkie_inbox`, `walkie_allow`,
`walkie_deny`.

**Three modes of one file:** `node walkie.mjs` (MCP server the host spawns), `node
walkie.mjs --relay` (HTTP plus SSE relay, default port 4747, for cross-machine), `node
walkie.mjs --watch <id>` (tails the journal, one line per arrival, so any harness that
reads lines can wake the agent).

**Transport:** same machine through file mailboxes at `/tmp/walkie/<id>/inbox`;
different machines through the relay over any tunnel or VPN. The agent does not know which.

**Durability:** every arrival is appended to `/tmp/walkie/<id>/journal.jsonl` before the
mailbox file is removed; `cursor` records how far the agent has read; unread messages
survive a server restart and the agent is told how many wait on startup. Journal is
append-only, never rotated, and lives under `/tmp`, which the OS clears on reboot.

**Trust gate:** unknown senders are held; the human allows or denies; knocks queue rather
than overwrite; the gate is rebuilt from the journal after a restart; a denial is never
rung out on the watch line; `WALKIE_ALLOW` pre-approves senders.

**Honest delivery status:** `walkie_send` reports sent, held at the gate, or queued with no
confirmation, so an agent is never left waiting on a reply that cannot come. Over a relay
only "left" is knowable, and the README says so.

**Resilience:** an unreachable relay fails the call, not the server.

**Push for Claude Code** via MCP channels, described in the README as a research preview.
`walkie_inbox` is the universal path.

**Env:** `WALKIE_ID` (required), `WALKIE_RELAY`, `WALKIE_ALLOW`, `WALKIE_DIR`, `WALKIE_PORT`.

## What it needs to belong to the garden

Work through Parts 1 to 6 in order. Part 7 is the list you hand back.

### Part 1 — Rename, end to end

Branch `pollen` off `main` in the walkie-clawkie workspace. Then, in one sweep, so the
tests fail first and pass after:

- `walkie.mjs` → `pollen.mjs`; `walkie.test.mjs` → `pollen.test.mjs`.
- Tools: `pollen_send`, `pollen_agents`, `pollen_inbox`, `pollen_allow`, `pollen_deny`.
- Env: `POLLEN_ID`, `POLLEN_RELAY`, `POLLEN_ALLOW`, `POLLEN_DIR`, `POLLEN_PORT`. Default
  dir `/tmp/pollen`. MCP server key `pollen` in `.mcp.json`. Watch-line prefix `[pollen]`.
- Every user-facing string, tool description, and error message re-read in the garden
  voice. Errors state what happened, why it matters, and the next action, in that order,
  with no jokes.
- README rewritten: one sentence of what it is, an install that runs, the three modes, the
  trust gate, the environment table, how it works, tests, and the garden footer line at the
  bottom (see the umbrella README's last line for the shape). The README is a product page;
  its rhythm and typography are judged like one.
- Grep the tree for `walkie`, `clawkie`, and `WALKIE` at the end. Zero hits, including the
  raw-URL install command, which will point at the future repo name (see Part 7).
- `node --test` green. Add one test if the rename exposed an untested path.

### Part 2 — The product brand layer

**The brand exists. Do not author it, do not re-pick it, do not "improve" it.** pollen's
product layer was designed and measured in the umbrella on 2026-09-05 and is committed at
`~/conductor/workspaces/plotplot-ai/cape-town-v1/.brand/products/pollen/`:

| File | What it fixes |
|---|---|
| `identity.md` | tagline **carry what matters.**, accent, tools, positioning, mark description, logo usage |
| `colors.md` | the accent claim: **anther gold `#C8B330`**, display/fill only on paper; **pollen-ink `#7E6A08`** for words; `#D9C44A` on soil-night |
| `voice.md` | signature phrase **strangers knock; you decide.** and the terminology table |
| `assets/pollen-mark.svg` · `assets/pollen-mark-night.svg` | the mark: two stems leaning apart, a grain carried between them |
| `petalsrc.example` | the `.petalsrc` to drop at the pollen repo root (rename it) |

The accent is registered in the umbrella's canonical `.brand/colors.md` (Product Accents,
Contrast Pairings, the `--pp-pollen` / `--pp-pollen-ink` custom properties) and in the derived
`tokens.css` / `tokens.json`. Every pair is measured by `scripts/palette_contrast.py`, which
now covers pollen and reports **0 failing pairs**: `#C8B330` on paper is **1.9:1 — decorative,
never a word**; ink on `#C8B330` is 6.7:1; `#7E6A08` on paper is 4.9:1; `#D9C44A` on soil-night
is 10.2:1.

What you do:

1. `/petals init --from <umbrella path or URL>` in the pollen repo. The fetch copies
   `.brand/` **including `products/pollen/`** — the layer arrives with it.
2. Rename `petalsrc.example` to `.petalsrc` at the repo root (it already sets
   `product: pollen`).
3. Use the accent as `colors.md` specifies. The trap: gold is **display/fill only on paper**.
   A `[pollen]` prefix, a kicker, or any word set in `#C8B330` on cream is off-brand and
   `check.sh` will not always catch it — use `--pp-pollen-ink` for words.
4. If you find a brand value the layer does not answer, **flag it in the handover**. Do not
   invent one, and do not edit `.brand/` in the pollen repo — it is a fetched copy.

### Part 3 — The landing page

`index.html` at the repo root, no build step, served later from `main:/` by GitHub Pages
exactly as tilth and umbel are. Follow pleach's page for structure and the umbrella's
`public/index.html` for how tokens are inlined in `:root` and mirrored for
`[data-theme="dark"]`. Required:

- Hero with the tagline, one real terminal pane showing a real exchange (two agents, one
  held at the gate, the watch line ringing), install as a verb-first button.
- A section that places pollen in the stack next to umbel and pleach, in the garden's own
  words: umbel holds each grower steady; pleach interweaves the branches; pollen is what
  growers say to each other, with the human at the gate.
- The trust gate explained: held, allowed, denied, and what the sender is told.
- The garden footer with the plotplot band, copied verbatim from
  `.brand/components.md`, changing only the identity column, the `.is-current` pill, and
  the repo links. The garden row must list the **full** garden: tilth, tend, petals,
  pleach, umbel, copeca, pollen. Note that the reference implementation in
  `components.md` lists only five tools; it is missing copeca. Fix that in the umbrella
  (Part 5) rather than copying the omission.
- `bash petals/scripts/check.sh index.html` → 0 errors. Known traps: anchors that look like
  hex (`#bed`) are read as colors; the `var()` resolver takes the last definition, so it
  validates dark-mode contrast; spacing on the 4px scale; `@media` only at 880 and 520;
  shadows ink-tinted, never pure black.
- Honour `prefers-reduced-motion`; content shows without JS.

### Part 4 — Release standard

Bring the repo to the standard of tilth (read how tilth does each item rather than
inventing):

- **LICENSE**: missing. The garden is MIT, but the license is the owner's call; add the MIT
  text on the branch and flag it as awaiting confirmation.
- **CI**: `.github/workflows/ci.yml` running `node --test` on push and PR on Node 18, 20,
  and current.
- **dependabot.yml**: for GitHub Actions at minimum (there are no npm dependencies).
- **FUNDING.yml**: exists; keep.
- **Versioning**: the file carries no version. Add a `VERSION` constant reported by the MCP
  `initialize` response and a `--version` flag; tag proposal `v0.1.0` in Part 7.
- **Repo hygiene**: propose description, topics, and homepage in Part 7. Confirm no secrets
  or personal paths anywhere in history (`git log -p | grep` for keys, tokens, home paths)
  and report what you found.
- **CODE_OF_CONDUCT.md, CONTRIBUTING.md, SECURITY.md**: umbel and copeca have them; copy
  the shape, adapt the content.

### Part 5 — Wire the bed into the umbrella

Create a worktree of the umbrella and work on a branch:

```
git -C ~/conductor/repos/plotplot-ai worktree add \
  ~/conductor/workspaces/plotplot-ai/pollen-bed -b jahala/pollen-bed origin/master
```

**The `.brand/` half of this is already done** on the umbrella branch
`jahala/plotplot-landing-page` (2026-09-05): the Product Accents row, the contrast pairings,
`--pp-pollen` / `--pp-pollen-ink` in `colors.md` + `tokens.css` + `tokens.json`, the beds list
in `identity.md`, the lowercase names line in `voice.md`, the reference garden row in
`components.md` (copeca and pollen both), the beds list in `CLAUDE.md`, and pollen + juniper in
`scripts/palette_contrast.py`. **Rebase onto or branch off whatever carries those commits, and
verify before you edit** — `grep -n pollen .brand/colors.md`. If a line is missing, add it; if
it is there, leave it alone.

What is left is the page itself. This list was derived by grepping for `copeca`; re-run that
grep and add anything it finds that is not here:

- `public/index.html`: the `--pp-pollen` var in `:root`; a bed card replacing the first
  placeholder (role, one-line description, status `soon` until Pages is live, link to
  `https://jahala.github.io/pollen/`); the footer garden row; the section heading that
  hardcodes "six tools, one garden".
- `README.md`: the garden table.
- `npm run check` → 0 errors on `public/index.html`.

The sibling product pages (tilth, umbel, copeca, and the pleach, petals, tend2 repos) each
carry a garden row that must gain pollen. Do not edit those repos; list them in Part 7 with
the exact line to add.

### Part 6 — Prove the two claims

**The kill criterion.** The direction memo adopts pollen only if a real multi-agent run
finds a message umbel cannot carry. Run it: with umbel, spawn two workers in separate
worktrees, each with pollen configured under a different `POLLEN_ID`, and give them a task
that needs a peer exchange umbel's supervisor channel cannot express (one asks the other a
question and waits for the answer; a third, unlisted worker knocks and is held at the
gate until you allow it). Capture the transcript excerpts and the watch lines. If umbel's
own send and read would have sufficed, say so plainly; that is a finding, not a failure of
the brief.

**The cross-harness claim.** The README says Claude Code, Codex CLI, and Gemini CLI. Prove
at least one non-Claude host: configure pollen as an MCP server for Codex or Gemini
(through umbel if convenient), send one message each way, and record the exact
configuration that worked. Also verify whether MCP channels are still a research preview
in the current spec, and update the README sentence to whatever is true now.

Record both runs in `docs/proof-2026-09.md` in the pollen repo.

## Part 7 — What you hand back

Your final message, and a copy at `docs/handover-2026-09.md` in the pollen repo:

1. Branches: `pollen` in the walkie-clawkie workspace and `jahala/pollen-bed` in the
   umbrella worktree, with commit lists. Test output before and after. `check.sh` output
   for both pages.
2. The features inventory from above, corrected against the code.
3. Decisions waiting for the owner, each with your recommendation:
   - license (accent and tagline are **settled** — see Part 2; do not reopen them);
   - GitHub repo rename `jahala/walkie-clawkie` → `jahala/pollen` (GitHub redirects the old
     URLs; the raw-file install URL in the README assumes the new name);
   - enable GitHub Pages from `main:/`;
   - proposed description, topics, homepage; tag `v0.1.0`; delete the two stale remote
     branches;
   - optional npm: `pollen` on npm is squatted at 0.0.5; `@plotplot/pollen` is free; the
     file installs by curl today, so npm is a choice, not a need;
   - whether journal state should stay under `/tmp` (cleared on reboot, never rotated) or
     move; state the trade-off, do not change it.
4. The exact garden-row line each sibling page needs.
5. What you could not verify, named.

## Acceptance

- Zero hits for `walkie`, `clawkie`, `WALKIE` in the pollen tree.
- `node --test` green; the tests still drive real servers with nothing mocked.
- `check.sh` reports 0 errors on the pollen page and on the umbrella page.
- `.brand/products/pollen/` arrived intact via the fetch, unedited, and `.petalsrc` names
  the product; any brand gap you hit is named in the handover rather than invented.
- The garden row on the pollen page lists all seven tools; the umbrella's reference footer
  lists all seven.
- Both proof runs are recorded with real output.
- Nothing pushed, renamed, published, or enabled on GitHub.
