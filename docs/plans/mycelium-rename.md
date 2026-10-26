# Renaming pollen to mycelium

**Objective.** The garden's peer-messaging tool takes the name mycelium, everywhere at once, so
the name pollen is free for the marketing bed.

## Background

On 2026-10-26 the owner set the garden's layers: one bed per layer of building a product, from
raw signal to release. Marketing became its own layer. Pollen leaves one flower and starts new
growth in many others, which is what marketing does for a product, so the marketing bed takes
that name. Mycelium is the network under a garden that links plants and carries signals between
them, which is what the messaging tool does for agents.

The rename is cheap now and dearer with every user: the repository has 5 stars and no forks, and
nothing is published under `@plotplot/pollen`. `@plotplot/mycelium` is free.

## Goals and non-goals

Goals: after the cutover, no live file, config, page or command uses the old name, and two
agents exchange a message through mycelium.

Non-goals: no alias and no compatibility shim. The old tool names, environment variables and
mailbox path stop working at the cutover. Every config that uses them is on the owner's machine
and is changed in the same step, so a clean cut is possible. Dated records (research notes,
plans, prompts, dogfood records, anything named with a date) keep the old name, because they
record what happened under it.

## Names

| Old | New |
|---|---|
| repository `jahala/pollen` | `jahala/mycelium` |
| page `jahala.github.io/pollen/` | `jahala.github.io/mycelium/` |
| `pollen.mjs`, `pollen.test.mjs` | `mycelium.mjs`, `mycelium.test.mjs` |
| tools `pollen_*` | `mycelium_*` |
| variables `POLLEN_*` | `MYCELIUM_*` |
| mailboxes `/tmp/pollen/` | `/tmp/mycelium/` |
| launcher `pollen-mcp` | `mycelium-mcp` |
| brand `products/pollen/`, accent anther gold | `products/mycelium/`, a new mark and accent; anther gold is kept for the marketing bed |

## Where the name lives today

Counted on 2026-10-26 with `git grep -il pollen` in each checkout at its default branch.

| Place | Files | Who changes it |
|---|---|---|
| the tool's repository | 31 | its agent, in one pull request |
| the umbrella | 59: brand, the stem's code and tests, contracts and fixtures, the law, the map page, dated records | cape-town |
| the website | 1 file, 12 mentions, including 3 links to the tool's page | cape-town |
| pleach | 18: README, agent notes, project page, fetched brand, dated dogfood records | cape-town: links and README; dated records stay |
| weeder | 12: README, project page, map pages, plans, a garden schema | cape-town: links and README; an issue for the schema |
| tend | 4: a plan, project page, a manifest schema fixture and its test | cape-town: links; an issue for the fixture |
| cultivate | 1 dated note | stays |
| the owner's machine | the installed copy and launcher, 3 global agent-CLI registrations, 11 workspace MCP configs, the mailbox directory, the umbrella agent's memory | cape-town, at the cutover |

## Order

Each step names how to undo it before it is taken.

1. **Prepare the tool's workspace.** Its committed `.mcp.json` hard-codes the peer id
   `cape-town`, so any agent opened there takes over the umbrella agent's mailbox. Set a local,
   uncommitted id for the agent that does the rename; the rename itself fixes the committed file.
   Undo: restore the file from git.
2. **Brand.** Draw mycelium's mark in the garden's mark grammar, choose its accent from the
   palette with `scripts/palette_contrast.py` passing, write `products/mycelium/`, and update
   `scripts/marks.py`. Land on green, tag brand v1.7.0. Undo: the tag stays; a later tag reverts.
3. **The tool's code.** Its agent renames everything in its repository, fetches brand v1.7.0,
   and opens a pull request with CI green. It merges on cape-town's word, because the merge
   starts steps 4 to 7, which run in one sitting. The installed copy on the machine is a separate
   file, so running agents keep working on the old names until step 7. Undo: revert the merge.
4. **Rename the repository** to `jahala/mycelium`. GitHub redirects the old repository address,
   but the Pages address moves without a redirect, so step 5 follows at once. Undo: rename back.
5. **Links and names outside the tool.** The website (brand tag, bed name, link, bloom, mark),
   then the umbrella (the stem's bed lists and tests, the manifest fixture, the law, the map page
   renamed to `mycelium.tend2.html`, the brand), then the links and READMEs in pleach, weeder and
   tend. Each is its own pull request, landed on its repository's check. Undo: revert each.
6. **Issues** on weeder and tend for their copies of the garden schema, for their agents' next
   run.
7. **Machine cutover**, with no agent running. Install `mycelium.mjs` and `mycelium-mcp` from the
   merged commit; change the 3 global registrations and the 11 workspace configs to the new server
   name, path and variables; move `/tmp/pollen/` to `/tmp/mycelium/` so the journals stay; move
   the old install and launcher to the Trash. Undo: restore from the Trash and put the configs
   back from the copies taken first.

## Checks

- The tool's own suite passes under the new names, with every test kept.
- `git grep -il pollen` in every changed repository lists only dated records and the tool's
  README line that says it was called pollen until October 2026.
- Every link to the tool's page returns 200.
- Two peers started from the installed copy exchange a message and each journal records it.
- The website's `npm run check` reports 0 errors; the umbrella's `plotplot check --strict`,
  `plotplot lock verify` and `scripts/marks.py --check` pass.

## Open issues

- **The old address.** When the marketing bed's repository is created as `jahala/pollen`,
  GitHub stops redirecting the old address to mycelium. Links in dated records then lead to the
  marketing bed. Accepted: the dated records name a date, and mycelium's README names its old
  name.
