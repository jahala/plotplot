# Capture plan: what one project asks mull to read

Each project keeps one capture plan in its repository: the subjects it listens about, the sources it reads with their configuration, and, for each private stream, the consent behind it and how long its records live. mull runs the plan; it does not write it. cultivate can draft one from "whom are we listening to", and a person can write one by hand.

Schema: `contracts/capture-plan.schema.json`. Fixture: `contracts/fixtures/capture-plan/valid.json`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `plan` | yes | the format version, `"1"` |
| `project` | yes | the project's name |
| `portfolio` | yes | whose salt and stores the project uses |
| `subjects` | yes | the products and projects the project listens about, each by name and exact identifiers (`contracts/subject.md`) |
| `sources` | yes, at least one | one entry per source read |

Each source entry holds:

| Field | Required | Meaning |
|---|---|---|
| `adapter` | yes | the adapter's id |
| `version` | no | the adapter's major version the entry was written for; mull refuses a different one |
| `config` | yes | valid against the manifest's config schema; `{}` for an adapter with none |
| `from` | yes | the earliest moment of saying to read; later runs continue from the cursor |
| `why` | no | whom the source lets the project hear |
| `private` | yes | true for a private stream |
| `consent` | for a private stream | `given_by`, `given_on`, `scope` and `evidence` (where the consent is kept) |
| `lifetime_days` | for a private stream | how long its records are kept before the prune command deletes them |
| `terms` | when the manifest sets `terms_per_site` | the site's own terms, as facts, in the manifest's shape |
| `ruling` | when the source's terms restrict the use | the owner's ruling: `use` (`allowed` or `refused`), `by`, `on` and `says` |

## Rules a reader may rely on

- **Public once, private apart.** A public source is fetched once into the shared public store, and every project reads its slice; a thousand projects never fetch the same forum a thousand times. A private stream's records go to that project's own store and nowhere else.
- **No private stream without consent and a lifetime.** mull refuses an entry that lacks either.
- **Each owner rules for their own use.** An entry whose source's terms carry restrictions, forbid commercial use, or come from an account whose terms forbid automation runs only when its `ruling` allows it; an entry whose ruling refuses never runs. No ruling changes a source's rate limit, its deletion requirements, or mull's refusal to work around a captcha, two-factor or a login it was not given.
- **No cadence here.** When the plan runs is the operating system's scheduler or a workflow file, never the plan (law §6).
