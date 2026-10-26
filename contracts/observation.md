# Observation: a number about a subject at a moment or over a period

Downloads, stars, prices, ranks, installs: a number the source gave about one subject. Observations carry the signal the plan calls movement, and the prices and installs that show people paying.

Schema: `contracts/observation.schema.json`. Fixtures: `contracts/fixtures/observation/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `observation`, `source`, `subject`, `metric`, the time key and `value`, joined by newlines, as lowercase hex. The time key is `at`, or the period as `from/to`; `value` is written as JSON writes the number |
| `source` | yes | the adapter that fetched it |
| `subject` | yes | the id of the subject the number is about |
| `metric` | yes | the source's own name for the number: `downloads`, `stargazers_count` |
| `value` | yes | the number as the source gave it |
| `unit` | no | the unit, when the source gives one |
| `at` | one of `at` or `period` | the moment a count read at a moment holds |
| `period` | one of `at` or `period` | the span a count over a span covers: `from` included, `to` not |
| `url` | no | where the number can be seen |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Rules a reader may rely on

- **The source's number, unadjusted.** No observation is derived, summed or smoothed by mull. A total over months is a reader's sum.
- **A revised number is a new observation.** If the source later reports a different value for the same span, both are kept, each with its `fetched_at`.
- **Never about a person.** An observation's subject is a product or project, never a person.
