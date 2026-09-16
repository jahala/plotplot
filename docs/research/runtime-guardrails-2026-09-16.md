# Runtime guardrails: what a 2026 blog post on the subject adds, and what it does not

Read on 2026-09-16 at the owner's request. The post argues that an invariant must not live inside
the probabilistic system it constrains, and sorts controls into prompts (shape behaviour),
permissions (expose or restrict capabilities), hooks (policy at lifecycle points) and sandboxes
(limit what executed code reaches). The owner keeps the link; this note maps its mechanisms to
the garden and names the one it lacks.

## Already the garden's

| the post's mechanism | the garden's |
|---|---|
| invariants in the runtime, preferences in the prompt | the law: code decides what code can decide; only the verifier writes pass; weeder blocks at commit; a guardrail changes only by allowance |
| action gates before a tool call | the planted hooks, weeder at commit-msg, the machine's git guard (rewritten today with tests) |
| completion gates that block "done" until verification passes | pleach's stop hook and gates, tend2 verify as the only stamp |
| compaction as state management | pleach's reaudit on compaction, Tried lines as the record |
| context admission control on large reads | tilth's outlines and structural reads |
| economic gates that stop before the call | pleach's policy.budget, dead until workers report usage (umbel 78) |
| routing as policy | the cast per node (pleach 90) |
| observability hooks that never block | the friction journal and its profile |
| fail closed for enforcement, fail open for observability | redaction fails closed by contract; friction never blocks |
| deploy new rules log-only and measure false positives | weeder's calibration ledger |
| "hooks are not a sandbox" | the audit envelope, umbel 92, filed as an OS-level need |

## The one gap, taken

An input gate: the post stops secrets before inference by refusing prompt content that carries
credentials. The garden scrubs at write (contracts/redaction.md) and fixed the process table
(umbel 93), but a worker that reads a .env or a key file still puts the value into the model's
context, and only the prompt says not to. Filed as jahala/plotplot 96: the stem plants a
read gate for secret-shaped paths in every bed's harness settings, with an allowance the way any
guardrail changes.

## Considered and left, to stay sharp

- A formatter run after every edit (mechanical work out of the prompt). CI already checks
  formatting and the workers run it; a second mechanism for the same invariant is bloat.
- Policy "above the thing it governs" (managed hooks an agent cannot edit). In a one-owner garden
  the repository guardrails already change only by allowance; the machine's git guard is
  agent-editable and relies on its test file, which is enough until it is planted by the stem.
- Testing outcomes rather than hook output. The guard's tests assert the hook's verdict; the
  harness turns a verdict into a refusal, so the end-to-end case adds nothing today.
