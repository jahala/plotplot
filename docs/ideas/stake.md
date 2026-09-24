# stake, the bed that keeps what a repository says about itself true

2026-09-24. An idea, not a loop. The owner asked whether a third product was missing beside
tend (the bets and their proof) and graft (where things belong and whom they serve), one that
"keeps things up to date, checks for drift". This page names it.

## The gap, stated plainly

A tended repository describes itself: a core page, an architecture document with its
decisions, an engineering document with its recipes, a README, a map whose prose says how
each part works, and, once tilth draws them, diagrams. All of it is true on the day it is
written and rots from the next commit on. The garden's law already says what must hold
(docs/building-the-garden.md §3, the references every bed keeps): cited paths resolve, a
changed shape changes its document in the same pull request, a monthly truth pass lands
corrections. Nothing owns those rules. weeder judges one diff now; copeca asks whether a
change helped; tend proves claims that have evidence scripts. No bed asks, over time, whether
the repository's account of itself is still the code.

## What stake does

Reads the repository's own words and holds them to the code, and says what moved without
its partner. Advisory, findings as SARIF beside the map, a pull request of corrections when
asked; it blocks only where a claim's anchor is gone, the way weeder's cited-path rule does.

- **Anchors.** Every claim in the core page, the architecture and engineering documents and
  the README that names a path, a symbol, a command, a number or a version is checked
  against the code (paths resolve, symbols exist through tilth, commands run, numbers equal
  what a named command prints). This is the doc-truth idea the direction memo of 2026-09-05
  reserved under the name graft; the name moved on 2026-09-24 and the idea is this bed's.
- **What moved without its partner.** From the repository's history: this document, test or
  file usually changes with that code and did not in this change. quadrat's experiment
  showed the signal on nine repositories (a document that usually moves with the code
  follows within five commits 31 percent of the time, against 3 percent for a random
  document; a missing co-change partner is changed within five commits 20 percent of the
  time on unseen projects, against 7 percent). quadrat's engine is the substrate.
- **Diagrams from code.** tilth emits the dependency graph, callers or a data model; the
  diagram is embedded in a tend page under a section of its own, and a check fails when it
  no longer matches the code, the way a wireframe no check cites is treated as decoration.
- **The truth pass.** On a schedule, or on a word, every anchor and every partner rule is
  re-read against the head of the default branch and the corrections land as one pull
  request a person reads.

## How it differs from weeder, and the line between them

weeder is the gate. Everything it judges is a property of one diff, or of the tree as it
stands at one moment: a deleted test, a skip, a stub, a swallowed error, a secret, an import
against the declared direction, a cited path that no longer exists. Each rule decides in
milliseconds from the bytes in front of it, with no history and no execution, and a block-level
finding stops the commit. Its `scan` face reads the whole tree the same way, never blocks, and
already carries the simplest anchor, a cited path or symbol that is missing.

stake is the caretaker. Everything it judges needs one of three things weeder never uses:

- **history**: this document or test usually moves with that code and did not; this partner is
  missing from this change; this claim has gone unverified through many changes;
- **execution**: the number in a document equals what a named command prints; a version
  matches the lockfile; a diagram derived from the code equals the one a page embeds;
- **a schedule**: on a day, or on a word, every rule is re-read against the head of the
  default branch and the corrections land as one pull request a person reads.

So the line is mechanical: no history, no execution, no schedule, weeder's; any of the three,
stake's. stake never re-implements a weeder rule. Its truth pass runs weeder's `scan` and adds
its own findings beside it, in the same SARIF channel, and it never blocks.

One honest alternative, which the rough check decides: if stake's own rules turn out to be few,
because the anchors are mostly weeder's scan, the partner rules are quadrat's engine and the
diagrams are tilth's, then stake is not a bed but a face on the stem, `plotplot truth`, that
composes the three on a schedule. A bed earns its place only if it holds rules no other bed
can.

## What it is not

Not a gate on prose quality or naming (a reader of meaning does that). Not a second home for
any fact: it points at documents, never restates them. Not a ranking of bad files; that was
tested and dropped in quadrat.

## Kill criterion, from the memo

If under 30 percent of the claims in the garden's own documents can be anchored mechanically,
the anchors half stays a document. The first step is a rough check, one afternoon: read the
umbrella's core page, README and architecture document, count claims, and count the ones a
path, a symbol, a command or a number would hold.

## Where it sits

The judge row, beside weeder (is this diff honest, now) and copeca (did any of it help): stake
asks whether what the repository says about itself is still true, over time.
