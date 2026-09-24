# stake, the bed that keeps what a repository says about itself true

2026-09-24. An idea, not a loop. The owner asked whether a third product was missing beside
tend (the bets and their proof) and graft (where things belong and whom they serve), one that
"keeps things up to date, checks for drift". This page names it.

## The gap, stated plainly

A tended repository describes itself: a core page, an architecture document with its
decisions, an engineering document with its recipes, a README, a map whose prose says how
each part works, and, once tilth draws them, diagrams. All of it is true on the day it is
written and rots from the next commit on. The garden's law already says what must hold
(docs/building-the-garden.md §3, the references every bed keeps): cited paths resolve, a
changed shape changes its document in the same pull request, a monthly truth pass lands
corrections. Nothing owns those rules. weeder judges one diff now; copeca asks whether a
change helped; tend proves claims that have evidence scripts. No bed asks, over time, whether
the repository's account of itself is still the code.

## What stake does

Reads the repository's own words and holds them to the code, and says what moved without
its partner. Advisory, findings as SARIF beside the map, a pull request of corrections when
asked; it blocks only where a claim's anchor is gone, the way weeder's cited-path rule does.

- **Anchors.** Every claim in the core page, the architecture and engineering documents and
  the README that names a path, a symbol, a command, a number or a version is checked
  against the code (paths resolve, symbols exist through tilth, commands run, numbers equal
  what a named command prints). This is the doc-truth idea the direction memo of 2026-09-05
  reserved under the name graft; the name moved on 2026-09-24 and the idea is this bed's.
- **What moved without its partner.** From the repository's history: this document, test or
  file usually changes with that code and did not in this change. quadrat's experiment
  showed the signal on nine repositories (a document that usually moves with the code
  follows within five commits 31 percent of the time, against 3 percent for a random
  document; a missing co-change partner is changed within five commits 20 percent of the
  time on unseen projects, against 7 percent). quadrat's engine is the substrate.
- **Diagrams from code.** tilth emits the dependency graph, callers or a data model; the
  diagram is embedded in a tend page under a section of its own, and a check fails when it
  no longer matches the code, the way a wireframe no check cites is treated as decoration.
- **The truth pass.** On a schedule, or on a word, every anchor and every partner rule is
  re-read against the head of the default branch and the corrections land as one pull
  request a person reads.

## How it differs from weeder, and the line between them

weeder is the gate. Everything it judges is a property of one diff, or of the tree as it
stands at one moment: a deleted test, a skip, a stub, a swallowed error, a secret, an import
against the declared direction, a cited path that no longer exists. Each rule decides in
milliseconds from the bytes in front of it, with no history and no execution, and a block-level
finding stops the commit. Its `scan` face reads the whole tree the same way, never blocks, and
already carries the simplest anchor, a cited path or symbol that is missing.

stake is the caretaker. Everything it judges needs one of three things weeder never uses:

- **history**: this document or test usually moves with that code and did not; this partner is
  missing from this change; this claim has gone unverified through many changes;
- **execution**: the number in a document equals what a named command prints; a version
  matches the lockfile; a diagram derived from the code equals the one a page embeds;
- **a schedule**: on a day, or on a word, every rule is re-read against the head of the
  default branch and the corrections land as one pull request a person reads.

So the line is mechanical: no history, no execution, no schedule, weeder's; any of the three,
stake's. stake never re-implements a weeder rule. Its truth pass runs weeder's `scan` and adds
its own findings beside it, in the same SARIF channel, and it never blocks.

One honest alternative, which the rough check decides: if stake's own rules turn out to be few,
because the anchors are mostly weeder's scan, the partner rules are quadrat's engine and the
diagrams are tilth's, then stake is not a bed but a face on the stem, `plotplot truth`, that
composes the three on a schedule. A bed earns its place only if it holds rules no other bed
can.

## Boundaries: tend, graft and stake

Each answers one question, from one kind of evidence, at one time, and writes one kind of
file. That is the whole rule; the rest follows from it.

| | tend | graft | stake |
|---|---|---|---|
| question | what should be true here, and is it | does what we built fit the people it is for | is what we say about ourselves still what the code does |
| evidence | the repository's own evidence scripts and tests, run now; agent checks graded by a rubric; a person's word | a calibrated judge over descriptions of places, features, people and jobs; never the code running | the code's structure (tilth), its history (the engine quadrat left), and what named commands print, re-read over time |
| when | at every landing, on the claim the landing touches | before a build, and again after it, on the whole product | on a schedule, or on a word, on the whole repository |
| writes | the map: loops with checks, personas, opportunities, flows, shapes, Tried, practices; stamps and proof records | findings as SARIF, a report with its maps, and proposals written into the map as pages marked guessed, once | findings as SARIF, and a pull request of corrections to the documents and diagrams |
| never writes | findings | a decision, a stamp, a page a person has edited | a decision, a stamp, a new claim |

**tend is not the documenting bed with the others checking.** A tend page is an article and a
fitness function in one file: it says what a thing is for and how it works, and it proves its
claims through checks that only the verifier stamps. tend checks its own claims. What it
cannot check is whether the product fits people (graft) or whether its prose still matches the
code and its history (stake); those two read tend's map and write back into it in the one
form each is allowed.

**One writer per kind of file.** Map pages are written by people and agents through tend, and
by graft once, as guessed proposals. Findings are written by the judges (weeder, copeca, graft,
stake) into the SARIF channel and never by tend, which only lists them beside a page's checks.
Decisions are Tried lines, written by a person or by tend's change skill. Corrections to the
references are pull requests stake opens and a person merges.

**Things that could have gone either way, settled:**

- tend's `discover` maps an existing codebase into loops; graft's `discover` proposes people,
  jobs and outcomes from docs and issues. Different inputs, different outputs; both stay, and
  the skills say which is which in their first line.
- The narrative-scope lint (prose that promises what no check proves) is tend's: it is a
  property of one page.
- The wireframe comparison (does the built screen realise the shape) is tend's agent check.
- The references law's rules split by the line above: a cited path that is missing is a tree
  fact, weeder's scan; a diagram that no longer equals the code is a tend check, so a landing
  that breaks it goes red; the monthly truth pass, and the scheduled re-verification the proof
  record's contract asks for, are stake's; the work order carrying the invariants a node
  touches is tend's.
- A persona nobody serves is graft's finding (a job no feature does). A loop that names a
  persona page that does not exist is tend's lint.
- Whether a repair helped is copeca's, not stake's: stake says what drifted, never what it cost.

**Flows and layout, the same rule applied.** Direction for a user flow, a feature flow or
what goes where on a screen is set by a person; the beds propose, record and prove.
graft proposes: from people, places and what each person reaches for, it proposes flows as
guessed flow pages written into the map, and its findings say what is forced, missing, split
by role, uncovered, and, per screen and state, which control is primary, visible, in a menu
or elsewhere. tend holds: the flow page with its moments and its walkthrough check, the
screen's shape (the wireframe, its regions, hierarchy and states) authored by a person or an
agent, and the agent check that compares the built screen with the shape. petals holds the
look: tokens, type, colour, voice. Nobody else is needed. graft never writes a wireframe or
a decision, and a finding it raises becomes direction only when a person records it in tend.
In practice graft supplies the direction and the person supplies the yes: on Savire the owner
did not know what each role needed, and the rounds said. A proposal has to be complete enough
that a yes is all it needs.

**Coherence is graft's too.** The Savire review found that most of what a reviewer objects
to is one thing: the same kind of thing should look and behave the same way everywhere.
People build a model of an app from its frame, where they are, how they move, where things
go, and each section had invented its own. That is not fit to people, it is fit to itself,
and it is decided mostly by code from the same inventory graft already reads: list each
page's navigation, sub-navigation, header, breadcrumb and content width, group the pages
that match, flag the odd ones; declare the rules a product keeps (one stable top level, one
breadcrumb grammar, one width rule, the house part kept apart from the person's part) in
`graft.toml` the way weeder.toml declares boundaries, with `docs/design.md` pointing at them,
and check every page against them. Of the four questions to ask of every control, two are
code (does it offer every source the product has; can anyone say what it feeds downstream)
and two need the judge (is it named for what the person gets; does its space follow its
importance). Starting points (a new customer's defaults, per persona) and ownership levels
(organisation, then brand) are the `own` round applied to settings. So graft's question
widens by one clause: does what we built fit the people it is for, and hold together as one
thing. No new bed: the inventory, the rounds, the findings and the declared rules are all
shapes graft has.


Once the owner settles these, they move to the law's §6a as settled boundaries; this page is
where they were proposed.

## What it is not

Not a gate on prose quality or naming (a reader of meaning does that). Not a second home for
any fact: it points at documents, never restates them. Not a ranking of bad files; that was
tested and dropped in quadrat.

## Kill criterion, from the memo

If under 30 percent of the claims in the garden's own documents can be anchored mechanically,
the anchors half stays a document. The first step is a rough check, one afternoon: read the
umbrella's core page, README and architecture document, count claims, and count the ones a
path, a symbol, a command or a number would hold.

## Where it sits

The judge row, beside weeder (is this diff honest, now) and copeca (did any of it help): stake
asks whether what the repository says about itself is still true, over time.
