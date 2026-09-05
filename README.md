# plotplot

**Grow what matters.** A garden of small, sharp tools for building with AI.

[plotplot.ai](https://plotplot.ai) is the umbrella for a family of tools — code
intelligence, orchestration, brand, planning — that agents and humans share. This
repo holds the **umbrella landing page** and the **canonical brand** for the whole garden.

## The garden

| Tool | What it does | Status |
|------|--------------|--------|
| [tilth](https://github.com/jahala/tilth) | code intelligence — an AST-aware map of your codebase for agents | live |
| [tend](https://github.com/jahala/tend) | feature mapping & narration across sessions | soon |
| [petals](https://github.com/jahala/petals) | brand intelligence for agents (extract + check) | soon |
| [pleach](https://github.com/jahala/pleach) | the conductor — gates agent work, ships only what's verified | soon |
| [umbel](https://github.com/jahala/rctrl) | fan out agent CLIs (Claude Code, Codex, Gemini) in tmux | live |
| [copeca](https://github.com/jahala/copeca) | cost per correct answer — a neutral benchmark for CLI coding agents | live |
| [pollen](https://github.com/jahala/pollen) | agent-to-agent messaging — mailboxes between agents, with a human at the gate | live |
| [weed](https://github.com/jahala/weed) | the judge of the diff — deterministic gates on tests, stubs, secrets and guardrails in what an agent changed | soon |

## Repo layout

- `public/` — the deployed site (`index.html` + favicon). No build step.
- `.brand/` — the canonical plotplot brand. Markdown is the source of truth; `tokens.css` / `tokens.json` are derived. Colors, type, voice, layout, surface/motion, identity, DESIGN, assets.
- `petals/` — vendored [petals](https://github.com/jahala/petals) skill: the brand tooling (`/petals` workflows + `scripts/check.sh`).
- `scripts/palette_contrast.py` — measures WCAG contrast for the palette and emits the Contrast Pairings table.
- `docs/` — [`deploy.md`](docs/deploy.md) runbook; `brand/` preview + reference shots.
- `CLAUDE.md` — project notes, decisions, and conventions for agents working here.

## Develop

```bash
npm install
npm run dev      # local preview (Cloudflare-accurate) at http://localhost:8787
npm run check    # audit the page against the brand (petals check) — must be 0 errors
```

## Deploy

Static site on **Cloudflare Workers (Static Assets)**. Merges to the default branch
auto-deploy via `.github/workflows/deploy.yml` (runs `wrangler deploy`). Manual:
`npm run deploy`. See [`docs/deploy.md`](docs/deploy.md) for the API-token secret and
the custom-domain steps (plotplot.ai + plotplot.io).

## Brand

The brand lives in `.brand/` and is enforced by petals — every page must pass
`/petals check`. Warm paper + deep-brown ink, vibrant growth green, a sunlight
accent, a bloom per tool, and a soil-night dark mode; Fraunces × Hanken Grotesk ×
JetBrains Mono; calm "unfold" motion. Full rules in `.brand/DESIGN.md` and
`.brand/voice.md`.

---

Part of the plotplot garden · built with [petals](https://github.com/jahala/petals)
