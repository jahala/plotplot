# How the garden works together, and what it costs an agent to know it exists

Status: architecture memo (September 2026). Companion to `garden-direction-2026-09.md`.
Numbers marked **measured** were taken on this machine on 2026-09-05 by querying each MCP
server's `initialize` and `tools/list` over stdio, or by counting source. Numbers marked
**≈** are guesstimates at about four characters per token. Treat every ≈ as a planning
figure, not a fact.

## 0. The shape, long term

Three shapes were on the table: separate CLIs with MCP servers, a unified app in the manner
of Conductor, or a proxy that intercepts model traffic and injects context. The answer is
none of the three as stated.

**The substrate is git, files, and harness hooks.** Every harness already reads AGENTS.md
and SKILL.md, runs lifecycle hooks, and commits to git. Those three seams are the most
stable, the most neutral, and the cheapest in context. Hooks in particular can do inside the
harness what a proxy wants to do outside it: inject context at SessionStart, deny a tool
call at PreToolUse, refuse a premature "done" at Stop, write a receipt at SessionEnd. They
work with subscription-billed CLIs, which a base-URL proxy cannot see, and they can block,
which a proxy cannot cleanly do. Hooks are the proxy that works.

**MCP is for channels only.** umbel's spawn and wait, pollen's inbox, tend2's write and
verify pens. Capabilities are CLIs behind skills.

**One stem.** Long term a user installs one thing, `plotplot`. It plants the beds as hooks,
CLIs and skills, keeps the garden block in AGENTS.md true, assembles the session briefing,
runs the checks, conducts a plan through pleach and umbel, and renders the walk. The beds
stay separately installable, but the default experience is one command and one version. The
garden is unified at the CLI and the file formats, never at a UI.

**The human surface is a document, not an app.** tend2's loop pages and the cross-repo walk
page, rendered from files at `file://` or by a local serve. Conductor stays the place where
sessions run; plotplot rides inside every session it starts. Complementary, not competing.

**The proxy idea survives as a repo, not a server.** Receipts and journals are files. Push
them to a shared repo and the "convergence store" the proxy wanted exists without touching
model traffic, with a privacy story anyone can read: you share what you commit. An egress
proxy is the enterprise variant, if ever, and it is not this garden.

Three horizons. Now: beds as separate repos, hooks as the primary seam, the trowel's first
spark, copeca proving each step. Next: the trowel as the daily driver, the loop end to end
in one command, one version for the garden. Later, only with a team: the receipts repo and
the org-level view built on it.

## 1. The loop, as one system

Who calls whom, and which contract crosses each seam:

```
 human ── shape ──▶ tend2 loop file  ◀── verify stamps pass  ◀── evidence (tests, judge, human)
                        │ emit-plan
                        ▼
                    pleach plan ──▶ pleach run ──▶ umbel spawn · send · wait ──▶ agent (any CLI)
                        ▲                │  gates: conflict markers · weed (smoke) · audit (other provider)
                        │                ▼
                        └──── node/<id> branch, published only when verified

 what the agent reads      tilth (code) · petals (brand) · graft-checked docs · the loop page
 what stops the agent      hedge (hooks, both directions) · weed (Stop and PreToolUse)
 who the agent talks to    pollen (peers, human at the gate) · umbel (its supervisor)
 what proves any of it     copeca, an A/B around any one bed, cost per correct answer
 what is remembered        the loop's Tried section (mull as its writer, if mull survives)
```

The contracts are the garden. Nine of them exist or are drafted: the loop format (tend2
`FORMAT.md`), the plan schema (pleach), the worker signature (umbel), the harness hook JSON
(Claude Code, Gemini, Codex), weed's findings JSON, the `.brand/` tree (petals), the
`.copeca` artifact, pollen's message envelope, and hedge's rules file. The trowel, the
`plotplot` CLI, is the one program that knows all nine. Everything below follows from
treating the trowel as the garden's context policy in code, not as an installer.

## 2. The context tax, measured

What each server pushes into an agent's window before the first prompt:

| Bed | Tools | Instructions | Tool schemas | Startup total | Basis |
|---|---|---|---|---|---|
| tilth | 7 | ≈ 480 tok | ≈ 1,950 tok for 5 of 7 counted from source; ≈ 2,500 for all 7 | ≈ 2,900 tok | measured from `src/mcp.rs` and this session's system prompt |
| umbel | 12 | 260 tok | 1,403 tok | 1,663 tok | measured |
| tend2 server | 6 | 0 | 654 tok | 654 tok | measured |
| tend2 SessionStart hook | | | | 807 tok on a 31-loop map | measured; grows with the needs-you list |
| walkie, future pollen | 5 | 193 tok | 226 tok | 419 tok | measured |
| petals skill | | one description line | | ≈ 175 tok | measured, 700 chars |
| tend2 plugin skills | 8 | eight description lines | | ≈ 500 tok | measured, ≈ 250 chars each |
| garden practice skills | ≈ 10 | ten description lines | | ≈ 625 tok | measured |
| mull server, if revived | ≈ 5 | | | ≈ 600 tok | ≈ |
| hedge, weed, graft, pleach, copeca | 0 | | | 0 | hooks and CLIs carry no schema |

Fully planted, no deferral, on one harness: **≈ 8,300 tokens** before the agent reads a word
of the task. Add the harness's own system prompt and built-in tools (≈ 15,000 to 20,000 on
Claude Code) and a project CLAUDE.md (≈ 1,500 here). Codex renders MCP tool definitions
fatter, about 550 to 1,400 tokens per tool by its own community's measurements, so the same
28 garden tools there are **≈ 15,000 to 40,000 tokens** per turn.

Three things make this worse than the raw count suggests:

- **It is paid every turn.** Prompt caching makes a static prefix cheap to re-read, but the
  prefix is rewritten at full price whenever anything in it changes: a new hook injection, a
  tool list that differs between sessions, a longer needs-you list. tilth's own cost work
  found cache writes dominate Haiku spend.
- **It displaces working memory.** Every token of catalogue is a token the agent cannot
  spend on the code it is reading, and it brings compaction forward. mull's CLAUDE.md rule
  six exists because compaction erases what the agent knew.
- **It dilutes choice.** A model choosing among sixty tools picks worse than one choosing
  among eight. Anthropic shipped ToolSearch because of this, and the field's own study
  found LLM-written context files lower task success while raising cost.

What the harnesses do about it today: Claude Code defers every MCP tool by default, loading
names and server instructions at start and schemas on demand, with an auto mode that
switches on above ten percent of the window. This session shows it: tilth's and umbel's
schemas were deferred; their instruction blocks were not. Codex loads full schemas each
turn as of mid-2026. Gemini's behaviour I could not verify. So the garden's answer must be
harness-neutral, because the bill differs by an order of magnitude between harnesses.

## 3. The doctrine: context by when it is needed, not by what is installed

Four tiers. Every bed, existing or future, declares which tier each piece of its context
belongs to.

**Tier 0, always present.** At most one line per bed, and ideally one block for the whole
garden: a ten-line section in AGENTS.md or CLAUDE.md that names what is planted, the one
orientation command, and the one help command. ≈ 150 tokens for the entire garden. No tool
schemas. The trowel writes this block and keeps it true.

**Tier 1, on demand.** Skill bodies, `--help`, a `describe` call. This is where ninety
percent of today's upfront text belongs: tilth's instruction block, umbel's help topics, the
practice skills' procedures. SKILL.md already works this way on thirty-plus harnesses: one
description line upfront, the body read when invoked. It is the garden's own principle,
files not servers, applied to context.

**Tier 2, event-driven.** Hooks that inject only when a deterministic condition says the
information is relevant now. tend2's SessionStart hook is the model: a repo with a map gets
its routing; every other repo gets silence. weed speaks at Stop only when it found
something. hedge speaks at PreToolUse only to deny, with a reason. Zero cost when quiet.
The rule from tilth's scout work applies: a hint that fires on an easy task costs more than
it saves, so hooks fire on evidence, never by default.

**Tier 3, never.** Full tool catalogues. Prose that duplicates a skill. A per-bed "how to
use me" block. Anything that scales with the number of beds rather than with the state of
the work.

The mechanism follows from the tier:

- **MCP only for channels.** Stateful, conversational surfaces that need push or a session:
  umbel's spawn, send, wait; pollen's inbox; tend2's write and verify pens. Even these keep
  schemas terse, collapse rarely used verbs behind one help or inspect tool, and rely on
  harness deferral where it exists.
- **CLI plus skill for capabilities.** Anything request-response. This is a question for
  tilth, the largest server: does tilth as a CLI behind a skill lose the cost-per-correct
  gain it measured as an MCP server? Nobody knows. copeca can answer it in one scenario, and
  the answer decides the pattern for every future capability bed.
- **Hooks for anything the agent should not have to remember.** hedge, weed, graft,
  orientation. These are the beds that cost nothing at startup by construction.

## 4. Greater intelligence: assemble the briefing from state

The static wall of instructions is the least intelligent possible use of the first two
thousand tokens. The alternative already exists in miniature in tend2's hook: compute what
this session needs from files, deterministically, and inject only that.

The trowel's `plotplot orient`, run as a SessionStart hook and again at PreCompact, would
assemble one briefing of ≈ 300 to 800 tokens:

- the routed next action and the loop page path, from tend2, if a map exists;
- a tilth outline of the files the loop names as evidence, if there are few;
- the brand's one-paragraph summary only if the loop touches UI or copy files;
- hedge's active rules in one line only if permissions are in bypass mode;
- weed's findings from the last Stop, if any remain;
- what the previous session tried and abandoned, from the Tried section.

Nothing about which tools are installed. The briefing is a function of the repo's state,
so it stays small in a small repo and grows only with real complexity. It is re-injected
after compaction, which closes the hole mull's rule six describes. And it is measurable: a
copeca scenario with the briefing versus without, on the same tasks, is the ship or kill
test. The prior from tilth's scout arc is sobering: a correct hint saved about twenty percent
on hard tasks and cost about nine percent on easy ones. So orient must fire on evidence,
such as a failing loop or a non-trivial map, and stay silent on a fresh repo.

For harnesses that do not defer schemas, the trowel adds a second face: `plotplot mcp`, one
gateway server with three tools, `garden_help(topic)`, `garden_describe(tool)`, and
`garden_call(tool, args)`, fronting every garden server. ≈ 400 tokens fixed regardless of
how many beds exist. It is Anthropic's code-execution-with-MCP pattern, where tools are a
directory the agent reads on demand, without the sandbox that pattern requires. The known
cost is one indirection and the model losing typed schemas, which `describe` gives back on
request. On Claude Code the gateway is unnecessary; ToolSearch already does this.

## 5. What this changes for each bed

| Bed | Today | Change | After |
|---|---|---|---|
| tilth | ≈ 2,900 tok as MCP | shorten the instruction block to a pointer; A/B CLI-plus-skill against MCP with copeca before deciding | ≈ 600 deferred, ≈ 150 as a skill line |
| umbel | 12 tools, 1,663 tok | fold capture, logs, actions, diff, status, ls behind one inspect tool; keep help | ≈ 6 tools, ≈ 800 tok |
| tend2 | 654 tok plus 807 injected | cap the injection at next-up plus a needs-you count with a command to list the rest | 654 plus ≈ 300 |
| pollen | 5 tools, 419 tok | collapse allow and deny into one gate tool; three tools total | ≈ 250 tok |
| petals | 700-char skill description | cut the description to one sentence; the body already loads on demand | ≈ 60 tok |
| practice skills | ten descriptions, ≈ 625 tok | one `plotplot` skill with topics; the procedures become its body | ≈ 200 tok |
| hedge, weed, graft | not built | hooks and CLIs; zero upfront by design | 0 |
| mull | dormant | batch process, no server; if it lives, it writes files | 0 |
| copeca, pleach | CLIs | unchanged | 0 |
| trowel | not built | garden block, orient hook, gateway for non-deferring harnesses | ≈ 400 |

Planning totals, all ≈: the fully planted garden goes from about 8,300 tokens upfront to
about 1,900 on a deferring harness and about 4,000 on a non-deferring one, and future beds
add nothing unless they are channels. The orientation briefing replaces most of what was
cut with 300 to 800 tokens that are about this repo and this session.

## 6. The rule for every future bed

Before a bed is planted it declares: is it a channel or a capability; which tier each piece
of its context sits in; and its startup cost in tokens on a deferring and a non-deferring
harness. A capability ships as a CLI with a skill and costs nothing at startup. A channel
ships as an MCP server with at most three tools and one help topic. Anything that wants to
speak at startup does so through the trowel's orient hook, on evidence, and proves itself
with a copeca scenario or does not speak.

## 7. Order of work

1. Diet what exists: tilth's instruction block, umbel's tool count, tend2's injection cap,
   pollen's tool count, the skill descriptions. Cheap, and each is measurable on its own.
2. The trowel, first spark: the garden block writer, `plotplot check`, and `plotplot orient`
   as a SessionStart and PreCompact hook that fires on evidence.
3. The copeca scenario for the context tax: same tasks, garden loaded three ways, cost per
   correct. This decides tilth's shape and whether orient ships.
4. The gateway face, only for harnesses that still load every schema by then.
5. hedge and weed as designed; they already obey the rule.

## What I could not verify

- tilth's served `tools/list`: the stdio probe was interpreted as a search query, so the
  seven-tool total is counted from source for five tools and estimated for two.
- Gemini CLI's schema loading behaviour.
- Codex per-tool token figures come from a community knowledge base, not from a probe.
- Claude Code's own baseline system prompt size is an estimate from this session.
