# glean, the tool that finds what is worth growing

2026-10-26. A draft, not a loop. Layer: finding what to grow (`docs/decisions/2026-10-26-layers.md`).
Gleaning is gathering what is left in the field after the harvest. glean gathers the needs that
existing products left behind.

## Who it is for, and their pain

An owner who wants to build products people need, and has a list of ideas they had themselves
and no evidence about which ones anyone else wants. Ideas from one head are a sample of one.
Reading forums and reviews by hand to check is slow, and nobody does it twice.

## What it owns, and what it does not

glean owns the question "what should exist?" across the whole world, and the seeds it plants.
It does not fetch (mull fetches), merge what people say into needs (cultivate merges and counts),
decide which seed to build (the gardener proposes, the owner says yes), or contact anyone.

cultivate asks who an existing product is for. glean asks which products should exist at all.

## How it works

1. **Where to look.** glean writes mull's source list for a hunt: places where people ask "is
   there a tool that...", complaints and poor reviews of popular products, feature requests a
   product closed as won't-fix, products being shut down whose users are stranded, people
   describing a spreadsheet or a manual workaround they run a business on.
2. **What people need.** cultivate turns mull's raw items into needs, each with the count of
   distinct people and the quotes behind it.
3. **Score.** glean scores each need against the compass: how many people, how much it hurts,
   signs they would pay, what already serves it and how well, how small a first version could
   be, and whether it fits what the owner wants the portfolio to be. Each part of the score
   cites the items it rests on. A part with no evidence is marked guessed, never filled in.
4. **Plant.** A need that clears the compass's line becomes a seed in the seedbank, in the format
   the owner's idea skill already writes, with the evidence linked by mull item id.

Its first source is the owner's own past projects and seedbank, before anything outside.

## What code decides

The score's arithmetic, the line a seed must clear, the evidence links resolving to real mull
items, and the refusal to score fit when no compass exists. A typed judge may rate how much a
quote hurts or whether it signals payment, in a shape code can refuse.

## Alone, and with the garden

Alone, glean reads raw items in mull's file format, which a person can write by hand, and a
compass file, and writes seed files. With mull it hunts on its own; with cultivate its needs are
counted properly; with the gardener its seeds become proposals in the owner's queue.

## Data and its lifetime

Seeds stay until the owner plants or archives them. Archived seeds keep their score and the
reason, so a seed is never proposed twice for the same reason. glean stores no personal data:
it cites mull's items, and mull holds authors only as hashes.

## Metric and kill criterion

Metric: the share of planted seeds the owner keeps after reading them. Later: the share of
built seeds whose landing page clears its signup line.

Kill criterion: in the first rough check (below), if fewer than 3 of 10 scored seeds are ones
the owner would build, the scoring is wrong. After 20 planted seeds, a keep rate under one in
three means glean is a reading list, not a tool.

## First step

By hand, before any build: score ten of the owner's own projects the way glean would, against a
draft compass, and show the owner the ten with their evidence. The best one is a candidate for
product one.

## Open questions

- Which outside places to read first, and their terms of use; mull's source list decides what is
  allowed per source.
- How a seed carries signs of payment honestly: a stated price, a paid rival, a workaround that
  costs money.
