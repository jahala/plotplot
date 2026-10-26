# Raw item: one thing one person said

The contract between every tool that reads the world and every tool that uses what it read.
mull writes raw items. cultivate, glean and harvest read them. Until mull is a fetcher, a tool
that fetches on its own (cultivate's source adapters, glean's rough test) writes the same shape,
so moving the fetch into mull later changes no reader. A person can write raw items by hand,
which is how each reader keeps full value alone (law §1, invariant 4).

Schema: `contracts/raw-item.schema.json`. Fixtures: `contracts/fixtures/raw-item/`. Test:
`contracts/test/raw-item.test.sh`.

## The shape

A file of JSON Lines, one item per line. Each item holds:

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `source`, a newline, `url` (or `locator`), a newline, and `text`, as lowercase hex |
| `source` | yes | the adapter that fetched it |
| `channel` | yes | where the person said it |
| `url` | when public | the public address |
| `locator` | when private and no url | a file and line, or a message id |
| `title` | no | the thread or page title |
| `text` | yes | what the person said, verbatim |
| `said_at` | yes, may be null | when they said it |
| `fetched_at` | yes | when it was fetched |
| `author` | yes, may be null | sha256 of the owner's salt, a newline, the channel, a newline, and the author's handle there, as lowercase hex |
| `parent` | no | the id of the item it replies to |
| `reactions` | no | counts of votes and reactions, each with the moment it was counted |
| `labels` | no | the source's own labels on the item as fetched, verbatim: issue labels, flair, tags |
| `signals` | no | numbers the source gave, under the source's own field names |
| `private` | yes | whether it came from a private stream |
| `raw_ref` | yes | where the raw response it came from is kept |

## Rules a reader may rely on

- **Verbatim.** `text` is what the person wrote. A paraphrase cannot be cited, so no adapter
  shortens, translates or cleans it beyond decoding.
- **Stable identity.** The same thing said in the same place has the same `id` on every run, so
  fetching is append-only and a re-run adds only what is new.
- **People count once, unnamed.** `author` is a salted hash. The salt stays with the owner and is
  never written into an item. mull computes the hash; an adapter only hands mull the handle
  (`contracts/protocol.md`). The same person on the same channel hashes the same, so readers count
  distinct people; across channels they stay apart.
- **Labels and reactions are snapshots.** They change after a person posts, so each is as of the
  item's `fetched_at`. The same `id` may appear on more than one line, appended again only when
  its labels or reactions differ from its latest line; the latest line is current, and the earlier
  ones answer "as of" a date. A source's labels are copied, never inferred.
- **Nothing is filled in.** A source that gives no date or no author writes `null`. Whether an
  undated item counts as evidence is the reader's rule, written in the reader's own docs.
- **Private items** carry `private: true` and follow the owner's lifetime for private data; public
  items are kept, because the history of what was asked is what compounds.
