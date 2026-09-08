# the friction profile

Pinned once, for the whole garden: which events the friction ledger records, which
attributes each carries, and the envelope every append-only stream in the garden writes,
per plotplot-ai issue 35. Not a schema of our own. Attribute names come from the
OpenTelemetry GenAI semantic conventions where they exist; everything specific to this
garden lives under a `plotplot.*` namespace, which the conventions permit. Source of
record: `docs/plans/friction-ledger.md` §3 to §5.

## the envelope (issue 35)

Every append-only stream a bed writes carries this envelope, additively: one JSON object
per line, UTF-8, one line per event, UTC time, and the four keys below present on every
line regardless of which bed or domain wrote it. A stream's own fields are never removed or
renamed to adopt the envelope; the envelope is added on top.

Universal, on every line of every stream:

| key | type | rule |
|---|---|---|
| `time` | string | UTC timestamp, RFC 3339 / ISO 8601, must end in `Z` (`2026-09-06T14:02:11.402Z`) |
| `event.name` | string | the event's own dotted name; a stream that already has an event field of its own (pleach's `event`) keeps it and adds `event.name` beside it, namespaced by source (`pleach.gate-retry`) |
| `plotplot.kind` | string | one of the pinned kinds below; a domain not yet in the list proposes an addition to this file rather than inventing its own attribute |
| `plotplot.count` | integer >= 1 | how many occurrences this line represents; 1 for a single event |

`plotplot.harness` and `gen_ai.conversation.id` are part of the envelope's required keys
on every line, but their value is `null` when the domain that wrote the line has no
harness or no single conversation to name (a pleach gate retry spans a node, not one
harness turn). The key is never omitted; a consumer that only checks for the key's
presence never has to special-case a stream. This mirrors the friction emitter's own
`plotplot.rule: null` / `plotplot.agent.id: null` convention in the example below, applied
one level up, to the envelope itself.

## the friction record shape

The friction emitter's own records carry the full set the friction ledger plan names,
which is the envelope above plus the fields below.

```json
{"time":"2026-09-06T14:02:11.402Z",
 "event.name":"plotplot.friction",
 "plotplot.kind":"tool.failed",
 "plotplot.harness":"claude-code",
 "plotplot.harness.version":"2.1.261",
 "gen_ai.provider.name":"anthropic",
 "gen_ai.request.model":"claude-opus-5",
 "gen_ai.conversation.id":"4f0c...",
 "gen_ai.operation.name":"execute_tool",
 "gen_ai.tool.name":"Bash",
 "gen_ai.tool.call.id":"toolu_01...",
 "plotplot.path":"src/search/symbol.rs",
 "plotplot.path.kind":"prod",
 "plotplot.rule":null,
 "error.type":"exit_code:1",
 "plotplot.count":1,
 "plotplot.agent.id":null}
```

Required on every friction record, beyond the envelope: `plotplot.harness` (non-null),
`gen_ai.conversation.id` (non-null). `gen_ai.request.model` is recorded only when the
harness supplies it in the hook payload; when absent the key is omitted, not nulled,
because "the harness never told us" and "the harness told us there is none" are different
facts and only the first happens in practice.

## the one required path attribute

There is exactly one attribute for a path anywhere in the garden's telemetry:
`plotplot.path`, always repo-relative, forward-slash separated, never absolute and never
outside the repository. A path outside the repository is not recorded as a value; the
record instead carries `plotplot.path.kind: "outside"` and omits `plotplot.path`. No
domain invents a second path-shaped attribute (`plotplot.file`, `plotplot.test_path`,
and similar are refused in review).

`plotplot.path` (and its companion `plotplot.path.kind`) is required, non-null, for the
kinds marked "yes" in the path column below, because those kinds are defined in terms of
one path. It is not present at all for kinds defined at the level of a session, a node, or
a worker, because forcing a path onto those would invent one.

## the pinned kinds

| `plotplot.kind` | meaning | derived from | path required |
|---|---|---|---|
| `tool.denied` | a hook refused a tool call | our own PreToolUse decision; Claude `PermissionDenied` | yes |
| `tool.failed` | a tool call errored | Claude `PostToolUseFailure`; Gemini `AfterTool` error; Codex `PostToolUse` non-zero exit | yes |
| `tool.retry` | same tool, same input hash, within N calls | consecutive `PreToolUse` payloads | no |
| `file.reread` | the same path read again in one session | `PostToolUse` for Read, `tilth_read`, `cat`/`sed` in Bash | yes |
| `search.fanout` | searches before the first edit of a session | `PostToolUse` for Grep, Glob, `tilth_search`, `rg` in Bash | no |
| `edit.churn` | the same path edited again in one session | `PostToolUse` for Edit, Write, `tilth_write` | yes |
| `test.loop` | a test command run again in one session | Bash `tool_input.command` matching a known test runner | yes |
| `stop.refused` | weeder's Stop hook blocked a premature done | our own Stop decision | no |
| `context.compacted` | the context was compacted | Claude `PreCompact`; Gemini `PreCompress` | no |
| `session.ended` | a session ended, with totals | Claude/Gemini/Codex `SessionEnd` | no |
| `gate.retry` | a pleach node retried after a gate | pleach's own journal (`event: "gate-retry"`) | no |
| `worker.wedged` | umbel detected a wedged worker | umbel's own run journal | no |
| `model.call` | one billed model call | mull's spend log (plotplot-ai#33), per issue 35 | no |

Adding a fourteenth kind is a change to this table, in a pull request, never a private
attribute invented by one bed.

## per-kind attributes beyond the envelope

| kind | required beyond the envelope |
|---|---|
| `tool.denied` | `gen_ai.operation.name`, `gen_ai.tool.name` |
| `tool.failed` | `gen_ai.operation.name`, `gen_ai.tool.name`, `error.type` |
| `tool.retry` | `gen_ai.tool.name`, `gen_ai.tool.call.id` |
| `file.reread` | `gen_ai.tool.name` |
| `search.fanout` | `gen_ai.tool.name` |
| `edit.churn` | `gen_ai.tool.name` |
| `test.loop` | `gen_ai.tool.name` |
| `stop.refused` | `plotplot.rule` (may be `null`) |
| `context.compacted` | `gen_ai.conversation.compacted` |
| `session.ended` | `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens` (either may be `null`; never estimated) |
| `gate.retry` | `plotplot.node`, `plotplot.gate` |
| `worker.wedged` | `plotplot.worker` |
| `model.call` | `gen_ai.provider.name`, `gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens` |

## never present

Prompt text, tool output, file content, user names, absolute home paths. A path outside
the repository is redacted to `plotplot.path.kind: "outside"` as above, never recorded by
value.

## the fixture journal

`contracts/fixtures/friction.jsonl` holds three lines, one JSON object each, in this
order, per issue 35:

1. a friction event (`plotplot.kind: "tool.failed"`), the emitter's own shape;
2. a pleach run-journal line (`plotplot.kind: "gate.retry"`), pleach's existing `event`
   field kept verbatim, the envelope's keys added beside it, authored from pleach's own
   `docs/journal.md` and the literal `journal.append({ event: 'gate-retry', node, gate })`
   call in `src/loop/run-node.ts`, since pleach has not yet landed the envelope change
   (jahala/pleach#60);
3. a spend line (`plotplot.kind: "model.call"`), mull's spend log shape born in the
   envelope per issue 35, authored ahead of mull's own `spend.jsonl` landing
   (plotplot-ai#33).

`contracts/test/friction-profile.test.sh` parses the fixture, checks it is one JSON object
per line, checks every line's `plotplot.kind` is in the pinned list above, checks every
line carries the envelope's required keys with the right types, checks every `time` value
parses as UTC and ends in `Z`, and checks the per-kind table's additional required
attributes for each line's kind.
