# What the Claude platform's cost guidance means for the garden

Status: research note, 2026-09-10, from Anthropic's article "Reducing cost and improving
performance with the Claude platform". Each item names the mechanism the article describes,
what in the garden it touches, and the issue that carries it.

| the article's lever | what it needs | where it lands in the garden | issue |
|---|---|---|---|
| Prompt caching: a repeated prefix is read, not recomputed, at a fraction of the input price | a byte-identical prefix across calls; variable text last; a 1-hour TTL for long tool calls | the garden block, tend2's work-order preamble, any SessionStart injection: stable bytes first, the task and the next-up line last; a fit check that the garden block is identical across planted repositories | jahala/plotplot 29 |
| Effort calibration: a stronger model at low effort can beat a weaker one at high effort at about 40 percent less | effort as a variable you can set and measure | pleach's cast carries effort where the CLI exposes it; the casting ledger keys cost per verified claim on provider, model and effort; copeca sweeps effort as a mode | jahala/pleach 90, jahala/tend 198, jahala/copeca 24 |
| Cache accounting: reads and writes are priced apart from fresh input | the counters per call | the spend line's usage attributes carry cache read and cache write tokens; the casting ledger and copeca price them apart, so a stable prefix shows as a saving | jahala/plotplot 29, jahala/tend 198, jahala/copeca 24 |
| Deferred tool loading keeps rarely used tools out of the prefix | a harness that supports deferral | beds declare `context.upfront_tokens` measured with deferral on; the stem's bundles mark deferrable tools where a harness supports it | on the stem loop as a note |
| Instruction audit: verification rituals, thoroughness boosters, mandatory procedures, stale examples, contradictory rules, dated configuration | a read of the instruction files | the six anti-patterns as a checklist in the doc-truth skill; the garden already moves rituals and procedures out of prompts into hooks | jahala/plotplot 4, as a comment |
| Batch API: unattended work at half price | calls that go through the API directly | not the garden's case today, which runs on the owner's subscription CLIs; a copeca runner option if API runs ever exist | none |

The garden's own claim, that agents produce and code decides, is the instruction audit's
conclusion from the other side: a prompt that carries a ritual is a prompt that could have
been a hook.
