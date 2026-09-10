# Dogfooding tend2, pleach and umbel while building the stem

Kept by the stem builder (agent beirut) during the stem build, September 2026. One line per
finding, dated, with the command that showed it. Misunderstandings, faults, missing features
and bugs in the garden's own tools, as met while using them for real work. Nothing here is a
fix; each was filed on the tool's repository on 2026-09-09 and 2026-09-10 or matched to an issue
already open: tend 157, 158, 167, 169, 175, 183 and 184; pleach 61, 63, 64, 66, 68, 69, 70, 71,
74 (79 closed as its duplicate), 77, 81 and 82; umbel 65, 67, 69, 70 and 75; pollen 20, 21 and
22; weeder 31 and 32. The Conductor finding has no garden repository.

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
- 2026-09-08 `tend2 next docs/tend2` reports "12 asks held back, their loops have no machine
  evidence to judge yet" without naming which loops or how to surface them; the human check on
  the stem loop is invisible until a code check has evidence.

- 2026-09-09 `tend2 verify` with `{evidence}` in `--runner` expands to the script path alone
  and drops the evidence line's argument (`scripts/fit/stem.sh bundles` runs as
  `bash scripts/fit/stem.sh`), and a `.sh` evidence has no default runner; the shape the law
  prescribes for fit evidence needs `--runner "bash {evidence} bundles"` per check (tend 184).
- 2026-09-09 `tend2 verify --check N` without `--force` reports `skipped-fresh` for a stamp
  written in another worktree; correct, since the stamp id is a payload identity and not a
  commit, but the word "fresh" reads as a claim about time. Not filed.

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
- 2026-09-08 pleach's run journal names workers only as `pl-<hex>`; after a SIGTERM abort the
  journal says "worker seam error: wait exited 143" and not which umbel session it was, so
  the operator cannot match sessions to nodes from the journal either.
- 2026-09-08 retracted: what looked like pleach spawning two workers for one node was two
  conductors on one machine sharing umbel's global session list (see umbel below). The
  teardown-leak line above stands corrected the same way: the session I killed after the
  abort was another run's.

- 2026-09-08 pleach executes `accept.smoke` and `accept.audit.command` as an argv array
  with no shell, so a smoke of three commands joined by `&&` fails with "command contains
  bare shell operator(s)". `pleach validate` accepted that plan without a word, so the first
  the conductor heard of it was after a full opus attempt of the crate node (about fifteen
  minutes of work, complete and green by hand) was thrown at a gate that could never run.
  validate should refuse or warn on a bare shell operator in a gate string, since the
  failure is deterministic and costs one attempt per node.
- 2026-09-08 on a gate failure pleach re-prompts the worker with "previous attempt failed;
  fix this and continue" and the gate's output tail. When the failure is in the plan (the
  gate string itself), the worker cannot fix it from its worktree and burns a second
  attempt confirming the code is green. A gate that fails before running any command
  (exit -1, "cannot exec") should fail the node without a retry and name the plan.
- 2026-09-08 the run journal records `gate-retry` with no output; the gate's stdout and
  stderr live only in the worker's re-prompt. The conductor's operator has to open the
  worker's transcript to learn why a gate failed.
- 2026-09-08 a SIGTERM abort ends with "failed after 0 attempt(s)" and no quarantine
  branch for a node whose worker had finished its slice; the work survives only in the
  pleach worktree until `pleach clean` sweeps it. Snapshotting the worktree's index by hand
  (`git write-tree`, `git commit-tree`) recovered it this time.

- 2026-09-09 pleach reads the plan once at run start, so a plan edit during a run (an audit
  provider whose vendor went down, a prompt correction for a node not yet started) cannot
  reach later nodes; the only path is to stop the run, edit, and re-run on the resume ledger.
  A `--plan-reload` on wave boundaries, or reading the file at each node start, would let a
  conductor recast an auditor without an abort.
- 2026-09-09 `pleach run --runner` casts the builder, but nothing at run time casts the
  auditor; when codex died mid-evening the only way to keep the integration node's audit was
  to edit the plan. An `--audit-provider`/`--audit-model` override would match the builder's.

- 2026-09-09 an audit whose command printed test output failed as `audit-egress-unparseable`
  on opencode and then twice on codex. Reading `src/core/audit-egress.ts` settled it: the
  auditor is a relay, and the audit command itself must print a fenced `tend-audit-result`
  JSON block (what `tend2 verify --audit-egress` prints, one check per invocation); pleach
  takes the last block in the reply. Neither `pleach schema`, `pleach --help`, nor the
  pleach-plan skill says so, and the skill's example (`Adversarially verify: <finding>`)
  suggests a free-form command. The plan contract should say what an audit command is, and
  `validate` could warn when an audit command is not a `tend2 verify … --audit-egress` form
  (pleach 77 amended). A `pleach audit <node>` face would still have saved the hand landing.

- 2026-09-09 collection ran `git add -A -- <touched paths>` including the worker's
  `.loop-scratch/` files, which `.gitignore` ignores; git refused, pleach called it a
  catastrophic isolate failure, the node failed after zero attempts with no quarantine, and
  the worktree was gone. Forty minutes of opus work recovered only by replaying the worker's
  transcript (pleach 79). pleach's own prompt promises that directory is never collected.

- 2026-09-09 a command node's verdict records provider claude and model claude-opus-5 though
  no worker ran (pleach 81); a node id that runs again overwrites its receipt file, so the
  failed audit's reasons were lost when the re-run passed (pleach 82).
- 2026-09-09 the doctor-live worker's session limit reached pleach as "Claude is waiting for
  your input"; the reset time was on the pane (umbel 75, commented).
- 2026-09-11 the fourth run's first attempt was lost at the commit gate: the node printed
  "commit gate failed" and pleach kept nothing, no `node/` branch, no quarantine, no receipt
  (UNDERIVABLE), the worktree gone and the worker's umbel session killed with its transcript.
  The commit was refused by the repository's own planted pre-commit hook, because a fresh
  worktree carries no `.plotplot/bin/` (ignored) and the hook fails closed without its judge;
  the plan's setup now runs `plotplot lock verify` first. A refused commit should quarantine
  the tree with the hook's words, never lose it (pleach 96; the hook's wording is
  plotplot 30).

## umbel

- 2026-09-08 `umbel ls` lists four `smk-trust-*` sessions dead since 2026-09-05 in temp
  directories that no longer exist; nothing prunes dead sessions, so the list grows forever.
- 2026-09-08 `umbel ls` is global across every conductor on the machine, shows the cwd
  truncated to its last path segments (`…pleach/worktrees/wt-xny6KM/wt`), and has no filter by
  repository or spawning process; two pleach runs in two repositories produce
  indistinguishable `pl-*` rows. I killed two sessions belonging to other agents' runs on the
  strength of that list. `umbel ls --cwd <prefix>` or an owner column (the spawning pid or
  a label pleach passes) would have prevented it; pleach should pass `name: <plan>-<node>`
  instead of an anonymous `pl-<hex>`.
- 2026-09-08 `umbel kill` deletes the session's state, including `meta.json` with the cwd,
  so after a mistaken kill nothing says whose session it was. `keepState` exists but the
  default erases the evidence.
- 2026-09-09 umbel has no wedge detection: a worker idle with a live session is not read as
  failure, so a wedged node waits out pleach's timeout (30 to 60 minutes here) unless the
  conductor aborts by hand (cape-town's observation on tend2's run, 2026-09-09).
- 2026-09-08 `umbel ls` shows a dash in the MODEL column for a freshly spawned claude worker for
  its first minute or so, then `claude-opus-5`; a row with no model looks like a probe or a
  failed spawn rather than a worker that has not reported yet.

## pollen

- 2026-09-09 a sender cannot tell whether a peer read a message; the umbrella's silence for
  an hour looked the same as a lost message (pollen 20).
- 2026-09-09 `pollen_inbox` prints messages without the journal's timestamps (pollen 21).
- 2026-09-08 `POLLEN_ALLOW` is read once at server start and `pollen_allow` does not persist;
  the project .mcp.json written after the session started had no effect on it (pollen 22).

## weeder

- 2026-09-09 `weeder check --strict --base origin/master` blocked (T2) on src/harness.rs
  because a test's three explicit `assert_eq!` calls became one assertion inside a loop over
  the same three cases; the count of assertion sites fell from 81 to 79 while the cases
  checked stayed three. A static site count reads a loop as a loss. Restored the three
  explicit assertions to pass the gate; one false block for weeder's calibration ledger
  (weeder 31).
- 2026-09-09 T5 warned "a recorded expectation changed alongside the code it judges" on two
  fixture payloads that were added, not changed, in the same change as the tests reading
  them (weeder 32).

## conductor (workspace tooling)

- 2026-09-08 the Conductor workspace was seeded on an empty "Initial commit" root with no
  relation to `origin/master`, and its target branch is `origin/main`, which does not exist on
  jahala/plotplot (the default branch is `master`). Re-pointed the branch with
  `git checkout -B <branch> origin/master` before any work could start.
