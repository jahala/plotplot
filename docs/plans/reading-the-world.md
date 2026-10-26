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

## Capture: a protocol, adapters on both sides, and plans per project

The owner's direction (2026-10-26): mull should capture any kind of data a project needs, for
thousands of projects. That is not thousands of adapters inside mull. mull is a small runner
around one protocol; most needs are configuration of a few generic adapters; the rare source gets
a small adapter an agent writes; and the output side is questions and formats, never one adapter
per tool. The old mull (tag `extraction-final` in `jahala/mull`) designed adapters on both sides
already, and its catalogue, cursor rules and transport middleware are read as prior art.

```
 sources          adapters in             mull                      adapters out         readers
 forum, HN    ─┐  generic: forum, feed,   runs adapters             query: questions     cultivate
 Discord log  ─┼► page to document,    ─► checks every record    ─► formats: JSON Lines, glean
 news article ─┤  export-file mapper,     stores once, shared        Parquet, Markdown    harvest
 downloads    ─┘  search (discovery);     census, limits, terms      digest, MCP tool     agents, people
                  custom: small programs  consent, lifetimes
```

### The protocol

An adapter is a small program in any language. It prints records, one JSON object per line, and
ends with one `state` record holding its cursor. mull runs it, checks every record against the
contracts, stores what passes and refuses what does not. The pattern is proven in open
data-integration tools with thousands of community connectors.

### Five record kinds

| Kind | Covers | Contract |
|---|---|---|
| post | anything one person said: comments, messages, reviews, emails | `contracts/raw-item.md` (landed) |
| document | a dated, verbatim snapshot of a page: an article, release notes, a pricing page | to draft |
| observation | a number about a subject at a moment: downloads, stars, prices, ranks | to draft |
| subject | one product or project and its exact identifiers across sources (a domain, a repository, a package, an app id) | to draft |
| state | where the adapter stopped, so the next run continues | part of the protocol |

mull links subjects only on exact identifiers. A fuzzy match is a judgment and belongs to
cultivate or glean.

### The adapter manifest

Every adapter ships a manifest (`contracts/adapter.md`, drafted by mull), recorded fixtures, and
tests that replay them. The manifest holds:

- the source, the adapter's kind (discover, expand or stream), the record kinds it emits, and the
  protocol version;
- the audiences and signals it carries;
- the source's terms (licence, attribution, commercial use, rate limit) and the adapter's use,
  allowed or refused with the reason;
- the hosts it may reach, and nothing else;
- for a generic adapter, the schema of its configuration;
- how a response splits into records, field by field;
- **access**: the method (`none`, `api-key`, `bearer`, `oauth2`, `basic`, `session-login`,
  `signed-request`, `export-file`); each credential by its Keychain name, fixed or refreshed;
  whose account it runs as; whether that account's terms allow automation; how access is
  obtained, whether approval is needed, its cost and its status; the least scopes it needs and
  whether they are read-only; when access expires and how it renews; any step a program must not
  take alone (two-factor, a captcha, a login the terms forbid automating); and whether calls are
  paid.

Code refuses a source whose terms forbid the use or the automation, and never works around a
captcha or two-factor: such a source becomes an export file the owner provides, or a refusal on
record. well (`docs/ideas/well.md`) owns every secret's life: storage, reading at call time,
refresh, and a warning before expiry. Access that is not ready reaches the owner as a request in
gate, not as a failed run.

### Generic adapters first

One forum adapter covers every site on the same forum software; one feed adapter covers every
feed; one page adapter turns any article into a document; one export mapper reads a chat, mail or
spreadsheet export through a field map. A custom adapter is only for what is left. An agent
writes it from the manifest template; it must pass its fixture tests, and mull refuses it until
it does. The runner composes login, rate limit, retry and recording around every adapter as
layers, so no adapter carries its own.

The first adapters, in order of what waits on them: Hacker News (glean's second test and
yardstick); local files and GitHub issues and discussions (cultivate's three, with cultivate's
tests still passing); web search as discovery (glean's stand-in); Discourse forums and Stack
Exchange as expanders. Until an adapter exists in mull, a tool may fetch with a stand-in script
beside the test that needs it, writing the same records; the stand-in moves into mull whole.

### Safety

Adapters are code from many authors. Each runs reaching only its manifest's hosts, receives its
keys from well at call time, and may write nothing but its records. The umbrella's redaction
contract (`contracts/redaction.md`) applies to documents that carry personal data. A source whose
site changes and breaks its adapter shows in the census, and upkeep opens the repair.

### Capture plans

Each project keeps a capture plan in its repository: its sources and their configuration, its
subjects, the consent behind each private stream, and each stream's lifetime. cultivate can draft
it from "whom are we listening to"; mull runs it. Public sources are fetched once into a shared
public store and each project reads its slice, so a thousand projects never fetch the same forum
a thousand times. Private streams stay in that project's own store.

### The output side: questions and formats

Readers never open mull's storage. They ask `mull query`, which answers the questions they need
and returns the contract formats:

| Reader | Asks |
|---|---|
| cultivate | every post in a window per channel or subject, with its thread |
| glean | posts and observations about a need as of a date, from sources reaching its people; one author's posts over time |
| harvest | posts after a release that mention a feature, and each asker's address, to reply there |
| upkeep | release notes, end-of-support notices and advisories for a product's dependencies |

Formats are delivery adapters, as the old mull named them: JSON Lines for tools, Parquet or CSV
for analysis, a Markdown digest for people and agents, and an MCP tool of at most three tools for
an agent in a session (law §1, invariant 5). A new reader needs a new question at most, never a
new store, and the storage behind `mull query` can change without any reader changing.

### Kill criteria, written before the build

- The generic adapters cover at least 80 percent of the first ten projects' sources by
  configuration alone.
- An agent writes a new adapter that passes its fixture tests in one session.
- A re-run over the same window gives byte-identical records.

## The store

Three layers, each owned by one tool. Each layer points to the one below by id and never copies
text, except the quote a seed cites.

| Layer | Owner | Holds | Where | Lifetime |
|---|---|---|---|---|
| Records | mull | one record per line, by kind, source and month: `<kind>/<source>/<channel>/<yyyy-mm>.jsonl` | a data folder outside git: one shared public store, one private store per project | public items kept; private items for the owner's set period, then deleted on schedule |
| Raw responses | mull | what each source returned, compressed, named by its sha256, cited from each item's `raw_ref` | the same data folder, outside git | kept while any item cites them |
| Census | mull | items per channel per period, per run | the data folder | kept, compacted to one line per channel per month after a year |
| Needs | cultivate | member item ids; distinct people counted by role (wanting, paying, building, gone quiet); counts per month; first and last seen | the product's or portfolio's repository | life of the portfolio |
| Scores | glean | per need, per compass version: each part's value with cited item ids, the rules and judge model versions, judge recording references | glean's repository | kept, so a seed is never proposed twice for the same reason |
| Seeds | glean | the seed files a person reads | the seedbank | until planted or archived |
| Outcomes | glean, then harvest | which needs later proved real, and the evidence | glean's repository | kept: this is glean's yardstick |

Files stay the truth. A closed month of raw items may be compacted into a columnar file
(Parquet) and read with an embedded query engine (DuckDB) for heavy counts; the compacted file is
derived and can always be rebuilt from the lines, and a full-text index is derived the same way.
The scale designed for starts at about ten million records (roughly 10 GB of lines, a few GB
compacted) and stays on files into the hundreds of millions when partitioned; on more than one
machine the same files live in a bucket.

**Measured before it is built.** Before mull builds the store, it loads the Hacker News corpus
plus about ten million generated records and times every question in the output table. If those
are too slow, or two machines must write at once, the next step is a real database. That breaks
the law's invariant 2, so it is the owner's decision, made with the numbers.

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

The raw item contract (already landed), the protocol, the adapter manifest, the document,
observation and subject contracts, and the role counts in cultivate's needs format. A wrong choice in any of these is paid for by every adapter and every reader.

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
