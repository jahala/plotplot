# Typed perception: what fifty years of stalled ideas were waiting for

Written 2026-09-17 at the owner's request, as the wide end of the series that starts with
`typed-judge-2026-09-17.md`. The measured notes say what the model does on our records: it
compares two texts well, it forecasts nothing, it cannot read intent. This note asks what that
capability is at root, which old problems it unblocks, what it offers each role in software
work, and where the new approach lies. It is an argument, not a measurement; the tests it
proposes are at the end.

## What the thing is

A function from any text and a question in plain language to a typed value with a calibrated
uncertainty, in a second, for a few hundredths of a cent, with no prose coming back. In
programming terms it is a predicate whose body is written in natural language and whose
argument is the unstructured world. The boundary between messy reality and code becomes a typed
function call. A generative model can answer the same questions, a thousand times dearer, ten
times slower, differently each time, and in prose that code must parse. That gap is the
difference between asking at a few gates and asking inside every loop.

Dual-process psychology has names for the two halves (Stanovich and West 2000; Kahneman 2011):
a slow deliberate system and a fast perceptual one. Generative models gave software the slow
system. This is the fast one.

## The old ideas that lacked exactly this

Again and again since the 1970s the reasoning machinery was worked out and the project stalled
for want of a cheap, reliable way to perceive meaning.

- **Expert systems.** Production rules fire on conditions (Forgy 1979). The medical systems of
  the 1970s even carried certainty factors (Shortliffe and Buchanan 1975). They died of two
  bottlenecks: acquiring the rules (Feigenbaum 1977) and evaluating the conditions against a
  messy world, which a person had to do by typing answers. A generative model now writes the
  rules; a typed judge evaluates the conditions, with certainty factors that are calibrated.
- **Fuzzy control.** Membership functions over numeric sensors ran trains and washing machines
  (Zadeh 1965; Mamdani 1974). Membership functions over meaning, "the customer is angry 0.83",
  let the same controllers act on semantic state.
- **Mixed-initiative interfaces.** Horvitz (1999) gave the decision theory for when software
  should act, ask or stay quiet: expected utility under the probability of the user's goal. The
  office assistant built on it failed for crude evidence, not for wrong theory. Calibrated
  probabilities of intent are the missing input, and "when should an agent interrupt its owner"
  is the same problem.
- **Context-aware computing.** Weiser (1991) and the context toolkits (Dey and others 2001)
  needed a trained model per context. Studies of trigger-action programming found people want
  triggers no sensor offers: when I am asleep, when nobody is home (Ur and others 2014). A home
  that reacts to what is going on is a rule engine with semantic triggers and asymmetric
  thresholds: dim the lights at 0.6, unlock a door at 0.99.
- **Management cybernetics.** Beer (1972) designed organisations as control loops with alarm
  signals that escalate when a level cannot cope, and built one on telex data in the early
  1970s. It could sense only numbers. Statistical process control (Shewhart 1931) has the same
  limit. Text is the exhaust of all knowledge work, and now it can be sensed.
- **The parser problem.** Text adventures since the late 1970s mapped open language onto a
  closed set of authored actions and players fought "guess the verb" (Lebling and others 1979).
  A choice over authored actions with a confidence is that parser, and the same shape serves
  any product with a finite command set: say what you want, the product picks the command or
  asks when unsure.
- **Requirements traceability.** Which requirement does this code, test or document satisfy.
  Retrieval-based recovery never reached usable precision (Antoniol and others 2002;
  Cleland-Huang and others 2014), so regulated industries maintain the matrix by hand at great
  cost. Judging a claim against an artefact is our strongest measured skill (0.90 to 0.95), and
  ten million candidate links cost a few hundred dollars.
- **The oracle problem.** Test automation can generate inputs without end and cannot say
  whether an output is right (Barr and others 2015). For textual behaviour a typed judge is a
  cheap pseudo-oracle: fuzz the product, judge every state against expectations written in
  plain language.
- **Content analysis.** Coding text against a codebook with measured reliability is a manual
  craft (Krippendorff 1980). A codebook is a set of typed questions with criteria.

## Calibration makes it an instrument

One property deserves its own heading. The sum of calibrated probabilities is an unbiased
estimate of a count, with a variance that can be computed. Prevalence estimation is a distinct
task from classification (Forman 2008; Bella and others 2010), and it means any question can be
measured over any pile of text with an honest error bar and no labels: what share of this
month's tickets are about onboarding, what share of our tests assert nothing, what share of
agent reports contain an unsupported sentence. Approximate query processing gave databases
answers with error bounds (Agarwal and others 2013); this gives them over text, which has been
second-class since Codd (1970). A classifier that is merely accurate cannot do this. An
instrument can be put on a control chart.

## By role

| role | what becomes possible |
|---|---|
| developer | assertions and property tests over meaning in CI and at runtime (Meyer 1992; Claessen and Hughes 2000); is this change breaking, per pull request, where semantic versioning has always leaned on judgment (Raemaekers and others 2014); names and comments that lie about the code (Host and Ostvold 2009); which hunks deserve a reviewer's eyes |
| agent | a fast path: gate every file read, tool output and memory by relevance before spending context; check each step against the task; decide when to ask; all at a thousandth of a deliberate step |
| repository | a semantic index kept fresh on every commit, so agents navigate typed facts and read less; health as measured rates: test theatre, false docstrings, drifted documents |
| project | a traceability matrix that maintains itself; issues folded across repositories; collisions between parallel workers seen before the merge |
| manager | process mining of unstructured work: label each step of a transcript or thread from a small taxonomy and the methods of van der Aalst (2011) apply, showing where rework loops; control charts on semantic rates; attention by exception. No forecasting: it senses the present |
| product | continuous content analysis of feedback against a codebook, with prevalence and error bars; every requirement linked to the work that satisfies it |
| designer | copy, tone and clarity linted on every string and every locale; heuristic evaluation over the accessibility tree: does this screen say what to do next |
| tester | pseudo-oracles at fuzzing scale; metamorphic relations over meaning (Chen and others 1998) |
| operations, security | every log line and alert triaged; session protocols checked; a second reader for commands a parser cannot classify, never the only barrier |

## The novel approach: compile deliberation into typed productions

Every decision an agent makes today is interpreted at run time by a large generative model,
the recurring ones included. Cognitive architectures solved this forty years ago: deliberate
problem solving is slow, so its results are compiled into fast productions, and deliberation
returns only at an impasse (Anderson 1982; Laird, Rosenbloom and Newell 1986; Newell 1990). The
two halves now exist as services.

1. The generative model meets a judgment and deliberates. It also writes the judgment down as a
   typed production: a question with criteria, a threshold, an action.
2. The production is backtested on the record, with the deliberate decisions and the later
   outcomes as labels, and scored with a proper scoring rule. The wording is revised against
   that score, which is training without touching a model.
3. Above a bar it is promoted and runs at a thousandth of the cost. Inside its abstention band
   it declares an impasse and the generative model is called, which is the only time it is.
4. A control chart watches each production and demotes it when it drifts.

The result is an agent that gets cheaper and faster at what it does often, whose expertise is a
readable rulebook with a track record, and whose control flow is ordinary code. Parts exist:
optimisers for prompt programs (Khattab and others 2023), skill libraries (Wang and others
2023), cascades (Chen, Zaharia and Zou 2023), runtime monitors for temporal properties
(Havelund and Rosu 2001) whose atoms could be judged, session types for protocols (Honda 1993).
We know of no system that closes the loop from deliberation to graded, promoted, monitored
typed productions. The garden already holds most of it: claims as the rule language, the
calibration ledger as the grader, kill criteria as the promotion rule, the conductor as the
runtime. The work of 2026-09-17 was this loop run by hand twelve times: a hypothesis, a packet,
a backtest on our records, a criterion, a verdict.

## What would prove it wrong, and what to test first

The claim fails if compiled productions do not hold their score on new records, if the
abstention band has to be so wide that the generative model is called as often as before, or if
writing and grading a production costs more than the calls it saves. The first test is small:
take one judgment the generative audit seat makes on every node, have a generative model
compile it into typed questions from a handful of its own past rulings, backtest on the rest,
and count how often the dear seat would still have been needed. The second is the measuring
instrument: pick three rates worth a control chart, estimate them over a month of records with
error bars, and check the estimates against a hand count of a sample.
