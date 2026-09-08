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

- 2026-09-08 `pleach schema` emits `policy` with `required: ["maxAttempts", "onDead",
  "reauditWhen"]` although each has a default, while `pleach validate` accepts a plan whose
  policy carries only `timeoutMs` (tend2's emitted plan does exactly that and validates). The
  emitted schema is stricter than the validator; a planner generating from the schema adds
  three fields it need not, a planner trusting the validator emits plans the schema rejects.
- 2026-09-08 the plan schema has no field for a worker's Tried handback although
  docs/building-the-garden.md §6a says pleach validates it in shape through an additive
  v1.3 field; `pleach --version` here is 0.0.1 and contracts/pins.json pins the plan schema
  at 1.1.5, so the field is not there yet and the run's Tried is whatever the worker writes
  in its final message.

## umbel

- 2026-09-08 `umbel ls` lists four `smk-trust-*` sessions dead since 2026-09-05 in temp
  directories that no longer exist; nothing prunes dead sessions, so the list grows forever.

## conductor (workspace tooling)

- 2026-09-08 the Conductor workspace was seeded on an empty "Initial commit" root with no
  relation to `origin/master`, and its target branch is `origin/main`, which does not exist on
  jahala/plotplot (the default branch is `master`). Re-pointed the branch with
  `git checkout -B <branch> origin/master` before any work could start.
