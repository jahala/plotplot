# harvest, learning whether a release helped

2026-10-26. A draft, not a loop. Layer: release, tell, measure, learn
(`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner who ships a change and never finds out whether it helped. The people who asked for it
are not told, so they do not come back. Release notes are written for developers. Support
questions pile up in an inbox. Whether the new feature was used is a guess.

## What it owns, and what it does not

harvest owns everything after a release, for people who already asked or already use the product:
release notes in users' own words, telling the people who asked, help pages, support replies,
reading whether a feature was used, and the verdict on whether a change helped. It does not reach
new people (pollen does), fetch what people say (mull does) or sort it into needs (cultivate does).

## How it works

1. **Release notes.** From the merged work and the jobs it serves, in the users' words cultivate
   recorded, in petals' voice.
2. **Tell.** For each need a release answers, harvest replies where the person asked: the issue,
   the thread, the review. It never contacts anyone anywhere else, and it never contacts another
   product's users. Each reply waits for a yes in gate unless the compass allows it.
3. **Support.** Support messages arrive through mull as a private source. harvest drafts replies,
   answers what the help pages already answer, and sends the rest to the owner through gate.
4. **Measure.** The analytics events trellis defines for each outcome say whether the feature was
   used and by how many.
5. **Verdict.** After the window the compass sets: helped, no change, or harmed, with the evidence.
   The verdict goes back to cultivate's outcomes and to glean's scoring.

## What code decides

Which needs a release answers (the trace from merged work to job), where each asker asked, the
yes, the event counts and the verdict's arithmetic. A typed judge may say whether a reply answers a
question, in a shape code can refuse.

## Alone, and with the garden

Alone, it writes release notes, tells the people who asked, and reports usage from an analytics
export. With the garden, it reads needs from cultivate and events from trellis, and its verdicts
change what cultivate and glean believe.

## Data and its lifetime

Verdicts and release notes are kept for the life of the product. Private support messages follow
mull's private-source lifetime, set by the owner, and are removed on schedule.

## Metric and kill criterion

Metric: the share of releases with a verdict, and the share of askers told.

Kill criterion: after one real release, harvest tells the askers with the owner's yes and reports
adoption within the first week. If analytics cannot answer "was the new feature used", that event
gap is fixed first.

## Open questions

- How long the verdict window is per kind of change.
- Whether help pages are part of the product (graft's screens) or harvest's own output; stake keeps
  them true either way.
