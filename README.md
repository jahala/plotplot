# plotplot

**Grow what matters.** A garden of small, sharp tools for building with AI.

This repository is the garden's root, the umbrella: the canonical brand every bed's page is
checked against, the law bed agents follow, the contracts every bed speaks, and the map of loops
that say whether a bed belongs. The stem, the `plotplot` binary that plants the garden into any
repository, will live here too. The website, [plotplot.ai](https://plotplot.ai), lives in
[jahala/plotplot-ai](https://github.com/jahala/plotplot-ai) and pulls the brand from here by tag,
like every bed's page does.

## The garden

| Tool | What it does | Status |
|------|--------------|--------|
| [tilth](https://github.com/jahala/tilth) | code intelligence — an AST-aware map of your codebase for agents | live |
| [tend](https://github.com/jahala/tend) | feature mapping & narration across sessions | soon |
| [petals](https://github.com/jahala/petals) | brand intelligence for agents (extract + check) | soon |
| [pleach](https://github.com/jahala/pleach) | the conductor — gates agent work, ships only what's verified | soon |
| [umbel](https://github.com/jahala/umbel) | fan out agent CLIs (Claude Code, Codex, Gemini) in tmux | live |
| [copeca](https://github.com/jahala/copeca) | cost per correct answer — a neutral benchmark for CLI coding agents | live |
| [pollen](https://github.com/jahala/pollen) | agent-to-agent messaging — mailboxes between agents, with a human at the gate | live |
| [weeder](https://github.com/jahala/weeder) | the judge of the diff — refuses deleted tests, stubs and secrets before they land | soon |

## What this repository holds

| Path | What it is |
|------|------------|
| `.brand/` | the canonical brand in petals' structure; `tokens.css` and `tokens.json` are derived |
| `docs/building-the-garden.md` | the law every bed agent follows: invariants, the fit contract, mandates, anti-goals |
| `docs/tend2/` | the umbrella map: one fit loop per bed, plus contracts, stem, friction, receipts |
| `docs/plans/` | plans for the unbuilt tools |
| `docs/prompts/` | the briefs bed agents were started from, and transcripts of channel messages |
| `contracts/` | the bed manifest and lockfile schemas, the friction and identifiers profiles, fixtures and tests |
| `scripts/fit/` | the fit runner that verifies a pinned bed against the contracts |

## Pinning the umbrella

A bed pins one umbrella version for brand, law and contracts together, by tag:

```yaml
# .petalsrc at a bed's root
brand:
  source: https://github.com/jahala/plotplot.git
  version: v1.0.0
  product: <bed>
```

Tags are `v<major>.<minor>.<patch>`. A change a bed must react to (a renamed token, a new
required manifest field) moves the major.

The tag is the pin. `Brand Version` in `.brand/identity.md` is the brand's own version and moves
only when the brand moves, so it can lag the tag when contracts or the vendored check change. A bed's
fetch holds its checkout to the tag's commit and prints the declared brand version beside it; the
one refusal is a tree whose declared brand version is newer than its tag, which is a mis-tag.
