# Requests for the owner's answer, and the answers

The contract between every tool that needs the owner's yes, gate (`jahala/gate`), which keeps the
requests and their answers, and hedge (`jahala/hedge`), which refuses an act until gate holds a yes
bound to that exact act (`docs/decisions/2026-10-26-layers.md`, rule 8). The rule they share, the
exact bytes a digest is computed over and when a yes holds, lives in the vector
`contracts/fixtures/request/vector.json`, which every implementation passes. Schema:
`contracts/request.schema.json`. Fixtures: `contracts/fixtures/request/`. Test:
`contracts/test/request.test.sh`. First drafted by gate.

## Files

| File | Holds |
|---|---|
| `requests/<digest>.json` | one request, written once when it is first asked |
| `answers/<digest>.json` | the answer to that request, written once |

Both live in the owner's gate folder, `~/.local/share/plotplot/owner/gate/` by default. `<digest>`
is the request's digest. Anything else in the folder is gate's own and not part of this contract.

## A request

| Field | Required | Bound | Meaning |
|---|---|---|---|
| `format` | yes | yes | `gate-request/1`; a change of meaning is a new version, never an edit |
| `asker` | yes | yes | the tool that asks, by name |
| `ask` | yes | yes | what is asked, in one sentence the owner can answer |
| `why` | yes | yes | why the owner's answer is needed |
| `evidence` | yes | yes | what the owner should look at, in order; may be empty |
| `options` | yes | yes | the answers that say go; never `no`, which every request may be answered |
| `act` | no | yes | the exact act a yes allows, below |
| `asked_at` | yes | no | when gate first recorded the request |
| `expires_at` | yes | no | the moment after which the answer is no |
| `waiting` | no | no | what waits on the answer, one item each; it ranks the request |

A request with an act has the options `["yes"]` and nothing else. Moments are UTC, ISO 8601, ending
in `Z`. Text is valid Unicode and is never normalized.

**Bound** fields are what the owner says yes to. A request that differs in any of them is another
request with another digest, and no answer of the first holds for it. The unbound fields say when
and how urgently, not what: asking the same question again with a later deadline is the same
request, and gets the answer already recorded. An asker that needs a fresh answer after a no asks a
different question, for example by naming in `why` or `evidence` what has changed.

## The act

| Field | Meaning |
|---|---|
| `kind` | `command`, the only kind in version 1 |
| `argv` | the command's words as the program receives them: quotes removed, nothing left to expand |
| `cwd` | the folder it runs in: absolute, with no empty, `.` or `..` segment and no trailing slash |

hedge reads a shell command into simple commands and the folders each may run in; an act matches
when one of them has exactly these words and runs in exactly this folder.

## The digest

1. Take the bound fields the request holds, and only those.
2. Write them as JSON with no whitespace, every object's keys sorted, arrays in their order. Each
   string is written between quotes, with `"` as `\"`, `\` as `\\`, U+0008 as `\b`, U+0009 as `\t`,
   U+000A as `\n`, U+000C as `\f`, U+000D as `\r`, every other character below U+0020 as `\u00`
   and two lowercase hex digits, and every other character as itself. This is RFC 8785, the JSON
   Canonicalization Scheme, for documents that hold only objects, arrays and strings, which is all a
   bound field may hold. Every key is lowercase ASCII, so sorting by bytes and by UTF-16 agree.
3. Encode the text as UTF-8. Its sha256, in lowercase hex, is the digest.

The act digest is the same over the act object alone. The vector's `digests` give, for each case,
the request as text, the exact bytes and the digest; `same` and `differ` give pairs whose digests
must match or must differ, `differ` naming the bound field that changed.

## An answer

| Field | Meaning |
|---|---|
| `format` | `gate-answer/1` |
| `request` | the digest of the request it answers |
| `answer` | one of the request's options, or `no` |
| `by` | `owner`, or `expiry` when gate answered because the request expired unanswered |
| `at` | when the answer was recorded, UTC |

An answer by expiry is always `no`, recorded at or after the request's `expires_at`. gate records an
owner's answer only before the expiry.

## When a yes holds

A yes holds for a request when its answer file names the request's digest, says `yes`, is `by` the
owner, and its `at` is before the request's `expires_at`, compared as instants, never as text.
Nothing else holds: no answer, even before the expiry; a no; an answer by expiry; a yes recorded at
or after the expiry. A request past its expiry with no answer recorded does not hold, and the next
writer that sees it records the no.

A yes holds for an act when it holds for a request whose act has the same canonical bytes. The
vector's `holds` and `holds_for_act` give the cases.

## Writing

Each file is written whole and once: to a temporary name in the same folder, then linked to its
name, which fails when the name exists. So a reader never sees half a file, two writers of one
request agree, and of two answers to one request the first wins and the second is refused. Nothing
is ever rewritten. No lock is needed.

## What a reader may rely on

- **Silence never acts.** Only the owner's yes, given in time, holds.
- **A yes is bound to exact text.** One word of the command, one folder, one line of evidence
  changed is another request.
- **Asked once.** A request already answered is answered from its record.

## Lifetime

Requests answered by the owner are kept with their answers: they are the proof an act was approved.
A request that expired unanswered is kept a year after its expiry, then gate compacts it to one
line and moves its two files to the Trash.

## Open questions

- Whether a yes for an act is spent by its first use, or holds for a window, and who records the
  use. hedge would keep the use in its own state, since gate's folder is gate's to write.
- Whether the owner can grant a standing yes for a class of acts, with an expiry, and how hedge
  reads it.
