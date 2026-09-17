# What a cheap, fast, calibrated judge makes possible across the garden

Written 2026-09-17 at the owner's request, after `typed-judge-2026-09-17.md` (the fit, by
argument) and `typed-judge-measured-2026-09-17.md` (the fit, by measurement). This note goes
wide: every place in the tooling where a judgment is needed that code cannot make, and that a
generative model made too slow or too dear to ask often. It names the research each idea
stands on, what our records could test today, and what would prove the idea wrong.

## What changed

A semantic judgment now costs three hundredths of a cent and takes one to two seconds. It comes
back typed, as a probability, a choice or a score, and the probabilities are calibrated. It
writes no prose, reads questions literally, cannot count, and can be swayed by text in its
state. Three things follow that were not practical before.

- **Ask everywhere.** A judgment can run at every event and on every item, where a generative
  judge was kept for the few gates that could afford minutes and dollars.
- **Decide with the number.** A calibrated probability supports a threshold, an abstention band,
  an expected-cost rule and a cascade: the cheap judge first, the dear one only when it
  abstains. This is the selective-prediction and learning-to-defer line of work (Geifman and
  El-Yaniv 2017; Madras, Pitassi and Zemel 2018; Mozannar and Sontag 2020) and the model
  cascades of Chen, Zaharia and Zou (2023). Conformal prediction (Angelopoulos and Bates 2021)
  turns a few hundred labelled cases into a threshold with a stated error rate, which is a
  better way to choose the flag line than reading a table.
- **Compose answers.** Pairwise yes or no answers build a ranking, a clustering or a dependency
  graph. Typed answers feed a table or a bandit without parsing prose.

## The classes of work this opens

Seven kinds of judgment recur across the beds. None can be done by a parser, and each was
priced out by a generative model.

1. **The spec, before any code.** A check line is short text, clean state, and the cheapest
   place to catch a fault. Requirements research has long named the smells: ambiguity,
   several behaviours in one line, implementation named in place of behaviour, no observable
   outcome (Femmer and others 2017; the EARS patterns of Mavin and others 2009). The heuristics
   of that work were brittle, and a generative reviewer was too slow for an open pen. At two
   seconds, `tend2 serve` can lint a check as it is written and `tend2 lint` can carry a
   semantic pass: is this verifiable, is it one claim, does it repeat a check elsewhere on the
   map, do the checks together cover the goal paragraph.
2. **A prior for a dear oracle.** Where the truth is deterministic and expensive, the judge
   orders the work and the oracle still decides. Predictive test selection chose tests per diff
   with a learned model (Machalica and others 2019); predictive mutation testing guessed which
   mutants survive without running them (Zhang and others 2016). Here: which tests a worker
   should run first after an edit, which mutants weeder's bite should execute, which nodes the
   generative audit should read. The gate always runs the whole oracle; only the order changes.
3. **The cheap-first cascade for workers.** This is the owner's question about model selection,
   and it is the largest money lever in the garden. Verifier-guided sampling is well studied:
   many cheap attempts plus a good verifier can match one expensive attempt, and the verifier is
   the bottleneck (Cobbe and others 2021; Brown and others 2024). A two second pre-gate makes a
   cheap worker's failed attempt nearly free, so pleach can try the cheap model first and
   escalate with the attempt in hand. The casting ledger then becomes a contextual bandit
   (Li and others 2010): the judge supplies features of the claim at plan time, such as
   cross-cutting, concurrency, end-to-end evidence, and the ledger learns which features need
   which model. Honest limit: our records hold one worker model for nearly every node, so there
   is nothing to learn from yet. Exploration comes first, a random tenth of nodes on a cheaper
   model.
4. **Plan-time structure.** Pairwise questions over the claims of a plan: does this node need
   that one, will these two touch the same files, is this node too large for one worker. A
   wrong `needs` edge or two parallel nodes in one file cost a merge and a retry. The pairs are
   quadratic, which is what made this unaffordable, and a plan of twenty nodes is 190 questions
   and under a cent.
5. **Consistency between two texts.** The claim and the test that proves it. The Tried line and
   the diff. The commit message and the diff. A documentation paragraph and the code it
   describes after a change. Comment and code drift is a research field of its own with trained
   classifiers (Wen and others 2019; Panthaplackel and others 2021); a general judge does it
   without training. Each is a warn-level SARIF note from `weeder scan` or `tend2 gate`, never
   a block.
6. **The long tail of text at the seams.** A failed gate's output: infrastructure, environment
   or a real failure. A provider's error in a pane. A handback: done, partial, blocked, with or
   without a reason. An inbound pollen message: for whom, how urgent. Patterns catch the known
   cases and the tail has been falling through.
7. **Exhaustive pairs over a population.** Issues across eight repositories: is this the same
   fault. The owner folded these by hand this week. Three hundred open issues are 45 thousand
   pairs and about a dollar. Entity matching with language models is recent work (Peeters and
   Bizer 2023); the calibrated band decides which pairs a person sees.

Two further uses are real and need more care.

- **Watching a worker mid-flight.** Is it still on the claim, is it thrashing, is it about to
  ask a person. Process supervision beats outcome supervision in the research (Lightman and
  others 2023), and a per-step check is affordable now. The state is a transcript, which is
  the most hostile text we hold, so this is advisory and research-grade.
- **Prose rules.** The owner's writing rules are mostly semantic: false suspense, manufactured
  stakes, significance narration, teacher voice. A regular expression cannot see them and a
  generative reviewer was too slow per paragraph. A battery of yes or no questions per
  paragraph is a linter for voice, for petals and for every agent's reply.

## What it still cannot do

Write, explain or give a reason. Count or compare dates. Judge intent, which the weeder ledger
showed is the person's (0.50 on "was this fine"). Stand alone against hostile text. Block or
stamp, by the law. Every use above is advisory or an ordering, so its absence costs nothing:
there is one such model today, and no gate may depend on it.

## Tested the same day, and what it did to the ranking

Two of the ideas above could be tested from our records at once, each against the kill
criterion written before the run. A third test, on context, is in the measured note.

- **Plan-time overlap: passes.** 178 pairs of done nodes on 23 loops; truth is whether the two
  nodes' own diffs share a changed source file. From the two claims alone the judge reaches an
  area under the curve of 0.76, against 0.55 for the rule on shared paths and identifiers, and
  0.80 against 0.67 when tests and documents count as files. The probabilities are calibrated:
  pairs predicted under 0.2 never overlapped, 0.2 to 0.4 overlapped 38 times in 100, 0.4 to 0.6
  overlapped 62, and 0.6 to 0.8 overlapped 82. Cost, half a cent. A conductor can use that
  number as it stands: run the likely-overlapping nodes one after the other.
- **Check-line lint as a forecast: fails.** 98 done nodes, 13 of which needed a second attempt
  or failed or blocked first. Eight answers about the claim text (one behaviour, observable,
  ambiguous, prescribes implementation, several modules, concurrency, end to end, a difficulty
  score) and a direct guess at a second attempt all sit between 0.36 and 0.58, which is chance
  at this size. Claim length does no better. The direct guess correlates with worker minutes
  at minus 0.28. Two readings, both honest: thirteen troubled nodes is a small set, and one
  careful author wrote every claim, so their quality barely varies. Either way our records do
  not support the idea, and the same result closes the casting half of the cascade: features of
  a claim do not forecast how hard a node will be.

The pattern across every test so far: the judge is good at reading what is in front of it, two
texts and whether they match (a diff and a claim 0.90, a finding and a change 0.95, two claims
and their files 0.76), and it cannot forecast an outcome (trouble, difficulty) or judge intent
(0.50). Uses that compare stay; uses that predict go.

## Ranked by value, after the tests

| rank | use | bed | standing |
|---:|---|---|---|
| 1 | pre-gate at handback | pleach, the stem's hook bundle | supported by the audit seat and claim check runs; the real test is log-only, going forward |
| 2 | plan-time overlap, for scheduling | pleach, tend2 emit-plan | passed on 178 pairs |
| 3 | cheap-first worker cascade | pleach | the pre-gate half stands; casting by claim features is closed; needs exploration runs on cheaper workers |
| 4 | consistency notes: claim and test, Tried and diff, docs and code | weeder scan, tend2 gate | a comparison, so likely; untested |
| 5 | issue folding across repositories | tend2 | a comparison; this week's folds as seeds |
| 6 | failure and handback classification | pleach, umbel | few labelled failures; record first |
| 7 | prose and voice lint | petals, harness hook | needs the owner's accepted and rejected text |
| 8 | bite scheduling | weeder | when bite exists |
| 9 | check-line lint at shape time | tend2 | no support in our records; reopen only with claims from several authors |
| 10 | mid-flight watch | umbel | research-grade |

Kill criteria still open: for the cascade, a hundred explored nodes with no cheaper model
passing first time at a useful rate closes it. For the pre-gate, a hundred nodes with no drop
in second attempts or in worker minutes per verified claim, or more than half the nudges
changing nothing, closes it.

## Beyond the beds: a comparator is a query engine

Asked the same evening: what is this kind of model for, at root, and what tool does the garden
lack that it makes possible. The tests answer the first half. It is a comparator: given two
texts it says, with a calibrated probability, whether one matches, supports, repeats or
collides with the other. Computer science has a long list of things to build once a comparator
is nearly free. The crowd-powered databases of a decade ago worked out sorts, joins, top-k and
group-by over a slow, costly, noisy human oracle (Franklin and others 2011; Marcus and others
2011), and the noisy-comparison literature gives the algorithms (Karp and Kleinberg 2007 for
search; Jamieson and Nowak 2011 for ranking). Recent work names the same operators over tables
with a language model as the oracle (Patel and others 2024). The garden writes a large record,
receipts, journals, Tried lines, issues, ledgers, session transcripts, and has no way to ask a
question of it. Four tools follow, none of which a bed holds today.

1. **Receipts for sentences.** The owner reads an agent's last message and little else. Each
   factual sentence in that message can be checked against the session's own tool results:
   supported, unsupported, contradicted. "Tests pass" with no test run in the transcript is the
   failure the garden exists to prevent, moved from code to prose. This is attribution and
   summary-faithfulness checking (Laban and others 2022; Bohnet and others 2022), applied to a
   coding agent's report for the first time we know of. Twenty sentences against the five most
   similar tool results each is a hundred short calls, a few cents and ten seconds, as a
   log-only stop hook in the stem's bundle. A generative checker costs a second agent run.
2. **A ledger for directives.** Every rule in an instruction file costs tokens in every session,
   and nobody measures whether it is followed. A yes or no question per rule and per agent
   message, over all transcripts, gives each rule a compliance rate beside its token cost. A
   rule always followed without being stated can go; a rule often broken should become a hook.
   Rules with a mechanical truth, such as a banned character, calibrate the judge before the
   semantic rules are read. Instruction-following research scores models on verifiable rules
   (Zhou and others 2023); this scores the rules themselves, which is the garden's cost per
   correct answer turned on its own instructions.
3. **Deja vu.** Before a worker starts, compare its stated approach with every Tried line in
   every repository: was this tried, and did it fail. Similarity search finds text that looks
   alike; the question here is a relation, tried and failed, which needs a judge per candidate
   and was never affordable across a whole history. The same relation keeps the memory
   directory and the documents honest: does a new entry repeat or contradict an old one
   (contradiction detection, de Marneffe and others 2008).
4. **A collision radar for parallel agents.** Text conflicts are git's; two changes that merge
   cleanly and break each other are the multi-agent problem. Awareness tools for human teams
   found these by building every pair of branches (Sarma and others 2003; Brun and others
   2011), which cost too much to run often. The overlap test passed on claims alone; the next
   step scores pairs of open diffs, and pollen carries the warning between the two agents.

Two smaller ones use the same comparator: a culprit finder that ranks recent landings by how
likely each caused a new fault, so a bisect tests the likeliest first (the SZZ line of work,
Sliwerski and others 2005), and grading the judge's own questions without labels from how they
agree with each other (Ratner and others 2017), which matters because our records are short of
negatives. One caution governs the first two: session transcripts are private text, and sending
them to any outside model is the owner's decision, not a default.

## The four tools, tested on real records the same night

Each tool was played back on records the garden already holds, with a truth that comes from a
record or from construction, and nothing new left the machine: the handbacks and diffs had
been judged earlier in the day, and the issues are public.

| tool | real data | result |
|---|---|---|
| receipts for sentences | 47 worker handbacks, each sentence against its own node's diff and against another node's | AUC 0.95; flagging below 0.3 marks 6 of 144 genuine sentences and catches 98 of 144 foreign ones |
| a ledger for directives | the same handbacks against the work order's rules | agrees with a regex 0.93 on a structural rule and 0.70 on finding a character; three semantic rules measured at 30, 30 and 18 of 30 |
| deja vu | 188 issues of the last month, the owner's hand-made folds as the true pairs | AUC 0.89 against 0.70 for text similarity; median rank of a true pair 141 against 275, among 3079 |
| collision radar | 356 ordered pairs of nodes; truth from the diffs, B uses a name A exports | AUC 0.77 from the two claims alone, against 0.48 for a shared identifier; the right direction in 26 of 38; pairs under 0.2 were dependent once in 95 |

What each run taught beyond its number:

- **Sentences.** A three-way answer matters: supported, not shown, contradicted. The lowest
  genuine sentences were true statements about things outside the evidence ("landed with the
  sibling nodes"), which is "not shown" and no accusation. Read by eye, the fifteen lowest held
  no real overclaim and one judge error at the line. Our history has honest workers, so the
  tool's worth is in the catch rate on foreign sentences and in a false-flag rate low enough
  to live with.
- **Directives.** Mechanical rules stay with code: the judge is weak on characters and no
  better than a regex on structure. The ledger still found things no judge was needed for: 17
  of 47 handbacks lack the Tried line the work order demands, all 13 from one bed and none of
  19 from another, and those reports end with the owner's global closing line, which is two
  directives competing. It also found a rule that does not fit: "what was rejected and why" is
  unmet in 12 of 30 Tried lines, every one a final-phase report where nothing was tried. A
  directive ledger has to store the directives in force beside each output.
- **Deja vu.** The owner's folds were two relations. Feature fan-outs and seam pairs scored
  0.74 to 0.86; a batch of unrelated fixes grouped into one loop scored 0.12 to 0.43, rightly,
  since nothing in the text joins them. The twenty best unfolded pairs were nearly all real:
  an umbrella contract issue and its bed's side, a scratch-collection cluster across two beds,
  and one true repeat, the same verifier fault filed a day after its first report was closed.
- **Collision.** The signal is strongest at the low end, which is what a scheduler needs:
  pairs it calls independent are.

## Before any run spends calls

Two of tonight's runs first measured a fault in their own design, and a third did in the
morning. The check that now goes first costs seconds:

1. The truth's base rate sits away from zero and one, with at least ten cases in each class.
   One label came out true for 95 pairs in 100 because a pattern matched everyday local names.
2. One packet of each class is read by eye.
3. The evidence is in the frame the text was written in. Handback sentences about "the working
   tree" meant the changes since the red-phase commit; against the whole node the judge called
   them contradicted, correctly, and the AUC was 0.87 where the right frame gives 0.95.
4. Where a negative is made by construction, count how often it could be true anyway.
