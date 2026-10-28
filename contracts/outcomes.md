# Outcomes: what people want to be true, and who wants it

The contract between cultivate, which writes outcomes, and the tools that act on them: graft reads which
outcomes a feature serves and how many people want each, glean reads where demand sits on the job, and
cultivate's own board draws people, outcomes and needs from it. It sits beside the needs document of the
same run (`contracts/needs.md`) and points into it by need id; the needs format does not change.

The method is Jobs-to-be-Done and Outcome-Driven Innovation (Ulwick, *What Customers Want*, 2005): people
use a product for a core job, mapped in universal steps, and a request is a proposed solution whose
desired outcome is what gets counted. The owner's agent drafts the job map and the outcomes from prompts
cultivate ships; the judge places every need on one outcome or none; code counts the people. Validated
on three products (jahala/cultivate `docs/research/validation-outcomes-2026-10-27.md`).

Schema: `contracts/outcomes.schema.json`. Fixtures: `contracts/fixtures/outcomes/`. Tests:
`contracts/test/outcomes.test.sh` here and `tests/test_outcomes_format.py` in cultivate, which also checks
every document before writing it. cultivate owns the content; the umbrella's `contracts/` is where it
lands, beside the needs format it points into (`contracts/needs.md`).

## The shape

One JSON document per run, named by the date of the needs document it places.

| Field | Required | Meaning |
|---|---|---|
| `format` | yes | `cultivate-outcomes/1`; a change of meaning is a new version, never an edit |
| `product` | yes | the product, by name, as in the needs document |
| `run` | yes | the date of the needs document whose needs it places |
| `model` | yes | the judge model behind every placement |
| `rule` | yes | when a need counts as placed: the judge picks that outcome from the whole list, or none, at 0.5 or more |
| `list` | yes | 12 hex characters naming the outcome list; it changes only when a saturation round adds outcomes |
| `core_job` | yes | the job the product's people are trying to get done, in one solution-free sentence |
| `steps` | yes | the 14 universal steps in order, each as below |
| `outcomes` | yes | every outcome on the list with people behind it or not, each as below |
| `needs` | yes | every need of the run by id: its `outcome` and `step`, or null when no outcome fits |
| `coverage` | yes | `needs` placed of `of_needs`, and distinct `people` behind a placed need of `of_people` |
| `round` | yes | the saturation-round rule and whether this run opened a round (`new`, `unserved`, `top_unserved`, `since`, `opened`) |

Each step holds `step` (define, locate, prepare, confirm, execute, monitor, modify, conclude, acquire,
personalize, sustain, integrate, collaborate, learn), `text` (the step in this product's words), and the
counts `outcomes`, `needs`, `people` and `open_people`.

Each outcome holds:

| Field | Meaning |
|---|---|
| `id` | `<step>-<n>`; its step is the part before the dash |
| `step` | the step it belongs to |
| `outcome` | the desired outcome statement: a direction, a metric, an object and a context ("Minimize the time it takes to…") |
| `needs` | the ids of the needs placed on it |
| `people` | distinct people behind those needs, counted once each as the needs document counts them |
| `open_needs`, `open_people` | the same, over the needs whose thread is neither answered nor closed |
| `cites`, `cites_placed` | how many needs the drafting agent cited for it, and how many of those the judge placed on it |

## Rules a reader can rely on

- People are counted once each across an outcome's needs and across a step's outcomes, from the same
  author hashes as the needs document, with the team and bots left out. Counts across outcomes overlap.
- A need sits on at most one outcome. Its step is its outcome's step.
- Outcome ids are stable while the list keeps them; a saturation round only adds outcomes.
- Personas built on outcomes are not in this document; they are in the run's people file, which names
  their outcomes by these ids.
