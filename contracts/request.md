# Requests for the owner's answer, and the answers

The contract between every tool that needs the owner's yes and hedge (`jahala/hedge`), which keeps
the requests and their answers and refuses an act until a yes bound to that exact act holds
(`docs/decisions/2026-10-26-layers.md`, rule 8; `docs/decisions/2026-11-05-gate-folds-into-hedge.md`).
This is version 2. An owner's answer holds only when it carries a signature from a key the owner
installed, and a request may offer real choices. The rules every implementation shares live in the
vector `contracts/fixtures/request/vector.json`: the exact bytes of a digest and of a signature,
the signatures that verify, the records hedge accepts, and when a yes holds. hedge implements this
page and pins the vector byte for byte; pollen is the first asker. Schema:
`contracts/request.schema.json`. Fixtures: `contracts/fixtures/request/`, written by
`contracts/test/request-vector.mjs`. Test: `contracts/test/request.test.sh`. First drafted by gate,
the queue's own tool until 2026-11-05; the format ids keep its name.

## Version 2 replaces version 1

The formats are `gate-request/2` and `gate-answer/2`. From the release that ships them, hedge
writes and accepts only version 2. Version 1 files stay in the folder as records. A version 1
answer never holds again, since any program could have written it. An asker that asked in version 1
asks again in version 2, and since `format` is bound, that is a new request with a new digest.

What changed:

- An option is an object: an id, a label, and what the asker does if it is chosen.
- A request may let the owner answer in their own words, and may carry what follows a no, a
  recommended option, the project it serves, facts and related rulings.
- An answer holds the choice or the words, an optional reason, and a signature.
- An owner's answer holds only when its signature verifies against a key the owner installed with
  sudo. Writing a file is no longer enough.
- A yes is spent by its first use, and lapses when its request expires.

## Files

| File | Holds |
|---|---|
| `requests/<digest>.json` | one request, written once when it is first asked |
| `answers/<digest>.json` | the answer to that request |
| `~/.config/hedge/owner-keys.json` | the owner's installed keys, owned by root |

The first two live in the owner's requests folder, `~/.local/share/plotplot/owner/requests/` by
default. `<digest>` is the request's digest. Anything else in that folder, such as hedge's record
of each use, is hedge's own and not part of this contract.

## A request

| Field | Required | Bound | Meaning |
|---|---|---|---|
| `format` | yes | yes | `gate-request/2`; a change of meaning is a new version, never an edit |
| `asker` | yes | yes | the tool that asks, by name |
| `ask` | yes | yes | what is asked, in one sentence the owner can answer |
| `why` | yes | yes | why the owner's answer is needed |
| `evidence` | yes | yes | what the owner should look at, in order; may be empty |
| `options` | yes | yes | one or more answers the owner may choose, below |
| `other` | no | yes | when present, the owner may answer in their own words; it says what the asker does with them |
| `no` | no | yes | what the asker does after a no |
| `pick` | no | yes | the id of the option the asker recommends |
| `for` | no | yes | the project the ask serves, by name |
| `facts` | no | yes | `{name, text}` pairs the owner reads before answering, for example "Most it can cost" and "1,000 kr, set aside before the first click" |
| `rulings` | no | yes | the owner's earlier rulings the asker found related, below |
| `act` | no | yes | the exact act a yes allows, below |
| `asked_at` | yes | no | when hedge first recorded the request |
| `expires_at` | yes | no | the moment after which the answer is no |
| `waiting` | no | no | what waits on the answer, one item each; it ranks the request |

Moments are UTC, ISO 8601, ending in `Z`. Text is valid Unicode and is never normalized. No field
holds anything but strings, arrays and objects.

**Bound** fields are what the owner says yes to. A request that differs in any of them is another
request with another digest, and no answer of the first holds for it. The unbound fields say when
and how urgently, not what: asking the same question again with a later deadline is the same
request, and gets the answer already recorded. An asker that needs a fresh answer after a no asks a
different question, for example by naming in `why` or `evidence` what has changed.

## Options

| Field | Meaning |
|---|---|
| `id` | lowercase ASCII, `^[a-z][a-z0-9-]*$`, unique in the request, and never `no` |
| `label` | what the owner reads in the list |
| `does` | what the asker does if the owner chooses it |

Every request may be answered `no`, so no option takes that id. The options are shown in their
order, and the order is bound. `pick`, when present, names one of the options.

A request with an act has exactly one option, with the id `yes`, and a `pick` on it can only be
`yes`. It may still have `other`, but an answer in the owner's words never makes the act hold.

Each item of `rulings` is `{text, source, relation}`. `text` is the ruling as the owner gave it.
`source` says where it comes from, as free text, for example "compass, 2026-11-04". `relation` is
`holds` when the ask agrees with the ruling, or `clashes` when it goes against it.

## The act

| Field | Meaning |
|---|---|
| `kind` | `command`, the only kind in version 2 |
| `argv` | the command's words as the program receives them: quotes removed, nothing left to expand |
| `cwd` | the folder it runs in: absolute, with no empty, `.` or `..` segment and no trailing slash |

hedge reads a shell command into simple commands and the folders each may run in. An act matches
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
the request as text, the exact bytes and the digest. `same` and `differ` give pairs whose digests
must match or must differ, `differ` naming the bound field that changed.

## An answer

| Field | Required | Meaning |
|---|---|---|
| `format` | yes | `gate-answer/2` |
| `request` | yes | the digest of the request it answers |
| `by` | yes | `owner`, or `expiry` when hedge answered because the request expired unanswered |
| `at` | yes | when the answer was given, UTC |
| `choice` | one of these two | an option's id, or `no` |
| `other` | one of these two | the owner's own words, only when the request has `other` |
| `reason` | no | the owner's reason, in their words |
| `signature` | when `by` is `owner` | `{alg, key, value}`, below |

An answer holds a `choice` or the owner's words in `other`, never both. An answer by expiry is
`"choice": "no"`, given at or after the request's `expires_at`, with no signature and no reason.
It needs no signature, because a no never acts.

hedge accepts an answer for a request only when it names the request's digest, its `choice` is one
of the request's option ids or `no`, and it has `other` only if the request does. An owner's answer
must also be given before `expires_at`, and its signature must verify. An answer by expiry dated
before `expires_at` is no answer. The vector's `accepts` cases give these rules, with the request
rules a schema cannot say: option ids are unique, and `pick` names an option.

## The signature

The owner's key is made in the Mac's Secure Enclave. It cannot leave the Mac, and it signs only
after the owner's fingerprint or login password. How hedge asks for that is hedge's own.

- **Bytes signed.** The answer object without its `signature` field, written by step 2 of the
  digest rule, encoded as UTF-8. Every other field of the answer is in those bytes, including
  `format`, `request` and `at`. So a signature binds one answer to one request at one moment. It
  cannot be moved to another request, another answer, or another kind of signed file.
- **Algorithm.** `alg` is `ecdsa-p256-sha256`: ECDSA on the P-256 curve over the SHA-256 of those
  bytes. This is what the Secure Enclave gives through `SecKeyCreateSignature` with
  `ecdsaSignatureMessageX962SHA256`.
- **Encoding.** `value` is the signature in X9.62 DER form, as that call returns it, in base64
  (RFC 4648 section 4, the standard alphabet, with padding).
- **Key id.** `key` is the SHA-256, in lowercase hex, of the public key as the 65-byte uncompressed
  point: the byte `04`, then X, then Y. That is the form `SecKeyCopyExternalRepresentation` gives
  for a P-256 public key.
- **Either half.** ECDSA signatures are random, and the signer does not normalize `s`. A verifier
  accepts `s` from either half of the group order. The vector has one of each.

The vector's `test_keys` are three software keys made from fixed text, with their private scalars
published so anyone can check the signatures. They sign nothing real, and no keys file outside a
test may list them. Its `signatures` cases each give an answer without its signature, its exact
bytes, the key id, the public point and the signature, and whether it verifies against the vector's
`keys`. Each failing case fails for one reason only: the signature was made over other bytes, the
key is not installed, or the answer is dated after its key retired.

## The installed keys

`~/.config/hedge/owner-keys.json` holds the public half of each key the owner has made:

```json
{"format": "hedge-owner-keys/1", "keys": [{"id": "…", "point": "…", "added": "…", "retired": "…"}]}
```

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | the key id, as above |
| `point` | yes | the 65-byte point, in standard base64 with padding |
| `added` | yes | when the owner installed it, UTC |
| `retired` | no | when the owner replaced it, UTC |

The owner installs the file with sudo, in the same step that installs hedge's policy, and root owns
it. hedge trusts the file only when root owns it and no one but root can write it. When the file is
missing, unreadable, or fails that check, no owner's answer verifies, and so no yes holds.

A replaced key stays in the list with `retired` set, so the answers it signed earlier still verify.
hedge's own tests verify with a software key they install under their own home folder. The
installed hedge reads only the root-owned file, and an agent has no way to point it elsewhere.

**Verifying.** An owner's answer verifies when its `key` is in the trusted list, its `at` is before
that key's `retired` if the key has one, and its signature checks over its bytes against the key's
point. Moments are compared as instants. Verifying needs no enclave, so tests and CI verify in
plain code.

## When a yes holds

A yes holds for a request at a moment *now* when all of these are true:

- the answer names the request's digest, and both are version 2;
- it is `by` the owner, with `choice` `yes`;
- its `at` is before the request's `expires_at`;
- *now* is before the request's `expires_at`;
- its signature verifies;
- hedge has recorded no use of it.

Moments are compared as instants, never as text. Nothing else holds: no answer, even before the
expiry; a no; another option; the owner's own words; an answer by expiry; a yes given at or after
the expiry; a yes checked at or after the expiry; a yes already used.

A yes holds for an act when it holds for a request whose act has the same canonical bytes. The
first run spends it: `hedge holds` records the use, and from then on the yes no longer holds. An
asker that needs to run the act again asks again, with its `why` saying so, which makes it another
request.

A file in `answers/` that fails these checks is no answer. `hedge list` shows it as a forged answer.

The vector's `holds`, `holds_for_act` and `peek` cases give these rules. Each holds case gives the
request, the answer, the moment of any use hedge recorded, and *now*.

**A known limit.** The record of a use is a file in hedge's folder, and an agent's code can delete
it. Then the yes holds again, and the act the owner approved can run a second time. Signing does not
close this, because nothing the owner holds is present when the yes is checked. The expiry bounds
it: a yes never holds at or after its request's `expires_at`. `hedge list` shows a yes whose record
of use has disappeared.

## `hedge holds --peek <digest>`

It answers whether the owner's yes holds and is unused, and writes nothing. When it holds, it exits
0 and prints `{"ok": true, "holds": true, "why": "..."}`. Otherwise it exits 1 and prints
`{"ok": true, "holds": false, "why": "..."}`, where `why` says what failed, for example that the
owner has not answered, that the yes was used at a given moment, or that the signature does not
verify.

`hedge holds <digest>`, without `--peek`, answers the same way and spends a yes that holds. So
`--peek` twice on an unused yes holds both times and records no use. A `hedge holds` after it passes
once, and a `--peek` after that says the yes is spent.

## Who writes what

- The signer app holds the key and shows the owner the question. hedge builds the bytes to sign and
  writes the answer file, so the canonical rule has one home. The app hands hedge the choice or the
  words, the reason and the signature. hedge refuses to write an answer whose signature does not
  verify.
- `hedge answer`, in the owner's terminal, asks the signer app to sign, and macOS asks for the same
  fingerprint or password.
- An asker reads the answer with `hedge ask`. It returns the choice, the words and the reason only
  when the signature verifies.

## Writing

Each file is written whole: to a temporary name in the same folder, then linked to its name, which
fails when the name exists. So a reader never sees half a file, two writers of one request agree,
and of two answers to one request the first wins and the second is refused. No lock is needed.

Nothing is ever rewritten, with one exception. A file in `answers/` that is no answer, such as a
forgery or an answer by expiry dated before the expiry, cannot block the owner. When the owner
answers, hedge moves that file to the Trash and writes the owner's answer in its place.

## What a reader may rely on

- **Silence never acts.** Only the owner's yes, given in time, holds.
- **Only the owner's key makes a yes.** A file any program can write is no answer until its
  signature verifies.
- **A yes is bound to exact text.** One word of the command, one folder, one line of evidence, one
  option changed is another request.
- **A yes acts once,** and never after its request expires.
- **Asked once.** A request already answered is answered from its record.

## Lifetime

Requests answered by the owner are kept with their answers and the record of their use: they are
the proof an act was approved. A request that expired unanswered is kept a year after its expiry.
Then hedge compacts it to one line and moves its files to the Trash. A file that is no answer goes
to the Trash when the owner answers. A retired key stays in the keys file as long as any answer it
signed is kept.

## Open questions

- Whether the owner can grant a standing yes for a class of acts, with an expiry, and how hedge
  reads it.
