# The architecture, attacked

An adversarial review of `plotplot-architecture.md`, written by its author, one day later.
Method: run the design through lenses that do not share its assumptions, name what is
theater, name what was under-thought, and end with the shape that survives. Where a finding
reverses a decision in the architecture doc, it says so plainly.

## 1. Lens: fan-in and fan-out. Which contracts are real?

A contract pays for itself when it has many producers or many consumers. Otherwise it is a
file format with a grand name. Applying that test to the eleven:

| Contract | Producers | Consumers | Verdict |
|---|---|---|---|
| findings | weeder, petals, graft, tend2 lint, hedge | check, pleach smoke, CI, Stop hook, PR comment | real, the keystone |
| manifest | every bed | the stem | real only if the stem exists (see §4) |
| briefing | several beds | one assembler | real only if push orientation is proven (see §5) |
| loop | tend2 | pleach, next, walk | real, exists |
| plan | tend2's bridge, hand-authored | pleach | real, exists |
| unit | umbel | pleach | a seam, not a contract; nobody else will write a runner soon |
| receipt | the stem | walk | speculative; the harness already writes a transcript and git already records the commit |
| message | pollen | pollen | an implementation detail |
| rules | hedge's config | hedge | a config file |
| brand | petals | petals, every page | real, exists, and it is petals' |
| artifact | copeca | copeca verify | real, exists, and it is copeca's |

**Finding.** Eleven was inflation: every file format got promoted to a contract. The umbrella
needs **three** that are new, findings, manifest, briefing, and leans on **two** that exist,
loop and plan. The rest belong to their beds. With five small schemas, the schema registry,
the conformance harness, and the additive-within-a-season rule are process theater for one
maintainer. What actually keeps contracts honest is one end-to-end run that pins versions.
Fixtures for satellites, yes. A registry, no.

## 2. Lens: what does the stem do that nothing else does?

Take each stem command and ask what breaks without it.

- `run`: a wrapper over `pleach run`. Adds a name, no capability. **Cut.**
- `walk`: a second attention surface next to `tend2 serve`, needing a registry of repos, for
  one user with fifty repos. `tend2 next` with several roots covers it. **Cut from v1.**
- `mcp` gateway: exists only for harnesses that load every schema. Codex will defer sooner
  or later, the string-dispatch `garden_call` loses typed schemas and makes the model worse
  at calling tools, and the garden's capabilities are CLIs anyway, which Codex agents can
  run from a shell. Only channels need MCP, and that is nine tools. **Cut.**
- `orient`: see §5. **Downgrade to experiment.**
- `init`, `doctor`, `check`: the three that survive.

Then a harder question: **is the stem a program at all?** Every harness now has a native
bundle format that carries hooks, skills, and MCP entries together: Claude Code plugins,
Gemini extensions, Codex skills plus config. Those formats are what the vendors themselves
keep stable. A `plotplot` plugin for Claude Code is a static directory: `hooks.json`,
`skills/`, `.mcp.json`. The same for Gemini. Generating three bundles is a build step in one
repo, not a CLI product, and the install becomes the vendor's own command. tilth's
twenty-host installer, which the architecture doc proposed moving into the stem, is the
thing to retire, not to inherit: it edits vendor settings files whose internals churn, while
bundle formats are a published contract.

**Finding.** The stem shrinks to three generated bundles plus a `doctor` that proves hooks
fire and a `check` that concatenates findings. `npx plotplot init` survives as a
convenience over the vendor installers, not as the architecture.

## 3. Lens: runtime. Is Node actually there?

The architecture doc chose Node as the single runtime because "every hook can rely on Node
being present". Check the premise: Claude Code installs as a native binary. Codex is Rust.
Gemini CLI is Node. A machine running Claude Code and Codex has no Node unless the user
installed it for something else. A hook that needs a runtime the harness does not ship is a
hook that silently never fires, which is the worst failure mode for a boundary.

**Finding.** The premise was wrong, and the decision reverses. Anything that runs as a hook,
hedge, weeder, orientation, must be a **static binary**. tilth already has that toolchain and
release pipeline: Rust, multi-platform binaries, crates.io, an npm wrapper. Glue that runs
inside an already-Node process, tend2 and pollen, may stay Node. umbel and pleach stay on
whatever they are; porting them off Bun buys nothing a user can feel and risks umbel's
delicate tmux handling.

## 4. Lens: Conway, and one maintainer who ships in sparks

Ten beds, a stem, a contracts package, a site, a proof repo, seasons. For one person with
three beds unshipped. Elegance that multiplies surface area is not elegance.

The monorepo argument was "atomic contract changes and one CI". With five small contracts
that stabilize fast, atomic changes are rare. One CI proving the loop can live in a `proof`
repo that pins versions, with no move at all. Separate repos are also what keeps the
standalone invariant honest: shared code cannot creep across a repo boundary, and each tool
keeps its own stars, issues, and landing page, which is how tilth got found.

**Finding.** The monorepo recommendation reverses. Keep separate repos. Add one small
`@plotplot/contracts` package and one `proof` repo holding the end-to-end run, the version
lockfile, and the generated bundles. The exception is the Rust side: weeder and hedge share
tilth's tree-sitter parsing and release pipeline, so they belong as separate crates in the
tilth **Cargo workspace**, each with its own binary and identity, depending on a `tilth-core`
library. That is a library dependency, not shared state, and standalone holds.

Then count again. The target surface is **tilth, weeder (with guard inside it, see §6), tend2,
pleach, umbel, pollen, petals, copeca**, plus contracts and proof. Eight beds, two support
repos. graft, mull, walk, receipts, and the gateway are gone or absorbed (§8).

## 5. Lens: push or pull. Is orientation theater?

The ETH result says context an agent did not ask for lowers success and raises cost.
tilth's scout arc says a correct hint saved a fifth on hard tasks and cost a tenth on easy
ones, so hints must fire on evidence. The architecture doc made `orient` a core stem command
and built a briefing contract around it. Nobody has measured push orientation against the
cheapest alternative: one line in `AGENTS.md` saying "run `tend2 next` first", and the agent
pulls what it needs. Claude Code's own memory works by pull: an index in context, files read
on demand.

**Finding.** Push orientation is a hypothesis wearing a contract. Default to pull. The garden
block in `AGENTS.md` names the commands; tend2's SessionStart hook remains as tend2's own
experiment; the briefing assembler ships only if a copeca scenario shows push beating pull.
If it never does, the garden lost nothing.

## 6. Lens: adversary. Does hedge hold?

hedge as designed is a deny list of shell patterns compiled to PreToolUse hooks. Deny lists
lose: `bash -c`, `python -c`, `find -delete`, `xargs rm`, `git -c alias.x='!rm -rf'`, a
script written to disk then executed. One community pack has 914 hooks and is still a deny
list. Determinism is not completeness. An honest hedge stops accidents and the obvious; it
does not stop an agent that wants to get around it.

The out-of-the-box move is to put the boundary where it cannot be talked around:

- **git hooks.** `pre-commit` runs weeder. `pre-push` runs weeder strict and refuses non
  fast-forward pushes to protected branches. `pre-rebase` and a `reference-transaction` hook
  refuse history rewrites. These are universal across every harness and every human, and the
  only bypass is `--no-verify`, which is one pattern to deny at the harness hook. The
  boundary moves from "match every dangerous command" to "guard the two things that matter:
  the working tree's history and the remote".
- **the OS sandbox.** Claude Code and Codex both ship sandboxes; bubblewrap and seatbelt
  exist. Deny writes and deletes outside the worktree and temp. That is `trash` versus `rm`
  solved by the operating system rather than by a regex.
- **harness hooks** only for what git and the sandbox cannot see: reading secret paths,
  network egress to unknown hosts, the `--no-verify` pattern.

Two other hedge features do not survive contact. Inbound quarantine of tool results is not
deterministic; natural-language injection has no fixed shape, so the claim would be theater.
Spend rules cannot see token usage from a hook on most harnesses. Both are cut.

**Finding.** hedge stops being a bed. Its git-hook half is weeder's `guard` subcommand, since
weeder already sits at commit and push and already classifies the diff. Its sandbox half is a
configuration the bundle installs. Its harness-hook half is a short deny list inside the
bundle. Boundary and evidence were one concern all along: code judging the agent's actions.

## 7. Lens: Goodhart. Will weeder be gamed, and will it be trusted?

Every false block costs a retry, and agents learn. If weeder blocks five percent of honest
commits, `Weeder-allow` trailers become reflex and the gate becomes noise. Two consequences
the design did not draw:

- Only unambiguous rules may block: a deleted test, an added skip, a stub in production
  code, a secret, a guardrail file edited, a conflict marker. Everything else warns.
- The consumer of a warning is a **human reviewer**, not the agent. An agent argues with a
  warning; a reviewer acts on it. Warnings belong in the pull request as one "garden check",
  posted by a plain GitHub Action running `weeder check`. tend2's pr-gate loop already points
  there. This makes the PR, not a walk page, the attention surface for everyone who is not
  the garden's author.

`weeder bite` is the strongest rule and needs the least code once pleach adopts one policy the
field already recommends: **the test phase commits separately from the implementation**.
With tests in their own commit, "does the test fail without the change" is a checkout and a
run, no hunk splitting. bite becomes a pleach policy plus a ten-line weeder subcommand.

## 8. Lens: subtraction. What is absorbed?

- **graft** (doc truth): deciding what counts as a claim is judgment, so it is a skill that
  proposes anchors, plus a checker small enough to be one weeder rule: "a path or symbol this
  doc names no longer exists". No bed.
- **mull**: if its review says it lives, its viable form is "write the loop's Tried section
  from this session's transcript". That is one judgment call at SessionEnd, a skill invoking
  `claude -p` with the loop and the transcript, not an eight-thousand-line pipeline.
- **walk**: a flag on `tend2 next`.
- **receipts**: the harness's transcript plus git already record what a session did. A
  receipt contract is worth building when a second person needs to read them.
- **the gateway**: cut, §2.
- **the ten-concern taxonomy** was a scaffold for design, not a truth. Four concerns are
  enough and make a better landing page: **plan** (tend2), **read** (tilth, petals), **run**
  (umbel, pleach, pollen), **judge** (weeder, copeca).

## 9. Lens: where a skill beats a CLI, and where neither

The rule the design stated but then violated in the stem:

- **Skill** when the step is judgment or a procedure a person would follow: shaping a loop,
  authoring a plan, extracting a brand, auditing a repo, deciding what a doc claims, writing
  Tried from a transcript, preparing a release.
- **CLI** when the step must be deterministic, fast, repeatable, or must produce an exit
  code: verify, lint, check, count, diff, run, measure.
- **Hook** when the agent must not have to remember: the boundary, the Stop gate, orientation
  where push proves itself.
- **git and the OS** when the boundary must hold against anyone: history, remotes, the
  filesystem outside the worktree.

Everything the stem tried to do beyond install and check was a skill or a flag pretending to
be a product.

## 10. Lens: absorption. What survives the vendors' next two quarters?

Every bed's single-harness version now exists or is announced. The design's defense is
neutrality, but neutrality only matters to people who run several harnesses, a minority.
The durable arguments are different and the pitch should say them: **your verification
layer is not rented**, it runs in your CI and your git hooks whatever model you use; and
**your state is files in your repo**, not a vendor's cloud. Four CLIs is the feature; not
being rented is the reason.

There is one move that turns the moat from tools into a standard. SKILL.md and AGENTS.md
won because they were tiny and obviously useful. A findings shape small enough to be a hook
output convention could be proposed the same way. A solo author proposing a standard is a
long shot, and the cost of trying is one page.

## 11. Lens: measurement. Is copeca the right instrument for everything?

The architecture doc gated the season on a whole-garden copeca scenario. That repeats the
mistake already learned once with SWE-bench: the wrong instrument for a tool's value. copeca
measures cost per correct answer on navigation and edit tasks. weeder, hedge, tend2 and pleach
do not lower that number; they lower risk, retries, and review burden, which copeca cannot
see. A null result would be likely and would mean nothing.

**Finding.** Invariant six stands, "measured or silent", but each bed declares its own
falsifiable metric: tilth, cost per correct; weeder, calibration precision and incidents
caught; tend2 and pleach, verified-close rate and human interventions per feature; pollen,
one message umbel could not carry. copeca is one instrument, not the gate.

## 12. Lens: the slogan. Does code really decide?

pleach's audit verdict is JSON written by a second agent on a different provider. Code
parses it and refuses anything that is not the fenced block, but the verdict itself is a
model's judgment. The deterministic gates are conflict markers, the test command, and weeder.
The audit is a second opinion with model diversity as its safeguard. That is a good design
and pleach's docs say it precisely. The garden's slogan overstates it, and the people this
garden is for will notice.

**Finding.** Keep the slogan, qualify it once on the page: code decides what can be decided
by code; where a judgment is unavoidable it comes from a second model, in a shape code can
refuse.

## 13. The shape that survives

- **Contracts**: findings, manifest, briefing, plus loop and plan where they already live.
  One small npm package with schemas and fixtures. No registry, no seasons ceremony.
- **Stem**: three generated harness bundles, `doctor`, `check`. `init` as a convenience.
  Nothing else.
- **Beds**: tilth · weeder (check, bite, guard) · tend2 · pleach · umbel · pollen · petals ·
  copeca. Four rows on the page: plan, read, run, judge.
- **Runtime**: hooks are static binaries in the tilth Cargo workspace. Everything else stays
  as it is.
- **Repos**: separate, plus `contracts` and `proof`. The proof repo runs the loop end to
  end on pinned versions and builds the bundles.
- **Boundary**: git hooks and the OS sandbox first; harness hooks only for what those cannot
  see.
- **Context**: pull first. The garden block names the commands. Push orientation is an
  experiment with a copeca scenario, not a core feature.
- **Attention**: the pull request, via one GitHub Action posting the garden check. tend2's
  pages for the author.
- **Measurement**: one falsifiable metric per bed, declared in its manifest.

What this costs the earlier design: four commands, six contracts, one monorepo, one
runtime decision, two beds, and a season ceremony. What it keeps: every invariant, the
loop, and a surface a single maintainer can carry.

## 14. What still needs thinking that this review did not do

- Whether tend2's loop file should be the garden's center at all, or whether the pull
  request already is, for everyone who is not its author. The review moved attention to the
  PR; it did not move intent there. That question deserves its own session with real users.
- Whether umbel's future runner is headless structured output rather than tmux, now that
  vendors ship SDKs. umbel's positioning doc anticipates it; nobody has built the adapter.
- How a feedback loop from false blocks, the `Weeder-allow` trailers, flows back into rule
  changes without a human reading every one.
