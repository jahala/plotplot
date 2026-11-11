# Capture plan: what one project asks mull to read

Each project keeps one capture plan in its repository: the subjects it listens about, the sources it reads with their configuration, and, for each private stream, how long its records live. mull runs the plan; it does not write it. cultivate can draft one from "whom are we listening to", and a person can write one by hand.

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
| `consent` | no | `given_by`, `given_on`, `scope` and `evidence` (where the consent is kept), for an owner who wants it recorded; nothing requires it |
| `lifetime_days` | for a private stream | how long its records are kept before the prune command deletes them |
| `terms` | when the manifest sets `terms_per_site` | the site's own terms, as facts, in the manifest's shape |
| `posting` | no | the place's own posting rules, as facts, in the manifest's `posting` shape (`contracts/adapter.md`). A place's rules are the source's `rules` plus the entry's. Its `promotion` is the entry's when the entry gives one, else the source's |
| `volume` | no | the place's recent volume, last measured, as facts, in the manifest's `volume` shape (one entry per kind of post counted) (`contracts/adapter.md`), for a generic adapter whose places differ |
| `ruling` | when the source's terms restrict the use | the owner's ruling: `use` (`allowed` or `refused`), `by`, `on` and `says`, copied from the owner's rulings file (`contracts/rulings.md`) |
| `bots` | no | handles on this source the owner knows to be programs, as the source writes them; their records are marked `author_kind` bot, from the plan |
| `budget` | when the adapter is paid or needs a credential | the well budget the entry spends; the owner chooses it, and mull refuses to run such an adapter without one, since its requests go through well |

## Rules a reader may rely on

- **Public once, private apart.** A public source is fetched once into the shared public store, and every project reads its slice; a thousand projects never fetch the same forum a thousand times. A private stream's records go to that project's own store and nowhere else.
- **`private` is a label, and a private stream has a lifetime.** A private entry's records go to the project's own store and carry `private: true`; mull refuses a private entry with no `lifetime_days`. No consent is required to read one (the owner's ruling of 2026-10-27: no privacy or security gates in the research tools).
- **Rulings are copied, not written per plan.** A tool that writes a capture plan copies each entry's `ruling` from the owner's rulings file: the adapter's entry under `by_adapter` when there is one, otherwise `all` (`contracts/rulings.md`).
- **Each owner rules for their own use.** An entry whose source's terms carry restrictions, forbid commercial use, or come from an account whose terms forbid automation runs only when its `ruling` allows it; an entry whose ruling refuses never runs. No ruling changes a source's rate limit, its deletion requirements, or mull's refusal to work around a captcha, two-factor or a login it was not given.
- **Robots are a default, not a rule.** For an adapter that reads any site, mull obeys the site's robots.txt as the ruling's `robots` says: `obey` when it says nothing, `agent` to keep only what the site tells mull and every agent, `ignore` to read what robots.txt disallows. mull still asks for robots.txt and keeps it, so a reader can see what the site asked. The limits that protect a source or the people who wrote there are not the user's to lift: rate limits, deletion duties, and never working around a captcha, two-factor or a login mull was not given.
- **No cadence here.** When the plan runs is the operating system's scheduler or a workflow file, never the plan (law §6).
