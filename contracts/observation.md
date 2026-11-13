# Observation: a number, or a name from a fixed list, about a subject at a moment or over a period

Downloads, stars, prices, ranks, installs: a number the source gave about one subject. Observations carry the signal the plan calls movement, and the prices and installs that show people paying. A categorical metric, such as the forum software a site runs, gives a name instead of a number.

Schema: `contracts/observation.schema.json`. Fixtures: `contracts/fixtures/observation/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `observation`, `source`, `subject`, `metric`, the time key and `value`, joined by newlines, as lowercase hex. The time key is `at`, or the period as `from/to`; `value` is written as JSON writes it: a number as a number, a moment or a name as a quoted string |
| `source` | yes | the adapter that fetched it |
| `subject` | yes | the id of the subject the number is about |
| `metric` | yes | the source's own name for the number: `downloads`, `stargazers_count`; for a categorical metric, the name its adapter's manifest gives it under `categories` |
| `value` | yes | the number, or for a metric that is a moment (such as a repository's last push) the date-time, as the source gave it; for a categorical metric, one of the names its adapter's manifest lists for it, in lowercase |
| `unit` | no | the unit, when the source gives one |
| `at` | one of `at` or `period` | the moment a count read at a moment holds |
| `period` | one of `at` or `period` | the span a count over a span covers: `from` included, `to` not |
| `url` | no | where the number can be seen |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Rules a reader may rely on

- **The source's number, unadjusted.** No observation is derived, summed or smoothed by mull. A total over months is a reader's sum.
- **A name comes from a fixed rule.** A categorical value is one of the names the adapter's manifest lists for its metric (`contracts/adapter.md`, Categories), each given by a fixed rule over what the source says about itself. mull refuses a name the manifest does not list. A source that could not be read gives no observation; a skipped line says why.
- **A revised number is a new observation.** If the source later reports a different value for the same span, both are kept, each with its `fetched_at`.
- **Never about a person.** An observation's subject is a product or project, never a person.
