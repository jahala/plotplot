# The spending ledger: one door for every paid call to the judge

The contract every program that spends on a shared budget follows until well exists, and the shape
well will keep. Today two clients write it: cultivate (`cultivate/ledger.py`) and the owner's judge
runner (`jev-run.mjs`). The rule lives in two runtimes, so its home is the test vector, not either
client: `contracts/fixtures/ledger/vector.json`. Schema: `contracts/ledger.schema.json`. Drafted by
cultivate; the umbrella's `contracts/` holds it as well's ledger.

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

## Decisions

Amounts are compared in millionths of a dollar, so a sum of cents never fails on rounding.
Spent is the sum of `settle` amounts in the period. Open reservations are `reserve` events with no
later `settle` or `release` for the same key, in this period or the one before.

| Event | Allowed when | Refused with |
|---|---|---|
| `reserve` | the key has no open reservation, and spent plus every open reservation plus this estimate is at most the cap | `unsettled` or `cap` |
| `settle` | the key has an open reservation; the charge may exceed the estimate | `no-reservation` |
| `release` | the key has an open reservation | `no-reservation` |

A reservation that never settles blocks sending the same request again until a person reconciles it,
because the provider may already have charged it. Only a rate limit (HTTP 429 or 529) or a failure
before the request left the machine may be retried under the same reservation.

## The lock

A writer takes the lock by creating `<ledger>.lock` as a directory, which is atomic on every
filesystem and the same call in every runtime. Holding it, the writer reads the ledger, decides,
appends at most one line, and removes the directory. A lock older than 60 seconds belongs to a
writer that died and may be removed by the next one. The lock holds no data.

## What a reader may rely on

- **The cap holds across programs.** Every program spending the same budget reads and writes the same
  file, so one program's call in flight counts against the next program's reservation.
- **Nothing is rewritten.** A mistake is corrected by a later event, never by editing a line.
- **The ledger holds money, not answers.** Each program keeps its own recorded answers for replay.

## Lifetime

One file per budget per month, kept. Months older than a year may be compacted to one summary line
per budget, kept for the books.
