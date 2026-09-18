# A component study of coding harnesses, read for the garden

Read 2026-09-18. The paper is "An Empirical Study of Harness Design for Coding Agents" (arXiv
2609.20804, submitted 2026-09-17). It fixes one execution loop and varies three parts, planning,
action space and context management, over 176 matched settings, four open-weight models of
different strength and two public benchmarks. Each setting ran once per task, so the directions
are firmer than the sizes.

## What it found

1. Context management pays as the window tightens, and almost all of the gain is avoided
   overflow. At the smallest window the unmanaged loop loses most tasks to overflow; at the
   largest the gain is under three points.
2. The cheapest strategy replaces the bodies of stale tool observations with short stubs first
   and summarises only later. Making the removed content recoverable adds nothing: the median
   use of the recall tool is zero, and the mean falls from 0.54 calls per task at the smallest
   window to 0.007 at the largest.
3. Planning is an accuracy scaffold for a weak model and a cost saver for a strong one. For the
   two strongest models it cut cost by about 30 percent with success within two points, mostly
   by "reducing redundant post-edit verification". The plan is supplied anew each turn and is
   never appended to the history.
4. Predefined file and search tools lift a model that is weak at the shell. The model best at
   the shell did better with the shell alone: success up 3.6 and 5.6 points, cost down 53 and
   30 percent, because it combines several operations in one call.
5. The authors' own summary: harness design is conditional on the model, the task type and the
   budget, and a component should be chosen for them and never adopted as a default.

## What the garden takes

- **Cost has to be measurable before any of this can be chosen.** Every finding above is a
  trade between success and cost per model. The garden records attempts, durations and audit
  outcomes per node, and no token counts: a verdict's telemetry is empty (umbel 78). Until that
  lands, a choice of worker model or tool surface in a plan is a guess. This is the item to
  raise.
- **The cast should carry more than the model.** A conductor that names a worker model can also
  name what that model needs: a strong worker gets a lean prompt and the shell with the
  garden's command-line faces; a weaker one gets predefined tools and explicit scaffolding. The
  garden met this the same day: mechanical nodes went to a smaller model, which ended its turn
  while its own command still ran until the prompt said not to (pleach 90).
- **Orientation is pushed, never pulled.** The plan that helped was supplied each turn from a
  store outside the conversation, and the recall tool went unused. That is the shape of a loop
  page and of `tend2 next`: state on disk, a few lines, re-read at the start of a session and
  after a compaction. It is evidence for the briefing experiment (jahala/plotplot 3) and
  against building any recall face.
- **A command-line face is a first-class face.** The law already counts what a bed costs at
  session start. This study says the shell is the cheaper surface for a strong model, so a
  bed's command line has to compose in one call (plain output, stable exit codes, no prompts),
  and its predefined tools earn their schema cost only for models that need them. tilth's
  outlines are elision at the source, which is the strategy that won here.
- **External gates matter more as workers verify less.** The strong models saved cost by
  verifying less after an edit. In the garden the check is not the worker's to skip: the
  conductor's gates, the pinned verifier and the diff judge run whatever the worker did.

## What does not transfer

The garden drives vendor command-line agents that own their loop and their context management,
so the context strategies are not ours to set; small nodes with a fresh worker each keep
trajectories short instead. The models studied are open-weight and the larger benchmark is one
language, so the crossover points are theirs, not ours.
