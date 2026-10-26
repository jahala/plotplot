# Adapter manifest: what one adapter reads, how it runs, and on what terms

The contract every mull adapter's manifest follows. mull owns its content. It sits beside the record contracts: a record says what one person said, what a page showed or what a number was, and the manifest says where it was read, how, and under what terms. mull reads the manifest before it runs the adapter (`contracts/protocol.md`), and readers use it to decide which records may count: glean scores a need only on sources that reach its people, and reports an audience no source reaches as a gap.

Schema: `contracts/adapter.schema.json`. Fixtures: `contracts/fixtures/adapter/`, a stream adapter (`valid.json`) and a generic forum adapter (`valid-forum.json`). Test: `test/contracts.test.ts`.

## The shape

One JSON object per adapter, kept beside the adapter's program.

| Field | Required | Meaning |
|---|---|---|
| `adapter` | yes | the adapter's id; every record it writes carries it as `source` |
| `version` | yes | the adapter's version; a change to how it reads a response is a new major version |
| `protocol` | yes | the protocol version it speaks, `"1"` |
| `run` | yes | the program and its arguments, run from the adapter's folder, never through a shell |
| `source` | yes | the source's own name and home address |
| `kind` | yes | `discover`, `expand` or `stream` |
| `emits` | yes | the record kinds it writes: `post`, `document`, `observation`, `subject`, `reaction`; empty for discover |
| `reaches` | yes | `hosts` it may fetch from, whether it reads `files`, for a generic adapter `config_hosts`, the configuration property naming more hosts, and for an adapter that reads files `config_paths`, the configuration property naming the files or folders it may read |
| `config` | yes, may be null | for a generic adapter, the JSON Schema its capture plan configuration must satisfy |
| `channel` | for expand and stream | how it fills a record's `channel`, in a sentence |
| `identity_space` | for expand and stream | where an author's handle names one person: the platform's account space as its host name, such as `github.com` for every GitHub repository, or `channel` when each channel has its own people; mull hashes authors over it |
| `audiences` | yes | the groups of people the source reaches, in words a compass would use |
| `carries` | yes | the kinds of signal the source carries (below) |
| `terms` | yes | the source's terms as read, and the adapter's verdict (below) |
| `terms_per_site` | no | true when each configured site has its own terms, which its capture plan entry then carries |
| `access` | yes | what reading the source needs (below) |
| `private` | yes | whether the source is a private stream the owner consented to |
| `split` | for expand and stream | for each kind it emits, where each field is read from (below) |

### Kinds

- **discover** finds places and returns addresses. It writes no record, so it has no `split`, no `channel` and an empty `emits`. Search is discovery and never evidence.
- **expand** reads one address into records, one post per person's post.
- **stream** reads a channel over a window, with a cursor.

One adapter has one kind. A source read two ways gets two adapters.

### What a source carries

`carries` takes the signal kinds of the umbrella's plan for reading the world (`jahala/plotplot:docs/plans/reading-the-world.md`): `wanting`, `paying`, `building`, `harm`, `stranded`, `movement` and `own`. A new kind is a change to this contract. The word is `carries` because a post's `signals` already means the numbers a source gives about one post.

### Terms

What the source's terms say, as read, as fact. A manifest never decides whether a use is allowed: mull serves many owners, and each owner rules for their own use, in their capture plan (`contracts/capture-plan.md`).

- `url` and `read_on`: where the terms were read, and the day.
- `licence`: the licence of what people post, as the terms name it, or the terms' own words when they name none.
- `attribution`: whether a reader that quotes a record must attribute it, and how.
- `commercial_use`: `allowed`, `forbidden` or `not-stated`.
- `rate_limit`: requests per span of seconds, `stated_by` the `source` or, when it states none, the `adapter`, which then keeps its own. mull holds this limit as a budget per span, whatever any ruling says.
- `restrictions`: each clause that restricts reading the source by program or using what is read, quoted, with where it was read. Empty when the terms restrict nothing.
- `deletion`: whether the source requires that what was read be deleted when its author deletes it or after a time, how soon, and how. mull complies, whatever any ruling says.

A generic adapter that reads many sites (`terms_per_site: true`) states the software's own limits here, and each capture plan entry carries the site's terms in the same shape.

### Access

`access` replaces the earlier `key`. It says what reading the source needs, so access that is not ready reaches the owner as a request instead of failing a run.

| Field | Meaning |
|---|---|
| `method` | `none`, `api-key`, `bearer`, `oauth2`, `basic`, `session-login`, `signed-request` or `export-file` |
| `credentials` | each secret by its Keychain name and whether it is `refreshed`; empty for `none`, at least one for a method that sends a credential. mull reads them at the moment of the call; the adapter never sees them |
| `account` | whose account the calls run as; null for `none` |
| `automation` | whether that account's terms allow a program to use it: `allowed`, `forbidden` or `not-stated` |
| `obtain` | `how` access is obtained, whether it needs `approval`, its `cost`, and its `status`: `ready`, `requested`, `not-started` or `refused` |
| `scopes` | the `least` scopes that do the job, and whether they are `read_only` |
| `expiry` | when access ends and how it renews; null when it does not expire |
| `human_step` | a step a program must not take alone: two-factor, a captcha, a login the terms forbid automating; null when there is none |
| `paid` | true when each call costs money |
| `cost` | when paid | what each call costs as the source states it, with `read_on` and `url`: a flat `per_request_usd`, or for a source priced by tokens or size an `estimate` saying how the cost is computed; well settles the real charge |

Calls that cost money or need a credential go through well (`jahala/well`), the garden's one door. mull sends each such request under `well run`, which reserves the estimate from `cost` against the capture plan entry's `budget` before sending, holds the owner's cap, reads the credential from the Keychain into the one sending process, settles the real charge, and never resends a call that may already have been charged. mull and the adapter never hold a secret.

One rule is held by the schema whatever any ruling says: a `human_step` makes the method `export-file`, so the owner exports the data and the adapter reads the file. mull never works around a captcha, two-factor or a login it was not given. An account whose terms forbid automation is a restriction like any other, ruled on in the capture plan. well, when it exists, owns each secret's life: storage, reading at call time, refresh, and a warning before expiry (`jahala/plotplot:docs/ideas/well.md`).

### Split

For each kind the adapter emits, `split` names each field the adapter writes and where it is read from in a response, in a sentence a person can check against a recorded response. An adapter whose source says what an account is writes `author_kind` (`person` or `bot`) and names where it reads it in `split`. Field names are those of the protocol's `record` lines (`contracts/protocol.md`): `handle` for the author, `parent_address` for the parent. A field the source does not give says so. The keys of `split` are exactly the kinds in `emits`.

## Rules a reader may rely on

- **One source per adapter.** An adapter's records all carry its `adapter` id as `source`.
- **The manifest is read before the source.** No adapter reads a host its manifest does not name, and no capture plan entry runs on a source with restrictions unless its owner's ruling allows it.
- **Same version, same records.** The same response under the same major version gives the same records.
- **Audiences are claims to check.** An audience names whom the source reaches by who posts there, not whom the reader hopes to hear.
