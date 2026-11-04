# Change: one event that changed a post's state or labels

A source that reports when a post was closed, reopened, labelled or unlabelled, such as GitHub's issue timeline, gives each event as its own record. A post's `status` and `labels` say how it stood when it was fetched (`jahala/plotplot:contracts/raw-item.md`); its changes say how it stood at any moment before. graft asks which requests were open on a past date, and only the events can answer it.

Schema: `contracts/change.schema.json`. Fixtures: `contracts/fixtures/change/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `change`, `source`, `target`, `field`, `op`, `value` (the empty string when null) and `said_at`, joined by newlines, as lowercase hex |
| `source` | yes | the adapter that fetched it |
| `channel` | yes | the target post's channel |
| `target` | yes | the id of the post that changed |
| `field` | yes | what changed: `state` or `label` |
| `op` | yes | the event as the source names it, verbatim, such as GitHub's `closed`, `reopened`, `labeled` or `unlabeled` |
| `value` | yes, may be null | the label's name, or the reason a post was closed as the source gives it; null when the event carries none, as a reopening does |
| `author` | yes, may be null | the salted hash of who made the change, made as a post's author is; null when the source names no one, as for a deleted account |
| `said_at` | yes | when the change was made |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Rules a reader may rely on

- **Replayable to any moment.** A post's state and labels at a moment are found by applying its changes said before that moment, in `said_at` order, to the post as it opened. A source writes changes only when it reports every change from the post's creation, as GitHub's timeline does.
- **One event, one id.** The id holds the moment, so a label added, removed and added again is three records, and the same event read on a later run is the same record.
- **Copied, never inferred.** `op` and `value` are the source's own words. A change the source does not report is not written.
