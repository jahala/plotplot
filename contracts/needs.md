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
| `format` | yes | `cultivate-needs/1`; a change of meaning is a new version, never an edit. Version 1 gained the state `unknown` on 2026-10-26, agreed with the umbrella before any reader depended on the three states alone, and the optional `noise`, a need's optional `statement` and the noise check's `top` on 2026-10-30, and a need's optional `sources` and `resolution` on 2026-11-04, and a need's optional `about` on 2026-11-05 |
| `product` | yes | the product the needs are about, by name |
| `run` | yes | `at` (the run's date), `previous` (the run it is compared with, or null), and `window` (`from`, `to`), the period the window counts cover |
| `scope` | yes | the channels read, each with how many raw items it gave, so a reader knows what was searched |
| `features` | no | where the feature tags came from (graft's pages, or a product's areas file) |
| `rules` | yes | every threshold the numbers used: the merge cut, the quiet period, the movement rule, the warning rule, `resolution`, the question and floor behind each need's `resolution`, when needs carry one, and `about`, the rule that marks a need as about the project, with the labels it reads, when needs carry one |
| `judge` | yes | the judge model and question set behind every judged field |
| `noise` | no | the build's noise check: `codebook` (where the rules for a need live), `sample` and `seed` (the needs drawn at random and graded), `clean` (how many the grader called a need), `share`, `bar` (fixed before the check), `top` (`sample`, `clean` and `bar` for the needs with the most people, graded too because tools read the top first), `fit` (share at least bar, and the top's clean count at least its bar), `grader` (a person, or a fresh agent reading blind) and `at` |
| `needs` | yes | the needs, each as below |

Each need holds:

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | `need:` and 16 hex characters, from the lead item's id; the same across runs while the need keeps its lead (a need whose lead changes gets a new id) |
| `lead` | yes | the raw item id of the post the need was first made from |
| `label` | yes | a short name; `label_source` says whether it is the lead's own title or a drafted label checked against the members |
| `statement` | no | `text`, one plain sentence of what the person wants or struggles with, written by the owner's agent from the lead post's own text in the job's or the product's terms, and `fits`, the judge's yes that the lead post says it; the label stays the title, as provenance |
| `kind` | yes | `ask` (wants something new), `problem` (something fails) or `remove` (wants something off or optional) |
| `about` | no | `product` or `project`: whether the need asks only for something around the product (its website, blog, the documentation site's frame, its repository and how it is run, its community spaces). Decided from the lead post's own labels, by the rule in `rules.about`: `project` when it carries a label the product's owner lists as the project's own, `product` when it carries other labels; absent when it carries none, or when the owner lists none. A reader may list project needs apart; a need marked `product`, or not marked, may still be about the project, since the rule finds about half of them |
| `features` | yes | the features it is about, from the feature list; `confident` is true only at a judge confidence of 0.9 or more |
| `sources` | no | `count` and `names`: the independent sources the need's member posts come from, counted by code, so a reader can tell a need several sources back from one a single source shows; a person on two sources counts as two, since authors are hashed per site |
| `resolution` | no | what kind of answer the person asks for, judged from the statement and the lead post, never from what the product can do: `label` is `change` (the product should do something new or different), `guidance` (how to do something, or what to do), `repair` (something should work and does not) or `outside` (no product in the job would do it), with the judge's `confidence`, at least 0.8. Under that floor, or when no label fits, `label` is null and `why` says which (`below confidence`, `no label fits`), never a guess. Whether a change is a new ability or an option on a feature is graft's to tell, against the features. The floor holds back most repairs, so a null label says nothing about the need's kind |
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
- **Use only a fit build.** A tool reads `noise` before it uses the needs, and uses them only when `fit` is
  true. A document without `noise` has not been checked, and says nothing about how clean it is.
- **Signals show their rule.** A judged signal appears only after its judgement passed a
  blind-graded bar, and names it.

## Lifetime

A needs document is written per run and kept for the life of the portfolio, in the product's or the
portfolio's repository. Raw items stay in mull's store; a needs document never holds their text.
