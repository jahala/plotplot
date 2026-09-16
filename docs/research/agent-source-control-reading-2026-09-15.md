# A reading of a source-control tool for coding agents: what to take, what to leave

Read on 2026-09-15 at the owner's request: a desktop application (Rust, MIT) that runs several
coding agents side by side against one repository, records every session locally, links commits
back to the session that produced them, and feeds every agent one shared memory. The garden's
research names ideas, never vendors, so the tool is not named here; the owner has the link.
Compared against the pinned pleach (411a1eb), the pinned tend2 (a33d5fb), umbel master (501c781)
and the law.

## What it is, and the part the law already excludes

The app, the editor, the semantic memory, the knowledge base and the organisation sync are all
outside the garden by §6 of the law (not an app, not model-extracted memory, not an orchestration
UI). Four of its engineering decisions are inside the garden's remit and sharper than what the
beds do today.

## 1. Redaction before persistence, as one pure library

It scrubs secrets on the way into its store, never on the way out. The redaction library is a
pure function, string in and redacted string out with a tally, six layers (entropy, a corpus of
some three hundred provider formats, provider prefixes, credentialed URIs, connection strings,
credential key/value pairs), no I/O, tested as a flat table, and the capture path fails closed if
scrubbing does not complete. Its stated reason: an agent that reads a `.env` writes it verbatim
into the transcript, so the guarantee holds at the point of writing or not at all, and one code
path is easier to keep than two. It also names over-redaction as a failure with its own tests.

The garden had no redaction anywhere. The contract now exists: `contracts/redaction.md`, one
implementation in the stem as `plotplot redact`, one claim per bed (jahala/plotplot 52; umbel 82,
pleach 107, tend 211). Tier one under the owner's rules.

## 2. No state without a reason and a next step

Its capture health is one of three states per workspace, OK, Degraded or Stopped, and every issue
carries a reason and a next step; its tests assert neither is empty, and a store written by a
newer build is a Stopped state with a reason, not a broken indicator. The garden's equivalents
were pleach's `blockedReason: null`, tend 208, umbel 71 and 73. Taken as the states rule in
`contracts/delivery.md`, held by `contracts/test/verdict.test.sh` (jahala/plotplot 53).

## 3. Every hop has a deadline, and the failure says which hop

Its decision record on bounded start paths is umbel 67 and 73 written down: a start row sat at
twenty-one minutes with no error because every hop could block and none reported. The fix has
four layers, each revertible alone: install is bounded and visible; every handshake hop has a
timeout and the error names the phase, how long it waited and the process's stderr; a stale
connecting entry is cancelled on restart instead of joined; the child runs in its own process
group so a kill takes the whole tree, with logs to a file so a report carries the phase that
stalled. That is the shape umbel 67 took (landed 2026-09-15). The record is also a good template:
it lists each layer's files, constants and pre-change behaviour so a revert needs no archaeology.

## 4. Links that survive history rewrites, or say they are orphaned

It observes commits rather than intercepting them (no hooks, so a commit made before the tool
existed still finds its session), re-points a checkpoint through amend and rebase by patch-id
reconciliation, and orphans on ambiguity instead of guessing. pleach's receipt binds its refs
outside the integrity envelope, bound by git, and umbel's landings are rebase-merged, so every
sha its receipts point at is off master. Taken as the rewrite check on the receipts loop
(jahala/plotplot 54; pleach 105).

## Considered and left

- **A wire protocol as the drive path.** It runs its agents as subprocesses over an agent client
  protocol on a frozen wire and ships no default agents. umbel 77 was the class of fault a wire
  protocol removes and tmux invites. umbel's mandate is the subscription-billed interactive CLI in
  tmux, and whether an adapter keeps subscription billing and a readable transcript is unknown. A
  kill criterion before any work: one worker run on both transports, same task, same account,
  with the bill and the capture compared.
- **Session handoff fact pack.** It assembles a curated fact pack plus the tail of the last session
  for an agent's first message. The garden's version is a person writing a brief and a Tried
  line; tend2's `next` could assemble the same from the loops, but no check asks for it yet.
- **Usage dashboard.** Confirms umbel 78 (empty telemetry) is the prerequisite for any efficiency
  claim.
- **Test stance.** Its crates test the public API against a real temporary directory, a real
  database and a real git repository, with no mocks. Already the garden's rule.
