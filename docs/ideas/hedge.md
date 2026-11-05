# hedge, the boundary before any tool call runs

2026-10-26. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`). Design
inputs: issue 147. Ranked first among new beds in `docs/garden-direction-2026-09.md`.

## Who it is for, and their pain

An owner who lets agents run commands on their machine and act in their name. An instruction
file asks agents to keep the owner's hard limits, and an agent can still delete a folder, reset a
branch, print a secret or publish without asking. A rule held only in a prompt is a request, not
a limit.

## What it owns, and what it does not

hedge owns the moment before a tool call runs, on every agent CLI: one policy file holding the
owner's hard limits and the acts that need a yes, compiled to each harness's own hooks, refusing
with a reason the agent can act on, and the owner's queue: every request that needs the owner's
yes, its answer, and whether a yes holds for an exact act. weeder keeps the moment of commit and
push, through git hooks. Showing the owner what waits belongs to the owner's screen (the stem's
`plotplot status`, later conducted); hedge supplies the data and the answer command. The queue was
its own tool, gate, until 2026-11-05 (`docs/decisions/2026-11-05-gate-folds-into-hedge.md`).

## How it works

1. **One policy file** per repository and one for the owner, outside every agent's writable
   paths. The seed rules are the owner's hard limits: delete with trash, never rm; never reset;
   never drop a stash; never force-push the default branch; never echo a secret; never delete a
   failing test.
2. **Compile** the policy to each harness's tool hooks, and where a harness can also hide a tool
   from the model, to that as well.
3. **Judge the whole command.** Allowlist complete commands, never a first word: unwrap wrappers,
   read heredoc bodies, bound the nesting, and deny when the command cannot be parsed.
4. **Deny with the replacement.** A refusal names the sanctioned way (rm to trash, reset to the
   revert path), so the agent acts instead of retrying variants.
5. **Needs a yes.** An act the compass lists (deploy, publish, send, spend above a line) is
   refused until hedge's queue holds the owner's yes bound to that exact act.
6. **Canary per harness.** Install is not done until one denied call has been attempted on each
   harness and the reason came back. A hooks file can be present and silent.

The seed is the owner's existing git guard hook and its 75 tests.

## The owner's queue

1. **Ask.** Any tool writes a request (`contracts/request.md`): what is asked, why, who asks, the
   evidence, the options, the expiry. On expiry the answer is always no, so silence never acts.
2. **Rank and cap.** Open requests are ranked by what waits on them and capped, so the owner never
   faces a wall. A question already answered is never asked again; its answer is reused.
3. **Answer.** Only the owner answers. An agent session that tries is refused by hedge's own
   boundary, the same way it refuses `rm`. The answer is bound to a digest of the exact request.
4. **Holds.** An act the compass lists passes only when a yes bound to that exact act holds.
5. **Data.** Open and answered requests are files under `~/.local/share/plotplot/owner/requests/`.
   Answered requests are kept, as the proof an act was approved; requests that expired unanswered
   are kept a year, then compacted to one line each.
6. **Metric.** The share of requests answered before they expire, and the number the owner faces
   per day. If the owner answers more often through something else, the queue's records move to
   that surface.

## What code decides

Everything. hedge has no model in it.

## Alone, and with the garden

Alone, it is the owner's hard limits enforced on every agent CLI they use, and a file-based
approval queue any script can wait on. It is what makes "needs the owner's yes" true.

## Data and its lifetime

Every refusal is a line in a log, kept a year, because refusals show which rules agents keep
hitting. Allow rules carry an expiry and lapse by themselves.

## Metric and kill criterion

Metric: refused calls that the canary corpus says should have passed (false refusals), and calls
that slipped past a rule (the target is none), per harness.

Kill criterion, as written in the direction memo: if a harness cannot express deny-with-reason on
shell calls, hedge is a document for that harness and says so.

## Open questions

- The precedence table: deny beats ask beats allow, a saved allow never beats a configured deny,
  an unknown profile denies, the last matching rule wins (issue 147, input 4).
- Whether hedge is a crate in the tilth workspace beside weeder, as the review proposed, or its
  own repository.
