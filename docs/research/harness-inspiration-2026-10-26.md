# What seven harness projects teach the garden, October 2026

Written 2026-10-26 by the umbrella agent, at the owner's request. Five Sonnet agents each read
one or two sources in full (the repository at a pinned commit, the paper, and the discussion
thread where one existed) and wrote a report with evidence. The umbrella agent then checked the
load-bearing claims against the cloned sources and against what pleach and tend2 hold today.
Products are described, never named; papers are cited by arXiv id. The owner keeps the links.

## The short version

1. Nothing read here changes the thesis. In every project, a model or a regex over the agent's
   own words decides what is adopted or "verified". Agents produce, code decides remains the
   garden's own claim, and it is now also the one nobody else has measured.
2. The field has written down the bar for "better", and most published harness gains do not
   meet it. Of 95 in-scope comparisons in one audit, 10 showed the change ran in the scored
   runs and 10 isolated the gain from run-to-run variation. Repeats of a task are not more
   tasks. Our "nothing ships unmeasured" needs a task count and a margin, written into the
   artifact.
3. The cheapest wins are design inputs for the two beds not yet built, hedge and stake, and
   three small nodes for pleach. The rest is a short list per bed, each with the check that
   would prove it.

## What to take, by bed

### hedge, the boundary (not built; the direction memo ranks it first)

- Compile to every harness and prove the deny fires. One harness editor reads a hooks file in
  the same schema as the harness we use most, but ignores it unless a trust flag is passed, and
  malformed hook JSON never fires rather than erroring. A hooks file that is present and silent
  is the failure to design against. Hedge ships a canary per harness: through umbel, attempt one
  denied call and fail the install if the deny reason does not come back.
- Allowlist the whole command, never denylist its first word. A probe of one kit's authority
  check showed `nice`, `xargs`, `sudo`, `timeout` and `time` in front of a forbidden git call all
  pass. Two projects carry a test corpus for this: wrapper unwrapping, heredoc bodies, a nesting
  bound, parse failure denies, and a hard deny never downgraded to ask. Take the corpus as
  hedge's first tests.
- A deny names the sanctioned replacement, from a table (`rm` to `trash`, `git reset` to the
  revert path). The agent can act on the reason instead of retrying variants.
- Precedence as a table-driven test: deny beats ask beats allow, a saved allow never beats a
  configured deny, an unknown profile denies, the last matching rule wins.
- Allow rules carry an expiry, and the policy file sits outside the agent's writable paths.
- Where a harness can both block a tool and hide it from the model, hedge compiles to both.

### copeca and every bed's numbers

- A claim record for "A beats B": the unit (a task), the count of independent tasks, repeats
  per task, how variation was measured, resources matched (calls, tokens, turns, attempts), and
  the margin test. An artifact that lacks one of these cannot say "better".
- Print the smallest certifiable difference beside every calibration set. With three candidates
  and one round, 32 items certify only a gap of 0.51 and 296 items a gap of 0.17 (a Hoeffding
  bound, conservative). Graft's calibration tracks hold 14 to 32 items, so they rank candidates;
  they do not certify one.
- A noise floor: run one known-null pair in every report and print its gap.
- Ledger identity: derive the expected call count from the plan, record each metered call under
  a unique label, and fail the artifact on a duplicate before a count mismatch, because a
  duplicate can satisfy the count.
- Regrade as a first-class command. One paper found grader defects in a standard benchmark that
  moved scores by up to 15 points, unevenly across models, larger than most effects it
  reported. Store the grader version and the pre-regrade result on each record, and print
  as-graded, the delta per correction, and newly failing items.
- A status class per call (ok, infrastructure failure, model error, truncated, empty), with the
  infrastructure-excluded count beside every score. Report a lower bound on a gap, never
  "matches".

### pleach

- Per-node write scope. A node declares the globs it owns and may append to. After the worker
  stops, diff the base commit against HEAD and the worktree, revert strays, keep a copy under the
  receipt, and scan the node's own commits, because a stray that was committed has left
  `git status`. This extends the v1.1.5 tamper rule, which refuses a node whose builder touched
  an audit command; today a node has no scope field. Verified against the pleach checkout.
- Sealed checks. A check marked sealed keeps its command and expected values outside the
  worker's worktree, and the worker sees pass or fail only. Add the judge-side assertion from the
  harness editor: fail the audit if an expected answer or a prior verdict string reaches the
  auditor's input.
- Fingerprinted resume. The receipt today seals facts and derived values under one hash; it
  does not hash the prompt, the input files and the worker configuration. Resume reuses a node
  only when that hash matches, and prints why otherwise.
- Cut-off workers. A session that ends at a token cap, a timeout or a context overflow has its
  plan and notes writes dropped, and the next node gets a bounded summary. A node whose prompt
  repeats verbatim stops the loop. Only the first failing case, capped, goes into the next
  implement prompt.
- A run board. Typed notes per run (fact, observed, fail, claim), a fail carrying a command and
  an exit code, the newest few rendered into sibling prompts, so parallel nodes stop retrying
  what a sibling disproved. A weak-evidence phrase check ("should work", "looks right", "tbd")
  runs on node notes before the auditor spends tokens.
- Blind audit. The auditor sees anonymised candidates from a seeded shuffle, and artifacts only.
- Plan quota in the receipt. Two providers expose the subscription's usage and reset time; read
  it before spawning. Our one-seat pacing becomes a check instead of a habit.

### tend2

- Checks by description. A check carries a description beside its command. The description
  resolves once per project into a ledger; an unresolved check refuses the start; a check that
  needs a missing resource reports skipped, never failed; an unknown stays unknown under
  negation. Issue 220 already asks for the unevaluable fold; this is the resolve step before it.
- Activation levels for anything a bed ships into a harness: available (the diff), reached (a
  receipt line shows the changed path ran), carried (an adherence check on the output). Record
  the level on the loop page. Updating a harness is not benefiting from it.
- A kill-criterion lint on every loop: an invalidation condition, observable signals, a time
  horizon. A check that errors out reports unresolved, never pass or fail.
- Fail entries authored by the verifier from command output, on the loop page, beside Tried.

### stake, doc truth (not built)

- Claims with typed evidence and computed status. A stored status is a schema error. A
  historical claim needs a scope note. Presentation text cannot verify a technical claim.
- A generated-sentence anchor: a claim produced by a script from a command's output, failing
  when the committed text differs from the regenerated text.
- Anchors match only on strong keys: file plus symbol, file plus line, or a test id.
- A rule proposed from a transcript cites the receipt line it answers, and stake rejects a
  citation that does not resolve. One paper measured proposers inventing a violation in 15 of
  60 runs on clean input.

### umbel

- Spike a protocol transport behind the same spawn, send, wait, read verbs. Two projects this
  month drive the same CLIs over a protocol instead of a terminal, so turn end and permission
  requests arrive as events rather than screen reads. First check: one turn over one provider's
  protocol, its wait reason compared with tmux's. Keep tmux where no protocol exists.
- The choice of one vendor's headless CLI over its chat CLI, which umbel made in September, is
  the same choice another project made this month.

### graft and weeder

- Record each judge's failure direction in the receipt (fails open, escalates, defers) and test
  it with the provider killed.
- Before an agent runs against a calibration set, strip the ground truth from the sandbox,
  history included. One preregistered benchmark found agents reading reference solutions from
  the working tree and from git history.

### pollen, petals, tilth

- pollen: two lanes, urgent at the next tool result and ordinary at the next turn, a backlog
  that spills to a file, and a held stranger that never reaches the urgent lane. Bind every
  message to a content hash and re-validate after fetch.
- petals: lint skills (description present, name matches directory, named in the always-on
  text), because a mismatch drops the skill silently.
- tilth: report how many tokens an instruction set forces a reader to load. One kit's mandatory
  manifest is about 135,000 tokens, which no agent follows literally.

## What we already hold

Only the verifier writes a pass; a second-provider auditor on every node; receipts sealed under
a hash; the tamper rule; record before use and replay for free; the human gate for strangers.
None of the projects read has all of these, and most have none.

## What contradicts our assumptions

- Code deciding is rare. The 1,024-agent run merged with required approvals off and status
  checks disabled, and its "verified" state came from a regex over the agent's own words. The
  harness editor lets a model selector pick the adopted output. The ledger paper lets the model
  declare done and code only vetoes on a failing public sample. No project measured the effect
  of a code-decided pass, which makes it the garden's test to run: the same plan with and
  without the auditor, on enough independent tasks for the bound.
- Repeats are not breadth. Three runs of 20 tasks is 20 tasks.
- Gains are small and noisy. A host agent's 0.8 points on a saturated benchmark; four points
  from 128 to 1,024 agents with no variance reported; the harness editor's rows carry standard
  deviations of 4.5 points, near its per-cell gains.
- Terminals are losing to protocols as the way to drive a CLI.

## What to skip

Self-evolving harness loops (a model adopts, gains near noise, thousands of rollouts per task).
Server stacks for coordination. The no-orchestrator thesis, shown on one benchmark with an
oracle binary. A chat-relay kit for a human between a browser and a terminal. The "recursive
self-improvement" framing.

## Order

1. hedge takes the design inputs above before its first commit.
2. The claim record becomes copeca's spec input, and every bed's next "better" uses it.
3. pleach: write scope, sealed checks, fingerprinted resume, one node each.
4. tend2: activation levels and checks by description.
5. umbel: the protocol spike, one afternoon.
6. stake takes its claim model when it starts.

## What was not verified

No agent re-ran a benchmark. The claim audit's verdicts were coded mostly by a model. Benchmark
charts published as images were not read. The umbrella agent checked, in the cloned sources:
the hooks file and trust flag, the weak-evidence phrase list, the stray-revert module, the
precedence rules, the quota reader, the audit tally, the cut-off guard, the claims schema and
the retest states; and in the garden: pleach's receipt hash, the absence of a scope field and of
a quota reader, and tend2's open issue 220.

## Sources

Seven sources read on 2026-10-26: a harness-editing framework (arXiv 2610.14224), a ledger
orchestration paper with code, a one-maintainer project kit, a host agent with an evolver
(arXiv 2609.33439), a survey with a claim audit (arXiv 2610.18813, citing 2607.24300,
2607.13083, 2605.30621), a decentralised 1,024-agent harness (arXiv 2609.26781), and a desktop
app that drives four vendor CLIs over one protocol. The agents' seven reports, with repository
links and commit hashes, are with the owner.
