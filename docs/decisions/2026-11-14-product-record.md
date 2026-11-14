# The product record, and one pattern for a bed that does a professional's work

2026-11-14. The owner's decision, worked out with the plotplot agent after a review of every
bed's code and real output. It amends `docs/decisions/2026-10-26-layers.md`. Where the two
pages differ, this page holds.

## What the owner said

On what plotplot is, 2026-11-14: "the current thinking is: 1. the repo is the harness - the tools
live as part of the repositories, and make those repositories run better, produce faster, be more
optimised, produce better output etc etc over time 2. software factory - the beds in plotplot -
as a whole - enables a user to run a near autonomous software factory, that produces apps and
small SaaS or similar things etc."

On agents inside a bed's command, 2026-11-11: "but the COMMAND can RUN AN AGENT TO DO SOMETHING"
and "our tools can be a mix of deterministic tooling and agent skills".

On the shape proposed below, 2026-11-14: "i agree with the general shape". On where the factory
runs while the owner's machine sleeps: "on cloudflare". On the compass: "plotplot has to work for
a thousand different compasses". On folding three planned beds: "almanac - ok", "upkeep - ok",
"stake - ok". On who defines an app's views: "isnt glean, or graft - supposed to give input on UX
and UI - essentially being able to define the views that an app should have?" It is graft, as
the layers decision already says, and this page spells it out.

Everything below that is not quoted above is the plotplot agent's design. The owner read it on
2026-11-14.

## Why

The review found a working build side and a product side that has never carried a real product.

- tend, pleach, umbel, mycelium, weeder, tilth, petals, hedge and well are in daily use.
- No hunt has run end to end in glean. graft has no command that defines a product. pollen sends
  nothing and counts nothing. No seed has been tested and no campaign has gone out.
- On the product side, a fixed chain of small model calls lost to an agent doing the job, every
  time it was measured. glean's reference handover took one agent about 23 minutes: 4
  opportunities and 94 verified quotes. glean's chain, on the same case, gave 24 needs, 23 of
  them resting on one post. In a proof run on 2026-11-14 one agent read the same posts as the
  chain. Code found all 55 of its quotes in the stored posts, and a blind judge preferred its
  handover on all seven points. It failed its depth mark, because the posts held about 20
  members of the group.
- The typed steps that did hold their marks are all judging steps: a claim gate, a fit check, a
  terms check, a place verdict. Typed calls hold as judges and fail as workers.
- glean and pollen each built their own way to find where a group talks, with different dials
  for the same question.
- What the factory knows about a product lives outside the product's repository, in each bed's
  own data folder.

## The decision

### 1. A product carries its own record

A tended repository carries the intent, law and proof for its code. A product's repository now
carries the same for the whole product: plain files, each written by one bed and read by the
next. Together they are the product record.

| Part of the record | Its bed | What it says | What code checks |
|---|---|---|---|
| People | cultivate | Who the group is, where they talk and under which rules, their own words, their needs with counts, what they use now | Every quote is in a post mull kept. Every count is counted again |
| Market map | glean | What serves each need today, what it costs, where people say it falls short | Every price and claim points at a dated page mull kept |
| Seed | glean | The opportunity, who pays, the riskiest assumption, the test and its pass mark, the owner's yes or no | Required parts present. Evidence resolves |
| Pitch, screens and flows | graft | The problem, the budget, what is left out. Each view the product has: what it is for and what is on it. How a person moves between views to finish each job. Acceptance examples | Every view traces to a need. Every job has a path that finishes it, with no dead end and always a way back. Examples become tend checks |
| Brand, mockups and components | petals | Tokens and voice. A mockup of each view graft defined. The components the views share | As today, and the built view matches its mockup |
| Map | tend | Loops with checks and stamps | As today |
| Ship state | transplant | What should be live, each step's undo, each release's verdict | The plan, the undo, the verdict against the last healthy release |
| Campaign | pollen | Who, where, the actual words and images, the owner's own parts as finished kits, the gates and the kill line | Claims true of the product. Each place's rule kept. Budget reserved |
| Outcomes | pollen, later harvest | What was predicted, what the owner said, what happened | Arithmetic only |

Raw posts stay in mull's store, outside the repository. The record points at them by id.

graft owns the experience: which views exist, what each holds and how a person moves through
them. petals owns the look: the mockup of each view and the components. The owner sees the
mockups before anything is built. tend's format already has a shape for a screen and a shape for
a flow, so graft's views and flows are tend pages.

### 2. One pattern for a bed that does a professional's work

This holds for cultivate, glean, graft and pollen, and for any later bed of the same kind.

| Part | What it is |
|---|---|
| The skill | How the best practitioners do this job, as a versioned file in the bed's repository |
| The command | The bed's own command starts an agent session with that skill, a fixed model and named tools, through the runner seam (`contracts/runner.md`). A development session doing the work by hand never counts |
| The handover | The practitioner's usual document, every claim pointing at its evidence |
| The checker | Code that reads the handover and can refuse it. It finds each quote in mull's own record, never in a file the agent wrote, so no worker stamps its own work (law §1, invariant 8) |
| The judged sample | Code can prove a quote exists. It cannot prove that the quote supports the claim, or that the writer belongs to the group. A typed judge on another provider checks a sample of those, in a shape code can refuse |
| The record | What was predicted, what the owner said, what happened later. Three runs on one case are compared on their claims, with the mark written first |

This is invariant 1 as the law states it: agents produce, code decides. The law never barred an
agent session inside a command. A line saying so in a bed's plan is withdrawn.

The parts already built stay, as tools the agent calls and the checker reuses: mull's readers,
the word-for-word quote check, the terms check, cultivate's counting, pollen's gates, well and
hedge. The typed steps that passed their marks stay as gates.

The checker's verdict is a file. One tend check per handover runs the checker as its evidence, so
the stem can run every bed's checker as one gate. tend's format does not change.

### 3. Alone, and with the garden

Invariant 4 is read as rule 4 of the layers decision words it: a bed works alone through files
any person could write by hand. Alone, a bed reads the file of the station before it, and a
hand-written claim shows as claimed. With mull's records behind it, the same claim shows as
proven. A bed never rebuilds another bed's work to be able to stand alone.

### 4. The compass is a format

Each owner fills in their own compass: whom they serve, what they never build, what a bet may
cost, which acts need a yes, and their demand gates. No tool carries a value from one owner's
compass. A tool that finds a field empty asks, or says the gate is not set.

### 5. Where the factory runs

plotplot hosts nothing. The tools stay local commands over files. A product's live parts (its
page, sign-up, analytics, mail and billing), the scheduled wake-ups and the owner's yes away from
the desk run in the owner's own Cloudflare account, planted and checked by transplant. This
keeps rule 6 of the layers decision: nothing is a proxy or a server of plotplot's.

An agent session started there reaches its model with a token, so it is a paid call, where a
session on the owner's machine rides a subscription. Every such session goes through well.

### 6. What changes in the beds

No new bed is invented.

| Bed | Change | Gains | Gives up |
|---|---|---|---|
| mull | Grows | Search inside a site. Reading on an agent's request through the same door, with a cap. One shared store as the default. Each place's facts kept once: its terms, its software, how active it is | Nothing |
| cultivate | Widens to the whole people layer | All of listening, for a group or for a product's users. The people file. The owner's interview kit | Its chain of typed sorting steps as the main road. Its counting stays |
| glean | Narrows and deepens | The market map, the argued shortlist, the test for each opportunity, the seed as a work order, the record of what became of each seed | Finding places, reading terms, sampling and membership, which go to cultivate. The formula that picked seeds: the owner picks |
| graft | Becomes the shaper | The pitch, each view and each flow, table stakes read from glean's market map, acceptance examples. It keeps the gap list for a built product | The ranking, choosing and pruning families that failed their tests |
| petals | Grows | A mockup of each view. Components. Still images made from a product's real screens | Nothing |
| tend | Stays | One check per handover. It shows a product's record | The plan for a page kind per layer (rule 2 of the layers decision). A handover is a contract file that a page points at |
| umbel | Stays | It is the default runner for every bed's agent session | Nothing |
| pollen | Narrows at the front, grows at the back | Going out, counting, answering, learning | Its own search for people, places and rules, and its own read of alternatives. It reads the people file and the market map |
| transplant | Planted next, on Cloudflare first | Shipping, and keeping a shipped product alive, which was upkeep's job | |
| the gardener | Planted after the pattern is in | It keeps the decision log that almanac was to hold | |
| hedge | Stays | The owner's yes reachable away from the desk | Nothing |
| the stem | Grows | `plotplot status` over a product's record, and every bed's checker run as one gate | Nothing |
| harvest | Waits | It starts when a product has users. pollen's outcomes carry the learning until then | |
| trellis | Waits | It is planted only if agents are seen breaking a product's shape | |

pleach, mycelium, well, coldframe, weeder, tilth, copeca and quadrat do not change.

Three planned beds fold away:

- **almanac.** What is due is read from the dates already in the record: a test's read date, a
  campaign's end, a renewal in the ship state. `plotplot status` lists it. The decision log goes
  to the gardener.
- **upkeep.** It folds into transplant. Shipping a product and keeping it alive are one job.
- **stake.** It is not planted. Its checks land as weeder `scan` rules, where the law already puts
  the cited-path rule (law §6a).

### 7. Who decides what

- The owner decides what is built, what people see, money, voice and accounts. Only the owner's
  own words are rulings.
- Code decides whether a handover stands.
- Each bed's lead agent decides how the bed does its work, and records it as its own call.

The umbrella keeps the contracts, the fit checks and the choice of what runs next. A dial the
umbrella agent set is a proposal with a range, like any other dial. Lines in a bed's docs that
cite a ruling by plotplot are relabelled as the umbrella agent's call unless they quote the owner.

## What this overturns

- Layers decision, rule 2: a page kind per layer. Replaced by section 2's last paragraph.
- Layers decision, the rows for almanac, upkeep and stake.
- Law §5, the mandates for cultivate and glean, rewritten in the same change as this page.
- Any bed's rule that no agent session runs inside a stage.

## What follows, in order

1. The contracts, each a page with a schema, a fixture and a test both sides run (law §8, point
   2): first the agent step and the people file, then the market map, the seed, the pitch with
   its views and flows, the campaign, the outcomes, the compass and the record's layout.
2. A brief for each bed whose job changes: cultivate and mull first, then glean, graft and
   pollen. At most three beds are busy at once.
3. One product of the owner's through the selling half, and one new product from a hunt through
   to its first users, with every hand step written down. That list is the gardener's
   specification and transplant's.
4. almanac, upkeep and stake leave the brand's bed list and the website's list with the next
   brand release.

Paused until a walk says otherwise: tuning glean's middle stages, pollen's wide proof on its
panel, and graft's ranking spikes.

## What would prove it wrong

- **The pattern.** On a group nobody tuned for, if three runs do not agree on most of their top
  opportunities, or a judge from another model family does not prefer the result to the chain's,
  or a draft written from the model's own knowledge with no posts reads as well, the pattern is
  wrong for glean and is not extended.
- **The shared home.** If one people file cannot serve both glean and pollen on a real product,
  listening has the wrong owner.
- **The record.** If the first walk needs facts that fit no part of the record, the parts are cut
  wrong and are redrawn before any contract is frozen.

## What is not settled

- cultivate's lead has not been asked about owning all of listening.
- The proof run was one tuned case, with a judge from the same model family. The pattern is shown
  for safety and for quality there, and not yet for a new group.
- Only one agent CLI's route to a Cloudflare sandbox was read. The others are unchecked.
- No practitioner has read any bed's handover.
