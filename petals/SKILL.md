---
name: petals
description: >-
  Brand compliance and design system skill. Use this skill when:
  - Building UI, components, layouts, landing pages, or any visual output
  - Writing copy, headlines, CTAs, microcopy, error messages, or marketing text
  - Checking if generated code/copy is on-brand (colors match palette, typography follows hierarchy, voice follows rules)
  - Extracting brand assets into a structured .brand/ directory from source files (PDFs, images, markdown, JSON)
  - Setting up or updating a project's brand context from a central brand repository
  - Any creative work where brand consistency matters: colors, typography, voice, UI, copy, compliance
---

# petals: brand that agents read

The brand lives in `.brand/` at the project root. Read it before you build, and check what you built. This file holds what every task needs; each rarer task has its own reference, listed at the end. Paths to `scripts/`, `references/` and `templates/` are relative to this file.

## Before you build

Read only the `.brand/` files the task needs:

| Task | Read |
|------|------|
| UI, components, layouts, pages | `.brand/DESIGN.md`, `.brand/colors.md`, `.brand/typography.md`, `.brand/components.md`, `.brand/layout.md` |
| Copy: headlines, buttons, errors, docs | `.brand/voice.md`, `.brand/identity.md` |
| Logos and other assets | `.brand/identity.md`, `.brand/assets/` |
| A control: button, input, checkbox, select | its contract in `.brand/components/`, if one exists |

A brand can be a family. When `.brand/products/<product>/` exists, read the product's file after the umbrella's: the product layer holds only its differences, and everything else comes from the umbrella. See `references/brand-architecture.md`.

If there is no `.brand/`, say so and offer `/petals init` (from a folder of guidelines) or `/petals init --from <brand repo>`. Do not guess a brand.

## Hard rules

These are not negotiable. Breaking one is a brand error.

1. **Colours from the palette only.** Every colour in what you build matches a value in `.brand/colors.md`. Never use a colour because it looks close. In a product, another product's accent is a warning: accents mark products.
2. **Type from the brand.** Heading and body faces, weights and sizes match `.brand/typography.md`. No substitutes.
3. **Logos from the assets.** Never invent, redraw or describe a logo. Use a file from `.brand/assets/`, or leave the logo out.
4. **Voice rules.** Copy uses none of the forbidden terms in `.brand/voice.md` and follows its tone, grammar and terminology.
5. **Reuse, never rebuild.** When a contract in `.brand/components/` covers the control you need, use the component from its home. Write a contract before you build a new shared control. See `references/components.md`.
6. **The brand wins.** When the brand and a framework default disagree, override the default: CSS framework config, resets and component library styles.

## After you build

Run the check on every file you made or changed:

```
bash scripts/check.sh <file>
```

Fix every error. Exit 0 is clean or warnings only, exit 1 is at least one error, and exit 3 means the check could not judge and says why. Exit 3 is never a pass. The script cannot judge typography roles, tone, chart series order or a stale brand book; that part is yours, in `references/check.md`.

## Commands

| Command | What it does | Read |
|---------|--------------|------|
| `/petals init <folder>` | Extract a brand from a folder of guidelines into `.brand/` | `references/init.md`, `references/extraction-guide.md` |
| `/petals init --from <repo-url>` | Fetch `.brand/` from a central brand repository | `references/init.md` |
| `/petals check [file]` | Check a file against the brand; with no file, check for brand drift | `references/check.md`, `references/update.md` |
| `/petals voice <text>` | Check copy against `.brand/voice.md` | `references/voice-guide.md` |
| `/petals color <name>` | Look up a colour in `.brand/colors.md` | `.brand/colors.md` |
| `/petals token <format>` | Export tokens (css, dtcg) with `scripts/export-tokens.sh`; `--check` exits 1 and lists every token the files and the markdown disagree on | `scripts/export-tokens.sh` |
| `/petals update` | Sync `.brand/` with the central brand, keeping project overrides | `references/update.md` |
| `/petals logo [variant]` | Give the path of the right logo file | `references/logo.md` |
| `/petals book` | Render `.brand/` as one self-contained brand book page | `references/book-guide.md` |
| `/petals review <page> [--note <text>]...` | Open pages in the owner's browser behind an index; `--stop` ends it and leaves nothing behind | `scripts/review.sh` |

## References

| When you need | Read |
|---------------|------|
| To install petals, find its scripts, or run it in CI | `references/setup.md` |
| The umbrella and product layers | `references/brand-architecture.md` |
| The gate, the git hooks, the edit-time hook | `references/guardrails.md` |
| The `.petalsrc` file | `references/petalsrc.md` |
| Component contracts: where they live, how to write one | `references/components.md` |
| To judge a colour finding (exclusions, contrast) | `references/color-audit-guide.md` |
| A rare task: a product frame, a bed page | `references/recipes.md` |
| The empty forms for each brand file | `templates/` |
