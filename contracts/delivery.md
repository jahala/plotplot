# delivery: what a worker may write, what a conductor collects, what a pin covers

Status: contract, 2026-09-11. One page both tend2 and pleach cite in their tests, so the two
never again disagree about whose file is whose.

## What a worker writes

- Product files under the node's scope, as the work order names them.
- Its dated `## Tried` line on the loop page it worked. That line is required by the law and is
  never drift: the payload pin covers the checks section of the page, not the page.
- Scratch under `.loop-scratch/` only. Nothing there is ever delivered, by tend2's own runner or by
  pleach's collector, and the work order may promise that because both keep it.

## What a worker never writes

- A stamp. Only the verifier writes pass.
- A whole-map artefact (a badge, a rendered index). Those are re-earned at land, on the merged
  stack, never asserted by a per-node suite.
- Anything under `.plotplot/`, which is machine-written state, or a guardrail file, which changes
  only by allowance.

## What a conductor collects

- Tracked changes under the node's scope, and untracked files that are not ignored, not scratch,
  not under `.plotplot/`, and inside the tree. Anything else is set aside and journaled, never
  staged, and never a reason for a finished node to die.
- The setup line a plan carries is one argv, never a shell string; a plan author who needs a shell
  wraps it once, in the emitter, in the one place that knows the rule.

## What a state must carry

- A conductor's verdict and a runner's status never report a non-progress state (blocked,
  failed, timed out, aborted, stopped, unevaluable) without a `reason` and a `nextStep`, both
  non-empty text derived from what was observed: the phase that stalled, how long it waited, the
  runner's last status, the tail of the worker's output, the claim that could not be evaluated.
  A null reason is a defect of the state, not a property of the failure (jahala/plotplot 53;
  the first conducted node on this repository ended `blocked` with `blockedReason: null` after
  twenty minutes). Held by `contracts/test/verdict.test.sh` against the pinned pleach.

## What a gate may trust

A conducted node proves the same claim at least four times: the worker in its phases, the smoke
gate, the audit, the land gate. Each has its reason, and three of them usually execute identical
evidence on a byte-identical tree. Where a proof takes seconds that is free; where it takes
minutes a node spends longer in its gates than in its work (jahala/plotplot 121). The law says
the same inputs and the same pinned tools give the same bytes, so a tree is proven once and the
record of that proof is what the later gates read.

- **The proof record.** After a first-hand execution that passes, the verifier writes one
  record: an in-toto Statement v1 whose subject is the tree (`gitTree` digest), predicate type
  `https://plotplot.ai/proof/v1`, shaped by `contracts/proof.schema.json`. The tree is the
  files as they stand when the verifier finishes, committed or not, computed after the stamp
  is written, through a throwaway index that honours the repository's excludes, so scratch and
  machine-written state never enter it. A conductor runs its gates on staged files and commits
  at settle, so a record that waited for a clean tree would never be written; it vouches for
  bytes, and when the conductor commits exactly those bytes the commit's tree is the same tree. Its key is the tree, the check (page, id and the digest of the
  claim's kind, words and evidence line, path and arguments), the verifier's name and revision, the runner string
  and the platform. It carries the result, the counts, the duration and the time. It holds no
  output, no prompt and no absolute home path.
- **Where it lives.** Under the repository's common git directory, never in the tree, never
  committed, never delivered. A record in the tree would change the tree it vouches for. A
  clone without records executes, and loses nothing but time.
- **Who writes it.** Only the verifier, and only for a pass. A failure, an unevaluable run and
  a killed run are always executed again. A worker never writes one, as it never writes a stamp.
- **The first gate always executes.** The first gate after a worker stops runs the evidence
  first-hand and is the one that produces the record. The conductor journals the record's
  digest, and a later gate accepts that digest and no other, so a record the conductor did not
  see written is never trusted.
- **When a later gate may read and not run.** Every key field equals what is in front of it:
  the same tree, the same check and claim digest, the same verifier revision, the same runner,
  the same platform. It then reports the claim as proven "from the record of" that time, in
  those words, never as executed. Any difference, or no record, and it executes.
- **Independence stays where it is cheap.** When the recorded execution took under the
  conductor's threshold (sixty seconds unless the plan says otherwise), the audit executes
  anyway. A second provider's judgement, whether the evidence proves the claim and the diff is
  honest, is owed on every node whatever the record says.
- **The land gate** executes when the composed tree differs from the proven tree, and reads the
  record when it is the same tree.
- **Flaky evidence is hunted on purpose.** A scheduled run re-executes the whole map on the
  default branch and ignores records. A pass record contradicted by an execution under the same
  key is a defect of the evidence, reported on its loop.

## What the pin covers

- `--expect-payload` hashes the loop's checks section (claims and evidence paths). A change there is
  drift; a Tried line, a narrative edit, a For edge is not.
