# gate, the owner's queue

2026-10-26. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner whose factory runs mostly alone and still needs their yes for a few things: the first
deploy to a domain, a campaign, spending above a line, anything public in their name, a question
only they can answer. Today those requests arrive in chat sessions, mixed with everything else,
and a yes given once leaves no record that a check could read.

## What it owns, and what it does not

gate owns requests for the owner and their answers. It does not decide what needs a yes (the
compass does, and hedge enforces it), carry messages between agents (mycelium does), or act on
an answer (the asking tool does).

## How it works

1. **Ask.** Any tool writes a request as a file: what is asked, why, who asks, the evidence, the
   options, the expiry, and what happens on expiry. On expiry the answer is always no, so silence
   never acts.
2. **Rank and cap.** Open requests are ranked by what waits on them, and capped, so the owner
   never faces a wall. A question already answered is never asked again; its answer is reused.
3. **Deliver.** Delivery adapters at the edge bring each request to the owner: a notification on
   the phone for urgent ones, a daily digest for the rest, the terminal for a session that is
   open.
4. **Answer.** The owner answers by command or by reply. The answer is recorded with the time and
   bound to a digest of the exact request, the way tend2 binds a stamp to a claim.
5. **Use.** hedge lets an act through only when gate holds a yes whose digest matches that act.
   A yes for one campaign cannot be spent on another.

Sources: the gardener (most), hedge (an act it blocked), graft and cultivate (their questions),
pollen, transplant and harvest (before they publish, deploy or send), transplant (alerts).
tend2's needs-you list is read as one more source, so tend2 stays standalone.

## What code decides

The request's shape, its expiry, the default no, the digest binding, the cap and rank, and
whether a yes matches an act. A person decides only the answer.

## Alone, and with the garden

Alone it is a file-based approval queue for any script that must wait for a person. With hedge,
it is what turns "needs the owner's yes" from a sentence into a check.

## Data and its lifetime

Open and answered requests are files in the portfolio repository. Answered requests are kept,
because they are the proof an act was approved. Requests that expired unanswered are kept a year,
then compacted to one line each.

## Metric and kill criterion

Metric: the share of requests answered before they expire, and the number the owner faces per
day.

Kill criterion: if, over product one, the owner answers more often through something else than
through gate, gate is the wrong surface and its records move into whatever the owner uses.

## Open questions

- Which delivery adapter comes first; the owner's phone is the likely one.
- Whether the owner can grant a standing yes for a class of act, with an expiry, and how hedge
  reads it.
