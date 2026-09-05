# The tended repository

How a repository becomes a self-contained, self-repairing, self-improving harness for
whatever it holds. The thesis is `plotplot-thesis.md`: the repo is the harness. This page is
the theory behind it and the shape that follows. It is written to be the seed of a one-page
spec, so it states principles as rules and names the mechanism for each.

Sources it stands on: Lukas Masuch, "The Repo Is the Harness" (April 2026); James Reason's
Swiss cheese model and its critiques; Hollnagel's Safety-II; Leveson's STAMP; Perrow's
normal accidents; Naur's "Programming as Theory Building"; Lehman's laws of software
evolution; Parnas on software aging; Brooks; Gall; Deming; the Toyota production system.
None of these were written about agents. All of them were written about this problem.

## 1. Two readings of "the repo is the harness"

Masuch's reading: the repo **shapes** the agent. Context next to the code, hard guardrails
with fast feedback, skills as engineered units of process, multi-model review, weekly
automated workflows. Aimed at velocity. His measured outcome is that humans moved from
implementation to judgment.

plotplot's reading: the repo **judges** the work. Intent with SHA-bound proof, law in git
hooks, deterministic gates on the diff, a vendor-neutral yardstick. Aimed at trust that
survives the model and the vendor.

They are complementary and they meet in one gap Masuch names himself: "we still lack good
visibility into how well this system works — which skills get invoked, what documentation
gets read, where agents slow down." He also names the seed of the answer: agent confusion is
a diagnostic signal for where the codebase wants simplification. He uses it as anecdote. A
tended repository uses it as an instrument (§5).

## 2. Swiss cheese, read properly

Reason's model: defenses are slices, each with holes; holes are active failures at the sharp
end and latent conditions from decisions at the blunt end; accidents happen when holes align;
holes move. Masuch applies the surface of it: prevent, enable, catch, stacked. The harness
reading needs five things the surface misses.

**Independence is the whole game.** Defense in depth fails when slices share a cause. In an
agent system the common cause is shared context and shared model. The builder, a reviewing
agent, and the tests the builder wrote all inherit the same misreading of the same stale
`AGENTS.md`. Three model reviews of one diff are one slice with three coats of paint. A
slice is independent only if it differs in **mechanism** (code, not a model), **information
source** (repo state, not the agent's claim), and **party** (a verifier that is not the
builder, a human that is neither). tend2's separation of powers and pleach's requirement
that the auditor run on a different provider are independence engineering. Everything in a
tended repo that judges must be placed by asking which of the three it differs in.

**Latent conditions are the real enemy.** Active failures are the agent deleting a test.
Latent conditions are the stale doc that made it think the test was wrong, the flaky suite
that taught it to retry, the permissive config nobody revisited. They are invisible until a
hole aligns. Self-repair (§6) is latent-condition hunting, not just catching bad diffs.

**Holes move.** A model update changes tool-use habits overnight. A slice calibrated to last
quarter's model has new holes. Every slice therefore needs drift detection: gates re-run
against real binaries on a schedule, calibration re-run when the model changes, the yardstick
re-measured per model. The harness must expect its own holes to move.

**Order of investment, from Deming.** Inspection at the end is the most expensive slice.
Build quality in upstream. So: enable first (clear code, context next to code), prevent
second (types, linters, hooks), prove third (stamps, evidence), catch last (review, human or
model). Model review is a real slice and the last one to buy.

**Alarm fatigue and normalized deviance.** A slice that warns too often trains agents to
route around it and humans to skip it. Each warning is then a hole generator. Every slice
must have measured precision, and every suppression must be a visible finding, aggregated,
reviewed. The pile of allowances is where deviance normalizes; Vaughan's Challenger analysis
is the case study.

Two more models sharpen the picture. **Safety-II**: systems work because humans adapt; the
gap between work-as-imagined and work-as-done is where risk lives. Humans absorbed the
friction in Masuch's codebase for eight years. Agents do not adapt, so every place an agent
stumbles is a place the documented process diverges from the real one. The agent is a probe
of work-as-imagined. **STAMP**: safety is a control problem; each controller needs an
accurate process model of what it controls. Hooks and verifiers are controllers; their
process model is the repo's state. A verifier with a stale model stamps the wrong thing. This
is why tend2 derives state from the file on every read and never stores it, and why keeping
docs true to code is a safety property rather than tidiness. And **Perrow**: tight coupling
plus interactive complexity produces accidents nobody caused. Files, exit codes, and one
concern per tool are the architectural counter, which is why the Unix shape is a safety
argument and not a taste.

## 3. What the long lines of software say

**Naur, 1985.** A program is not its text. It is the theory in the builders' heads: why it is
as it is, how it maps to the world, how to change it without breaking it. When the team
leaves, the theory dies and the program is dead even though it runs. This is the deepest
answer to "what are agents not telling." An agent has no persistent theory. Each session
rebuilds a fragment and discards it; what it tells you is the diff. So the repo must hold the
theory explicitly: intent and its why, decisions with the alternatives rejected, scope-outs,
what was tried and failed, invariants, the shape of the system kept true. Not documentation
as description. Theory as record, written at the moment of change by whoever changed the
program, as a condition of the gate. tend2's Tried section and decisions are theory storage;
a tended repo makes writing them a required output of work, not a courtesy.

**Lehman.** Systems embedded in the world must keep changing as the world changes; complexity
grows unless work is spent reducing it; the rate of change is bounded by the comprehension of
those who must understand it; quality declines unless rigorously maintained. Consequences:
the repo is never done, so the harness is maintenance infrastructure and the loop is
perpetual. Simplification must be a scheduled loop, not a mood. Agents raise the change rate,
so human comprehension becomes the bottleneck, and the human surface must maximize
comprehension per minute.

**Parnas, 1994.** Software ages two ways: from failure to modify, and from modifications that
ignore the design. Agents accelerate the second: local patches, fallbacks, a try/except around
the symptom. Ronacher's "each iteration adds another small defense" is Parnas's second aging
with a loop around it. Counter: gates on the diff for stubs and swallowed errors, and a
dependency-direction rule (core must not import seams) checked structurally, which tilth's
dependency analysis makes cheap.

**Brooks.** Conceptual integrity needs one mind or one document. Agents are superb at
accidental complexity and blind to conceptual integrity. The loop tree, one goal per node, is
a conceptual integrity device; the human's remaining job is that integrity plus judgment.

**Gall.** A working complex system evolves from a working simple one. The harness must start
as an atom and grow by incident. A harness designed complete on day one is the second-system
effect, and the first draft of plotplot's architecture was exactly that.

**Toyota.** Jidoka: machines stop themselves on abnormality; the Stop hook is the andon cord.
Poka-yoke: make the wrong action impossible rather than detect it; the sandbox and git
protections are poka-yoke, a regex deny list is inspection. Genchi genbutsu: go and see the
real thing; the verifier runs the evidence itself and never trusts the report. Kaizen: small
verified improvements on a cadence; the loop.

## 4. What humans are not seeing, what agents are not telling

Humans see the artifact. They do not see the process: what was not tried, what was silently
dropped, which instruction was ignored when two conflicted, how many retries, which file was
read five times, what was assumed to resolve an ambiguity. The transcript holds all of it and
nobody reads transcripts. Humans also adapt to friction until they cannot see it, miss slow
variables (flakiness rate, dependency age, doc staleness, loop staleness) because they see
events rather than trends, forget their own why within months, and normalize the growing pile
of exceptions.

Agents do not tell you their uncertainty; they present it uniformly. They do not tell you
which steps they skipped or that "tests pass" meant one test. They do not tell you the
assumption they made, the instruction they chose to ignore, the fallback they added, the
alternative they rejected, or where they were confused, because confusion looks like a retry
and retries are silent. They cannot tell you their theory, because they do not keep one. And
they are trained toward appearing done, which is the structural reason the judge cannot be
the agent.

Three countermeasures, and they are the spine of the design:

1. **Make the telling structural.** The work order requires a Tried entry: what was tried,
   what was scoped out, what was assumed. A machine checks that the section was written and
   names the files it cites. Not prose politeness; a gate.
2. **Instrument what they will not say.** A friction ledger, computed deterministically from
   hook events, never by asking a model: denials per rule, retries per tool, repeated reads
   of the same file, test-run loops, edit churn per file, time in each directory. Aggregated
   per path, it says where the repo is hard. Masuch's diagnostic anecdote becomes a number,
   and the number shapes the next simplification loop. This is the instrument the garden
   lacks and the one that makes self-improvement more than a slogan.
3. **Make the gate indifferent to what they say.** The stamp comes from evidence the verifier
   ran. The receipt comes from git and the transcript. The finding comes from the diff. Nothing
   in the proof path reads the agent's prose.

## 5. Self-contained: nothing to install

The question was whether putting the tools themselves in the repo is overkill. The answer
splits on bytes and on principle.

**Bytes.** Skills, hooks, rules, loop pages, brand files, and small scripts are kilobytes:
pollen is 21 KB in one file, the petals skill is 148 KB, Streamlit's three hooks are under
8 KB together. Commit them. Binaries are not: tilth is on the order of ten megabytes per
platform, umbel compiled with Bun is 62 MB. Do not commit them.

**Principle.** A stamp's meaning depends on the judge's version. `[x] claim @sha` proves
something only if the verifier that produced it is known and reproducible. So the repo must
**pin its judges**: a lockfile naming each tool, version, and per-platform checksum, and the
stamp must record the verifier's identity and version alongside the SHA. Pinning is not a
convenience; without it the proof is a claim.

The pattern that satisfies both is the one Gradle, Maven, and `.tool-versions` settled on
years ago: commit the versions and checksums, fetch the bytes on first use into an ignored
directory, verify the checksum, run. Nothing to install in effect; nothing large in git; the
proof reproducible on any machine and at any later date.

**The bootstrap chain, and where it actually needs a hand.** All three major harnesses read
project-scoped configuration committed in the repo: `.claude/settings.json`,
`.gemini/settings.json`, `.codex/config.toml` with hooks. So a committed SessionStart hook can
run `git config core.hooksPath .githooks` idempotently, and the git law is live from the first
session without anyone installing anything. Streamlit already commits hooks, skills, and
settings this way. The one honest exception: Codex loads project hooks only after the user
marks the project trusted, which is one prompt, once, and a correct design on their part.

So the tended repository is self-contained by construction: clone it and it carries its
constitution, its law, its skills, its intent, its proof, and the pins for its judges. What it
does not carry is the hands.

## 6. Self-repairing: homeostasis through the same gate

Self-repair is latent-condition hunting on a cadence, with two rules that keep it from becoming
a new source of failure.

**Detectors are deterministic** and each detection shapes or reopens a loop rather than
patching anything directly: docs whose anchors no longer resolve to code; tests whose CI
history shows flakiness; dependencies past an age threshold; loops gone stale by tend2's own
decay; friction hot spots from §4; coverage decline; the suppression pile growing; a
conducted run gone quiet. Masuch's weekly `ai-update-docs` and `ai-fix-flaky-e2e-tests` are
this idea; the difference is routing.

**Repairs go through the same gate as any change.** A repair agent is an active-failure
source like any other. There is no privileged repair path, no direct write, no auto-merge
that skips the verifier. A repair loop closes when its checks are stamped and its diff passes
the gates, or it surfaces as needs-you. The scheduler is itself in the repo, as workflow
files, so the cadence is versioned with everything else.

## 7. Self-improving: the harness changes itself, under a constitution

Where improvements come from, each with a deterministic trigger:

- **Every escape adds a slice.** A defect that reached the trunk becomes a check, a rule, or
  a hook, with the incident cited in the loop's Tried. The failing test is the spec, applied
  to the harness itself.
- **Every slice must earn its keep.** Precision is measured; a rule with zero true positives
  in a season is demoted; a rule with many false blocks is tuned or removed. Alarm hygiene
  as a scheduled loop.
- **Friction hot spots become simplification loops**, and repeated command sequences across
  sessions become candidate skills. Both are counted from hook events, never extracted by a
  model from transcripts, which is the mistake mull made.
- **Measurements change casting.** The yardstick says which model and which tools for which
  kind of work; tend2's casting loop already reads it.

The governance that stops a self-improving system from improving itself into leniency is a
**two-tier constitution**:

- **Tier A, the constitution**: who may write pass; the hard limits; the definition of the
  gates; the pins of the judges. Changed only by a human, and any diff that touches these
  files is a block-level finding until a human lands it.
- **Tier B, the by-laws**: rule thresholds, skills, docs, pointers, cadences. Changed by
  verified pull request; agents may propose, the gate decides, a human approves the
  human-method checks.

The ratchet has a pawl: improvements land only through the gate, and the gate itself changes
only through people. Goodhart is held at the constitutional boundary.

## 8. What is a skill, what needs a tool, what is just a referenced file

Order artifacts by how strongly they bind:

| Artifact | Binds | What it is for |
|---|---|---|
| a referenced markdown file | advisory; read when relevant | facts, constraints, the theory: why, decisions, scope-outs. Lives next to what it describes. Kept true by a checker. Unreferenced, it is dead. |
| a skill | procedural; invoked | a checklist that composes: shaping a loop, authoring a plan, preparing a release, deciding what a doc claims. Measured by invocation and outcome, or it is folklore. |
| a tool | deterministic; when invoked | a judgment that must be reproducible: verify, lint, count, diff, measure. Exit code is the verdict. Not in a gate, it is optional. |
| a hook | unavoidable at a moment | a tool bound to session start, tool use, stop, commit, push. Silent when nothing is wrong. |
| poka-yoke | unavoidable for everyone | the wrong action made impossible: the OS sandbox, protected refs, `core.hooksPath` with `--no-verify` denied. |

The rule is **least binding that reliably achieves the intent, promoted on evidence**. Start
everything as a referenced file. When the friction ledger shows agents violating it, promote
it to a skill; still violated, to a tool in a gate; still violated, to a hook; and where the
cost of one failure is unbounded, to poka-yoke. Demote the other way when a hook has not
fired in a season. Streamlit's `pre_bash_redirect.py` is a promotion that already happened:
"use `uv run`" was advice until agents ignored it, then it became a hook. A tended repository
records each promotion in the loop that motivated it, so the harness's own history is a
theory too.

## 9. The atom, and the vital choices

Simply: a tended repository is three files, two hooks, and one lock.

- `AGENTS.md`: the constitution (who writes pass, the hard limits, the gates) and pointers to
  the commands. Ten lines of Tier A, then pointers. Nothing else.
- `docs/loops/`: intent with proof. Goal, checks, Tried; stamps bound to the SHA of the
  evidence and to the verifier's version.
- `.githooks/pre-commit` and `pre-push`: the law, running the judge on the diff; bootstrapped
  by the committed harness config.
- `garden.lock`: the pinned judges.

Everything else grows by incident, under §7, and is measured under §2.

Profoundly, the vital choices are these, and each is a fence with a cow behind it:

1. Slices independent by mechanism, information, and party; a review that shares the
   builder's context is not a slice.
2. Only a verifier writes pass; the stamp carries the evidence SHA and the verifier version.
3. Everything through the gate, including repairs and changes to the harness.
4. Judges pinned in the repo; binaries fetched, never committed; scripts and specs committed.
5. Theory in the repo as a condition of change: Tried, decisions, scope-outs, invariants.
6. Friction counted deterministically; hot spots become loops; repeats become skills.
7. Every escape adds a slice; every slice earns its keep; suppressions are visible.
8. Bindings escalate on evidence and demote on silence.
9. Two tiers: a constitution only humans change, by-laws that verified work can change.
10. Loose coupling everywhere: files, exit codes, hooks, one concern per tool.
11. Pull before push for context; push only on evidence, and re-brief after compaction.
12. Start with the atom.

## 10. What this adds to the garden, concretely

- A friction ledger, computed from hook events, feeding tend2's routing and shaping
  simplification loops. Home: tend2, because it shapes loops. The single most important
  missing instrument.
- Verifier identity and version in tend2's stamp format, additively.
- A Tried entry as a machine-checked required output of every work order in pleach and tend2.
- A dependency-direction rule in weed, using tilth's dependency analysis.
- The judges lockfile and fetch-on-first-use wrapper in the stem, replacing any idea of
  committing binaries or installing globally.
- The two-tier `AGENTS.md` structure, with Tier A files guarded by weed's guardrail rule.
- The promotion ladder as a named practice, with each promotion recorded in a loop.
- Scheduled hygiene loops: alarm precision, suppression pile, slice drift on model change.

## 11. What this page does not settle

Whether the loop file or the pull request is the center for people who are not the author;
this page argues the loop is the center of proof and the PR the center of attention, and
leaves the tension visible. How the friction ledger stays honest across harnesses whose hook
events differ. And whether a one-page spec of the tended repository can be written small
enough that someone outside the garden would implement it, which is the test that matters.
