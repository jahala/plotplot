# plotplot

**Grow what matters.** A garden of small, sharp tools for building with AI.

This repository is the garden's root, the umbrella: the canonical brand every bed's page is
checked against, the law bed agents follow, the contracts every bed speaks, the map of loops
that say whether a bed belongs, and the stem, the `plotplot` binary that plants the garden into
any repository and proves it is planted. This repository is itself planted with it. The website,
[plotplot.ai](https://plotplot.ai), pulls the brand from here by tag, like every bed's page does.

## The garden

| Tool | What it does | Status |
|------|--------------|--------|
| [tilth](https://github.com/jahala/tilth) | code intelligence: an AST-aware map of a codebase for agents | released |
| [weeder](https://github.com/jahala/weeder) | the judge of the diff: refuses deleted tests, stubs and secrets before they land, as SARIF | released |
| [umbel](https://github.com/jahala/umbel) | fans out agent CLIs (Claude Code, Codex, Gemini, OpenCode) in tmux | released |
| [copeca](https://github.com/jahala/copeca) | cost per correct answer: a neutral benchmark for CLI coding agents | released |
| [pollen](https://github.com/jahala/pollen) | agent-to-agent messaging: mailboxes between agents, with a human at the gate | released |
| tend | loops and the verifier: intent with proof, checks only a verifier can close | not yet public |
| petals | brand intelligence for agents: extract a brand, check a page against it | not yet public |
| pleach | the conductor: gates agent work in isolated worktrees, ships only what passed | not yet public |

## The stem

`plotplot` plants every bed into any harness and proves it is planted. It is a static Rust
binary with nothing to install at runtime.

```
plotplot init [--harness claude,gemini,codex] [--beds a,b] [--profile minimal|full] [--lock <template>]
plotplot doctor [--live]
plotplot check [--strict] [--format sarif|table]
plotplot lock verify
plotplot bundle build [claude|gemini|codex]
plotplot receipt seal|verify|show
plotplot hook <harness> <event>
plotplot version
```

- `init` writes `garden.lock` from a template, fetches each pinned judge and refuses bytes
  whose sha256 is not the one pinned, generates a Claude Code plugin, a Gemini CLI extension
  and a Codex plugin from the beds' manifests, installs each at project scope, writes the git
  hooks and points `core.hooksPath` at them, plants the garden block in `AGENTS.md`, and
  writes the pull request gate. A second run changes nothing and says so.
- `doctor` proves the planting: every bundle byte-identical to what the stem generates, every
  hook entry the stem's dispatcher, the hooks live, the judges the lock names; `--live` drives
  one real session per harness and proves from the friction journal that each hook fired.
- `check` runs every gate the manifests declare and returns one SARIF 2.1.0 log: exit 2 on a
  block, 3 when a gate could not run.
- `receipt seal` attaches an unsigned in-toto statement to a commit under
  `refs/notes/plotplot/receipts`; `verify` re-derives the subject digests from git.

Build it from this repository:

```bash
cargo install --locked --path .
```

The crate's tests are the specification: `cargo test` runs them, and `scripts/fit/stem.sh
<check>` is the evidence each check of the stem's loop names.

## What this repository holds

| Path | What it is |
|------|------------|
| `src/`, `tests/` | the stem, a Rust crate at the root |
| `.brand/` | the canonical brand in petals' structure; `tokens.css` and `tokens.json` are derived |
| `docs/building-the-garden.md` | the law every bed agent follows: invariants, the fit contract, mandates, anti-goals |
| `docs/tend2/` | the umbrella map: one fit loop per bed, plus contracts, stem, friction, receipts |
| `docs/plans/` | plans for the tools that were unbuilt when they were written |
| `docs/prompts/` | the briefs bed agents were started from, and transcripts of channel messages |
| `docs/dogfood/` | the pleach plans the stem was built with |
| `docs/research/` | records, among them what building the stem taught about the garden's own tools |
| `contracts/` | the bed manifest and lockfile schemas, the friction and identifiers profiles, fixtures and tests |
| `scripts/fit/` | the fit runner that verifies a pinned bed against the contracts, and the stem's own evidence |
| `garden.lock`, `garden.json`, `.githooks/` | this repository, planted: the pinned judges, its own manifest, the git law |

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

## Support

If the garden is useful to you, [buy me a coffee](https://buymeacoffee.com/jahala).

## License

MIT
