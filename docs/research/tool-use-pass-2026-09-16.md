# How the garden's agents use their tools: a tool-use pass over the session logs

Date: 2026-09-16. Scratch: `/tmp/tool-use-analysis/` (per-session TSV extracts, indexes, issue bodies).

## Method

One JSONL per harness session under `~/.claude/projects/<workspace-dir>/`. For each file read, `jq` streamed every `tool_use` (timestamp, id, tool name, a 600-character summary of the input: the command for Bash, the path for Read/Edit/Write, recipient and text for pollen, name and text for umbel) and every `tool_result` (timestamp, id, `is_error`, 400 characters). The two were joined by id into `/tmp/tool-use-analysis/j.tsv` (12,865 tool uses, 7,653 of them Bash). Counts below come from regex passes over that table; sequences ("what follows X") walk the table in session order. Token costs are estimated as characters divided by four from the raw result lengths of three sessions; wall time comes from timestamps.

Sessions read fully (tool_use lines, results only where an error needed them):

| agent | sessions | tool uses | dates |
|---|---|---|---|
| cape-town (umbrella) | 6d9de859, bf523d9d | 1,425 | 2026-09-05 to 09-16 |
| cayenne (pleach) | 7b42ed04, 443a2c8e | 2,156 | 2026-08-16 to 09-16 |
| missoula (tend2) | 80c82780 and six others over 1 MB | 5,046 | 2026-07-01 to 09-16 |
| beirut (stem) | 0cc973f6, e1331e62 | 768 | 2026-09-08 to 09-11 |
| bandung (weeder) | 204fc970, bdd85b56 | 1,799 | 2026-09-05 to 09-11 |
| workers (pleach-conducted) | 30 of 237 worktree sessions, every eighth by date | 1,671 | 2026-09-05 to 09-16 |

Notes on the corpus:

- The missoula directory holds 830 files. 823 are headless runs, not the agent: the July session-log analyzer ("THIS IS DATA, NOT A TASK", 2026-07-21 to 07-25) and tend2 verify's independent alignment judge (78 files on 2026-08-19, three to eight tool calls each). Excluded.
- umbel's own workspace directory holds no top-level session. umbel's work appears as `pleach-v1-umbel` worktree sessions (three in the worker sample) and inside cayenne's session.
- One sampled worktree session belonged to another project (paragate) and was swapped for a plotplot worktree session.
- The worker sample spans pleach-v1 (8), weed (6), feature-map (6), plotplot (5), pleach-v1-umbel (3), plotplot-run (1), tibetanwriter (1).
- The evidence bar for filing: at least three occurrences across at least two sessions or two agents. Everything below it sits in "observed, not filed".

## Findings, ranked by cost

### 1. Conductors babysit `pleach run` with tail polls

Pattern. `pleach run` is backgrounded 88 times of 89 (bandung 55, beirut 17, cayenne 12, missoula 4). The conductor then waits by hand: 806 consecutive `tail` polls of the run log in 201 runs of three or more polls, across 25 sessions and 6 agents (first 2026-07-01, last 2026-09-16); 168 `jq` queries on the journal in 8 sessions (30 of them `select(.event=="verdict")`); 28 `until [...]; do sleep` loops in 6 sessions, three killed at the harness's ten-minute cap. The harness's own wait primitive (Monitor) was used 74 times against those 806 polls.

Counts. missoula 80c82780: 217 polls in 53 runs. bandung 204fc970: 175 in 46. cayenne (both sessions): 137 in 37. beirut 0cc973f6: 74 in 16. Evidence: bandung 204fc970 2026-09-09T16:26Z; missoula c9b6f479 2026-09-08T21:47Z to 22:24Z (six loops, two killed at 21:59Z and 22:14Z); cayenne 443a2c8e, 269 polls.

Consequence. Each poll returns about 1,000 characters into context: in three sessions alone 764 polls carried roughly 208k tokens of log tail (bandung 204fc970 78k, cayenne 443a2c8e 88k, missoula c9b6f479 42k). The wait is re-invented per session and the ten-minute cap kills the loops.

Proposal. `pleach wait <journal> [--node] [--timeout]` blocking until settle with one line per node, `pleach status` for the non-blocking glance, `pleach run --wait` as sugar; the conducting skill points at them. Filed as jahala/pleach#114 (P1), cross-referencing #63.

### 2. Agents hand-edit tend2 maps with python heredocs

Pattern. 455 writes to `.tend2.html` maps across 13 sessions and 6 agents (2026-08-18 to 2026-09-16): 166 through Edit or Write, 253 through `python3 - <<'PY'` whole-file string replacement, `sed -i`, `perl` or `node -e`, the rest through other shell. 115 of them append a dated ledger line (`- 2026-09-11 <who>: <what>`) to a check's Tried list (bandung 38, capetown 39, missoula 28, beirut 9). In capetown's session 26 of 41 `tend2 lint` calls came within two calls of a map edit.

Evidence. bandung 204fc970 2026-09-10T08:12Z; missoula c9b6f479 2026-09-11T14:37Z; capetown 6d9de859 2026-09-15T21:27Z; cayenne 443a2c8e 2026-09-16T16:15Z.

Consequence. Each Tried line costs a whole-map read into a heredoc, a regex that must hit the right check, and a lint round. The format is a contract only the linter knows, so every agent re-derives it; tend#211 (redact before transcribing) and tend#142 (a predict line) both assume tooling writes Tried lines and nothing does.

Proposal. `tend2 tried <map> --check <n> "<text>"`: dated, attributed, redacted, lint-enforced on write, one writer shared with verify's own transcription, named in the `next` work order. Filed as jahala/tend#218 (prio:1, area:ledger).

### 3. The landing chain is typed by hand, differently by each agent

Pattern. Across 8 sessions and 5 agents: `gh pr create` 62, `gh pr checks` 178 (48 `--watch`, 130 polled by hand, 25 as `sleep N; gh ...`), `gh run view --log` 71, `gh run view|list` 74, `gh pr merge` 88 in five flag shapes (`--merge --delete-branch` 28, `--merge` 20, `--rebase` 10, `--rebase --delete-branch` 3, four with hand-written `--subject`/`--body`), plus `until gh pr view ... != OPEN` loops (bandung 204fc970 2026-09-09T20:52Z, 20:53Z killed at ten minutes, 21:03Z). The `cppr` skill was invoked twice in the corpus.

Per session. bandung 204fc970: 10 creates, 51 checks, 45 run views. cayenne 7b42ed04: 14 creates, 57 checks, 31 merges. missoula 80c82780: 14 creates, 46 checks. beirut 0cc973f6: 5 creates, 21 checks, 9 merges.

Consequence. About 470 tool calls of plumbing; merge strategy chosen per agent, not per repository; branch deletion on 31 of 88 merges; CI waits as sleep loops the cap kills.

Proposal. `plotplot land`: push, PR, wait on the ruleset's required checks, merge with the repository's policy from `.plotplot`, delete the branch, print the sha and receipt. Filed as jahala/plotplot#93 (tier three).

### 4. Tools are re-pinned by cloning under /tmp

Pattern. 15 clone or trash-and-clone events of `/tmp/pleach-pinned`, `/tmp/tend2-pinned`, `/tmp/tend2-agent` in 4 sessions and 3 agents (2026-09-11T09:42Z to 2026-09-16T20:28Z; missoula a278e2e7 re-cloned tend2 four times on 09-16). 45 `bun install`/`npm ci`/build steps under pinned dirs in 7 sessions; 41 hand-made PATH shims in 3 sessions; `/tmp/tend2-pinned` named 147 times, `/tmp/pleach-pinned` 119; capetown refreshed `/tmp/plotplot-umbrella` with one three-command prefix 48 times. Pinned path and bare name mixed in one session (cayenne 443a2c8e: 12 pinned tend2 against 17 bare).

Consequence. Two agents pinning on the same day pin different commits; a replaced clone loses its `node_modules`; a stale bare `tend2` on PATH answers half the calls.

Proposal. The lock pins the conductor and the verifier and materialises them at a stable path. Already filed as jahala/plotplot#39; evidence added as a comment.

### 5. Bash written for Linux runs in zsh on macOS

Pattern. `no matches found` 63 results, 22 sessions, 6 agents (2026-07-02 to 2026-09-16); 18 from unquoted `--include=*.ts`. `${PIPESTATUS[0]}` in 38 commands, 6 agents, 12 sessions, printing `exit ` with no code (capetown 6d9de859 2026-09-05T13:44Z after a fatal `npm run check`); one agent learned `$pipestatus` and used it 42 times. `command not found: timeout` 12 results, 11 sessions, 6 agents.

Consequence. A red check read as green, an aborted command read as an empty result, an unbounded test run; each agent pays the lesson per session.

Proposal. A four-line shell section in the AGENTS.md block that `plotplot init` plants, held by `plotplot check`. Filed as jahala/plotplot#94 (tier two).

### 6. Leftover workers after a run are found and killed by hand

Pattern. `umbel ls` 158 calls, 8 sessions, 5 agents; 53 listings showed leftover `pl-` sessions (6 sessions); 11 bulk-kill shell loops over `umbel ls | awk '/^pl-/'` in 5 sessions and 4 agents; 69 `pgrep`/`ps` checks for a lingering `pleach run`; `pleach clean` answered "nothing to clean" 6 of 12 times while `umbel ls` in the same session still listed live workers (beirut 0cc973f6 2026-09-08T21:39Z and 22:46Z; missoula 80c82780 2026-09-08T22:12Z; capetown 6d9de859 2026-09-16T16:25Z).

Consequence. Seats and panes stay busy, the next run's `ls` is confusing, and a prefix kill can hit another conductor's live run.

Proposal. `pleach clean` and `pleach stop` kill the sessions the journal names (jahala/pleach#115, P2); `umbel kill` takes many names, `--prefix`, `--all`, `--dry-run`, and `umbel ls --json` (jahala/umbel#94, tier three).

### 7. Land conflicts stop pleach and get merged by hand

Pattern. "land conflict on node/X, resolve manually" or "culprit: X" 12 times in 3 sessions and 3 agents (bandung 204fc970 seven times on 2026-09-05 and 09-06; missoula 80c82780 three on 2026-08-20; beirut 0cc973f6 2026-09-10T22:43Z). In 8 of 9 continuations the conductor ran `git merge node/X` within four commands. Two `pleach land` calls were killed at the ten-minute cap mid land gate (bandung 2026-09-06T10:15Z, 2026-09-09T16:26Z).

Consequence. The hand merge carries no gate, receipt or journal event, and the next land may name a different culprit.

Proposal. On conflict, rebase the node onto the landed stack in its worktree, re-run its gate, land on green, refuse only on a rebase conflict; land gates settle through `pleach wait`. Filed as jahala/pleach#116 (P2), cross-referencing #98.

### 8. A check stamps with `tests: unknown`

Pattern. A stamp line ending in `tests: unknown` 8 times across 3 sessions and 2 agents, once on a suite that was skipped (cayenne 443a2c8e 2026-09-11T11:23Z, exit 0), four times on the umbrella's contracts loop (capetown 6d9de859 2026-09-11T11:20Z, 09-15T22:13Z, 09-16T16:43Z and 18:33Z), where the receipt's audit reads `pass` beside the unknown. 100 of the 225 `--runner` values in the corpus are the `bash scripts/check/run.sh {evidence}` shape that yields this.

Consequence. A skipped suite reads as proven downstream.

Already filed as jahala/tend#207; evidence added as a comment.

### 9. Ack rounds and inbox polls stand in for a delivery receipt

Pattern. 550 pollen sends and 471 inbox reads across 5 agents; 65 percent of sends over 300 characters. The umbrella wrote "no ack" into 34 of its 217 sends and two other agents picked the phrase up. 92 runs of consecutive inbox polls with no send between (capetown 28 runs, 72 polls); 60 reads returned nothing, with the watcher running.

Consequence. Message rounds spent on receipts, or on asking not to send them.

Already filed as jahala/pollen#20 (a sender cannot tell whether a peer received or read a message); evidence added as a comment.

## Summary table

| # | pattern | n | sessions | agents | first | last | cost | issue |
|---|---|---|---|---|---|---|---|---|
| 1 | tail polls after `pleach run` | 806 polls, 201 runs | 25 | 6 | 2026-07-01 | 2026-09-16 | ~208k tokens in three sessions; 3 loops killed | pleach#114 |
| 2 | hand edits to `.tend2.html` | 455 | 13 | 6 | 2026-08-18 | 2026-09-16 | whole-map heredoc + lint round per line | tend#218 |
| 3 | landing chain by hand | 62 create, 178 checks, 88 merge, 145 run views | 8 | 5 | 2026-07-24 | 2026-09-11 | ~470 calls; policy per agent | plotplot#93 |
| 4 | re-pin by clone under /tmp | 15 re-pins, 45 installs, 41 shims | 7 | 3 | 2026-09-11 | 2026-09-16 | divergent pins, lost node_modules | plotplot#39 (comment) |
| 5 | zsh glob, PIPESTATUS, timeout | 63 + 38 + 12 | 22 | 6 | 2026-07-02 | 2026-09-16 | lost exit codes, aborted commands | plotplot#94 |
| 6 | leftover workers killed by hand | 158 ls, 11 loops, 69 pgrep | 8 | 5 | 2026-07-02 | 2026-09-16 | seats held, prefix kills | pleach#115, umbel#94 |
| 7 | land conflicts merged by hand | 12 | 3 | 3 | 2026-08-20 | 2026-09-10 | unrecorded merges | pleach#116 |
| 8 | `tests: unknown` stamps | 8 | 3 | 2 | 2026-09-11 | 2026-09-16 | skipped suite reads as proven | tend#207 (comment) |
| 9 | ack rounds, inbox polls | 34 "no ack", 92 poll runs | 6 | 5 | 2026-09-05 | 2026-09-16 | message rounds | pollen#20 (comment) |

## Observed, not filed

- guard-git hook blocks: 66 across 17 sessions and 6 agents. `git init` in a fresh `mktemp -d` blocked 22 times (5 agents), `git worktree remove` 12 (5 agents, mostly pleach's own worktrees), force push 10, `git branch -D` 7, `.git` deletion 7. The hook also blocks any command whose text merely contains the phrase (this analysis's own grep was blocked once). The hook lives in `~/.claude/hooks/`, not a garden repository. Recommendation: allow init when the cwd has no `.git` and sits under the temp directory; allow worktree removal for paths under `.git/worktrees/*/pleach/worktrees`.
- "Shell cwd was reset" 442 results in 15 sessions; 2,099 of 7,653 Bash commands (27 percent) begin with `cd <path> &&`. Harness behaviour in agent threads, no garden fix.
- Red-phase gate refusing an existing red test: jahala/pleach#113, filed on 2026-09-16 by the agents themselves; "red phase" appears in 50 results across 12 sessions.
- tend2 verify per-check timeout: jahala/tend#181 already open; the 14 `timed out after` results in this corpus are harness caps on `until` loops, `pleach land`, a merge and a cargo build, not verify.
- Verify re-run loops: 10 (session, map, check) tuples with 3 to 5 verify calls; the red-green loop working as designed.
- weeder: 53 checks, 3 blocks (bandung, cayenne, one worker), 22 warns; after a block the agent inspected the diff and reworded; `Weeder-allow` trailers mentioned 20 times. No consistent misuse above the bar.
- Workers: the 30 sampled sessions run the suite six times on average (max 13); 2 of 30 install dependencies; 2 symlink `node_modules` into a temp worktree; none land PRs (pleach lands). Expected shape.
- `pleach validate` precedes only 25 of 77 runs; no cost seen since run validates.
- Edit refusals "File has not been read yet": 16, all missoula in 2 sessions; a single agent's habit.
- pollen "go" versus "resume": "resume" in 83 sends (capetown 53). A misread cannot be shown from tool-use lines alone; not counted.
- A process listing in one tool result carries an API key. The class is filed as jahala/umbel#93; nothing quoted here.
- tend2 verify's judge creates one harness session per run (78 files on 2026-08-19 in the missoula directory). Worth knowing when `~/.claude/projects` grows.

## Issues filed and comments added

| repo | number | title | kind |
|---|---|---|---|
| jahala/pleach | #114 | Conductors poll the run log with tail because pleach has no wait or status face | new, P1 |
| jahala/pleach | #115 | pleach clean leaves the run's worker sessions alive; conductors kill them by hand from umbel ls | new, P2 |
| jahala/pleach | #116 | A land conflict stops pleach and the conductor merges the node by hand outside the record | new, P2 |
| jahala/tend | #218 | Agents append Tried lines with python heredocs and sed; a tend2 tried face writes them | new, prio:1, area:ledger |
| jahala/tend | #207 | verify stamps a check whose runner ran zero tests | comment with counts |
| jahala/plotplot | #93 | Every agent lands pull requests with its own gh chain; a plotplot land face carries the repository's policy | new, tier three |
| jahala/plotplot | #94 | Agents write bash for Linux in a zsh on macOS: unquoted globs abort, PIPESTATUS is empty, timeout is missing | new, tier two |
| jahala/plotplot | #39 | The lock pins the conductor and the verifier, not only the judges | comment with counts |
| jahala/umbel | #94 | umbel kill takes one name, so conductors loop over ls to kill leftover workers | new, tier three |
| jahala/pollen | #20 | A sender cannot tell whether a peer received or read a message | comment with counts |
