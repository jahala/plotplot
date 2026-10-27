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
| `author` | yes, may be null | sha256 of the owner's salt, a newline, the source's identity space, a newline, and the author's handle there, as lowercase hex. The identity space is the platform's account space for platform accounts (`github.com` for every GitHub repository) and the channel itself for a source with no shared identity; the adapter manifest declares which |
| `author_role` | no | the source's own statement of the author's relation to the channel as fetched, verbatim: GitHub's author association (MEMBER, OWNER, COLLABORATOR, CONTRIBUTOR, NONE) |
| `author_kind` | no | `kind` (`person` or `bot`) and `from` (`source` when the source says so, such as GitHub's Bot accounts; `plan` when the owner's capture plan lists the handle under `bots`) |
| `parent` | no | the id of the item it replies to |
| `reactions` | no | counts of votes and reactions, each with the moment it was counted |
| `labels` | no | the source's own labels on the item as fetched, verbatim: issue labels, flair, tags |
| `status` | no | the source's own statement of the thread's state as fetched, verbatim: `state` (such as open or closed), `reason` (such as completed or not_planned) and `changed_at`, when it last changed |
| `signals` | no | numbers the source gave, under the source's own field names |
| `licence` | no | the source's own licence statement for the item, verbatim, such as Stack Exchange's `content_license` (`CC BY-SA 4.0`); absent when the source states none per item, never inferred. With `url`, which links to the item and names its author there, and the source the adapter manifest names, it is what attribution needs |
| `private` | yes | whether it came from a private stream |
| `raw_ref` | yes | where the raw response it came from is kept |

## Rules a reader may rely on

- **Verbatim.** `text` is what the person wrote. A paraphrase cannot be cited, so no adapter
  shortens, translates or cleans it beyond decoding.
- **Stable identity.** The same thing said in the same place has the same `id` on every run, so
  fetching is append-only and a re-run adds only what is new.
- **People count once per platform, unnamed.** `author` is a salted hash. The salt stays with the
  owner and is never written into an item. mull computes the hash over the source's identity space,
  which the adapter manifest declares; an adapter only hands mull the handle (`contracts/protocol.md`).
  The same account hashes the same across every channel of one platform, so readers count a person
  once there; across platforms, and across sources with no shared identity, people stay apart.
- **Labels, reactions, status and author role are snapshots.** They change after a person posts,
  so each is as of the item's `fetched_at`. The same `id` may appear on more than one line, appended
  again only when its labels, reactions, status or author role differ from its latest line; the latest line is current, and
  the earlier ones answer "as of" a date. A source's labels and status are copied, never inferred.
- **Programs are marked, never guessed.** `author_kind` is set only when the source says what an account is or the owner lists the handle as a bot in the capture plan, and it says which; a plan's listing wins over the source. A reader counting people leaves out authors marked bot.
- **Nothing is filled in.** A source that gives no date or no author writes `null`. Whether an
  undated item counts as evidence is the reader's rule, written in the reader's own docs.
- **Private items** carry `private: true` and follow the owner's lifetime for private data; public
  items are kept, because the history of what was asked is what compounds.
