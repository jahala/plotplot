# A typed judge, measured on the garden's own records

Read on 2026-09-17, after `typed-judge-2026-09-17.md` placed a calibrated, non-generative judge
in the garden by argument. This note places it by measurement. Every number comes from the
garden's own records: pleach's receipts, weeder's calibration ledger, the maps' Tried lines.
The scripts, packets and answers name the vendor, so they stay with the owner, outside the
public repositories.

## What was measured

Four tracks and one probe. Each names its truth. Every label is either a record a person or a
verifier wrote, or certain by construction.

1. **The audit seat.** 296 cases from 174 pleach receipts across five beds. For each done node
   three cases: its own contiguous commits paired with its own claim, the check line on the loop
   page (done); the red-phase commit alone, kept only when it touches test paths (not done);
   the full change paired with a neighbouring check's claim from the same page (not done).
   Four quarantined nodes with their quarantine commit (not done). Questions in one call: does
   the diff implement the claim; is the change tests only; does it weaken a test; completeness
   on four described levels; a state among done, partial and wrong.
2. **Weeder's rulings.** The 20 blocked commits a person classified in the calibration ledger
   (`docs/calibration/judgements.toml`), each with the rule's definition, the finding as weeder
   states it, and the real diff at the paths the finding names. Questions: is the finding's
   claim literally true of this change, which is the audited question under the ruling of
   2026-09-06; did the change really weaken something; the ledger's three-class verdict.
3. **Routing.** 64 issues each cited by exactly one loop page's Tried line on its map, offered
   every loop of that map as a choice, each loop described by its title and goal paragraph.
4. **A probe.** The seven distinct weakening hunks weeder's recall audit planted (a loosened
   pin, a broadened throw, a swallowed error, a forbidden import), each with its reversal.

## Results

| track | n | headline | Brier | cost |
|---|---:|---|---:|---:|
| audit seat, accuracy at 0.5 | 296 | 0.90 | 0.099 | $0.09 |
| audit seat, abstaining in 0.4 to 0.6 | 256 judged | 0.93, abstained 0.14 | | |
| weeder rulings, claim literally true | 20 | 0.95, three of four false positives caught | 0.099 | $0.005 |
| weeder rulings, really weakened | 20 | 0.50 | 0.246 | |
| routing, top-1 on a bed's own map | 32 | 0.74 to 0.78 among 27 to 43 loops | | $0.01 |
| routing, top-1 on the umbrella map | 22 | 0.09 among 13 loops | | |
| probe, planted shapes and reversals | 14 | 1.00 | 0.039 | |

Audit seat by kind: own claim 0.91 (Brier 0.067), red snapshot 0.94, swapped claim 0.85. By
bed: pleach 0.90 on 145, tend 0.89 on 74, umbel 0.89 on 54, weeder 0.88 on 17. Calibration of
the implements answer: predicted 0.0 to 0.2 was done 0 times in 53; predicted 0.8 to 1.0 was
done 96 times in 100; the middle bands sat at 0.38 and 0.56. Latency two seconds per call at
seven thousand tokens. Everything in this note, three discarded runs included, cost under a
dollar.

## What the misses say

- **Audit.** Of 128 done nodes, 11 were called not done, and 6 of those are end-to-end or
  documentation nodes whose deliverable is a test or a page. The question said tests alone do
  not implement a claim, and for those claims that is false. A claim whose deliverable is a
  test needs a question written for it. Swapped claims within one loop are the hardest
  negative: neighbouring checks share code, and the change often does part of the neighbour.
  The state question called 59 of those 100 "partial", which is the right word.
- **Weeder.** The judge reads the literal claim well and the person's "acceptable" badly.
  Acceptable is a reading of intent: a feature taken out together with its tests, a move the
  rule cannot follow. That reading is the person's and stays so.
- **Routing.** The umbrella's stem page cites bed issues it tripped over while building the
  stem. The judge routed those to the bed's own fit loop, which is where a person would file
  them. The label was the citing page, not the loop that owns the fault. On a bed's map a
  citation means ownership, and three in four land.

## Three packet faults, and the recipe that survived

The first batch scored 0.38 and measured its own builder: diffs taken from the merge base with
the trunk, a hundred commits long, cut in the middle; claim text missing in 51 of 65 packets;
verdict labels borrowed from a later state of the same node. Two more followed: a commit range
that reached an older run of a reused node name, and whole-loop nodes whose claim is every
check on the page, not the first. The recipe that survived: the receipt is the case record
(status, page, checks, diff ref, handback); the node's own contiguous commits are the diff;
files that change more than six hundred lines are named and left out; negatives are made, not
found, because the records hold almost no judged-not-done work with a diff. That recipe belongs
in the calibration ledger's plan, since any judge the ledger grades needs the same set.

## Where it fits, now by measurement

1. **The audit seat, as one vote.** Calibrated on 296 past nodes, right nine times in ten,
   93 in 100 when it abstains in the middle band, at three hundredths of a cent per node. Of
   pleach 117's three conditions two are met: calibrated, and under a tenth of the cost. The
   third, at least as right as the generative auditor, is unmeasured, because that auditor
   only ever saw done nodes and stamped them all. One run of the same 296 packets through the
   generative seat settles it.
2. **Weeder's calibration, as the blind second reader.** The audited question is the one it
   answers well. A typed reader beside the person on every blocked commit, its agreement
   written into `metric.json`'s regrade table, at a hundredth of the generative blind auditor's
   cost. Never the "acceptable" call, never a block.
3. **Routing on a bed's map, as a suggestion.** Top three with probabilities, for an issue
   that has no loop yet. On the umbrella map only once the label is the loop that closed the
   issue, which tend2 can record.
4. **Advisory shapes.** The probe says direction and shape are read reliably in small hunks.
   A warn-only rule for shapes weeder's parsers cannot reach is possible; a metric needs a
   labelled set weeder does not yet hold.

Where it does not fit is confirmed rather than changed: blocking, stamping, judging intent,
anything needing prose.

## The tools to build now, in order, each with its kill criterion

1. **One face, one page.** `plotplot judge` in the stem: a packet file and a questions page
   in, typed answers with probabilities out as JSON; endpoint and model in configuration, the
   key by Keychain or environment reference, never on a command line. Every use below goes
   through it, and one contract page names the packet. Kill: a second consumer that cannot use
   the same packet shape means the face is a wrapper, not a contract.
2. **pleach: `accept.audit.provider: typed`, log-only.** Answers into the receipt's audit
   packet beside the generative seat's, then the comparison run above. Gating comes only after
   that, with the abstention band, the verifier primary. Kill: worse than the generative seat
   on the same 296, or one planted instruction in a handback that flips an answer in a fixture.
3. **weeder: the second reader in `cargo xtask calibrate`.** The claim-true question on every
   blocked commit, agreement in `metric.json`. Kill: agreement below the blind generative
   auditor's across two ledger seasons.
4. **tend2: a suggested loop in `ingest`.** Top three on the bed's map for an issue without a
   loop. Kill: a season in which the suggestion is taken less than half the time.
5. **Records before judges.** For handback states under the delivery contract and for a
   pane that waits on a person, the records hold no negatives. pleach and umbel should keep
   the packet and the eventual outcome, so a set exists before a judge is asked. No tool yet.
6. **The ledger's first gradable judge.** Probabilities make a Brier score possible, which a
   prose judge never gave the ledger. That is plotplot 2's design paying out, not new code.

## The claim check, simulated on merged pull requests

Added the same day. The tool this list points to beyond the garden is a claim check on a pull
request: each check line the pull request claims, judged against its diff, probabilities
posted, never a block. Our own history can play it back. A landing is a first-parent commit
that carries a pull request number. A closed check is a (code) check the landing stamped whose
evidence file the pull request touched, so the work is in that pull request. A negative is a
check still open on the page at the landing (10 cases), or a check that another pull request
of the same repository closed, not stamped at this landing, none of whose named files this
pull request touched (167 cases): the overclaim the tool exists to catch. That makes 366 cases
from 60 pull requests in four repositories, at most six closed checks per pull request. The
loop pages stay out of the diff, since the stamp would leak. The files a claim names come
first, whole files only, up to 60k characters; 254 of the 366 cases had files left out. umbel
gave no cases: it squashes, and its stamps and evidence arrive in separate landings.

| | the diff alone | with the pull request's title and body |
|---|---:|---:|
| accuracy at 0.5 | 0.92 | 0.94 |
| Brier | 0.076 | 0.052 |
| flag below 0.4: verified checks falsely flagged | 9 of 189 | 10 of 189 |
| flag below 0.4: overclaims caught | 154 of 177 | 172 of 177 |
| flag below 0.2: verified checks falsely flagged | 0 of 189 | 4 of 189 |
| flag below 0.2: overclaims caught | 85 of 177 | 146 of 177 |
| cost per pull request | $0.004 | $0.004 |

On the diff alone, a predicted probability under 0.2 was a closed check 0 times in 85, and one
over 0.8 was a closed check 130 times in 130. By repository: pleach 0.98, weeder 0.92, tend
0.90, the umbrella 0.80 on 20 cases. Where the whole diff fit the window, closed checks scored
0.96; where files were left out, 0.89.

- **As an advisory it works.** Flagging below 0.4, one verified check in twenty draws a false
  flag and seven overclaims in eight are caught, for four tenths of a cent per pull request.
  Flagging below 0.2 it never cried wolf in 189 and still caught half.
- **The description sways it.** With the title and body it catches more, because here the
  description honestly describes other work. A pull request that overclaims would say so in its
  description too. The honest estimate for the tool is the diff alone, and the description
  serves only to find which checks are claimed. This is the fit note's caution, that state is
  data and never instruction, measured: in weeder, whose descriptions do not list every check,
  accuracy on closed checks fell from 0.92 to 0.78 once the description was added.
- **Large pull requests cost seven points, not the tool.** Ordering the diff by the files a
  claim names holds up, and it fixes the tool's shape: one call per claimed check, never one
  per pull request.
- **Kill criterion.** A season on a real repository in which the reader dismisses more than
  half the flags raised below 0.4.

## What the owner does

Rotate the generative auditor's key before the comparison run; it appeared in a process
listing. Decide where the vendor-named packets and answers live: with the owner, or in a
private repository.
