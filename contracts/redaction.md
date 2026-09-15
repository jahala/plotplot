# redaction: a secret never reaches disk, because scrubbing runs at the point of writing

Status: contract, 2026-09-15 (jahala/plotplot 52). One page and one fixture table that the stem
implements once and every bed calls before it writes agent-produced text anywhere.

## Why at the point of writing

A worker that reads a `.env` file puts its contents verbatim into its output, and a shell
command that prints a token puts it in the tool result. umbel captures that output, pleach
journals it and seals the handback into a receipt, tend2 transcribes it into a Tried line, and
the umbrella commits receipts and loops. The owner's hard limit is that a secret never reaches a
log or a committed file. That holds only if scrubbing happens before persistence, on the one
path that writes, so every reader downstream (verify, ledger, export, a pull request) inherits
the guarantee and there is one code path to keep rather than one per bed.

## The face

`plotplot redact` reads all of standard input, writes the redacted text to standard output, and
exits 0. With `--tally` it also writes one JSON object to standard error:
`{"total": n, "by": {"entropy": n, "format": n, "prefix": n, "uri": n, "connection": n, "keyvalue": n}}`.
Every replaced span becomes the literal `[REDACTED]`. If redaction cannot complete (input that
is not UTF-8, an internal error) the face writes nothing to standard output and exits 1: the
caller then fails closed and does not write. Exit 2 is a usage error. The face touches no
network, reads no file but its input, and logs nothing about the content it processed.

Callers pass one write's worth of text per invocation, a capture chunk, a journal line, a
handback, a Tried line, so a multi-line secret (a PEM block) is seen whole. Splitting a text at
arbitrary byte boundaries before redacting is a caller's defect.

## The six layers

A span is redacted if any layer flags it. Each catches a class the others miss:

| layer | catches | why the others miss it |
|---|---|---|
| entropy | random-looking tokens of 32 characters or more with high Shannon entropy | the only layer that works on a format nobody has a rule for |
| format | known provider formats, a vendored corpus in the stem | entropy too low, or alphabet too narrow, to clear the threshold |
| prefix | `sk-`, `sk_live_`, `ghp_`, `xox`, `npm_`, `AIza`, `SG.`, `sb_secret_`, `AKIA` and the like, with a plausible tail | corpus rules are length-pinned and miss a key quoted alone |
| uri | `scheme://user:password@host`: the password in the userinfo | a memorable password has low entropy and matches no format |
| connection | JDBC, keyword and semicolon connection strings: the password field | the credential is structural, not a token |
| keyvalue | `KEY=value`, `key: value`, `"key": "value"` where the key names a credential (password, secret, token, api key, auth) | the key is the only evidence; the value has no shape |

## Over-redaction is a failure too

The garden is full of strings that look like secrets and are not: git commit shas, sha256
digests in locks and receipts, UUIDs, version pins, `{fromEnv:NAME}` and `${NAME}` references,
prose that names a password without holding one, and the `[REDACTED]` token itself. The fixture
table holds one row for each, and a row that changes is a defect of the same weight as a
leaked secret: a lock whose digests are scrubbed verifies nothing.

## The fixture table

`contracts/fixtures/redaction/cases.jsonl`, one JSON object per line: `id`, `in`, `out`,
`count` (the tally's total the row expects), and `layer` (which layer must catch it, or `none`).
`in` and `out` are stored base64-encoded. A fixture that looks like a secret is treated as one
by every scanner, and the platform's push protection refused the plain table on its first push
(Slack, Stripe, SendGrid and Supabase shapes), which is the right behaviour and the reason the
rows are encoded at rest rather than allowed through. `contracts/test/redaction.test.sh` decodes
each `in`, feeds it to the face, and compares standard output to the decoded `out` byte for
byte and the tally to `count`; it prints row ids, never values. The values are fabricated; none
is a live credential, and a fixture that ever held one would be replaced, not rotated.

## What each bed proves

One claim per bed's fit script, verified by the umbrella: a planted secret in a worker's output
never reaches disk. umbel: capture, read, logs and the opencode plugin's event and log files
(jahala/umbel 82). pleach: the journal, the run log, the receipt's handback and quarantine notes
(jahala/pleach 107). tend2: Tried lines and the ledger (jahala/tend 211). A bed calls the face
through the planted stem; a bed that reimplements the layers has drifted.
