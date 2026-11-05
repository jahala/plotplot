# gate folds into hedge

2026-11-05. The owner's decision, after the umbrella agent traced why gate was a tool of its own.

## What the owner said

Asked why gate was a separate tool, the umbrella agent found that it had proposed the split itself
on 2026-10-26, under its own rule of one owner per layer. The owner's only words then were "your
queue - ok .. i assume these are coming from the almanac and other places (maybe mycelium/rill?)".
On 2026-10-27 the owner asked whether "hedge and well belong together". On 2026-11-05, given the
proposal below, the owner said: "ok, do that -- plan it out and effectuate".

## The decision

gate stops being a tool. Its three jobs move to the homes that already need them:

| Job | Was | Now |
|---|---|---|
| Keep requests for the owner's yes and their answers; expire to no; answer "does a yes hold for this exact act" | gate | hedge |
| Show the owner what waits and take the answer | `gate list` and `gate answer`, a terminal | the stem's `plotplot status`, and later conducted's question cards; hedge supplies the data and the answer command |
| The request and answer format | `contracts/request.md` | unchanged, now between the asking tools and hedge |

## Why

- hedge is the only tool that asks whether a yes holds, because it is the one that blocks the
  act. Split across two tools, nothing stopped an agent from answering its own request as the
  owner (jahala/gate issue 9, jahala/hedge issue 37). In one tool, the answer path and the check
  live together and are guarded together.
- A terminal command was never how the owner works. The owner's view of what waits belongs to the
  owner's screen.
- gate had answered no real request: its folder did not exist on the owner's machine.

## What follows, in order

1. hedge takes over the request store, `ask`, `holds`, `list`, `answer`, `expire`, `prune` and
   the tend2 source, porting gate's code as prior art, with the owner-only answer path from the
   start (jahala/hedge issue, linked from umbrella issue for this decision).
2. pollen's owner-yes edge (`src/edges/gate.ts`) calls hedge instead.
3. Only then is jahala/gate archived, and its install removed from the owner's machine.
4. gate leaves the brand's bed list and plotplot.ai's list in the same change as this decision.

Supersedes the gate row of `2026-10-26-layers.md`.
