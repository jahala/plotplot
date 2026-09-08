# Dogfooding tend2, pleach and umbel while building the stem

Kept by the stem builder (agent beirut) during the stem build, September 2026. One line per
finding, dated, with the command that showed it. Misunderstandings, faults, missing features
and bugs in the garden's own tools, as met while using them for real work. Nothing here is a
fix; each is a candidate issue on the tool's repository.

## tend2

- 2026-09-08 `tend2 emit-plan docs/tend2` writes every node's verify command as
  `node '<cwd>/dist/cli.js' verify …`. That file does not exist in a repository that is not
  tend2 itself (tend2 is installed at `/opt/homebrew/bin/tend2`). The default should be the
  running binary; `--verify-bin` exists but the default is wrong for every consumer.
- 2026-09-08 `tend2 emit-plan` emits one node per loop, with every unchecked check of that
  loop in one node under one timeout (30 minutes default). The stem loop has nine code checks;
  one worker cannot close them in one node. A `--per-check` split, or an `emit-plan <loop>`
  form that emits one node per check with the loop's shared narrative, is missing.
- 2026-09-08 `tend2 emit-plan` on the umbrella map emitted all thirteen loops as nodes wired by
  their `## Needs` tags, with no way to select a subset (`--only stem`); the stem node needs
  contracts, weeder, tend2, pleach and umbel, none of which this repository can verify, so the
  emitted plan cannot run here at all.
- 2026-09-08 `tend2 next docs/tend2` reports "12 asks held back — their loops have no machine
  evidence to judge yet" without naming which loops or how to surface them; the human check on
  the stem loop is invisible until a code check has evidence.

## pleach

## umbel

## conductor (workspace tooling)

- 2026-09-08 the Conductor workspace was seeded on an empty "Initial commit" root with no
  relation to `origin/master`, and its target branch is `origin/main`, which does not exist on
  jahala/plotplot (the default branch is `master`). Re-pointed the branch with
  `git checkout -B <branch> origin/master` before any work could start.
