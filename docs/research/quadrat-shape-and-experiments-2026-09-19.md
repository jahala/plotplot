# quadrat: what it is for, what good code looks like in a history, and what to test next

Written 2026-09-19 by the umbrella agent, at the owner's request, after the first two measures
of the quadrat experiment (hotness and spread) were dropped on training. It reads the field's
analyzers and the research on them from memory; the papers are named so they can be checked
before anything here is quoted outside this repository. Products are described, never named.

## The short version

1. The experiment so far tests one use (rank files for repair) with one unit (the file) and one
   outcome (was it repaired later). Two things the research is clear about are missing from
   it: the strongest simple predictor (what was repaired before gets repaired again) and an
   effort-aware reading (a reviewer reads lines, not files). Add both before judging anything.
2. An agent needs quadrat at two other moments, and the facts we extracted can test both:
   before a change (what moves together with the thing I am about to touch) and after a change
   (what did this change do that history says rarely sticks). Each gets its own kill criterion
   and its own simple baseline, registered before the held-out projects are read.
3. Classic code metrics (complexity, maintainability index, cohesion scores) are mostly size in
   disguise and flag the same files for ever. They enter quadrat only as change over time
   (drift), never as a threshold on the code as it stands.

## How today's analyzers work, and what is known about them

- **Size and complexity counts.** Lines, cyclomatic complexity (McCabe 1976), cognitive
  complexity, nesting, the maintainability index. They are cheap and nearly all of their
  predictive power is size: once size is controlled they add little (El Emam and others 2001,
  "The confounding effect of class size"; Gil and Lalouche 2017). They are also stagnant: a
  large complex file is flagged in every release, so people stop reading the list (Rahman and
  Devanbu 2013, "How, and why, process metrics are better").
- **Design metrics and smells.** Coupling and cohesion suites (Chidamber and Kemerer 1994),
  god class, long method, feature envy. In a controlled study of real maintenance work, smells
  explained little effort once file size and number of changes were known (Sjøberg and others
  2013).
- **Clone detectors.** Most copies are harmless. The harmful ones are copies that are changed
  inconsistently (Juergens and others 2009, "Do code clones matter?").
- **History and process measures.** Churn and relative churn (Nagappan and Ball 2005), recent
  change, number of authors and minor contributors (Bird and others 2011), scattered changes
  (Hassan 2009, change entropy), co-change (Gall 1998; Zimmermann and others 2004; D'Ambros
  and others 2009), and prior repairs: a file repaired recently is repaired again, which as a
  plain ranking did as well as far heavier models (Kim and others 2007; Rahman and others
  2011). Process measures beat code measures repeatedly (Moser and others 2008).
- **Behavioural hotspot tools.** Commercial tools of this family rank change frequency times
  complexity. That is our baseline, churn times size, under another name, and our tables agree
  with their claim: it finds a later-repaired file in about six of ten picks.
- **Architecture from history.** Error-prone files cluster around a few patterns: an interface
  many files depend on that keeps changing with them, files that change together with no
  structural dependency between them, cycles (Xiao, Cai and Kazman 2014; Mo and others 2015,
  "Hotspot patterns").
- **Change-level prediction.** Whether one change will need repair is predicted by its size,
  how scattered it is, whether it is itself a fix, and the history of what it touches (Kamei
  and others 2013, just-in-time quality assurance).
- **Effort-aware evaluation.** When the cost of inspection is counted in lines, many models are
  no better than reading small files first, and fine-grained findings gain (Arisholm and
  others 2010; Mende and Koschke 2010).
- **Model-based reviewers.** They read code for meaning (names, error handling, intent). That
  is a different kind of evidence. quadrat can tell them where to read.

## What long-lived, maintainable code looks like in its own history

Stated as things a deterministic reader of history can observe:

1. **Changes stay local.** A typical change touches few files in one part. Files that change
   together live together, so the co-change groups agree with the directory structure. This is
   what separation of concerns and vertical slicing look like from outside.
2. **Foundations are stable.** Code that much else calls changes rarely, and when it changes
   its callers are not repaired afterwards. The reverse, widely used and often repaired, is
   the most expensive pattern there is.
3. **Each piece of knowledge has one home.** No similar bodies in different files that have to
   be edited in the same commits.
4. **Units stay bounded.** Functions and files do not ratchet upward in size, branching and
   nesting through many small additions by many hands.
5. **Changes stick.** What is written is not reworked, reverted or repaired within the next
   few commits, and a repair does not set off repairs next door.
6. **Tests move with the code.** Source changes arrive with changes to their tests, and the
   tests themselves are not what keeps being repaired.
7. **The code is tended.** Some share of changes are moves, extractions and renames. A
   repository where that share falls to nothing is accreting.

Not observable this way, and left to readers of the code: whether names are good, whether
errors are handled, whether an abstraction is the right one.

## What quadrat is for

The memory of what the code has been doing to the people and agents who change it, delivered
at three moments, always as an observation with its evidence, never as a gate:

- **Before a change** (context when an agent opens a file): what usually changes with this,
  whether this place has been repaired before, how much depends on it.
- **After a change** (a hook on the finished change, advisory): what this change did that
  history says rarely sticks. A partner file that always moves with these and is missing. A
  near-copy of something that exists. A change scattered across many parts. A large step up
  in a function's branching. Source touched and its tests not.
- **Periodically** (scan): where repair effort goes. This is the only use the experiment tests
  today.

## Experiments to run on the benchmark we have

All of them are pure functions over the facts already extracted (commits, files, functions
followed by identity, restorations, rapid reworks, bulk edits, authors, times, file classes,
callers and imports at cutoffs, declared fixes), on the nine training histories first.

### Fix the ruler first

- **R1. The prior-repair baseline.** Past corrective events per file and per function,
  weighted toward the recent. If this simple ranking is the strongest bar, every measure has
  to beat it, and the tool should show it plainly.
- **R2. A function-level outcome.** A declared fix names its files, and the facts name the
  functions each commit touched, so "this function was touched by a fix, restored or reworked"
  exists for every history. Function measures are then judged on functions, where the
  baselines are weak today (they name a restored function in about one pick of ten).
- **R3. An effort-aware reading.** The same lists under a budget of lines to read beside the
  budget of five files. Printed beside the registered reading, never replacing it.

### The scan face: candidates beyond the four

- **S1. Hotness, second reading.** Churn beyond what functions of that size have in the same
  repository, weighted toward the recent, judged on R2.
- **S2. Co-change hubs.** Files with many strong co-change partners outside their own
  directory (the unstable-interface and implicit-dependency patterns). Uses lift.
- **S3. The inward split.** A file whose functions fall into groups that never change
  together has been maintained as several things under one name.
- **S4. Near-copies that change together.** Similar bodies in different files changed in the
  same commits. Needs one new capability, a body fingerprint index.
- **S5. Fragile foundations and fix ripple.** Callers times repair rate; and a repair followed
  within a few commits by repairs in its partners.
- **S6. Drift**, as registered, and a second reading that looks for steps and for accretion.
- **S7. The test gap.** Source changed repeatedly while its test partner stood still.

### The diff face: is this change likely to stick?

- **D1. Change survival.** Outcome per commit: the code it wrote is reworked, restored or
  repaired within the next few commits that touch the same functions. The facts already
  classify rapid reworks and restorations. Predictors, each an observation a hook could print:
  scatter, a missing co-change partner, touching a previously repaired function or a fragile
  foundation, introducing a near-copy, a step in branching, source without its tests, growth
  of an already large function. Baseline: the size of the change. Criterion: at a budget of
  the top twentieth of changes, the observations name more non-surviving changes than size
  does, on held-out histories.

### The context face: what moves with this?

- **C1. Co-change completion.** For each later commit with several files, hide one and ask
  whether the past predicts it from the others. Baselines: files in the same directory, files
  joined by an import. Criterion: better recall in three suggestions than both, on held-out.
- **C2. The omission alert.** When the past says a partner is missing from a change, how often
  is that partner changed within the next few commits anyway? That rate is the alert's truth.

### Over time, descriptive only

Share of changes that are refactors, rework rate, growth of co-changing near-copies, how local
changes are, how well co-change groups agree with the directory structure. Read across all
the histories against their repair share.

## Order, and what waits

Rulers first (R1 to R3), since every later number is read against them. Then C1 and C2, which
need only lift and are the most direct help to an agent. Then D1. Then S2, S3, S1 and S6. Then
S4, S5 and S7, which need a fingerprint index, callers over time, or test pairing.

The held-out projects have not been read. They should stay unread until the three criteria
(scan, change survival, co-change completion) are registered and their candidates are chosen
on training; then they are read once for all three.

## Dropped or parked, with the reason

- Thresholds on complexity, the maintainability index, cohesion scores as findings: the
  research says they are size in disguise and stagnant. They survive only as drift.
- Questions about names, comments, error handling, input validation: they need a reader of
  meaning. Not quadrat's evidence; quadrat can rank where such a reader spends its budget.
- Pricing a change by agents trading on it: its own author says there is no oracle yet.
- Ownership and minor contributors: parked, not dropped. Cheap from the facts, but its
  meaning is unclear where agents write most commits; worth one table once agent trailers are
  read.
- Reading a coverage report, agent trailers, survival analysis of time to next repair, stable
  finding codes, the inconclusive band: capabilities and presentation for the tool, not
  measures to test now.
