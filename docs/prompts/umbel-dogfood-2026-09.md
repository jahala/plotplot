# umbel — the dogfood brief

Recorded by cape-town (the umbrella agent) on 2026-09-10 for the agent that takes umbel's
open issues. Self-contained: you have not seen any prior conversation. You work in a Conductor
workspace of `jahala/umbel`.

## Read first, in this order

1. `docs/building-the-garden.md` in `jahala/plotplot` (public): the law. Invariants, the fit
   contract, mandates per bed (umbel: the runner; twelve MCP tools today against a mandate of
   six plus help), anti-goals, and §2: shape before code, only the verifier writes pass, Tried
   on every handback, where work lives, never kill what you did not start, a guardrail changes
   only by allowance.
2. `docs/tend2/umbel.tend2.html` in the same repository: umbel's fit loop and every decision
   already made about umbel in its Tried.
3. umbel's own README, docs and its `docs/tend2/` map, which you create with `tend2 init` if
   it is absent; module loops live there, never on the umbrella.
4. The fourteen open issues on `jahala/umbel`, all filed in the last three days by agents that
   built with umbel under pleach. They are the specification. Read every one before shaping.

## The work, in this order

Tier one, each of which lost real work or a worker this week:
- 72: the trust-dialog dismissal presses Enter on "No, exit", so every worker in a fresh
  worktree dies in seconds.
- 53: `spawn --provider opencode` destroys a user's opencode config that carries comments.
- 67: a wedged worker, live and idle past a threshold, is a failure; a provider error line on
  the pane is a terminal signal.
- 77: codex 0.154.0 never submits the pasted prompt; codex is out of the audit seat until this.
- 73: a dead worker leaves nothing to inspect; keep the capture, the log and the meta.

Tier two, each of which blinded a conductor:
- 65: `ls` shows each session's repository and conductor, and state survives a kill.
- 68: `actions` is blind for codex sessions.
- 71 and 70: `ls` tells idle from busy and prunes sessions whose directory is gone.

Tier three: 69, 74, 75, 66. Deferred until the owner decides on mull: 64.

Triage first, without a worker: labels and milestones on the fourteen, cross-links where one
loop closes several, one issue closed as a duplicate only where it is one. Then one loop per
tier-one issue, checks first, evidence scripts named, conducted through `tend2 emit-plan`,
pleach and umbel itself with Claude Opus 5 as the worker model and `weeder check --strict` as
the smoke gate; audits on opencode with deepseek-v4-pro until 77 lands. A failing test is the
spec: for 72 the test drives a real claude session in a fresh worktree and proves it survives
the dialog.

## Rules of the house

- Never kill a session, worktree or process you did not start; read `meta.json` first.
- Delete with `trash`, never `rm`. Never `git reset`. No stubs, mocks, TODOs or fallbacks.
- Merge your own pull request on green with a merge commit; close the issue with the commit.
  Tags and publishing are the owner's.
- Record every stumble in tend2, pleach or weeder in `docs/dogfood/<loop>.md` and file the real
  defects on the tool's repository, commenting where an issue already exists.
- Pacing: one node at a time, and hold whenever cape-town says so; the session window is shared.

## Coordination

Put `POLLEN_ALLOW=cape-town` in your workspace `.mcp.json` (git-ignored), run the pollen watcher,
and send `cape-town` your workspace path first. Report each landing with its link. Anything
you would ask the owner, ask cape-town.
