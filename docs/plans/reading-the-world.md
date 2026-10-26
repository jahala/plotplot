# Reading the world: sources, adapters and the evidence store

**Objective.** One way for the garden to read what people say and do, from every kind of source,
into evidence that cultivate counts and glean scores, kept as files with a stated lifetime.

## Background

The layer decision (`docs/decisions/2026-10-26-layers.md`) gives reading the world to mull,
needs to cultivate and seeds to glean. glean's first rough test (`jahala/glean`,
`docs/rough-test/findings.md`, 2026-10-26) showed what goes wrong without this design: web search
returned whole pages, not people, so of 847 passages only 3 showed a person stating the need;
sellers' descriptions of a gap passed for users feeling it; one person counted twice because
search gives no author; and no source reached half the compass's people. cultivate's survey of
open sources (`jahala/cultivate`, `docs/research/open-data-sources-2026-09-19.md`) has already
tested a dozen sources and their limits.

## Goals and non-goals

Goals: every source reads into the same raw items (`contracts/raw-item.md`), one person's post per
item; every source says which people it reaches and on what terms; every score can be reproduced
as of any past date; every store says what it keeps and for how long.

Non-goals: no server, no database (law §1, invariant 2). No scraping a source whose terms forbid
it. No model inside mull: extraction and judgment happen in cultivate and glean, where code can
refuse them.

## Sources, by the signal they carry

| Signal | Sources (✓ tested by cultivate) | Whom it reaches |
|---|---|---|
| People wanting something | ✓ GitHub issues and discussions, ✓ Discourse forums, ✓ Stack Exchange, ✓ App Store reviews, ✓ Steam reviews, ✓ WordPress plugin support; community forums whose terms allow it | depends on the community |
| People paying | reviews of paid tools, plugin and app marketplaces with prices and installs, freelance posts asking to hire someone for a task | small businesses, makers |
| Builders and their outcomes | ✓ Hacker News (monthly "what are you working on" threads, launches), launch sites, builders' milestone posts | technical people |
| Harm and complaints | ✓ vehicle owner complaints, ✓ medical device reports, consumer complaint registers | consumers, patients |
| People left stranded | shutdown and deprecation notices, "alternatives to" threads | users of dying products |
| Movement | package downloads, repository stars over time, search interest | technical people |
| The owner's own | support inbox, call notes, product analytics, landing-page signups from pollen | the owner's users |

Two rules, from the rough test:

1. **Search finds places; it is not evidence.** A web search source (the first is the DeepSeek
   Search API) is a discovery adapter: it returns addresses. An expanding adapter for that site
   then reads the thread one post at a time, with each author. A page is never an item.
2. **Every source declares whom it reaches.** glean scores a need only on sources that reach the
   need's people, and reports an audience no source reaches as a gap, never as a low score. Today
   the compass's elders, people with aphasia and Dharma learners have no tested source; finding
   one is work, not a reason to score them on nothing.

## Adapters

mull owns every adapter. One adapter reads one source. Each ships:

- **A manifest**, a contract beside the raw item (`contracts/adapter.md`, to be drafted by mull):
  the source, its kind (discover, expand or stream), the audiences and signals it covers, its
  terms (licence, attribution, commercial use, rate limit), the key it needs by name (read through
  well, `docs/ideas/well.md`), and how it splits a page into posts.
- **One command shape:** given a window and a cursor, it returns raw items, the raw responses it
  read and the next cursor. The same window gives the same bytes, so fetching is incremental and
  append-only.
- **A census on every run:** items per channel per period, and a warning when a channel goes
  quiet or a label stops being used.
- **A refusal** when the source's terms forbid the use, rather than a workaround.

The first adapters, in order of what is already waiting on them:

1. Hacker News, for glean's second test and its standing yardstick (below). Seed:
   `hn-wayw-fetch.py` in the owner's accelerando workspace.
2. Local files and GitHub issues and discussions, moving cultivate's three adapters
   (`cultivate/sources.py`) into mull with cultivate's tests still passing.
3. Web search as discovery, moving glean's rough-test search script into mull behind its door.
4. Discourse forums and Stack Exchange as expanders: per-post, with authors, open terms.

Until an adapter exists in mull, a tool may fetch with a stand-in script kept beside the test
that needs it, writing the same raw items. The stand-in moves into mull whole.

## The store

Three layers, each owned by one tool. Each layer points to the one below by id and never copies
text, except the quote a seed cites.

| Layer | Owner | Holds | Where | Lifetime |
|---|---|---|---|---|
| Raw items | mull | one post per line, partitioned by source and month: `raw/<source>/<channel>/<yyyy-mm>.jsonl` | a data folder outside git, one per portfolio | public items kept; private items for the owner's set period, then deleted on schedule |
| Raw responses | mull | what each source returned, compressed, named by its sha256, cited from each item's `raw_ref` | the same data folder, outside git | kept while any item cites them |
| Census | mull | items per channel per period, per run | the data folder | kept, compacted to one line per channel per month after a year |
| Needs | cultivate | member item ids; distinct people counted by role (wanting, paying, building, gone quiet); counts per month; first and last seen | the product's or portfolio's repository | life of the portfolio |
| Scores | glean | per need, per compass version: each part's value with cited item ids, the rules and judge model versions, judge recording references | glean's repository | kept, so a seed is never proposed twice for the same reason |
| Seeds | glean | the seed files a person reads | the seedbank | until planted or archived |
| Outcomes | glean, then harvest | which needs later proved real, and the evidence | glean's repository | kept: this is glean's yardstick |

Files stay the truth. A closed month of raw items may be compacted into a columnar file
(Parquet) and read with an embedded query engine (DuckDB) for heavy counts; the compacted file is
derived and can always be rebuilt from the lines. The scale designed for is about ten million
items: roughly 10 GB of lines, a few GB compacted, on a local disk.

## Time, identity and reproducibility

- Every item keeps when it was said and when it was fetched. Reaction counts are snapshots, each
  with its moment, so "as of" a date is a filter, not a guess.
- An edited post is a new item (new text, new id) with the same address; both are kept.
- People count once per channel through the salted author hash; across channels they stay apart,
  by design. The same text under two addresses is two items; cultivate's merge joins them.
- Every score records the needs snapshot, compass version, rules version and judge model it came
  from, and replays from recordings with no new call. "Score this need as of March 2025" is an
  honest query, which glean's second test depends on.

## Hacker News as signal and as yardstick

The monthly threads carry five signals: who is building for a need (posts), whether the need
proved real (the same builder later reporting users or revenue), people wanting it (replies),
what died (projects that stop appearing), and what is rising (builders per need per month).
cultivate counts them by role; glean reads them as parts of the score.

Their history is also glean's yardstick. Each early need's later outcome is a label, so the test
"does glean's score, as of the post, predict which needs proved real" can be re-run on every new
quarter of threads. That AUC is glean's declared metric (law §1, invariant 6). Its limit is the
audience: the threads speak for technical builders, so the yardstick measures glean on technical
needs only, and says so.

## What is hard to change

The raw item contract (already landed), the adapter manifest, and the role counts in cultivate's
needs format. A wrong choice in any of these is paid for by every adapter and every reader.

## Open issues

- **The data folder's home**, when tools run on more than one machine. Next step: one folder per
  portfolio on the owner's machine; revisit when a second machine runs adapters.
- **Sources for the compass's untechnical people.** Next step: glean lists candidate communities
  per group, mull checks each one's terms, and only open ones become adapters.

## Checks

- mull's kill criterion (layer decision, and `docs/ideas/glean.md`): a re-run over the same window
  returns byte-identical items; a channel that goes quiet raises the census warning; cultivate's
  three adapters run through mull with cultivate's tests passing.
- glean's second test runs on per-post raw items and reports its AUC with an interval.
- Every store in the table above names its lifetime in its owner's `docs/architecture.md`.
