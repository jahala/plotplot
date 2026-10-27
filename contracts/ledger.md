# The spending ledger: one door for every paid call

The contract every program that spends on a shared budget follows: well's ledger (`jahala/well`).
well implements it (`src/ledger.ts`) and offers it as a command, `well run`, which a program in any
language can spend through. Programs that carry the rule themselves must pass the same vector:
cultivate (`cultivate/ledger.py`) and the owner's judge runner (a pinned copy of well's
`src/ledger.ts`). The rule's home is the test vector, not any client:
`contracts/fixtures/ledger/vector.json`. Schema: `contracts/ledger.schema.json`. First drafted by
cultivate; changed since by well.

## Files

| File | Holds |
|---|---|
| `budgets.json` | one cap per budget per period, set by the owner: `{"judge": {"period": "month", "cap_usd": 5}}` |
| `<budget>/<yyyy-mm>.jsonl` | the ledger: one event per line, append-only, never edited |
| `<budget>/<yyyy-mm>.jsonl.lock` | an empty directory that exists only while one writer appends |

All three live in one folder, by default `~/.local/share/well/`. A program that finds no budget for
what it is about to spend refuses to spend.

## Events

| Field | Meaning |
|---|---|
| `at` | when the event was written, UTC, ISO 8601 |
| `budget` | which budget, such as `judge` |
| `period` | `yyyy-mm`, the file it belongs to |
| `tool` | the program that wrote it |
| `key` | sha256 of the request's canonical JSON (its model, state and questions, keys sorted, no spaces), lowercase hex |
| `event` | `reserve`, `settle` or `release` |
| `usd` | for `reserve`, the estimate; for `settle`, what the provider charged; for `release`, 0 |
| `why` | on `release` only: `not-sent` (the call never left), `refused` (the provider refused it without charging), or `reconciled` (a person checked the bill) |
| `cap_usd` | on `reserve` only: the cap the decision compared against. Reserve lines written before this field existed lack it, and are never rewritten to add it |

## Decisions

Amounts are compared in millionths of a dollar, so a sum of cents never fails on rounding.
Spent is the sum of `settle` amounts in the period. Open reservations are `reserve` events with no
later `settle` or `release` for the same key, in any period up to this one. A reservation stays open
until it is settled or released, however old, because never resending blind is the point; a person
clears an old one with a `release` whose `why` is `reconciled`. Until then it is held against the cap.
The vector's `across_months` cases hold this.

| Event | Allowed when | Refused with |
|---|---|---|
| `reserve` | the key has no open reservation, and spent plus every open reservation plus this estimate is at most the cap | `unsettled` or `cap` |
| `settle` | the key has an open reservation; the charge may exceed the estimate | `no-reservation` |
| `release` | the key has an open reservation | `no-reservation` |

An allowed `reserve` writes the cap it was decided under as `cap_usd`. The vector marks each one
with `writes`, and a step with `owner_sets_cap_usd` changes the budget's cap before it is decided.

A reservation that never settles blocks sending the same request again until a person reconciles it,
because the provider may already have charged it. Only a rate limit (HTTP 429 or 529) or a failure
before the request left the machine may be retried under the same reservation.

## The lock

A writer takes the lock by creating `<ledger>.lock` as a directory, which is atomic on every
filesystem and the same call in every runtime. Holding it, the writer reads the ledger, decides,
appends at most one line, and removes the directory. A lock older than 60 seconds belongs to a
writer that died. Only one waiter may remove it: the waiter first creates `<ledger>.lock.break` as
a directory, checks the lock's age again, removes the lock only if it is still older than 60 seconds,
then removes the breaker. Without the breaker, a waiter that saw the old lock could remove the fresh
lock another writer had just taken, and two writers would decide at once (well measured the cap
passed in 5 of 20 trials). A breaker older than 60 seconds belongs to a waiter that died and may be
removed. A writer that finds its lock gone when it releases it reports that, because another writer
may have decided at the same time. The lock and the breaker hold no data. Every client passes the
lock cases in `contracts/fixtures/ledger/lock.json`, so clients in different runtimes share the lock
safely.

## What a reader may rely on

- **The cap holds across programs.** Every program spending the same budget reads and writes the same
  file, so one program's call in flight counts against the next program's reservation.
- **Nothing is rewritten.** A mistake is corrected by a later event, never by editing a line.
- **The ledger holds money, not answers.** Each program keeps its own recorded answers for replay.

## Breaches

A month breaches when its settled spend passes the highest `cap_usd` its reserve lines carry. A
charge may exceed its estimate, so spend can pass the cap even when every reservation fit. A month
whose reserve lines carry no cap, because they were written before reserves carried it, is judged
against the cap the budgets file names today, and a reader says which cap it used: `ledger` or
`budgets`. A cap the owner changes later never moves the line for a month the ledger already
records. The vector's `breaches` block holds the cases.

## Lifetime

One file per budget per month, kept. Months older than a year may be compacted to one summary line
per budget, kept for the books, except that every reservation still open is kept as it was written.
