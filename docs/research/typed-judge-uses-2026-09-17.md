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

## Ranked by value, and what our records can test today

| rank | use | bed | testable now from records |
|---:|---|---|---|
| 1 | check-line lint at shape time | tend2 | yes: does the claim text alone predict a second attempt, a failed or a blocked node (209 verdicts) |
| 2 | pre-gate at handback | pleach, the stem's hook bundle | partly: mutants and claim-to-test alignment; fully only log-only, going forward |
| 3 | cheap-first cascade and casting | pleach, tend2 cast | no: needs exploration runs on cheaper workers |
| 4 | plan-time overlap and needs | pleach, tend2 emit-plan | yes: file overlap between node diffs is ground truth (174 diffs) |
| 5 | consistency notes: claim and test, Tried and diff, docs and code | weeder scan, tend2 gate | partly: made negatives by swapping pairs |
| 6 | failure and handback classification | pleach, umbel | thin: few labelled failures; record first |
| 7 | issue folding across repositories | tend2 | thin: this week's folds as seeds |
| 8 | prose and voice lint | petals, harness hook | needs a labelled set of the owner's accepted and rejected text |
| 9 | bite scheduling | weeder | when bite exists |
| 10 | mid-flight watch | umbel | research-grade |

Kill criteria travel with each. For rank 1: if claim-quality scores do not separate troubled
nodes from clean ones better than claim length does, the lint is decoration. For rank 3: if a
hundred explored nodes show no cheaper model passing first time at a useful rate, the cascade
is closed and the casting ledger stays a report. For rank 4: if predicted overlap does not
beat a rule on shared path words, the rule wins.
