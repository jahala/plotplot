# Document: a dated, verbatim snapshot of a page

An article, release notes, a pricing page, a notice of shutdown: what a page said at the moment it was fetched. A document is not a person's post; a page with many people's posts is read as posts by an expanding adapter.

Schema: `contracts/document.schema.json`. Fixtures: `contracts/fixtures/document/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `document`, `source`, the `url` (or `locator`) and `content`, joined by newlines, as lowercase hex |
| `source` | yes | the adapter that fetched it |
| `channel` | yes | the site or publisher |
| `url` | when public | the page's address |
| `locator` | when private and no url | a file and line, a message id |
| `title` | no | the page's title |
| `media_type` | yes | `text/html`, `text/markdown` or `text/plain` |
| `content` | yes | the page as served, decoded to text, never cleaned or shortened |
| `published_at` | yes, may be null | when the page says it was published |
| `subjects` | no | ids of the subjects the page is about, known by an exact identifier |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Rules a reader may rely on

- **A changed page is a new document.** Same address, new content, new id; both are kept, so "what did the pricing page say in March" is a filter.
- **Content is the source's, whole.** A reader that wants plain text from HTML extracts it with its own code. A binary file such as a PDF is kept as a raw response and is not a document in this version.
- **Personal data follows the redaction contract** (`jahala/plotplot:contracts/redaction.md`) before a document leaves mull for any reader that writes it into a repository.
