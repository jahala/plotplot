# Reaction: one person's reaction or vote on a post

For a source that names who reacted, such as an issue tracker's reaction lists, each reaction is its own record, so readers count distinct people who wanted something and not only a total. A source that gives only totals keeps them in the post's `reactions`, each count with its moment (`jahala/plotplot:contracts/raw-item.md`).

Schema: `contracts/reaction.schema.json`. Fixtures: `contracts/fixtures/reaction/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `reaction`, `source`, `target`, `kind` and `author`, joined by newlines, as lowercase hex |
| `source` | yes | the adapter that fetched it |
| `channel` | yes | the target post's channel |
| `target` | yes | the id of the post reacted to |
| `kind` | yes | the reaction as the source names it: `+1`, `heart`, `upvote` |
| `author` | yes | the salted hash of who reacted, made as a post's author is; never null, since a source that does not name people gives counts instead |
| `said_at` | yes, may be null | when the reaction was given |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Rules a reader may rely on

- **One person, one reaction of a kind, one id.** The id leaves out the moment, so a reaction read again on a later run is the same record.
- **A withdrawn reaction is not deleted.** Records are append-only. A reader that needs current reactions compares the post's latest reaction counts.
