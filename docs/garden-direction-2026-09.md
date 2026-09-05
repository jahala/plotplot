# plotplot — what the garden is, and where it grows next

Status: direction memo (September 2026), not a committed plan. Grounded in the six
bed repos, `.brand/`, the pleach engineering doctrine, the tend2 architecture, the umbel
positioning doc, the copeca plans, the seedbank and cuttings, prior session memory, and
a September 2026 check of the outside landscape (sources at the end).

## 1. The one-sentence answer

plotplot is the **vendor-neutral harness layer**: the deterministic, file-based tools that
sit around stochastic agents so a human can still trust what comes out.

The invariant is already written down, in pleach's doctrine: **agents produce; code
decides.** Every canonical decision — done, verified, merged, on-brand, cheaper — is made
by code that reads files, never by an agent's prose. That sentence, not "dev tools", is
what the six beds have in common.

## 2. The stack as it exists

Armin Ronacher's "The Coming Loop" (June 2026) describes the shape of the work now: an
outer harness loop wrapping the inner agent loop, deciding "done" on a signal that only
has to be "useful enough". He ends on an open question: how do we not abdicate judgment,
keep engineering, and keep a responsible human supervising. The garden is one answer to
that question, one bed per harness concern:

```
what should be built         tend2    intent + fitness function; only a verifier writes pass
how the agent reads code     tilth    AST-aware reading; measured cost-per-correct
how the agent reads brand    petals   brand as files; deterministic check + agent layer
run one unit reliably        umbel    the execution boundary; cross-provider
run a plan and gate it       pleach   deterministic conductor; ships only verified branches
did any of it help           copeca   cost per correct answer, signed artifacts
```

| Harness question | Bed | Status | What eats the Claude-only version |
|---|---|---|---|
| Intent and done-gate | tend2 | soon (npm publish pending) | Anthropic Managed Agents "Outcomes" grader |
| Reading code | tilth | live, 339 stars | Claude Code LSP mode, tree-sitter MCPs |
| Reading brand | petals | soon (repo private) | Onbrand, Frontify, zeroheight, Figma MCP servers |
| One unit of work | umbel | live | Claude Code built-in subagents (nested to depth 5) |
| Conducting a plan | pleach | soon (repo private) | Dynamic Workflows, Microsoft Conductor |
| Measurement | copeca | live, on PyPI | Artificial Analysis cost-per-task (no correctness, no CI) |

Two readings of that table. First, the stack is complete as a loop: plan, read, run,
gate, measure. pleach's proof run (Run 3) closed it once, end to end, with real claude
workers, a real codex auditor, and real tend verification. Second, half of it is not
installable by anyone else. The composition is the product, and the composition is
private.

## 3. What plotplot is not

- **Not a dev-tools grab bag.** mrkd, pechas.com, fiken-mcp, smeos, decker are good work
  in other gardens. Adding them would dilute "for building with AI" into "things Jan made".
- **Not an orchestration platform.** umbel's own positioning says the DAG layer is crowded
  (n8n, Temporal, Conductor, Gas Town) and belongs to the caller. pleach is a conductor
  for verified work, not a workflow product with a dashboard.
- **Not a Claude Code plugin collection.** Anthropic has shipped a Claude-only version of
  nearly every bed. Neutrality across Claude Code, Codex, Gemini CLI, and OpenCode is the
  only structural moat any bed has (umbel's insight generalizes: no first-party vendor can
  orchestrate, gate, or measure its competitors).
- **Not the income plan.** Memory is explicit: plotplot is the reputation-and-gift garden
  in the venture-altruist model; Surraura is the paid product. tilth went from 53 to 339
  stars between March and August 2026, which is the compounding this garden is for. Where
  a paid layer could attach without bending the garden is in §8; none of it replaces
  Surraura.

## 4. Landscape check, September 2026

What changed since the beds were planted, and what it means for each:

- **Skills won as the distribution unit.** SKILL.md is an open standard supported by
  30+ platforms; marketplaces list hundreds of thousands. petals is already a skill. The
  garden's own practices (repo audit, oss prep, competitor research, audit batch, lock
  architecture, the seedbank/cuttings idea system) are skills in one person's home
  directory. That is a distribution channel sitting unused.
- **Anthropic shipped cross-session memory (auto memory, `/dream`, Dreaming for Managed
  Agents), built-in and nested subagents, Dynamic Workflows, and an Outcomes grader.**
  Each is a Claude-only tend2, umbel, or pleach. Confirms the spine: the garden survives
  only as the neutral version.
- **"Loop engineering" and "harness engineering" are now named categories** with
  awesome-lists, surveys, and small tools (scheduled loops with worktree isolation and a
  second-agent verification gate). tend2 and pleach's thesis is no longer contrarian; it
  is a category with no file-based, vendor-neutral occupant yet.
- **Code intelligence is contested on the idea, not the number.** Claude Code still greps
  by default; LSP is opt-in; tree-sitter MCP servers multiply. tilth's defensible claim is
  the measured cost-per-correct delta, and copeca is what makes that claim verifiable.
- **Brand-for-agents went commercial as servers** (Onbrand, Frontify MCP, zeroheight MCP,
  Figma MCP). DESIGN.md is emerging as a convention. petals' position — files, no server,
  never guesses — is distinct and should read and write DESIGN.md natively.
- **Cost tracking is commodity; cost per correct is still unclaimed.** Public leaderboards
  publish cost per task; none combine correctness with confidence intervals against a
  shared baseline. copeca's scoreboard plan is still the only one with that shape.
- **Safety hooks are crowded for Claude Code only** (one project ships 914 hooks mapped to
  OWASP). Gemini CLI and Codex now have hook systems too. Nobody ships one policy file
  compiled to all three.

## 5. Gaps in the loop

Harness questions no bed answers today:

| Gap | Evidence it is real | Dormant seed in Jan's repos |
|---|---|---|
| **The boundary.** Hard limits are prose in CLAUDE.md; prose is advisory. | Jan's global CLAUDE.md hard limits (trash never rm, never git reset, never drop a stash, never echo secrets, never delete a failing test) are re-encoded in every repo and enforced by nothing. | none |
| **Doc truth.** README/CLAUDE.md/ARCHITECTURE drift from code and mislead the next agent. | This repo's CLAUDE.md lists five beds; the page ships six. Jan's audit skill has "doc-truth" as a manual step. | none |
| **Peer messaging** between agents with a human trust gate. | umbel is supervisor to worker; nothing is peer to peer. | walkie-clawkie (live, 3 stars, one file, pushed August 2026) |
| **Cross-session learning**, vendor-neutral. | Dreaming owns the Claude slice; sessions from Codex/Gemini/OpenCode learn nothing. | mull (dormant since April; the skein doc already calls it family) |
| **The living document** an agent keeps current. | The polyglot file is Jan's signature format; tend2 and decker both use it. | o-o (14 stars, dormant since February) |
| **The on-ramp.** No "plant the garden" command; each bed installs differently. | The page says "use one, or grow the whole plot" and offers no way to do the second. | none |

## 6. Candidate beds, ranked

Filter (a candidate must pass every line):

1. Answers a harness question no bed answers.
2. Agents produce, code decides: a deterministic core; an agent layer only on top (petals' split is the template).
3. Files, not servers. Local-first.
4. Works on Claude Code, Codex, Gemini CLI, OpenCode.
5. Small enough to finish in one spark.
6. Has a kill criterion written before the first commit.

| Rank | Working name | Harness question | Passes | Kill criterion |
|---|---|---|---|---|
| 1 | **hedge** | the boundary | all six | Gemini/Codex hooks cannot express deny-with-reason on shell calls |
| 2 | **plotplot** (the trowel) | the on-ramp | 1–5; it is the umbrella's own product, not a bed | bed installs cannot be made idempotent one-shot |
| 3 | **graft** | doc truth | all six | under 30% of claims in the garden's own docs can be anchored mechanically |
| 4 | **pollen** (adopt walkie-clawkie) | peer messaging | all six; already built | one real multi-agent run finds no message umbel cannot carry |
| 5 | **mull** (revive) | cross-session learning | 1–4; spark uncertain | Dreaming goes cross-harness, or under half of proposals are accepted |
| 6 | **o-o** (revive, rename) | living documents | 1–4; spark uncertain | no second user beyond Jan after a public release |

**hedge.** A garden boundary. One policy file per repo, ten rules, not 914, compiled to
Claude Code `PreToolUse`, Gemini CLI `BeforeTool`, and Codex hooks. The seed ruleset is
Jan's own hard limits: delete with `trash`, never `rm`; no `git reset --hard`; no stash
drop; no force-push to the default branch; no secrets echoed to stdout; no deletion of a
failing test. Deny with a reason the agent can act on. Differentiators over the crowded
Claude-only packs: cross-harness, curated, and each rule smoke-tested against the real
binaries on a schedule (umbel's provider-drift discipline). Also the first bed a non-coder
can install and feel.

**plotplot, the trowel.** `npx plotplot init` plants the garden in a repo: tilth's MCP
entry, the petals skill, a tend2 map, hedge rules, and one CLAUDE.md block. `plotplot
check` runs every bed's deterministic check in one pass and prints one verdict. This makes
the page's "grow the whole plot" true. It is the umbrella product, and it is also where
the practices pack lives (the skills in §4), so the practices ship with the tools instead
of living in one home directory.

**graft.** Every claim in CLAUDE.md, README, and ARCHITECTURE anchored to a symbol, a
path, or a command's output; tilth resolves the symbols. The check fails when an anchor
is gone or an output changed. The agent layer proposes the rewrite; only the human
lands it. This is tend2's "only the verifier writes pass" applied to prose. Swimm sells
line-anchored docs as an IDE product; nothing does it as files for agents.

**pollen.** walkie-clawkie already is this: push-to-talk between agents, file mailboxes on
one machine, an HTTP relay across machines, unknown agents need human approval. It needs a
garden name (the current one is cute, which the voice forbids), the garden footer, and a
bloom. A2A is the enterprise protocol for this; the local, one-file, human-gated niche is
open.

**mull and o-o** are Jan's own dormant seeds in exactly the two gaps the vendors are
racing to fill (memory, living documents). Both would be neutral, file-based answers.
Neither should be started unless the spark returns; a half-revived bed is worse than an
honest "seed".

## 7. Not beds, and why

- **The intelligence proxy** (seedbank). Venture-scale, needs a team and an enterprise
  motion; stays in the seedbank with its research intact. tilth is the natural context
  engine if it ever grows.
- **decker.** The provenance idea (a tile is a record with a picture attached) is
  garden-adjacent, but decks are not "building with AI".
- **A cost dashboard.** Commodity; a dozen exist. copeca on real transcripts could be a
  copeca feature, not a bed.
- **A Beads or Task Master competitor.** tend2's niche is the feature-level loop that
  feeds execution; the memory from March 2026 already settled this.
- **Any Claude-only thing.** It dies on the next Anthropic release.

## 8. Where it should move

1. **From six beds to one loop.** The composition is the product. Publish tend2, petals,
   and pleach; then show one real run on plotplot.ai, end to end: tend2 shapes the loop,
   pleach conducts it, umbel drives the workers, tilth and petals inform them, copeca
   reports the cost. The hero terminal already hints at this; make it the story.
2. **Say the thesis out loud.** "Agents produce; code decides" belongs on the landing page
   and in every README. The field has named the category (loop engineering, harness
   engineering) and keeps lists; plotplot should be on them as the file-based, neutral
   answer.
3. **Neutrality is the spine.** Every bed on four harnesses, always. This is the one
   property Anthropic, OpenAI, and Google cannot ship.
4. **Publish the contracts together.** The plan schema (pleach), the loop format (tend2),
   the worker signature (umbel), the `.copeca` artifact, the `.brand/` structure (petals).
   Shared contracts are what make six repos a garden; today they are five private facts.
5. **Nothing ships unmeasured.** Every bed already lives by a copeca number or a
   deterministic check. Make it the public promise on the page.
6. **Money, honestly.** Paid layers that would not bend the garden: petals for brand
   teams (the market exists, though what they buy today is servers), copeca verified runs
   for vendors (behind a neutrality wall), hedge policy packs for teams. None is the income
   plan; Surraura is.

## 9. Sequencing (order, not dates)

1. Ship the three "soon" beds publicly. The loop becomes installable.
2. Plant hedge. The desktop grid has two empty placeholders; this fills the first.
3. Adopt pollen. Fills the second; the grid is full at eight.
4. Build the trowel (the plotplot CLI with the practices pack inside).
5. Build graft.
6. mull and o-o only if the spark returns.

## 10. Housekeeping surfaced while reading

- `CLAUDE.md` in this repo lists five beds; copeca is missing.
- The page hardcodes "six tools, one garden" and two placeholder beds.
- copeca's PyPI project is owned by the personal account; the plotplot org transfer is an
  open decision.
- tend2's npm publish under the plotplot org is pending; pleach and petals repos are private
  while their READMEs speak as if public.

## Sources

- Ronacher, "The Coming Loop": https://lucumr.pocoo.org/2026/6/23/the-coming-loop/
- Claude Code changelog (Sept 2026): https://www.gradually.ai/en/changelogs/claude-code/
- Claude Managed Agents updates (Aug 2026): https://explainx.ai/blog/claude-managed-agents-memory-domain-controls-console-update-august-2026
- Dreaming for Managed Agents: https://platform.claude.com/docs/en/managed-agents/dreams
- Agent skills ecosystem report: https://agentman.ai/blog/agent-skills-ecosystem-report-2026
- Every agent supporting SKILL.md: https://www.agensi.io/learn/every-ai-agent-that-supports-skill-md-2026
- Hook systems compared (Gemini, Claude Code, Codex): https://lilting.ch/en/articles/gemini-cli-hooks-research
- cc-safe-setup (Claude-only safety hooks): https://github.com/yurukusa/cc-safe-setup
- awesome-harness-engineering: https://github.com/ai-boost/awesome-harness-engineering
- loop-harness (verification-gated loops): https://github.com/lSAAGl/loop-harness
- Microsoft Conductor: https://opensource.microsoft.com/blog/2026/05/14/conductor-deterministic-orchestration-for-multi-agent-ai-workflows/
- Artificial Analysis coding agent index: https://artificialanalysis.ai/agents/coding-agents
- Claude Code tree-sitter/LSP: https://srilav.medium.com/inside-claude-code-how-tree-sitter-lsp-and-mcp-kill-token-inflation-38ae87035a3f
- Design MCP servers 2026: https://slidespeak.co/blog/best-design-mcp-servers
- DESIGN.md and MCP: https://slidespeak.co/blog/design-md-vs-mcp-ai-brand-guidelines
- A2A protocol: https://en.wikipedia.org/wiki/Agent2Agent
