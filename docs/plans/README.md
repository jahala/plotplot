# Plans: what a file under docs/plans/ carries

The law (§2) says planned work gets one file here, linked from its loop. What belongs in it is
decided by one question: what is the penalty for being wrong? A decision that is expensive to
reverse (a format, a seam, a dependency, where state lives) is written down with its
alternatives; one that an afternoon can undo is not. Sections, each optional, in this order:

- **Objective**, one sentence, plain language.
- **Background**: why now, what problem, what was tried before.
- **Goals**, stated as impact, never as implementation. **Non-goals**: what a reader might assume
  is in scope and is not.
- **Scenarios**: how the finished thing is used, as numbered steps.
- **Interfaces**: the CLI semantics, the file format, the contract page it changes.
- **What is hard to change**: the dependencies and formats a wrong choice would lock in.
- **Open issues**: each with the problem, the options, and the immediate next step. **Resolved
  issues**: the decision, kept with its discussion.
- **Alternatives considered**: a few lines each, why not.
- **Checks**: which loop, which check lines, what evidence proves the plan is done.

No timeline: the garden records what and in what order, never when. Decisions taken while the
plan is worked go on the loop as dated Tried lines, not back into the plan.
