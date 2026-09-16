# Four readings: ordewell, absurd, capsule, and how to write a design doc

Read on 2026-09-16 at the owner's request. Sources: ordewell/ordewell (README, the ADR list,
docs/why-a-separate-planner.md), earendil-works/absurd (README), withcapsule.app (the product page),
and Michael Lynch's "How to write an effective software design document" (refactoringenglish.com,
2026-06-24). Compared against the pinned pleach (411a1eb), tend2 (a33d5fb), umbel master (b772dbb)
and the law.

## ordewell: a planner that only reads, a plan that is a file, a verdict that is a marker

ordewell turns one goal into an ordered plan of coding-agent tasks, each with its own runner,
model, thinking effort and mode, then runs and verifies them. Most of it is what pleach and
tend2's emit-plan already are: the plan is a file edited before anything runs and completed work
survives edits (pleach keeps closed nodes across a re-run); one runner and model per task (a
node's worker); a task is done when its completion marker appears in the runner's output and
never on exit code alone (pleach's stop hook and the verifier); coding agents as planners on the
subscription you hold (Opus 5 through Claude Code); runners as plugin manifests, not code
(garden.json); every UI command is also a subcommand (the law's CLI-first rule).

Three things are sharper than ours.

1. **The planner is read-only, by envelope, not by prompt.** ADR-0008 and ADR-0011: the planner's
   shell is sandboxed, reads run in parallel, anything reaching outside the workspace asks once,
   and a command that would write is refused outright. The garden's auditor, a second model whose
   verdict code can refuse, runs with no such envelope: it can repair the tree it judges and then
   report green. umbel has --allowed-tools for claude only. Filed: jahala/umbel#92 (the
   envelope), jahala/pleach#111 (the audit seat uses it).
2. **Effort is a per-task setting the plan shows.** Each task carries thinking effort beside model
   and runner, assigned by the planner and visible before anything runs. The garden already filed
   this (pleach 90, effort in the cast; tend 198, ledger keyed on effort); the reading confirms the
   shape: a field on the node, never a global.
3. **The planner interrogates a vague goal before outlining.** "grilling" requires at least three
   probing questions, and the PRD mode drafts, waits for an OK, then commits the plan as its final
   message. tend2's shape step is the garden's equivalent and is prose today; a minimum of named
   questions answered on the loop before emit-plan runs is a check a fit script could hold.

Left: the TUI and the VS Code extension (the law: not an app, not an orchestration UI); the local
daemon the CLI drives (not a service); twenty-five provider keys (the garden bills through the
harness).

## absurd: durable execution as a queue plus a state store, on one database

absurd is durable execution on Postgres alone: a task is decomposed into steps, each step's result
is a checkpoint, a crashed or suspended task replays its checkpoints and continues, events are
cached so the first emit wins and there is no race, workers pull, nothing pushes, and there is no
service beside the database. Its own summary is the lesson: it is absurd how much you can
over-design such a simple thing.

The garden's conductor is already this shape in the small: a node's phases are steps, the
quarantine branch and the receipt are the checkpoint, and a killed run resumes from quarantine on
the same command (pleach's "Nothing is lost" milestone). Two readings worth keeping:

- **Code outside a step may run more than once.** absurd says so on the first page. pleach's
  equivalent rule is not written: which parts of a run are idempotent on resume (setup, the
  gates, the audit) and which are checkpointed (a phase's commit). One line in
  contracts/delivery.md the day resume is next touched; not filed, since no defect is known.
- **First emit wins.** umbel 86 (read at Stop races the transcript) is an event race of exactly
  the kind absurd removes by caching the first emit. The runner contract now says stop means
  the handback is readable in full; the fix in umbel should be the same shape: the event is
  recorded once, complete, before it is reported.

Left: the database (the garden's state is files in the repository, by the law), the SDKs, the
web UI. Its "install the bundled skill into the project" is the garden's SKILL.md face.

## capsule: a document that runs

capsule packs an application, its data and its media into one portable file that opens offline
on any desktop, with no cloud and no account. The garden already made this decision for its
one document: a tend2 loop page is one HTML file carrying its markdown, rendered by a vendored
script, readable offline, shareable as a file. Nothing to take; the reading confirms that the
renderer stays vendored and the page stays self-contained, and that a map is a folder of such
files rather than a site.

## The design doc article: what belongs is what is expensive to get wrong

Lynch's rule for what goes in a design doc is one question: what is the penalty for being wrong?
Language and storage belong; a "load more" button does not. His sections, each optional: title,
metadata (author, date, status, who signed off and when), a one-sentence objective, background,
related documents, goals stated as impact rather than implementation, non-goals, scenarios,
diagrams with editable sources, glossary, constraints, service level objectives, monitoring,
timeline as milestones that produce artefacts, interfaces, dependencies (what is hard to change),
security, privacy, logging, open issues (problem, options, next step), resolved issues kept with
their discussion, alternatives considered in a few lines each.

The garden has most of this spread across its records: the loop page carries goal and checks,
Tried lines carry decisions with dates, the human check is the signoff, the manifest's metric is
the objective, friction is the monitoring, and the law's §6 is the non-goals. What it lacks is
the filter and the shape for the plan file the law asks for under docs/plans/: the three plans
there each invented their own sections. Taken: a plan template, docs/plans/README.md, with the
sections that fit the garden (objective in one sentence, background, goals as impact, non-goals,
scenarios, interfaces, what is hard to change, open issues with a next step, resolved issues,
alternatives), the penalty-for-being-wrong filter as its first line, and no timeline (the owner's
rule: what and in what order, never when).

Left: SLOs as such (a bed's metric command is the garden's measurable objective), diagrams as a
requirement (a plan may carry one; none is asked for), the review process (the loop and its
stamps are the review).

## Order

1. The plan template (this memo's pull request) and the audit envelope (umbel, then pleach).
2. Effort per node stays where it is filed (pleach 90, tend 198).
3. The shape step's minimum questions as a fit claim on tend2's loop, once the fit script exists.
