# The friction ledger — plan

Where the repository is hard, as numbers. Denials, failures, retries, re-reads, test loops,
edit churn, refused stops and compactions, counted per path from the hooks every harness
already fires, reduced into hot spots, and routed by tend2 into simplification loops. It is
the instrument that turns "agent confusion is diagnostic" from an anecdote into a
self-improving loop. Deterministic; nothing in it asks a model anything.

Status: plan, 2026-09-06. Owners as settled on the channel with tend2: the event profile is
a contracts deliverable; each bed emits its own domain's events; tend2 owns the reducer and
the hot spots in `next`. This plan adds one decision: the harness-session emitter ships in
the stem's bundles, because the stem is the thing installed in every harness, and tend2's
plugin emits only until the stem exists.

## 1. Why this, and why deterministic

Humans adapt to friction until they stop seeing it. Agents do not adapt: every place an
agent retries, re-reads the same file, loops on a test, or gets denied is a place where the
documented process and the real one diverge. Masuch's "The Repo Is the Harness" names agent
confusion as the diagnostic signal for where a codebase wants simplification, and names the
lack of visibility into it as his one open gap. mull spent 8,400 lines and 26,000 lines of
research learning that extracting this from transcripts with a model produces slop. The
harness hooks already carry every event needed, as structured JSON, with no prompt text
required. Counting is enough.

Non-goals, so the tool stays small: no dashboard, no OTLP exporter, no observability
product, no prompt or output capture, no cross-machine aggregation, no per-person metrics.
A file the repo ignores or commits, reduced on read.

## 2. Standards it stands on

- **OpenTelemetry GenAI semantic conventions** for attribute names, from the dedicated
  `semantic-conventions-genai` repository (all of it in Development status as of 2026-09;
  agent and tool conventions provisional). We use only names that exist: `gen_ai.operation.name`
  (`execute_tool`, `invoke_agent`, `chat`), `gen_ai.tool.name`, `gen_ai.tool.call.id`,
  `gen_ai.provider.name`, `gen_ai.request.model`, `gen_ai.conversation.id`,
  `gen_ai.conversation.compacted`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`,
  `gen_ai.usage.cache_read.input_tokens`, `error.type`. Everything of ours is under a
  `plotplot.*` namespace, which the conventions permit for custom attributes.
- **JSON Lines** as the file format, one event per line, UTC timestamps. No OTLP protobuf,
  no collector.
- **Harness hooks** as the only source: Claude Code, Gemini CLI, Codex CLI lifecycle hooks,
  whose payloads are documented and verified below.

## 3. The profile (contracts deliverable)

`contracts/friction-profile.md` pins which events the garden records and which attributes
each carries. One record shape:

```json
{"time":"2026-09-06T14:02:11.402Z",
 "event.name":"plotplot.friction",
 "plotplot.kind":"tool.failed",
 "plotplot.harness":"claude-code",
 "plotplot.harness.version":"2.1.261",
 "gen_ai.provider.name":"anthropic",
 "gen_ai.request.model":"claude-opus-5",
 "gen_ai.conversation.id":"4f0c…",
 "gen_ai.operation.name":"execute_tool",
 "gen_ai.tool.name":"Bash",
 "gen_ai.tool.call.id":"toolu_01…",
 "plotplot.path":"src/search/symbol.rs",
 "plotplot.path.kind":"prod",
 "plotplot.rule":null,
 "error.type":"exit_code:1",
 "plotplot.count":1,
 "plotplot.agent.id":null}
```

Required on every record: `time`, `event.name`, `plotplot.kind`, `plotplot.harness`,
`gen_ai.conversation.id`, `plotplot.count`. `plotplot.path` is required for every kind that
has a path and is always repo-relative. Never present: prompt text, tool output, file
content, user names, absolute home paths. `gen_ai.request.model` is recorded only when the
harness supplies it in the hook payload.

The kinds, and what produces each:

| `plotplot.kind` | Meaning | Derived from |
|---|---|---|
| `tool.denied` | a hook refused a tool call | our own PreToolUse decision (hedge rules, weed on commit); Claude `PermissionDenied` |
| `tool.failed` | a tool call errored | Claude `PostToolUseFailure`; Gemini `AfterTool` with an error in `tool_response`; Codex `PostToolUse` with non-zero exit |
| `tool.retry` | same tool, same input hash, within N calls | any harness, from consecutive `PreToolUse` payloads |
| `file.reread` | the same path read again in one session | `PostToolUse` for Read, `tilth_read`, `cat`/`sed` in Bash, path from `tool_input` |
| `search.fanout` | searches before the first edit of a session | `PostToolUse` for Grep, Glob, `tilth_search`, `rg` in Bash |
| `edit.churn` | the same path edited again in one session | `PostToolUse` for Edit, Write, `tilth_write` |
| `test.loop` | a test command run again in one session | Bash `tool_input.command` matching the test runners weed already knows |
| `stop.refused` | weed's Stop hook blocked a premature done | our own Stop decision |
| `context.compacted` | the context was compacted | Claude `PreCompact`; Gemini `PreCompress`; `gen_ai.conversation.compacted=true` |
| `session.ended` | a session ended, with totals | Claude `SessionEnd`, Gemini `SessionEnd`, Codex `SessionEnd` |
| `gate.retry` | a pleach node retried after a gate | pleach's journal (its own emitter, later) |
| `worker.wedged` | umbel detected a wedged worker | umbel's run journal (its own emitter, later) |

Each kind's attribute set is a table in the profile, and `contracts/test/friction-profile.test.sh`
validates a fixture journal against it.

## 4. Where the events come from, per harness

Verified against the vendors' hook references on 2026-09-06.

| Harness | Config | Events used | Fields used |
|---|---|---|---|
| Claude Code | project `.claude/settings.json` hooks, or the stem's plugin `hooks/hooks.json` | `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionDenied`, `Stop`, `PreCompact`, `SessionStart`, `SessionEnd` | `session_id`, `cwd`, `hook_event_name`, `tool_name`, `tool_input`, `tool_use_id`, `tool_error`, `agent_id`, `stop_hook_active`, `startup_reason`, `end_reason`, `model` (SessionStart, optional) |
| Gemini CLI | project `.gemini/settings.json` hooks, or the stem's extension `hooks/hooks.json` | `BeforeTool`, `AfterTool`, `AfterAgent`, `AfterModel`, `PreCompress`, `SessionStart`, `SessionEnd` | `session_id`, `cwd`, `hook_event_name`, `tool_name`, `tool_input`, `tool_response`, `llm_request.model`, `llm_response` (token usage when present), `trigger`, `source`, `reason` |
| Codex CLI | project or `~/.codex/hooks.json`, or the stem's plugin `hooks.json` | `PreToolUse`, `PostToolUse`, `Stop`, `SessionStart`, `SessionEnd` | `session_id`, `cwd`, `hook_event_name`, `turn_id`, `model`, tool name and command as Codex provides them |

Constraints that shape the emitter: Claude Code's `SessionEnd` hooks share a 1.5-second
budget, so the emitter never does more than append lines; Gemini's `AfterTool` can carry the
model's usage but only in `AfterModel`, so token totals are best-effort and never estimated;
Codex's payload names are the same as Claude's for the events it has, and its per-tool
fields are verified at build time against the version installed, since its documentation is
thinner.

## 5. The emitter

One face on the stem binary: `plotplot friction emit`, reading the harness payload on stdin,
deciding the kind from `hook_event_name` and the tool, and appending one line to
`.plotplot/friction/<yyyy-mm>.jsonl`. It exits 0 always; it never blocks a hook. Re-read,
churn, retry and test-loop kinds need session memory: a small per-session state file under
`.plotplot/friction/state/<session_id>.json` holding path counters and the last few input
hashes, pruned at `SessionEnd`. The emitter is registered by the stem's bundles on the events
above; until the stem exists, tend2's plugin hooks emit the same lines with the same
profile, and are removed when the bundle takes over.

Redaction is structural: the emitter copies only the fields the profile names. Paths are
made repo-relative or dropped; a path outside the repository becomes `plotplot.path.kind:
"outside"` with no value. Nothing else in the payload is read.

pleach and umbel emit their own domains from their own journals, in the same shape, into the
same directory, when they choose to; the profile is the only coupling.

## 6. The reducer (tend2)

`tend2 friction reduce [dir]` reads every journal, groups by `plotplot.path`, and computes a
friction score per path over a window (default 90 days, with older months weighted by half
per quarter):

```
score(path) = Σ_kind w_kind · count_kind(path) / sessions_touching(path)
```

with default weights: denied 5, refused stop 5, failed 3, test loop 3, churn 2, reread 1,
fanout 1, retry 2. Weights live in `.tend2/friction.toml` and are reported with the result.
The score is per session that touched the path, so a hot file is one that is hard every time,
not one that is merely touched often. Minimum support: three sessions.

Output: `.plotplot/friction/hotspots.json` (sorted paths with score, support, the kind
breakdown, and the window), committed or ignored as the repo chooses, and a one-line summary
in `tend2 next` beside needs-you:

```
hot spots (3)
  src/search/symbol.rs   friction 9.4 over 11 sessions — reread 31 · test loops 12 · churn 8
```

Shaping a simplification loop from a hot spot is `tend2 shape --from-friction <path>`, which
seeds the goal ("this path stops being hard") and one check ("friction for <path> falls below
<threshold> over the next N sessions"), which is how the loop closes on evidence.

## 7. Governance

- The ledger is never read by a model to produce facts; only the reducer reads it, and only
  a human or a shaping skill turns a hot spot into a loop.
- Weights are visible in every report; a change to weights is a by-law change through a PR.
- The journal directory is ignored by default; committing the reduced `hotspots.json` is a
  repo choice, and committing raw journals is discouraged (they name paths and models, not
  people, but they grow).
- A repo can opt out per path prefix in `.tend2/friction.toml` (vendored code, fixtures).

## 8. Checks (on the umbrella's `friction` loop; module checks in tend2's and the stem's maps)

- the profile exists in contracts with a fixture journal that validates, and every kind above
  has an attribute table;
- the emitter, fed each harness's documented payload for each event, appends exactly the
  profile's fields and nothing else, proven by fixture payloads for Claude Code, Gemini CLI
  and Codex;
- the emitter never writes prompt text, tool output or absolute home paths, proven by a
  fixture payload containing all three;
- `SessionEnd` emission completes under 200 ms on the fixture;
- the reducer, fed a fixture journal, ranks the path with the most denials and re-reads first
  with the documented weights, and `tend2 next` shows it;
- one hot spot named by the ledger on a garden repo is confirmed by the simplification loop it
  shaped (the kill criterion, human-owned).

## 9. Kill criteria

- If a hot spot the ledger names is not confirmed by the next simplification loop it shapes,
  the ledger is cut, per tend2's own plan.
- If the emitter cannot be made silent (any hook it is attached to fails or slows because of
  it), it is removed from that event.

## 10. Order of work

1. Profile and fixture journal in contracts (rides the contracts loop, step 1 of the garden).
2. Emitter as a face of the stem binary, registered by the stem's bundles; tend2's plugin
   emits the same lines meanwhile.
3. Reducer and hot spots in tend2.
4. `tend2 shape --from-friction`.
5. pleach and umbel emit their own domains when they choose.

## 11. Decisions for the owner

- Whether raw journals are ignored by default (recommended) or committed.
- The default weights, once a month of real journals exists to argue from.
- Whether the harness-session emitter lives in the stem (recommended, this plan) or stays in
  tend2's plugin; tend2 has agreed to the profile split and will be asked about this on the
  channel.
