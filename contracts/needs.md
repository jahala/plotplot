# Needs: what people need, merged and counted

The contract between cultivate, which writes needs, and every tool that acts on them: glean scores
them, graft judges features against them, harvest checks them after a release, and cultivate's own
board shows them. A need is what several people said, judged to be the same request or the same
problem, with the distinct people behind it counted by role over time.

Schema: `contracts/needs.schema.json`. Fixtures: `contracts/fixtures/needs/`. Tests:
`contracts/test/needs.test.sh` here and `tests/test_needs_format.py` in cultivate. cultivate owns the content; the umbrella's `contracts/` is where it
lands, beside the raw-item format it builds on (jahala/plotplot `contracts/raw-item.md`).

## The shape

One JSON document per run. It holds the run, what was read, the rules every number used, and the
needs.

| Field | Required | Meaning |
|---|---|---|
| `format` | yes | `cultivate-needs/1`; a change of meaning is a new version, never an edit. Version 1 gained the state `unknown` on 2026-10-26 and the optional `about` on 2026-10-29, agreed with the umbrella before any reader depended on the three states alone |
| `product` | yes | the product the needs are about, by name |
| `run` | yes | `at` (the run's date), `previous` (the run it is compared with, or null), and `window` (`from`, `to`), the period the window counts cover |
| `scope` | yes | the channels read, each with how many raw items it gave, so a reader knows what was searched |
| `features` | no | where the feature tags came from (graft's pages, or a product's areas file) |
| `rules` | yes | every threshold the numbers used: the merge cut, the quiet period, the movement rule, the warning rule, and `about` (the cut that marks a need as about the project) when the run asked |
| `judge` | yes | the judge model and question set behind every judged field |
| `needs` | yes | the needs, each as below |

Each need holds:

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | `need:` and 16 hex characters, from the lead item's id; the same across runs while the need keeps its lead (a need whose lead changes gets a new id) |
| `lead` | yes | the raw item id of the post the need was first made from |
| `label` | yes | a short name; `label_source` says whether it is the lead's own title or a drafted label checked against the members |
| `kind` | yes | `ask` (wants something new), `problem` (something fails) or `remove` (wants something off or optional) |
| `about` | no | `product` when the need is about the product itself, `project` when it is about the project around it: its website, blog or feeds, documentation site, repository files (README, contribution rules, issue templates), release notes page or community channels. Judged per need, `project` at 0.5 or more; absent when the run did not ask. A reader judging the product can leave `project` needs out |
| `features` | yes | the features it is about, from the feature list; `confident` is true only at a judge confidence of 0.9 or more |
| `members` | yes | the raw items merged into it: `item` (raw item id), `url` when public, `said_at`, `lead`, and `same`, the judge's probability that it asks the same as the lead (null for the lead) |
| `first_seen`, `last_seen` | yes | the earliest and latest moment any person engaged with the need, in what was read |
| `people` | yes | distinct people by role, below |
| `state` | yes | `open`, `answered`, `closed` or `unknown`, with when and how, read from the lead post's own `status` as its source states it; `unknown`, with `at` and `how` null, when the source states none, such as call notes |
| `activity` | no | events (reactions and comments) per week over the window, for movement and sparklines |
| `signals` | yes | reasons the need matters now, each with its rule and basis; may be empty |

### People, by role

A builder and a buyer are different evidence, so people are counted by role, never only in total.
Each role holds the same fields:

| Field | Meaning |
|---|---|
| `count` | distinct people in this role over everything read, or null when no source read measures this role |
| `in_window` | distinct people in this role inside the run's window, or null |
| `first_seen`, `last_seen` | the first and last moment anyone in this role engaged, or null |
| `by_month` | distinct people in this role per calendar month, keyed `yyyy-mm` |
| `basis` | one sentence: how the role was counted, or why it was not |

| Role | Who |
|---|---|
| `wanting` | people who asked for it, reported it, commented on it or reacted to it |
| `paying` | people who say they pay, or would pay, for it, or whom a paid source shows paying |
| `building` | people who say they built something for it, or whom a builders' source shows building it |
| `gone_quiet` | wanting people whose last engagement is older than `rules.quiet_after_days`; their `by_month` is keyed by the month they were last seen |

### Signals

Each signal names its `kind`, its `basis` (`counted` by code, or `judged` under a passed bar), the
rule it passed, its numbers, and the raw item ids that show it (`evidence`). Kinds in version 1:
`movement` (the pace changed by the movement rule), `intent_to_leave` (people say they would leave,
above the warning rule), and `answered` (the lead thread closed as done since the previous run).

How cultivate reads a source's status into a state is cultivate's rule: a post whose status is closed
with the reason completed is `answered`; closed for any other reason is `closed`; open is `open`; no
status is `unknown`. Words are compared without regard to case, because sources spell them differently.

## Rules a reader may rely on

- **Ids, never copied text.** A need points to raw items by id. Its only words are its label; the
  quote a reader shows comes from the raw item.
- **People count once, unnamed.** Distinct people come from the raw items' salted author hashes,
  per channel. Counts in different needs overlap and do not add up.
- **Nothing is filled in.** A role no source measures has a null count and says why in `basis`. A
  zero means the role was measured and nobody was found in what was read, which is not a claim that
  nobody exists.
- **Positive claims only.** No field says a feature, a person or a need is missing.
- **Every number is reproducible.** The rules, the judge model and the scope that produced a number
  are in the same document, and a need's members are listed, so a reader can recount.
- **Signals show their rule.** A judged signal appears only after its judgement passed a
  blind-graded bar, and names it.

## Lifetime

A needs document is written per run and kept for the life of the portfolio, in the product's or the
portfolio's repository. Raw items stay in mull's store; a needs document never holds their text.
