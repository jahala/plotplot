# almanac, the record of what is due and what was decided

2026-10-26. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner whose products each carry dates nobody holds in one place: a domain that renews, a
framework version that stops getting fixes, a key that expires, a weekly report, a campaign that
ends, a seed to look at again. Today these live in a person's head or in scattered notes, and the
first sign of a missed one is a broken product.

## What it owns, and what it does not

almanac owns dates and the decision log: what is due, when, for which product, whose job it is,
and what the gardener decided and why. It decides nothing. It does not hold the work itself:
work lives in each product's map, and an almanac entry points at it.

## How it works

- **Due dates and recurring work** are iCalendar (RFC 5545) to-dos and events with recurrence
  rules, one file per product in the portfolio repository, so any calendar application can show
  them.
- **The decision log** is JSON Lines, one line per gardener turn: inputs read, the choice, its
  reason, the alternatives passed over, the outcome.
- **The command:** `almanac due [--by <date>]` lists what is due; `almanac add` writes an entry
  with its source and owner; `almanac log` appends and reads the decision log. Every command is
  idempotent and takes the date as an argument, so tests pass time in.

Entries come from upkeep (renewals, end-of-support dates), transplant (certificates), pollen
(campaign ends), harvest (when to read a change's verdict), glean (seeds to revisit) and the
owner.

## What code decides

Whether an entry is valid iCalendar, whether it names an owner bed and a source, whether its
pointer resolves, and what is overdue. All of it is code.

## Alone, and with the garden

Alone it is a calendar of a project's obligations that a person can read in any calendar
application. With the gardener, it is what the gardener reads first.

## Data and its lifetime

Done and expired entries move to a dated archive file per year and are kept, because past
decisions explain present ones. The decision log is kept for the life of the portfolio; entries
older than a year are compacted to one line each.

## Metric and kill criterion

Metric: entries that passed their date with no action. The target is none.

Kill criterion: if after product one nearly every entry duplicates a loop already on a map,
almanac folds into tend2 as a due-date field, and the decision log moves with the gardener.

## Open questions

- Whether almanac lives in the portfolio repository only, or each product carries its own file
  and the portfolio reads them all.
