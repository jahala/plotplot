# well, the one door for every call that costs money or needs a key

2026-10-26. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner whose tools spend money on their own: judge calls, model calls, ads, domains, paid
APIs. Each tool that spends keeps its own cap, so each stays under its cap while together they
overspend. A call that timed out may already have been charged, and a naive retry pays twice.
Keys sit in environment variables and files where agents can read and print them. Today three
clients of one judge key in the garden each keep their own cache and their own cap.

## What it owns, and what it does not

well owns budgets, reservations, the record of what was charged, and access to keys. It does not
decide what to spend on (the tool and the gardener do), approve large spends (gate does), or make
the call itself: the caller still calls the provider, through well's wrapper or library.

It is not a proxy and not a server (law §6). It is a command and a library over files.

## How it works

1. **Budgets.** One file the owner sets: a cap per budget per period (judge, models, ads,
   domains), and the subscription seat and its windows as a budget measured in sessions rather
   than money.
2. **Reserve.** Before a call, the caller asks well to reserve the estimated cost against the
   budget, with a key for the request (a hash of what is sent). well refuses when the
   reservations in flight plus the spend so far would pass the cap. Every spender of the same
   budget shares the same ledger, so the cap holds across tools.
3. **Call.** `well run --budget <name> --key <secret-name> -- <command>` puts the secret into the
   child process's environment only, read at that moment from the operating system's keychain,
   never printed, never written, never passed on a command line. A library form does the same
   inside a program.
4. **Settle.** After the call, the caller records what was charged. A reservation that never
   settled blocks a resend of the same request until a person reconciles it, because the provider
   may have charged it.
5. **Replay.** An answered request is kept, so the same request later replays from the record for
   free. This is what lets CI re-run a judged check without paying.

The design ports the claim-before-send ledger already proven in the owner's UX-review skill's
judge runner, which claims each request before sending, refuses past a ceiling, and refuses to
resend an unsettled claim without an explicit flag.

## What code decides

All of it: the cap, the reservation, the refusal, the resend block, the replay. well has no
judgment in it.

## Alone, and with the garden

Alone, it is a spending cap and key door for any script that calls a paid API. With the garden,
the three judge clients move behind it first, then every tool that spends, and pleach reads its
ledger for the cost-per-correct report.

## Data and its lifetime

The ledger is append-only JSON Lines per budget per period. Periods older than a year are
compacted to one summary line per budget, kept for the books. Recorded answers are kept while a
check cites them, and removed when no check does.

## Metric and kill criterion

Metric: the number of times charged spend passed a cap (the target is none), and the monthly
difference between well's ledger and the providers' bills.

Kill criterion: if the three existing judge clients cannot move behind well and keep their replay
records, well has the wrong shape.

## Open questions

- How a reservation is estimated for calls whose cost is known only afterwards, such as an agent
  session on a subscription.
- Where the shared ledger lives when tools run on more than one machine.
