# identifiers

The five identities every join in the garden rests on, and the marker grammar that
carries them between files, per plotplot-ai issue 34. Pinned once, here, so `tend2 ledger`
(tend#154), the grade step (plotplot-ai#33), the prune fold (tend#151) and the ratchet
(plotplot-ai#30) join records by the same keys instead of each inventing a convention.

## session id

The harness's own id for one agent session: an opaque string the harness mints, not a
garden-defined format. Claude Code calls it `session_id` in hook payloads; the friction
profile carries the same value as `gen_ai.conversation.id` (`contracts/friction-profile.md`),
which is the literal attribute name the OpenTelemetry GenAI conventions already provide
and the one the friction profile in this repository uses. (Plotplot-ai issue 34 refers to
this loosely as "the friction profile's `plotplot.session`"; the pinned attribute is
`gen_ai.conversation.id`, and this page is the one place that reconciles the two names.)

Carried by: transcripts (the harness's own field), friction events
(`gen_ai.conversation.id`), receipts (`predicate.sessions[]`), mull's cache and proposals.

Grammar: a non-empty string, no path separators (`/`), no whitespace, no control
characters, under 128 bytes. The garden does not constrain the shape further, because the
harness owns it; a fixture shows the common case (a UUID) without requiring it.

## commit sha

The full git commit sha, never abbreviated in a stored record. An abbreviated sha in a
commit message or a trailer meant for a human to read is fine; an identifier stored for a
machine to join on is always full length.

Carried by: receipts (the in-toto subject's `gitCommit` digest), trailers, reverts, minted
corpus tasks (`origin.sha`), kept artifacts.

Grammar: `^[0-9a-f]{40}$` for a sha1 object database (git's default today), or
`^[0-9a-f]{64}$` for a repository using the sha256 object format. Lowercase hex only. A
consumer that joins on commit sha accepts either length and never truncates one to compare
against the other.

## loop id and check ordinal

`<loop-id>:c<ordinal>`. `<loop-id>` is the loop file's name with its `.tend2.html`
extension removed (`contracts.tend2.html` becomes `contracts`). `<ordinal>` is the check's
1-based position in the loop's `## Tests` list, matching the `c<n>` tend2 already uses in
its own Tried lines (`docs/tend2/FORMAT.md`, tend2's `## Tried` grammar: "attacked c6",
"judged c3").

Carried by: stamps, Tried lines, plan nodes (`node.id` equals the loop id), proposals, the
PR projection.

Grammar: `^[a-z][a-z0-9-]*:c[1-9][0-9]*$`. The loop id is lowercase, starts with a letter,
and may contain digits and hyphens; the ordinal has no leading zero and is never `c0`,
because tend2's checks are numbered from one.

## proposal id

`<source>:<season>:<slug>` (`mull:2026.09:three-seeds`, `recurs:2026.09:no-print`).
`<source>` names the tool that proposed it. `<season>` is the season below. `<slug>` is a
short, lowercase, hyphenated name.

A landed file claims a proposal with one of three markers, matched to where the claim
lives:

| Where | Marker |
|---|---|
| a practices line | a trailing `· from <proposal-id>` |
| a shape (code, config, a rule) | a `# from <proposal-id>` comment |
| a skill or a loop | a `<!-- from <proposal-id> -->` comment |

Carried by: proposals, and the marker itself once a proposal lands.

Grammar for the id: `^[a-z][a-z0-9-]*:[0-9]{4}\.(0[1-9]|1[0-2]):[a-z][a-z0-9-]*$`.
Grammar for the three markers, each capturing the id in one group:

- practices line: `· from ([a-z][a-z0-9-]*:[0-9]{4}\.(0[1-9]|1[0-2]):[a-z][a-z0-9-]*)\s*$`
- shape comment: `^# from ([a-z][a-z0-9-]*:[0-9]{4}\.(0[1-9]|1[0-2]):[a-z][a-z0-9-]*)\s*$`
- skill or loop comment: `^<!-- from ([a-z][a-z0-9-]*:[0-9]{4}\.(0[1-9]|1[0-2]):[a-z][a-z0-9-]*) -->\s*$`

Open, per issue 34: whether the marker should instead be a trailer on the landing commit.
Both may be true at once: the file marker is what the grade step reads at rest, and a
trailer is what a receipt carries; this page does not decide against either.

## season

`garden.lock`'s `season` field, and nothing else. A lever change (a weight, a threshold,
a default) is a change to the lock; the calibration re-run and any season-scoped fold key
on this value alone.

Carried by: `garden.lock` (`contracts/lock.schema.json`), calibration runs, season-scoped
folds, and the season component of a proposal id above.

Grammar: `^[0-9]{4}\.(0[1-9]|1[0-2])$`: four-digit year, dot, two-digit month `01`-`12`.

## fixtures

`contracts/fixtures/identifiers/` holds one fixture file per identity:

- `session-id.txt`
- `commit-sha.txt`
- `loop-check.txt`
- `proposal-marker.txt` (three lines, one per marker form above)
- `season.txt`

`contracts/test/identifiers.test.sh` parses each fixture against the grammar above with a
small parser written in the test itself (POSIX regexes matched with the shell's own
pattern matching and `grep -E`), not a separate library, since each grammar is a single
anchored regular expression and a second file would only be one more place for the two to
drift.
