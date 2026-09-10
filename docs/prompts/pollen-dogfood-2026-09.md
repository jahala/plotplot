# pollen — the dogfood brief

Recorded by cape-town (the umbrella agent) on 2026-09-10 for the agent that takes pollen's
open issues. Self-contained. You work in a Conductor workspace of `jahala/pollen` (public).

## Read first

1. `docs/building-the-garden.md` and `docs/tend2/pollen.tend2.html` in `jahala/plotplot`: the
   law, and pollen's fit loop with every decision already made in its Tried (pollen is a
   channel: three tools, a human at the gate, the journal as the inbox of record).
2. pollen's README and `pollen.mjs`, and its `docs/tend2/` map, created with `tend2 init` if
   absent.
3. The five open issues on `jahala/pollen`, filed this week by the agents that lived on the
   channel. They are the specification.

## The work, in this order

- 22: `POLLEN_ALLOW` is read once at start and `pollen_allow` does not persist. This cost the
  garden a day of one agent's silence and held every message from the umbrella at gates that
  had already been opened. An allow persists across restarts, in a file beside the mailbox.
- 19: the watcher rings the same knock hundreds of times, and nothing on the MCP side names who
  is at the gate. One ring per arrival; `pollen_inbox` names the peer at the gate.
- 23: `pollen_agents` lists every mailbox ever created with no liveness; show which peers have a
  live process.
- 20: a sender cannot tell whether a peer received a message; the send result names delivered,
  queued or held, and the journal records the read.
- 21: inbox lines carry their timestamps.

Shape before code, one loop per issue on your own map, checks first, conducted through
`tend2 emit-plan`, pleach and umbel with Claude Opus 5 as the worker and `weeder check --strict`
as the smoke gate. `pollen.test.mjs` is where the failing test goes first.

## Rules of the house

Never kill what you did not start. Delete with `trash`, never `rm`. No stubs, mocks or
fallbacks. Merge your own pull request on green with a merge commit and close the issue with the
commit; tags and publishing are the owner's. Record every stumble in the tools in
`docs/dogfood/<loop>.md` and file real defects where they belong. One node at a time; hold when
cape-town says so.

## Coordination

`POLLEN_ALLOW=cape-town` in your workspace `.mcp.json`, the watcher running, your workspace path
to `cape-town` first. Report each landing with its link.
