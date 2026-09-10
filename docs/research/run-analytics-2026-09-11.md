# What the run journals say, 2026-09-11

Status: analysis, read from pleach's run journals in the garden's own repositories on
2026-09-11 (weeder, tend2, pleach; 119 closed nodes since July, 113 with a verdict, 104 of them
in September). Dates come from each closed node's commit, since journal lines carried no
timestamp before the envelope landed on 2026-09-10. Nothing here is a controlled measurement.

## Closed nodes per day, all tools

| day | nodes | first attempt | attempts per node | median minutes |
|---|---|---|---|---|
| 2026-09-05 | 10 | 100% | 1.00 | 25.0 |
| 2026-09-06 | 15 | 67% | 1.33 | 36.5 |
| 2026-09-09 | 17 | 88% | 1.12 | 14.0 |
| 2026-09-10 | 61 | 89% | 1.11 | 10.2 |

## Per tool, September

| tool | runs | closed nodes | first attempt | median minutes per node |
|---|---|---|---|---|
| weeder | 53 | 33 | 85% | 25 to 37 by day |
| tend2 | 24 | 41 | 90% | 8.5 |
| pleach | 10 | 39 | 85% | 9 to 11 by day |

Attempts in September: 90 nodes closed on the first attempt, 14 on the second, none needed a
third. weeder's landings show the pain the other two did not: 7 landing conflicts, 5 landing
gate retries, 4 aborted runs, 4 quarantines; pleach's ten runs had none of those.

## What the numbers say, honestly

- The per-day fall in minutes per node is mostly composition. weeder's nodes take 25 to 37
  minutes on every day it ran; tend2's and pleach's take 9 to 11. The 10th of September was a
  tend2 and pleach day with per-check nodes, and the 6th was a weeder day with calibration
  nodes. No tool's own median moved.
- The first-attempt rate rose from two thirds to nine tenths across the week, but weeder's own
  stayed at 85 percent while the tools that joined later ran at 85 to 90 from their first day,
  so the rise is mostly the mix again. What did change on every tool: a retry never exceeded one,
  and every second attempt this week was a gate refusing something real (an already-green proof,
  a fixture secret, a dropped flag), so the retries are the gates earning their keep, not waste.
- Losses to the conductor itself (aborted runs, quarantines, nodes dying at collection) belong
  to the days before pleach's teardown and collection fixes landed on the 10th; the two loops
  after them ran seven of seven with no retry. Two loops is not a trend, and it is the right
  direction.

## What cannot be measured yet, and why

- **Cost.** Not one of the 132 verdicts in the three journals carries token counts; every
  `telemetry` field is empty, because umbel does not report a worker's usage to pleach. The
  casting ledger's cost per verified claim therefore rests on durations alone. Filed as
  jahala/umbel 78 with a note on pleach 81.
- **The stem.** Its journal lost its first nineteen nodes between 2026-09-09 and 2026-09-10 by a
  cause nobody can name (jahala/pleach 97); their close receipts survived beside it and say: nineteen
  nodes, every one closed on its first attempt, all Claude Opus 5, durations from under a minute to
  49 minutes with a median of 11. They are not in the tables above because a receipt carries no
  date the tables can key on.
- **Improvement of the tools on the work.** The nodes are different tasks in different tools
  on different days; a fall in attempts or minutes cannot be attributed to a tool change without
  a fixed task set run before and after it. That is the experiment the garden withdrew for want
  of a yardstick, and it is the only way to turn this table into a claim.

## What would make it a measurement

A fixed set of twenty nodes, the same tasks conducted before and after a tool change, with
token counts in every verdict and three repetitions per cell. Until then this page is a
description of one week, updated as the journals grow.
