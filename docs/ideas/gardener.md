# the gardener, the factory's brain

2026-10-26. A draft, not a loop. Around every layer (`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner with several products and several tools, each with its own agent, who wants work to
keep moving while they are away. Today a person or one long chat session decides what runs next,
starts each agent, paces the one shared agent seat and the subscription limits, and writes down
what happened. When the session closes or the person sleeps, the factory stops.

## What it owns, and what it does not

The gardener owns the choice of what runs next, and the brief it hands to the agent that runs
it. It does not plan the work inside a product (tend2 does), run it (pleach and umbel do), judge
whether it is done (tend2's verifier does), keep the calendar (almanac does), or decide anything
the compass reserves for the owner (gate holds those).

## How it works

The gardener is an agent, and each wake-up is one turn, the same command every time:

1. **Read.** What is due in almanac; the owner's answers in gate; messages on mycelium; the
   factory's health from `plotplot status`; what each product's map says is next
   (`tend2 next`); the budgets and the seat in well.
2. **Choose.** One action, or none, from the candidates the maps and almanac offer, under the
   compass's priorities. If the action needs a yes, it asks through gate and chooses something
   else meanwhile.
3. **Act.** Start the bed's agent through umbel with a brief built from the work order: who the
   work is for, the paths, the acceptance checks, the rules. One seat unless the compass allows
   more.
4. **Record.** The choice, the reason, the alternatives it passed over and the outcome, in
   almanac's log.

It stops at the first failure, records it, and tells the owner through gate when the failure
needs a person.

## What code decides

Whether it may act at all: hedge's limits, well's budgets and seat, gate's yeses. Which
candidates exist: tend2 and almanac, never the gardener's own invention. Whether work is done:
the verifier. The gardener's judgment is only the choice among allowed candidates and the
writing of the brief, and every choice is recorded so it can be checked.

## Alone, and with the garden

Alone, it reads a repository's plan files and issues, picks the next item, and starts an agent
CLI directly. With the garden it reads the maps, almanac, gate and well.

## Time

The gardener holds no timer (law §6). The operating system's scheduler or a workflow file wakes
it; each turn is idempotent, so a missed or doubled wake-up does no harm.

## Data and its lifetime

Its log lives in almanac and follows almanac's lifetime. Briefs are kept with the run's receipt.

## Metric and kill criterion

Metric: how often the owner overrules its choice, and how long the seat sat idle while work was
ready.

Kill criterion: replay first. The umbrella agent's own decision log from September and October
2026 records each choice with its inputs. Give the gardener the same inputs and compare its picks.
If it agrees on fewer than half, it is not ready to choose alone. After its first 40 live
choices, an override rate above one in four sends it back to proposing only.

## Open questions

- How briefs are kept self-contained without copying documents (law §3: a work order names the
  documents and carries the numbered invariants).
- Which model runs the gardener's own turn, and its cost per turn, through well.
