# mull — a fresh look

Brief for a Fable 5.1 agent. Self-contained: you have not seen any prior conversation.
Read this whole file before touching anything.

## Your posture

You are the lead engineer and product strategist taking over a dormant codebase that a
non-developer owns. Your deliverable is a decision, written down, with the evidence that
forced it. You are not here to fix mull. You are here to find out whether mull can be made
viable, and if so, as what.

- Execute the work yourself. No subagents, no delegation. Your final message is the
  deliverable: the paths of the two files you wrote plus the verdict paragraph.
- Ground truth over inference. Every claim about the code cites a `file:line` or a command
  you ran with its output. Every claim about output quality quotes the actual extracted
  item. If you did not run it, you do not know it.
- Falsify, don't defend. mull has 26,000 lines of docs arguing for itself. Your job is to
  find what would prove it wrong, then check.
- Decide. The owner will not arbitrate technical trade-offs. If you are genuinely torn
  after real groundwork, give at most two options, the pitfall of each, and why the call is
  hard. Never a menu.
- No time estimates. No phases. No TODOs. Nothing silently dropped: anything you could not
  verify goes in a "could not verify" section, named.

## Guardrails

- Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash. Do not
  commit or push. Do not edit anything under `src/`; if you find a bug, record it in the
  audit with file:line, do not fix it.
- Session transcripts contain secrets and private text. Never paste a transcript excerpt
  into the memo without redacting keys, tokens, paths under `~/.ssh`, and email bodies.
  Never echo a secret to stdout.
- mull shells out to `claude -p` and spends money. Cap each run with the budget in
  `.mull/config.yaml` or the `--budget` flag; keep total spend under $5 and report it.
- Revert every file you mutate for testing. `git status` at the end must show only your
  two new docs plus whatever `.mull/` state your runs produced, which you list in the audit.
- If the tilth MCP tools are available, use them for code reading and search; otherwise
  grep and cat.

## Where things are (verified 2026-09-05)

| What | Where |
|---|---|
| The code | `~/conductor/workspaces/mull/sarajevo` on branch `jahala/mull-standalone-rewrite`, which equals `origin/master` at `9c3e560`. `~/conductor/repos/mull` is a stale checkout; ignore it. |
| Tests | `npx vitest run` in that directory: 47 files, 905 tests pass, 3 skipped, under one second. |
| Source | `src/` 8.4k lines TypeScript. `test/` 11k lines. `scripts/` e2e probes and quality probes. `defaults/musings/` the five shipped musings. `defaults/templates/` external-source templates. `v2/` an abandoned exploration. |
| Docs | `docs/` 26k lines. Start with `docs/WHY-MULL.md`, `docs/architecture/engine-design.md`, `docs/research/strategic-findings-2026-03-30.md`, `docs/research/mull-devils-advocate.md`, `docs/research/mull-multi-vendor.md`, `docs/research/mull-as-unix-tool.md`, `docs/ideas/potential-features.md`. |
| Untracked findings | `docs/relay-mull-findings.md`, `docs/relay-mull-all-items.md` (mull run on the agent-relay repo), `docs/research/trendradar.md`. |
| mull run on itself | `.mull/` in the same directory. `.mull/knowledge/decisions.md` is 120 KB. `.mull/decisions/metrics.jsonl` has run metrics. `.mull/decisions/curation-log.md` has the curation history. |
| Prior session memory | `~/.claude/projects/-Users-jahala-conductor-repos-mull/memory/*.md`, four files. Read all of them first. The gate-quality file records the last known root cause. |
| The repo's own rules | `~/conductor/workspaces/mull/sarajevo/CLAUDE.md`. Rule 3 there is the one that matters: quality means output quality, never code cleanliness. |
| Session logs on this machine | Claude Code: `~/.claude/projects/` (278 project dirs, about 2,980 `.jsonl` files). Codex: `~/.codex/sessions/` (41 files). Gemini CLI: `~/.gemini/tmp/` (15 files). OpenCode: `~/.local/share/opencode/` (213 files). |
| Other repos with real session history | `~/conductor/workspaces/feature-map/missoula` (tend2), `~/conductor/workspaces/copeca/cancun` (copeca), `~/CascadeProjects/tilth` (tilth). |

Garden context, read before Part 2:

| Doc | Path | Why |
|---|---|---|
| plotplot direction memo | `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/garden-direction-2026-09.md` | What the garden is; the six-line filter; where mull was slotted and why it was ranked fifth. |
| pleach engineering doctrine | `~/conductor/repos/pleach/ENGINEERING.md` | The invariant: agents produce, code decides. |
| umbel positioning | `~/conductor/repos/rctrl/docs/positioning.md` | How a bed in this garden argues for itself; the neutrality moat. |
| tend2 loop format | `~/conductor/workspaces/feature-map/missoula/FORMAT.md` | The `## Tried` section is the garden's existing per-loop memory. |
| copeca methodology | `~/conductor/workspaces/copeca/cancun/docs/methodology.md` | How the garden measures whether a tool helps: cost per correct answer, A/B, pre-registered ship and kill bars. |
| petals skill | `~/conductor/repos/petal/petals/SKILL.md` | The template for a bed: deterministic core that runs with zero tokens, agent layer on top. |

## What mull is today, so you do not rediscover it

mull reads sources (Claude Code session JSONL, git history, agent trajectory files, Hacker
News, GitHub, RSS, web), cuts them into chunks, denoises, scores relevance with BM25 plus
an optional local embedding, sends surviving chunks to `claude -p` with a YAML "musing"
(focus, fields, negative examples, quality gate), dedups the extracted items, stores them
under `.mull/<musing>/`, curates them against the current code, and delivers them to
`.mull/knowledge/*.md` plus a compact index injected into CLAUDE.md plus an MCP server.
After each run it rewrites the musing YAML with `[learned]` refinements. Maintenance
clusters, consolidates, decays, and detects drift.

History in nine commits: started 2026-03-29 as a decision extractor; pivoted 2026-03-30 to
a generic signal engine with processors as plugins; watches renamed to musings; external
sources added; last commit 2026-04-07. Dormant since.

Known and unresolved at the time it went dormant:

- The relevance gate passes garbage because the Claude session adapter produces 5k to 33k
  character mixed blobs of system preamble, tool output, and conversation. Root cause is
  upstream of scoring. See the gate-quality memory file.
- Survival rates in `.mull/decisions/metrics.jsonl` run 18% to 55%. Twenty-four LLM calls
  produced six surviving items in one run.
- `.mull/knowledge/decisions.md` on mull's own repo is 120 KB. That is not knowledge an
  agent reads on every turn; that is a second codebase.
- Only Claude Code sessions are parsed, despite `docs/research/mull-multi-vendor.md`
  arguing the neutral parser is the whole point.
- No external consumer exists. `~/conductor/workspaces/plannr/auckland/.mull/` contains
  only lock and realtime dirs.
- `docs/research/strategic-findings-2026-03-30.md` records the ETH Zurich finding
  (arxiv 2602.11988): LLM-generated context files reduced task success by 0.5 to 3
  percent and raised cost 20 percent or more; developer-written files improved success
  only about 4 percent. This is the largest threat to mull's premise and the prior
  session knew it.
- **The owner's stated reason for stopping:** mull could not replicate, let alone beat,
  Claude Code's built-in `/insights` (a 30-day scan of local sessions by Opus that yields
  friction analysis and ready-to-paste CLAUDE.md rules), and what it did extract was slop
  repeated from subagent transcripts. Nothing in `docs/`, `memory/`, or `.context/`
  mentions `/insights`; the comparison was never written down. Treat `/insights` as the
  bar mull already failed, and find the duplication mechanism: the adapter at
  `src/sources/claude-session.ts` never checks `isSidechain`, and subagent output also
  echoes back into the parent transcript as tool results. Today subagent transcripts live
  under `~/.claude/projects/<project>/<session>/subagents/` (1,679 files in 27 dirs on
  this machine); the adapter's `readdir` is not recursive, so confirm which path the
  duplicates took in April: inline sidechain entries, tool-result echoes, or the
  trajectories adapter.

## What changed in the world since April 2026

- Anthropic shipped auto memory and `/dream` in Claude Code (March), Dreaming for Managed
  Agents (May), and memory orchestration across agent teams (August). Codex and Gemini CLI
  have their own memory. A Claude-only memory tool is dead on arrival; only a
  vendor-neutral, repo-local, auditable one has any ground to stand on, and only if
  extracted knowledge demonstrably helps.
- SKILL.md became the cross-harness distribution unit (30+ platforms). AGENTS.md is the
  cross-harness instruction file (60k+ repos). Anything mull delivers to CLAUDE.md should
  probably deliver to AGENTS.md.
- Gemini CLI and Codex now have lifecycle hooks, so a session-end trigger is possible on
  three harnesses, not one.

## Part 1 — Ground-truth audit

Read `src/` end to end before running anything. Then:

1. **Pipeline as built versus as documented.** Map the real data flow from
   `src/pipeline.ts` outward. Where do README, `docs/architecture/engine-design.md`, and
   the code disagree? List each disagreement with file:line.
2. **Run it, twice, on real data.** First on the mull repo itself with a small budget.
   Second on one other repo with real session history from the table above. For each run
   record: LLM calls, dollars, chunks in, chunks past the gate, items extracted, items
   survived. Then read every surviving item and score it on one axis: **valuable** (a new
   agent would be misled or slower without it, and it is not derivable from code, git log,
   or the existing CLAUDE.md), **derivable**, **wrong**, or **stale**. Report the four
   percentages per run and quote three items from each class, redacted.
3. **Re-verify the gate root cause.** Sample five chunks the gate passed and five it
   rejected. Is the mixed-blob finding still the cause? Does `stripMarkup` in the adapter
   still miss the preamble and tool output?
4. **Check the parser against reality.** Take a Claude Code `.jsonl` written this month
   from `~/.claude/projects/` and diff its entry shapes against
   `src/sources/claude-session.ts`. Has the schema drifted? What is silently dropped? Then
   open one Codex, one Gemini, and one OpenCode log and state what a parser for each would
   need, in a table: format, message shape, tool-call shape, where the project path lives.
5. **Inventory the weight.** For `v2/`, each `src/sources/*` adapter, each maintenance
   module, `defaults/templates/`, and the top twenty docs by size: keep, delete, or
   harvest, with one line of reason each. Which tests pass without exercising real logic?
6. **Privacy.** `.mull/knowledge/` is meant to be git-tracked. Trace whether any path lets
   secret-looking strings from a transcript reach a delivered file. Is there redaction?
   Where?
7. **The `/insights` head-to-head.** Run `/insights` in a Claude Code session on the same
   project you ran mull on (it writes an HTML report; read it, do not screenshot it). Put
   mull's surviving items and the report's friction findings and CLAUDE.md suggestions
   side by side. For each mull item: could `/insights` have produced it? For each
   `/insights` finding: did mull find it? Report the two overlap counts. Anything mull
   produces that `/insights` could also produce is not a reason for mull to exist.
8. **Subagent duplication.** Count, in your two runs, how many extracted items trace back
   to the same underlying event seen twice or more (lead transcript plus subagent, or a
   tool-result echo). Name the mechanism with file:line in the adapter and in the dedup
   stage that let it through.
9. **Deterministic versus LLM.** For every stage, mark whether code decides or an LLM
   decides. Extraction is an LLM judgment by nature. What could be made deterministic and
   is not: curation against code (does the referenced symbol still exist; tilth can answer
   that), dedup, supersession, staleness, the delivery diff, the budget.

## Part 2 — The hard questions

Answer each with evidence from Part 1 and the docs. Opinion without a citation does not
count.

0. **Why would this time be different?** The owner stopped because mull lost to
   `/insights`. Any revival must rest on a structural difference, not on better prompts or
   a tuned gate. Only these count, and you must show which one your verdict rests on:
   a **different question** (`/insights` summarises a month of everything; a tool that
   knows the goal and the checks of one loop can ask "what was tried against this goal
   and failed", which `/insights` cannot); a **different consumer** (a machine-checked
   file such as a tend2 loop or a proposed CLAUDE.md diff that a verifier or human lands,
   not a report a human reads); a **different input** (Codex, Gemini, OpenCode sessions
   that `/insights` will never see, and only if the owner's cross-harness use is real,
   which the counts above suggest); or **code deciding** (curation against the current
   code via tilth, so stale and wrong items die deterministically). If none of the four
   holds for your chosen shape, the verdict is kill, and you say so in the first sentence.
1. **Does the premise survive?** Given the ETH finding and Dreaming, under what conditions
   does extracted knowledge help rather than hurt? Test the hypothesis that the only
   class not derivable from code is negative knowledge: hard constraints, things tried
   that failed, and the reason. Check it against your scored items: which class did the
   valuable ones fall in?
2. **Who is the user?** Four candidates: one developer on one harness (Dreaming wins); one
   developer on several harnesses (neutral wins, if the pain is real; the counts above are
   one such developer); a team sharing a repo (repo-local wins, but AGENTS.md exists); a
   fleet of agents in a pleach or umbel run whose sessions nobody reads (the "four agents,
   one picture" case). Pick one, with evidence. The garden's own fleet runs are the case
   with a guaranteed consumer; say whether that is enough.
3. **What is the smallest sharp thing?** Evaluate each of these against the six-line filter
   in the direction memo (a harness question no bed answers; code decides; files not
   servers; four harnesses; finishable in one spark; a kill criterion written first).
   For each give: pass or fail per line, the copeca A/B that would prove it, the kill
   criterion, and the one-spark scope in files and commands.
   - (a) **The negative-knowledge extractor.** Only constraints, gotchas, and failed
     attempts. Output is a proposed diff to CLAUDE.md or AGENTS.md; a human lands it.
   - (b) **tend2's `## Tried` writer.** Reads the sessions of one pleach or umbel run and
     appends what was tried, decided, and scoped out to the loop file. Consumer guaranteed.
   - (c) **The neutral session reader.** `mull read <any harness log>` yields one
     conversation shape; nothing else. Check whether copeca's runner parsers already do
     this and whether this is a bed or a library the other beds share.
   - (d) **The external-signal engine** (HN, RSS, web). State plainly whether this belongs
     in a garden of tools for building with AI, or in a different garden.
   - (e) **Kill and harvest.** The Claude session parser to copeca; curation-against-code
     to a future doc-truth tool; the research to `docs/research/` in plotplot-ai.
4. **What is the measurable claim?** Design the copeca scenario for your chosen direction:
   the repo, the task set, baseline versus arm, the metric (cost per correct answer), the
   pre-registered ship bar and kill bar. Follow the copeca methodology doc. Do not run it
   unless one run costs under a dollar; design it so the next agent can run it unchanged.
5. **Name.** "mull" is on-brand and already treated as family in the tend2 docs. Keep it
   unless the direction makes the name a lie.

## Part 3 — Write two files

Create both under `~/conductor/workspaces/mull/sarajevo/docs/`. Do not commit.

**`audit-2026-09.md`** holds facts only: the tables and quoted items from Part 1, the
commands you ran, spend, and the "could not verify" list.

**`direction-2026-09.md`** holds the judgment, in this order:

1. The verdict, one sentence: revive as X, or kill and harvest, or park with a named
   condition. The owner is not a developer; this sentence and the next paragraph must
   stand alone for them.
2. The user and the pain, with the counts that prove the pain exists.
3. The chosen direction: what is deleted (name the files and directories), what is kept,
   what is new. Be specific enough that a builder could start.
4. The kill criterion and the copeca scenario from Part 2.4.
5. The first loop, in tend2's shape: goal, checks a verifier could close, and a `## Tried`
   seeded with what the previous sessions already tried and abandoned, so no one repeats it.
6. Why this might be wrong: the strongest case against your own verdict, in a paragraph.

Voice: calm, precise, literate. Product names lowercase (mull, tilth, tend2, pleach,
umbel, copeca, petals, plotplot). Sentence case. No exclamation marks. Numbers live in
tables, not prose. Never these words: supercharge, unlock, 10x, magic, synergy,
revolutionary, game-changing, cutting-edge, seamless, effortless, next-gen, AI-powered.

## Acceptance criteria

- Every code claim carries a file:line or a command with output. Every quality claim
  quotes an item.
- You ran mull at least twice on real data and reported the six run numbers and the four
  quality percentages for each.
- You diffed the parser against a Claude Code log written this month and inspected one log
  from each of Codex, Gemini CLI, and OpenCode.
- You read the four memory files and the strategic-findings doc, cited them, and did not
  redo their research.
- Every candidate in Part 2.3 has a filter verdict per line, a kill criterion, a copeca
  design, and a one-spark scope.
- The verdict is a decision.
- Spend is reported. `git status` shows only your two docs plus listed `.mull/` changes.
- Final message: both paths plus the verdict paragraph, nothing else.
