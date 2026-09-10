# Does the harness pay for itself? The planted-versus-bare experiment

Status: plan, 2026-09-10. The first measurement after wave one, ahead of the harness loop and
signed receipts, because everything after it assumes the answer is yes. Companion to the orient
experiment (jahala/plotplot issue 3), which adds a third arm once this one has a baseline.

## 1. The claim

A repository planted by the stem (weeder on commit and stop, the deny list, the garden block,
the lockfile) lowers cost per correct answer on non-trivial tasks by more than it raises it on
trivial ones, on the same repository, under the same harness and model, and lowers the number
of human interventions per task.

The prior is mixed. Every hook adds latency and a few tokens; every refused stop costs a turn.
Against that, a refused test deletion or a caught stub saves a retry a person would otherwise
pay for later. Nobody has measured which side wins, and the garden's own claim is that code
decides, so this is the number the garden owes.

## 2. The arms

| arm | what the worker gets |
|---|---|
| bare | copeca's baseline: a minimal environment, no hooks, no MCP, no instructions beyond the task |
| planted | the same, plus `plotplot init --profile minimal` run in the worktree as the mode's `setup`, and the stem's Claude Code bundle as the mode's `agent_config` overlay, so PreToolUse, Stop and the git hooks fire |

One variable changes between arms, which is what copeca's modes exist for. The planted arm's
validity gate is the friction journal the hooks write into the worktree: an arm whose journal
holds no hook event did not run planted, and its result does not count.

## 3. The tasks

copeca's existing corpus, which needs no minting: fifty-two tool-agnostic tasks on ripgrep, gin,
express and fastapi, each pinned to a commit, each verified to fail on mutated code, plus the
six-task control set where no tool should help. The split the claim needs is already in the
corpus: `fix` and `debug` tasks are the non-trivial set, `locate` and `trace` the trivial one,
and the control set is the regression guard. Three repetitions per cell, the seed fixed where
the runner allows one.

## 4. What is measured

- Cost per correct answer per arm per set, with intervals, from `copeca analyze`.
- The control delta: a planted win on tool-neutral tasks means the arm did something other than
  judge, and the result is suspect.
- Interventions per task from the friction journal: refused stops, denied commands, hook
  latency in the harness's own timing.
- Wall clock per task, because a hook that costs seconds per tool call shows up here first.

## 5. Kill criteria

- If the planted arm is not cheaper per correct answer on the non-trivial set by more than the
  interval, the harness does not pay for itself on agent work, and the stem's default profile
  is wrong until it does.
- If the planted arm loses on the trivial set by more than it wins on the non-trivial set, the
  same, because most sessions are trivial.
- If hook latency exceeds one second per tool call at the median, the dispatcher's budget is
  the first fix, before any result is read.

## 6. What is built to run it

One scenario file, `scenarios/harness-payoff.yaml` in the copeca repository, with the two modes
above; the stem's bundle already exists and `init` already plants a fresh worktree. No new
tool, no new bed. The scenario runs on the owner's machine under umbel, since the subscription
CLI lives there, one worker at a time within the session window; the results artifact lands in
the umbrella under `docs/research/harness-payoff-2026-09/` with copeca's integrity manifest.

## 7. Decisions for the owner

- Which model runs both arms: the recommendation is Claude Opus 5, the model the garden builds
  with, so the result describes the work it does.
- Whether to run all fifty-two tasks or a first slice of twenty, three repetitions each; the
  slice answers the kill criteria faster and the full run follows if it is close.
