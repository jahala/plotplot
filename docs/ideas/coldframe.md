# coldframe, the bench where tests run before work goes out

2026-10-30. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`). A cold frame is the low glass box beside a garden where seedlings harden off before they go outside. coldframe is where a change's tests run, on the owner's own machine, before the change goes out.

This draft follows the CI review and plan written in the accelerando workspace (`docs/ci-review-2026-10-29.md`, `docs/ci-machine-tooling-plan.md`) and two groundwork reports in the plotplot-ai workspace (`docs/research/coldframe-groundwork-ci-2026-10-30.md`, `coldframe-groundwork-garden-2026-10-30.md`). Those carry the evidence. This page carries the shape.

## Who it is for, and their pain

An owner who builds many projects with agents, and runs their tests and CI on their own Mac to keep paid minutes near zero. On 2026-10-29:
- **Runners:** 29 self-hosted runners on an 18-core Mac held 35 GB, 6.5 GB of it logs and old runner versions that nothing removes.
- **Waits:** one project's heavy jobs queued about 10 minutes and then waited up to 19 more for a quiet machine, to run 3 to 6 minutes of tests.
- **Paid minutes:** they stopped CI twice, at $25 in September and $25 in October.
- **"A run leaves nothing behind":** built six separate ways in six projects. A machine-wide "wait for a quiet machine" script exists in three versions.
- **Rules nothing checks:**
  - every project's test strategy with measured times;
  - tests that hold on a loaded machine;
  - leave nothing behind;
  - spend paid minutes sparingly;
  - re-check what merges together.

Each project solves the same problems again, and none of the fixes bounds the machine as a whole.

## What it owns, and what it does not

coldframe owns how tests run: where, when, at what weight, at what cost, and that each run leaves nothing behind. It has two sides of one job:
- **the machine:** the runners on it and the load they put on it;
- **the project:** its declared test tiers and the hygiene of each run.

It does not own:
- **judging a change to tests or workflows.** Deleted, skipped or weakened tests and workflow edits are weeder's rules T1 to T8 and C3, read from the diff.
- **writing a project's workflows, git hooks or lock.** Those are the stem's. coldframe reads them and reports.
- **stamping a claim as passed.** Only tend2's verifier writes pass.
- **pacing agents or the subscription seat.** That is the gardener's and well's.
- **deploys.** Those are transplant's.
- **choosing what a project tests.** That is a person's call, with the test-setup skill's method. coldframe holds the declaration, measures it and keeps it true.

It is not a server and not a daemon (law section 6). It is one command, run by the runner's job hooks, by LaunchAgents the owner installs, and by hand.

## What it can do, in families

| Family | Capability | What it does |
|---|---|---|
| **A. The bench (machine)** | A1 runners | `runner add`, `remove`, `adopt`: set up, remove, or bring existing runners under coldframe. They replace the ci-runner skill's scripts. A runner is restarted only while it is idle. |
| | A2 job hooks | `hook started` and `hook completed`. Completed clears the checkout, as `after-job.sh` does today. Started takes a weighted slot and keeps the Mac awake for the job. |
| | A3 weighted slots | A machine-wide cap counted in weight, not jobs: a heavy tier uses more of the machine than a light one. This answers the port-exhaustion failure that one project now avoids only by running heavy jobs one at a time. A wait has a ceiling, after which the job runs and an overflow is counted, so a slot bug slows CI but never stalls it. |
| | A4 lifetimes | `prune`: runner logs older than 7 days, superseded runner versions, update leftovers, and coldframe's own logs. |
| | A5 health | `status`: every runner online, busy or idle; slots and who holds them; disk per runner; awake and logged in; the last prune and audit with their outcomes. It exits non-zero when unhealthy, and a notification goes out only then. |
| | A6 measure | `measure --since 14d`: the job timeline per repository and machine-wide, queued against running time. It is the instrument for every before-and-after claim. |
| **B. The test plan (project)** | B1 declare | A project declares its test tiers once: name, command, when it runs (each change, heavy, main line, release), weight, and whether it needs a clean machine. The home is a `tests` block in the project's `garden.json`, added as a contract in the umbrella beside `smoke`. It is not a second file. |
| | B2 measure times | `time`: runs each tier and records its measured time and the machine it ran on, under `.coldframe/` in the project. |
| | B3 keep the record true | `check plan`: the test strategy in the project's CLAUDE.md names every declared tier, and its stated times are within twice the measured ones. A stale time is reported. |
| | B4 pick what a change touches | `affected`: given the changed files and the tiers' declared paths, which tiers a change must run. When unsure, all of them, as the owner's rule says. |
| **C. Run hygiene** | C1 a run scope | `run <tier>` runs a tier inside a scope coldframe opens. The scope gives it its own temp folder and its own port range, then records what the run started. After the run, it reports any file outside the scope, any process still alive and any container or image left. It removes what it made, and fails the run when anything leaked. This is "leave nothing behind" in one home, replacing six designs. |
| | C2 under load | `run --load`: the same tier with the machine deliberately loaded, so timing assumptions break here first, not on CI. |
| | C3 test for the rule | Every project's suite gets the check that "a run leaves nothing behind" from C1, without writing its own. |
| **D. Audit (reports, never changes)** | D1 cost | The month's paid minutes from the billing API, and which repositories and jobs spent them. |
| | D2 workflow traps | The ci-runner skill's five traps, read from every repository's workflow files: push without a branch filter beside pull requests, schedules re-testing unchanged commits, hosted jobs in private repositories, fixed shared names and ports, and missing way-back variables. |
| | D3 runner safety | Self-hosted runners on public repositories, and private repositories with outside collaborators, where a pull request runs code as the owner. Plus runners on repositories idle for 30 days. |
| **E. Faces for agents and git** | E1 skill | How an agent runs a project's tests: `coldframe run` for the tiers a change touches, and what a red result means. It replaces the test-setup and ci-runner skills' mechanics, while their judgement stays as text in the skill. |
| | E2 pre-push guard (optional) | Through the stem's git dispatch, `coldframe guard pre-push` runs the tiers the change touches before a push leaves the machine, so CI rarely runs a change that fails locally. |

Out of scope, named so nothing drifts in:
- **Landing a batch of pull requests together** (drafts, one merge branch, one CI run, faults placed by changed files). It needs an owner, but that owner is a landing tool, not coldframe.
- **Linux and Windows runners, and moving repositories into an organization.** These come later, if ever.

## Guardrails

These are what coldframe never does. Each one gets a test that breaks it once.

1. **It changes no project's tests, workflows, hooks or lock.** Its audit reports, and that is all.
2. **It deletes only what it made or what a runner made, and it says which.**
   - Runner files: logs, old versions and update leftovers inside `~/actions-runner-*`.
   - Files a `run` scope created.
   - Nothing else, ever.
   - It uses `trash` by default. Deleting for good is the owner's stated exception (open question 1), and it is limited by code to runner-made files.
3. **It never restarts a busy runner.** It checks for a running job worker just before each restart.
4. **A slot can slow CI, never stall it.** A wait has a ceiling, then the job runs and the overflow is counted. A crashed job's slot is reclaimed when its process is gone.
5. **Hooks never call GitHub and never read a key.** The runner's session cannot read the Keychain. `audit` runs in the owner's normal session, from a LaunchAgent.
6. **It refuses to add a self-hosted runner to a public repository.** A pull request from anyone would run code as the owner. Its audit flags any that exist.
7. **It spends nothing.** It reads billing, it never buys minutes, and it changes no spending limit.
8. **Its own data has a lifetime.** Slots are bounded by their count, its logs are kept 30 days, audit reports are capped at the last 12, and measured times are kept per tier as only the latest.
9. **It never writes "pass".** It reports an exit code and what leaked. tend2's verifier decides a claim.

## How it is added

- **To the machine,** once: install it with `cargo install --locked --git` at a tagged commit, then `coldframe runner adopt` points every runner's hooks at it, one idle runner at a time. Two LaunchAgents run in the owner's session: `prune` daily and `audit` weekly.
- **To a project** ("the repository is the harness"): the stem plants it like any bed. `garden.lock` pins it, and `plotplot init --beds coldframe` installs the skill and, if chosen, the pre-push guard. The project declares its tiers in its `garden.json` `tests` block. coldframe writes only `.coldframe/` (measured times, git-ignored or committed, as the owner chooses) and changes nothing else.
- **Alone,** with no garden: everything works with a hand-written `tests` block and the runners on the machine. It needs nothing else.

## What code decides

These are all pure functions from data to data, with the disk, the process table and `gh` at the edges:
- which slot to take, given the slot folders and which processes are alive;
- what to prune, given a runner folder's listing, its links and today's date;
- which tiers a change touches;
- what a run left behind;
- which audit findings hold.

There is no model and no judge.

## Alone, and with the garden

- **Alone:** a machine tool and a project test bench.
- **With weeder:** weeder blocks the diff that weakens a test, and coldframe runs the tests that remain.
- **With tend2:** a verify script can call `coldframe run` and read its exit code and leak report as evidence.
- **With the stem:** the stem plants it and dispatches its pre-push guard. The stem's `garden` workflow job should take its runner label from coldframe's convention (`CI_RUNS_ON`), not hard-code a hosted runner. Today it hard-codes one, and hosted runs fail for private repositories.
- **With the gardener and almanac, later:** `status` and `measure` are what they would read.

## Data and its lifetime

| Data | Where | Kept |
|---|---|---|
| Slots | `~/Library/Application Support/coldframe/slots/` | bounded by the slot count; a dead holder's slot is reclaimed |
| Logs | `~/Library/Logs/coldframe/` | 30 days |
| Audit reports | `~/Library/Application Support/coldframe/audit/` | the last 12 |
| Measured times | the project's `.coldframe/times.json` | the latest per tier |
| Config | `~/.config/coldframe/config.toml` (slot weights, wait ceiling, log lifetime) | the owner's |

## Metric and kill criterion

**Metric:** the owner's machine-wide CI health, measured by `measure` before and after:
- paid minutes per month (target $0);
- the 90th-percentile wait of the slowest repository's heavy jobs;
- runner disk growth per week;
- the count of per-project "leave nothing behind" and "quiet machine" mechanisms (target one, coldframe's).

**Kill criteria:**
- **Slots:** if `measure` shows CI ran more than the slot cap at once in under 5 percent of busy minutes, the cap changes nothing, so slots (A3) are dropped.
- **Run scopes:** if C1's run scope cannot reproduce the verdicts of two projects' existing leftover checks (it misses a planted leak or flags a clean run), the per-project checks stay and C1 is dropped.
- **Size:** if, once projects have moved over, coldframe is not smaller in total than the scripts and per-project mechanisms it replaced, it was the wrong shape.

## Build order, each step small first

The garden's pre-flight rule holds for every step: list the step's assumptions, check each at $0 on real data, dry-run, then fix its bar and control.

0. **Measure and spike, before the tool exists.**
   - Run `measure` as a script over the last 14 days of jobs in the self-hosted repositories, excluding load the owner's agents put on the machine by hand.
   - On one runner of a quiet repository, show that a started hook can block a job, that its wait counts toward the job's timeout, and which job variables the hook can see.
1. **Lifetimes and health (A4, A5, A1's `adopt`).** This is the disk fix, and it does not wait on step 0.
   - Bar: tested on a copy of a real runner folder, then run on all runners, with the measured sizes dropping by the expected amount.
   - Control: a deliberately broken prune turns `status` red.
2. **Run scope (C1), proven on a known case.** Pre-flight: a run scope can offer a temp folder and a port range, but only tests that read them honour them. Check first how many of the two projects' tests take their temp folder and ports from the environment. Plant a leak in a copy of each of two projects, graft and mull, whose own leftover checks already catch leaks.
   - Bar: coldframe catches every planted leak and passes the clean runs, the same verdicts as the projects' own checks.
3. **Under load (C2), proven on a known case.** Run the umbrella's `two_beds_run_at_the_same_time` test both ways: at the commit before PR 189 (the timed version, which failed on loaded CI) and after it (the rendezvous version).
   - Bar: under `--load`, the old test fails and the new one passes in 10 of 10 runs.
4. **The test plan (B1 to B3).** Add the umbrella contract for the `tests` block, then declare it in two projects.
   - Bar: `check plan` flags a stale time planted in a copy of a CLAUDE.md, and passes the true one.
5. **Weighted slots (A3), only if step 0 passes.** Small batch first, on two repositories.
   - Bar: under a burst from three repositories, the weight in use never exceeds the cap, and a killed job's slot is reclaimed within one poll.
6. **Audit (D).**
   - Bar: rules are tested against workflow files and API answers recorded today, and once against the live API. It finds the review's open items and nothing the review cleared.
7. **Faces (E) and hand-over.** The skill replaces the mechanics of ci-runner and test-setup, and projects drop their own mechanisms one at a time, each with its `measure` before and after.

## Open questions

1. **Deleting runner files for good** (the owner's call). Either a stated exception, limited by code to runner-made files inside the runner folders (recommended), or turning on Finder's 30-day Trash setting. Until then, prune uses `trash`.
2. **Whether `.coldframe/times.json` is committed** (so CLAUDE.md's times can be checked in CI) or git-ignored (so every machine measures its own).
3. **The stem's `garden` workflow job** should take its runner from `CI_RUNS_ON`. That is a change in the stem, proposed to its owner, not made by coldframe.
4. **Who owns landing a batch of pull requests.** It is out of scope here, and still needs a home.
