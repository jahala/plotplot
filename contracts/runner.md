# runner: the wait reasons umbel returns are the table pleach classifies by

Status: contract, 2026-09-15 (jahala/plotplot 69). One page and one fixture table that umbel
(the runner) and pleach (the conductor) both cite in a test, so a reason one side adds is a
seam change both sides see before either lands it.

## Why a page

`umbel wait` ends with a reason and an exit code; pleach turns that reason into a verdict for
the node: retry, block, declare the worker dead, or stop. On 2026-09-15 umbel 67 added
`provider-error` (exit 122) and the pinned pleach classified it, like any reason it did not
know, as terminal: a rate-limited worker would have ended a node for good. Both sides were
right by their own code and wrong at the seam. The law's seam rule (§8) says the page and the
test come first.

## The table

`contracts/fixtures/runner/reasons.jsonl`, one JSON object per line: `reason`, `exitCode`,
`meaning`, and `classification`, the verdict class a conductor must give. The classes are
pleach's: `terminal` (the worker is done and the gates decide), `retryable` (another attempt
may succeed, bounded by the node's policy), `blocked` (a person or the operator is needed),
`dead` (the session is gone; resume or fail per policy).

| reason | exit | meaning | classification |
|---|---|---|---|
| stop | 0 | the worker's stop hook fired: it says it is done | terminal |
| file | 0 | the awaited file appeared | terminal |
| pattern | 0 | the awaited pattern appeared on the pane | terminal |
| provider-error | 122 | a provider error on the pane, then stillness: the model refused or failed, the worker did not | retryable |
| idle | 123 | no activity across pane, events and transcripts past the threshold: wedged, or waiting on something it never said | blocked |
| timeout | 124 | the wait's own clock ran out with the worker still active | retryable |
| dead | 125 | the session or its process is gone | dead |
| input | 126 | the worker is asking a person something | blocked |
| aborted | 130 | the wait was interrupted from outside | terminal |

A reason not in the table is a seam change: umbel adds the row here first, pleach maps it,
the test goes green, then either lands.

## What a wait result carries

Beside the reason: a `message` naming what was observed (for idle, each source and how long it
was still; for provider-error, the pane line that matched; for input, the prompt text), and the
pane's tail. This is the states rule of `contracts/delivery.md` applied to the runner: no
non-progress reason without a reason and a next step, and a conductor's verdict carries them
through unchanged.

## What each side proves

`contracts/test/runner.test.sh` reads umbel's exit table and pleach's reason union and
classification at their pinned checkouts (PLOTPLOT_UMBEL_SRC, PLOTPLOT_PLEACH_SRC), and is red
when either names a reason the table does not, or classifies one differently. umbel's own test
suite cites this page for the table; pleach's cites it for the classification.
