<!-- tend2:begin -->
## tend2 — this project plans on loops

- Orient first: `tend2 next docs/tend2` (or the `loop_next` MCP tool) — next up, running now, needs-you, gone stale.
- Nothing gets built that is not a loop first: shape the goal and its checks on the map before any code.
- Only `tend2 verify` writes a pass. Never hand-flip a checkbox — a naked [x] renders claimed, not proven.
- Record decisions, scope-outs and dead ends in the loop's `## Tried` — append-only memory for whoever comes next.
- The map holds bets, not maybes: ideas attached to a loop park in its `## Tried`; free-standing ideas park in `docs/ideas/`; shaping is the only transition onto the map.
- Full craft lives in the tend2 plugin skills (next, shape, run, verify, discover, change).
<!-- tend2:end -->

<!-- plotplot:begin -->
## plotplot: what is planted here

Planted: weeder 0.2.1.
Season: 2026.09.

- Orient first: `tend2 next docs/tend2` says what is next, what is running, and what went stale.
- Gates: `plotplot check` runs every planted gate and returns one SARIF log.
- Help: `plotplot doctor` says whether the law is live, and what is missing when it is not.
- The hard limits are enforced at the boundary: `--no-verify` is refused, and `garden.lock` and `.githooks/` are the stem's to write.
<!-- plotplot:end -->
