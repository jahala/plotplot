# A typed-question judge: where a calibrated, non-generative model fits the garden

Read on 2026-09-17 at the owner's request, who has access to the API. The garden's research
names ideas, never vendors; the owner keeps the link. Compared against the law, pleach at
411a1eb, tend2 at e270dd9, umbel at 16f6ccb, and the calibration ledger's shape (jahala/plotplot 2).

## What it is

A model that does not generate text. It takes a text state (a string, an object or an array, up
to about 32k tokens) and a map of typed questions, and returns for each a typed answer: a
yes/no as a probability from 0 to 1, a choice among named options with a distribution and a
confidence, or a score on two to ten described levels with a distribution and a confidence.
Many questions ride on one call at the cost of the extra question tokens. It is trained for
calibrated probabilities, which hold over groups of answers and never for one answer. It
reads questions literally, cannot count or compare dates, does not treat its state as hostile,
takes text only, and costs about a twentieth of a cent per ten thousand input tokens; output
is free. Its own cookbooks are the honest guide to its reach: citation checking, screening,
routing, re-ranking, skill selection from a large catalogue, and a repeatability study where
its answers flipped less than most generative models' and abstained on a threshold.

## Where it fits, ranked

1. **The audit seat.** The law's slogan, a second model's judgment in a shape code can refuse,
   is this model's output type. Today pleach relays the verifier through a second generative
   provider and reads prose. A typed judge answers, per claim, questions such as: the diff
   implements the claim as its check states; the diff weakens or deletes an existing test; the
   evidence exercises the code the claim names rather than a mock; which of the delivery
   contract's states this node is in; how complete the work is against the contract page.
   Confidence gates the verdict, and the calibration ledger grades the judge over its history,
   which is the only way calibration can be read. Filed as an experiment with a kill criterion:
   jahala/pleach 118. Deterministic gates stay primary; this is one vote.
2. **The pane's ambiguous states in umbel.** umbel 67 and 73 made death and stillness
   deterministic; whether a still pane is a worker asking a person something is still a
   pattern match per provider. A yes/no over the last lines of the pane is provider-neutral
   and cheap. Second, because 67 just landed and the regexes should be given their run first.
3. **copeca's grading.** A calibrated scorer for did-it-help measurement over corpora is what
   copeca lacks, and copeca is not set up yet; noted for when it is.

## Where it does not fit, by the law

weeder judges diffs deterministically and may block; a probability has no place in a block.
The verifier alone writes pass; a judge cannot stamp. A human check stays human. quadrat's
explain face needs prose, which this model cannot write. Skill routing for the harness would
be a second mechanism beside the harness's own; that is bloat.

## The cautions that shape the adapter

- State is data, never instruction: a worker's diff or handback may carry text that reads as a
  directive, and the model does not defend against it. Criteria per question, and never a
  question whose answer the state can rewrite.
- No reason comes back. The states rule wants a reason and a next step; the reason is the
  question that failed and its probability, which the receipt records.
- Vendor-neutral by construction: one adapter behind pleach's audit provider, an endpoint and
  a model id in configuration, the token by environment reference (umbel 93), the questions in
  the plan or a contract page.
- Log-only first. Nothing gates on it until the ledger shows it calibrated on a fixed set of
  past nodes with known outcomes, at least as often right as the generative auditor, and at
  under a tenth of the cost. That is the kill criterion; without the set, this stays a note.

## What the owner does

Put the API key where the umbrella's sessions can read it (the workspace's harness
configuration or the shell profile), never in chat or a file under version control. Then the
first probe runs: the redact node's diff and its contract as state, the five questions above,
and the answer compared with what the verifier and a person concluded.
