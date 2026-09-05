# The repository is the harness

The thesis behind plotplot: what is actually new, why the vendors cannot do it, and the shape
that follows from it. Everything in `plotplot-architecture.md` and its review should be read
as implementation of this page.

## The shift

Today, "done" is decided by whoever ran the agent. Claude Code's grader, Codex's cloud, a
second agent in a loop harness, or a human squinting at a diff. The judgment lives in the
session, in the vendor's product, in a dotdir under your home folder or in their cloud. It
evaporates when the session ends and it changes when you change models.

plotplot moves the judgment into the repository. The repo carries its own intent, its own
law, its own proof, and its own yardstick, as files and git hooks. Any agent, any model, any
human is then a worker against that. The vendors sell you better hands. plotplot makes the
work verifiable regardless of whose hands did it.

That is the paradigm change, in one line: **the fitness function lives in the repo, and every
agent is a stochastic optimizer against it.** Ronacher asked how loops can run without the
human collapsing into a messenger between a machine that says done and a machine that
judges. The answer is that the judge is not a machine that talks. It is the repository.

## What a tended repository carries

Five things, all files, all in git, all readable by a person:

1. **Intent with proof.** A loop file per feature: the goal, falsifiable checks, what was
   tried. A check passes only when a verifier ran the evidence, and the pass is a stamp
   bound to the SHA of that evidence. An agent that ticks a box has claimed, not proven, and
   the page renders the difference. That stamp is the smallest idea in the garden and the
   one everything else rests on: done becomes a fact about the repo's state, portable,
   auditable, forgeable only by editing the verifier.
2. **Law.** Git hooks the repo ships: a diff that deletes a test, weakens an assertion, adds
   a stub, or touches its own guardrails is refused at commit; history rewrites and
   non-fast-forward pushes are refused at push. The operating system's sandbox fences the
   filesystem. These bind every agent and every human identically, and the only bypass is a
   flag the harness hook denies.
3. **Structure.** Code read as syntax trees rather than grepped, so an agent reads the
   definition and its callers instead of guessing. Brand and voice as files, checked
   deterministically. Instructions as one short block that points at commands.
4. **A findings channel.** Every gate, in every tool, emits one shape. It reaches the agent
   at Stop, the conductor as a gate, and the reviewer as one check on the pull request.
5. **A yardstick.** Signed measurement artifacts: the expected dollar cost of a correct
   answer with a confidence interval, on fixed axes, so a model, a harness change, or a
   tool can be compared honestly.

The beds are the reference implementation of those five: tend2 for intent and proof, weed
for law, tilth and petals for structure, the findings contract for the channel, copeca for
the yardstick, umbel and pleach to run and conduct workers against all of it, pollen for
the workers to talk with a human at the gate.

## Why this fares better than what exists

Each existing tool puts one piece in the repo. AGENTS.md puts instructions there. Beads and
spec-kit put tasks and specs there. Swimm anchors docs there. CI verifies code there. None
puts **proof that only a verifier can write, bound to a SHA**, or **conduct enforcement in
git hooks that judge the diff for honesty**, or **a vendor-neutral yardstick**. Loop
harnesses verify with a second model, which is an opinion. Vendor graders verify in the
vendor's cloud, which is rented.

And the vendors will not build it, for structural reasons rather than lack of talent. Their
judgment belongs inside their loop, because that is the product. Their state lives in their
dotdir or their cloud, because that is the retention. Their hooks bind their product only.
And none can be neutral across competitors: Anthropic cannot conduct Codex, OpenAI cannot
audit Claude, neither will publish a fair number about the other. A repo-owned harness is
the one thing a model vendor is structurally unable to sell.

## Why use it with Anthropic, Conductor, or Cursor rather than instead

Because they are the hands and the cockpit, and this is the ground they work on. Conductor
runs your sessions in parallel worktrees; every one of those worktrees inherits the repo's
hooks, so the law holds inside Conductor without Conductor knowing. Claude Code is the best
worker; pleach can conduct ten of them and have Codex audit the result. Cursor edits; the
commit still passes weed. Nothing here replaces a harness. It makes the harness
interchangeable.

What you get that they cannot give you: your judgment is owned, not rented. Switching models
costs nothing and loses no proof. Your rules are enforced, not suggested. And you can put a
number on the hands.

## Grounded in the operating system, on purpose

The reliability and speed come from using what is already there instead of building a
runtime:

- **git** is the ledger, the isolation (worktrees), the enforcement point (hooks shipped
  with the repo via `core.hooksPath`), and the proof primitive (content SHAs).
- **the OS sandbox** (seatbelt, bubblewrap, the harnesses' own) is the filesystem fence.
- **tree-sitter** is the eye: structural reading for tilth, structural judgment for weed.
- **static binaries** run the hooks in milliseconds with no runtime to be missing.
- **the filesystem** is the bus: mailboxes, journals, loop pages that render from disk.
- **the harness's lifecycle hooks** are used only for what git and the sandbox cannot see.

No daemon, no database, no account, no server. A tended repo works on a plane.

## The shape that can be built upon

The durable artifact is not a tool. It is **the convention of a tended repository**: the five
things above, specified as small file formats and hook behaviours that anyone can
implement. git, npm, AGENTS.md and SKILL.md were built upon because each was a tiny
convention with tools around it. plotplot publishes the convention and ships the reference
tools, and the extension points already exist in each: loop check methods, weed rules,
pleach runners and ledgers, tilth languages, copeca runners and tasks, petals extractors.

So the long-term shape is: a one-page spec of the tended repository; a handful of
independent tools that implement it and work alone; one thin way to plant them in any
harness; and a yardstick that keeps everyone honest, including the garden itself.

## Who needs this first

People who ship under review. Open-source maintainers drowning in agent-authored pull
requests who need a deterministic gate before they spend a minute reading. Teams that run
more than one harness and want one law. Anyone who wants the proof of what their agents did
to survive the vendor they used. A casual user will take the vendor's grader and be happy.
This is for people who want to own the judgment.
