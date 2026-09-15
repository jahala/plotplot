# What [vendor] has that the garden should take, and what it must not

Read on 2026-09-15 at the owner's request: [vendor], "source control for coding agents"
(Rust and Tauri, MIT, 4.5k stars, pushed the same day). Sources: its README, ARCHITECTURE.md,
ADR-0001 to ADR-0008, and the `[vendor]-checkpoint` and `[vendor]-redact` crates. Compared against
the pinned pleach (411a1eb), the pinned tend2 (a33d5fb), umbel master (501c781) and the law.

## What it is, and the part the law already excludes

[vendor] is a desktop app that runs Claude Code, Codex and any ACP-registry agent side by side,
records every session to a local SQLite store, links commits back to the session that made
them, and feeds every agent one shared memory index plus the notes you wrote. The app, the
editor, the semantic memory, the knowledge base and the organisation sync are all outside the
garden by §6 of the law (not an app, not model-extracted memory, not an orchestration UI).
Four of its engineering decisions are inside the garden's remit and sharper than what the
beds do today.

## 1. Redaction before persistence, as one pure library

[vendor] scrubs secrets on the way into its store, never on the way out. `[vendor]-redact` is a
pure function, string in and redacted string out with a tally, six layers (entropy, a corpus
of about 310 vendor formats, provider prefixes, credentialed URIs, connection strings, and
credential key/value pairs), no I/O, tested as a flat table, and the capture path fails
closed if scrubbing does not complete. Their README states the reason exactly: an agent that
reads a `.env` writes it verbatim into the transcript, so the guarantee holds at the point of
writing or not at all, and one code path is easier to keep than two. It also names
over-redaction as a failure with its own tests.

The garden has no redaction anywhere. umbel writes captures, logs and event files raw; pleach
writes the journal, the run log and the receipt's verbatim handback raw; tend2 transcribes
Tried lines raw; and the umbrella commits receipts. The owner's hard limit says secrets never
reach logs or committed files, and today nothing enforces it. Proposed shape, kept to the
law's composition rule: a contract page `contracts/redaction.md` with a fixture table of
strings in and strings out (including strings that must not be touched), implemented once in
the stem as `plotplot redact` reading stdin, and called by umbel at capture, by pleach at
journal, log and receipt write, and by tend2 at transcription. Each bed's fit script gets one
claim: a planted secret in a worker's output never reaches disk. This is tier one under the
owner's rules, since a committed secret is an irreversible loss.

## 2. No state without a reason and a next step

[vendor] reports capture health as one of three states per workspace, OK, Degraded or Stopped,
and every issue carries a `reason` and a `next_step`; its tests assert that no issue has an
empty next step, and a store written by a newer build is a Stopped state with a reason, not a
broken indicator. The garden's equivalents are pleach's `blockedReason: null` on the umbrella's
first conducted node, tend 208 (an unevaluable check recorded as failed), umbel 71 (nothing in
`ls` tells idle from busy) and umbel 73 (a dead worker leaves nothing to inspect). Proposed
shape: one line in `contracts/delivery.md`, that a conductor's verdict and a runner's status
never report a non-progress state without a reason and a next step, held by the seam test,
which then fails on a null `blockedReason` before any bed moves.

## 3. Every hop has a deadline, and the failure says which hop

ADR-0008 (accepted 2026-09-12) is the closest thing to umbel 67 and 73 written down anywhere:
a start row sat at 21 minutes with no error because every hop on the start path could block
indefinitely and none reported. Their fix has four layers, each revertible alone: install is
bounded and visible; every handshake hop has a timeout and the error is typed
`TimedOut { agent, phase, after, stderr }`; a stale connecting entry is cancelled on restart
instead of joined; and the child runs in its own process group so a kill takes the whole tree,
with logs to a file rather than stderr so a report carries the phase that stalled. That is the
shape 67 should take in umbel: a deadline per phase (spawn, prompt accepted, first output,
idle past threshold), a typed failure naming the phase and carrying the tail of the pane, a
process-group kill, and the pane's last lines written to the capture before the worker is
gone, which is what 73 asks for. The ADR is also a good template: it lists each layer's
files, constants and the pre-change behaviour so a revert needs no archaeology.

## 4. Links that survive history rewrites, or say they are orphaned

[vendor] observes commits rather than intercepting them (no hooks, so a commit made before the
tool existed still finds its session), and re-points a checkpoint through amend and rebase by
patch-id reconciliation; when a squash makes the link ambiguous it orphans instead of
guessing. pleach's receipt keeps its integrity envelope over facts and derived status and
binds `diffRef`, `quarantineBranch` and `quarantineSha` outside the envelope, "bound by git".
umbel's repository forbids merge commits, so its first landing under the garden's method was
rebased and every sha the receipts point at is now off master. Proposed check on the umbrella's
receipts loop, with a fit test: after a rebase or squash of the landed branch, `receipts.sh
verify` resolves each receipt to the commit that carries its patch, by patch-id, or reports
it orphaned, and never resolves it to the wrong commit.

## Considered and left

- **ACP as the drive path.** [vendor] runs Claude Code and Codex over the Agent Client Protocol
  as subprocesses on a frozen wire, and ADR-0002 ships no default agents. umbel 77 (a pasted
  prompt never submitted) is the class of fault a wire protocol removes and tmux invites. The
  law does not forbid a second transport, but umbel's mandate is the subscription-billed
  interactive CLI in tmux, and whether an ACP adapter keeps subscription billing and a
  readable transcript is unknown. A kill criterion before any work: one worker run on both
  transports, same task, same account, with the bill and the capture compared.
- **Session handoff fact pack.** [vendor] assembles a curated fact pack plus the tail of the last
  session for an agent's first message. The garden's version is a person writing a brief and
  a Tried line; tend2's `next` could assemble the same pack from the loops, but no check asks
  for it yet, so it stays an idea.
- **Usage dashboard.** Confirms umbel 78 (empty telemetry) is the prerequisite for any
  efficiency claim; nothing to add beyond its priority.
- **Test stance.** Their crates test the public API against a real temporary directory, a real
  database and a real git repository, with no mocks. That is already the garden's rule.

## Order

1 (redaction) and 2 (reason and next step) are contract changes the umbrella writes first,
each with a seam or fit test. 3 is the shape for umbel 67 and 73, already the next item in
pleach's slot. 4 is a new check on the receipts loop. None is filed as an issue by this memo.
