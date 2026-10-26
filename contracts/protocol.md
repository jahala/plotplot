# Protocol: how mull runs an adapter, version 1

An adapter is a small program in any language. mull starts it, writes JSON lines to its standard input, and reads JSON lines from its standard output. The adapter asks mull to fetch, reads the answers, prints the records it finds, and ends with one `state` line holding its cursor. mull checks every line, completes and stores the records that pass, and refuses the rest.

Schema: `contracts/protocol.schema.json`, one line in either direction. Fixtures: `contracts/fixtures/protocol/`. Test: `test/contracts.test.ts`.

## Why the adapter asks mull to fetch

The adapter never opens a connection, never reads a key and never sees the salt. It asks; mull fetches. That one choice gives every adapter, from any author, the same guarantees:

- **Replay is native.** mull keeps every response it fetched. A replay answers the same requests from the kept responses, so an adapter whose output depends only on its input writes the same bytes.
- **Hosts are enforced.** mull refuses a request to a host the manifest does not name.
- **Secrets stay in mull.** mull adds credentials from the Keychain to the request; the adapter's code and its logs never hold them. Authors are hashed by mull, so the salt never leaves it.
- **Limits and doors are held once.** Rate limits, retries of free requests, and the claim-before-send door for paid calls live in the runner, not in each adapter.

## A run

1. mull reads the adapter's manifest (`contracts/adapter.md`) and refuses to start when the manifest is invalid, its use is refused, or its access is not ready. Access that is not ready becomes a request to the owner.
2. mull starts the program named by the manifest's `run`, from the adapter's folder, never through a shell, with an empty environment except `PATH`.
3. mull writes one `start` line: the protocol version, the window, the cursor from the adapter's last `state` line for this capture plan entry (or null), and the entry's configuration. The property the manifest names in `reaches.config_paths` arrives resolved to absolute paths, so the adapter can name them in `file:` requests.
4. The adapter writes `fetch` lines. For each, mull writes back one `response` line, live or replayed, or one `refused` line with the reason. Answers come in the order the fetches were asked.
5. The adapter writes `record` lines, each naming the fetch it was read from. When the source says an item it read before was deleted, the adapter writes a `gone` line for its address. mull keeps these with the run, and `mull prune` removes every line at that address, and every response that held one, for a source whose terms require deletion.
6. The adapter writes one `state` line. mull closes the adapter's input, and the adapter exits 0.

The run is kept whole or not at all. Only a run whose last line is a valid `state` and whose exit status is 0 has its records stored and its cursor saved. Responses are kept either way, because a fetched response may have been paid for.

## Lines

| Line | Direction | Holds |
|---|---|---|
| `start` | mull to adapter | `protocol` (`"1"`), `window` (`from` included, `to` not), `cursor`, `config` |
| `fetch` | adapter to mull | `id` (the adapter's name for the fetch, unique in the run) and `request`: `method` (`GET` or `POST`), `url` (`https://` or `file:`), `headers` (never credentials), `body` (base64) |
| `response` | mull to adapter | `id`, `status`, `headers` (names in lowercase), `body` (base64), `received_at`. A `file:` request is answered with the file's bytes; for a folder, with a JSON array of `{"name", "folder"}` entries sorted by name, under the header `content-type: application/vnd.mull.folder+json`; for a path with nothing there, status 404 |
| `refused` | mull to adapter | `id` and `reason`: `host-not-declared`, `access-not-ready`, `ceiling`, `unsettled-claim`, `not-recorded` (a replay with no kept response), `rate-limit-exhausted`, `unanswered` (the request failed before the source answered; a paid request stays claimed and is never resent) |
| `record` | adapter to mull | `kind` (`post`, `document`, `observation`, `subject` or `reaction`), `from` (the fetch id), `fields` |
| `gone` | adapter to mull | `from` (the fetch whose response said so) and `address` (the url or locator of an item the source says was deleted) |
| `state` | adapter to mull | `cursor`: any JSON value, read back on the next `start` |

Standard error is free text. mull keeps it in the run's log and never parses it.

## What mull completes in a record

An adapter writes what it read. mull writes what only mull can know, so no adapter can get it wrong:

- `source`: the manifest's `adapter`.
- `fetched_at`: the `received_at` of the response named by `from`.
- `raw_ref`: where that response is kept.
- `private`: from the capture plan entry.
- `author`: the adapter writes `handle`, the author's handle on the channel, or null; mull writes the salted hash.
- `author_kind`: the adapter writes `author_kind` as `person` or `bot` when the source says what the account is; mull writes it with `from: source`, or as `bot` with `from: plan` when the capture plan entry lists the handle under `bots`, which wins.
- `parent` and a reaction's `target`: the adapter writes `parent_address` or `target_address`, the post's address; mull writes the id of the post at that address, from the same run or the store.
- a document's `subjects` and an observation's `subject`: the adapter writes exact identifiers (`subject_identifiers`, `subject_identifier`); mull writes the subject ids.
- `id`: computed by each kind's rule after the rest is complete.

A record's `fields` may not carry any of `id`, `source`, `fetched_at`, `raw_ref`, `private`, `author` or `target`. The completed record must then validate against its kind's contract; one that does not is refused, and the refusal is counted in the census.

## Rules an adapter keeps

- **Its output is a function of its input.** It reads no clock, no random number, no environment and no file except through `fetch`. The window and each response's `received_at` are its only time.
- **One record per thing said, shown or counted.** A page is never a post; a thread is many posts.
- **Nothing filled in.** A field the source does not give is left out or written as null where its contract allows.
- **It stops with its state.** An adapter that cannot go on writes no `state` line and exits non-zero, and the run stores nothing.

## Rules a reader may rely on

- **Same input, same bytes.** A run replayed from kept responses with the same `start` line writes byte-identical records.
- **Every record names its response.** `raw_ref` leads to the bytes it was read from.
- **The protocol version is in the manifest.** A change that breaks an adapter is version 2, and mull runs each adapter by the version its manifest names.
