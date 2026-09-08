# The garden in 2031

A scenario, not a forecast: what repositories look like five years out if the structural
forces already visible keep pushing, what the platforms will have absorbed, where tools like
ours prevail, the actual tool set spelled out, the long bets, and the first moves. Where a
claim is a guess it is marked as one. Companion to `plotplot-thesis.md` and
`tended-repository.md`.

## 1. The forces that decide the shape

- **Volume outruns attention.** Agents run for hours, fleets are normal, cost per token keeps
  falling. Software output grows faster than anyone can read it. Lehman's law holds: the rate
  of change is bounded by comprehension. Human review becomes the scarce resource, and every
  tool that makes verified work legible in less human time wins.
- **Vendors integrate vertically.** Each model vendor ships harness, memory, grader, and cloud
  runner as one product. Judgment stays inside their loop because that is the product.
- **Standards come from tiny files.** AGENTS.md, SKILL.md, MCP, A2A each won in months because
  they were small and obviously useful. Expect a cross-harness lifecycle-hook standard (the
  three major CLIs already converged on Pre and Post naming) and an agent config directory
  standard (`.agents/` is the likely winner).
- **Provenance becomes compliance.** The EU AI Act and Cyber Resilience Act obligations land
  in 2026 and 2027. "What wrote this change, under what verification" stops being a nice
  idea and becomes a field an auditor asks for.
- **The standards for the boring parts already exist.** SARIF for findings, consumed natively
  by GitHub code scanning and every editor. in-toto attestations signed with Sigstore for
  provenance, which GitHub already issues for artifacts. OpenTelemetry's GenAI conventions
  for agent telemetry. Branch protection rulesets, CODEOWNERS, and git hooks for law. Dev
  Containers and Nix for environments. A tool that invents a rival to any of these loses.

## 2. What a repository looks like

```
repo/
  AGENTS.md              constitution and pointers: who writes pass, hard limits, gates, commands
  .agents/
    skills/              engineered units of process, composable, measured
    hooks/               lifecycle hooks in the cross-harness shape
    rules/               the judge's configuration
  .githooks/             pre-commit and pre-push: the law, live from the first session
  docs/loops/            intent with proof: goal, checks, tried; stamps carry evidence SHA and verifier version
  docs/decisions/        the theory: why it is as it is, what was rejected
  .brand/                design and voice as files
  garden.lock            pinned judges: tool, version, checksum per platform
  evals/                 the repo's own agent-eval corpus and results
  .github/workflows/     the scheduler: garden check on PRs, hygiene loops on a cadence, attestations
```

And how it is run, on an ordinary day:

- A pull request arrives, authored by an agent on any vendor, from a worktree or a cloud
  runner. Required checks run: the test suite; the **garden check**, which posts SARIF
  findings (a deleted test, a stub, a dependency crossing a boundary) as code-scanning alerts;
  the **attestation check**, which requires a signed statement of which harness and model
  produced the change and which gates it passed; and the **loop check**, which requires the
  claims the PR closes to carry verifier stamps.
- The human reads a verified summary and a short needs-you list, not the diff. They approve
  the human-method checks, or refute them. Their job is conceptual integrity and judgment.
- Nightly, hygiene loops run in the scheduler: doc anchors that no longer resolve, flaky
  tests by CI history, dependency age, loops gone stale, friction hot spots. Each finding
  shapes a loop; agents work them; the same gates decide.
- Quarterly, the repo re-casts: its own eval corpus is run against current models and tools,
  and the casting file says which hands do which kind of work.

The vendor of the model is a line in an attestation. Nothing in the repo depends on it.

## 3. Did GitHub make the harness the repo?

Mostly, and predictably. (A guess with high confidence about direction, low about detail.)
Actions is already the scheduler; rulesets are already the law at push; code scanning already
consumes SARIF; artifact attestations already exist; Agent HQ already aims to run several
vendors' agents from one place. The incentive is clear: keep the repo on GitHub, sell minutes
and seats. Expect native "AI change review" rulesets and repo-scoped agent memory within the
window. GitLab follows; Forgejo and Gitea serve the self-hosted.

What the platform does not do, structurally:

- **Enforce at the moment of the agent's action.** Platform law runs at push. The commit
  hook, the Stop hook, and the sandbox run inside the agent's loop, locally, before anything
  leaves the machine. That layer stays local by nature.
- **Measure neutrally.** A platform with its own coding agent will not publish a fair number
  about a competitor's. Neutral measurement stays with a third party or does not exist.
- **Hold intent as a fitness function.** Issues are conversations. A loop with checks only a
  verifier can close is a different object, and a platform has weak incentive to make issues
  that strict.
- **Serve people off the platform.** Regulated, self-hosted, air-gapped, and the maintainer
  who wants ownership.

## 4. Did tools like ours prevail, and how?

Three outcomes, and all three will happen to different pieces:

- **Absorbed.** The obvious deterministic checks (test tampering, stubs, secrets) become
  native platform features. Our versions become the reference implementations that nudged the
  shape, and survive as the local-first, neutral option. Reputation compounds; tilth's arc
  is the template. High probability for weeder's static rules.
- **Standardized.** One tiny piece becomes a convention harnesses adopt, the way AGENTS.md
  did. The candidate is the **stamp**: a checkbox grammar for verified claims,
  `[x] claim · evidence @sha · by verifier@version`, renderable by any markdown viewer,
  writable by any verifier. Low cost to propose, one page, and it needs one large adopter.
  Modest probability, outsized payoff.
- **Durable independents.** Tools that are measured and neutral survive as their own things,
  the way ripgrep and tree-sitter did. copeca, if it becomes the way teams choose models on
  their own repos, is the best candidate. tilth survives by becoming the parsing substrate
  for judgment as cheap context makes pure navigation less scarce.

How they prevail, in practice: by being the neutral reference implementation, by running
where the platform cannot (inside the loop, offline, off-platform), by speaking the existing
standards so they interoperate with everything rather than competing with it, and by carrying
their own measurements.

## 5. The tools, spelled out

What, how, why. Each leans on a standard rather than inventing one.

| Tool | What | How | Why |
|---|---|---|---|
| **tend2** | intent with proof: loops of goal, checks, tried; state derived; stamps only a verifier writes | markdown in a thin HTML shell, rendered from disk; a verifier CLI; stamps carry evidence SHA and verifier version and may be emitted as in-toto statements; two-way sync with issues | the fitness function must live in the repo and outlive the vendor; issues are not fitness functions |
| **weeder** | the judge of the diff: deleted or weakened tests, skips, stubs, swallowed errors, secrets, guardrail edits, dependency-direction violations; `bite` proves a test fails without the change; `guard` protects history and remotes | a static binary in the tilth workspace; runs at pre-commit, pre-push, Stop, and in CI; **emits SARIF** | honesty of work is checkable by code; the platform cannot check inside the loop; SARIF makes every finding native in GitHub and every editor |
| **tilth** | structural reading of code for agents, and the tree-sitter substrate weeder judges with | binary, CLI and MCP, skill; measured by cost per correct | reading by syntax tree beats grep on a measured number; as context gets cheap its value shifts to being the eye of the judge |
| **petals** | brand and voice as files, checked deterministically | a skill plus a check that emits SARIF | design truth agents read before generating, and a gate after |
| **umbel** | one unit of agent work on any provider, with a typed result | adapters: tmux today, headless structured output as vendors ship it; the unit contract | no vendor can run its competitors; someone neutral must |
| **pleach** | a DAG of verified nodes in worktrees, gated, audited by a different provider, published only when verified | git worktrees; gates are weeder, tests, markers; ledger is tend2; runnable in Actions; the test phase commits separately | fleets need gating at the node, before the PR |
| **pollen** | agent-to-agent messages with a human at the gate | file mailboxes, a relay, MCP with three tools | fleets need lateral talk; the local Unix version of A2A |
| **copeca** | cost per correct answer with confidence intervals on fixed axes, signed | runs harness CLIs on a task corpus; `copeca init` scaffolds a repo's own corpus from its history | measurement is how slices earn their keep and how teams choose hands; nobody neutral does it |
| **friction ledger** | where the repo is hard, as numbers: denials, retries, re-reads, test loops, churn per path | hook events recorded in OpenTelemetry GenAI shape, reduced per path, routed by tend2 into simplification loops | the instrument that turns "agent confusion is diagnostic" into a self-improving loop; deterministic, never model-extracted |
| **receipts** | what produced a change and what verified it | in-toto attestations signed with Sigstore, attached to commits and PRs | provenance is compliance now and trust always; proof outlives the vendor |
| **the stem** | one way to plant all of it in any harness, and to prove it is planted | generated bundles per harness, `doctor`, `check`, and the lockfile wrapper that fetches pinned judges | nothing to install, judges pinned, holes detected when they move |

What is deliberately absent: an app, a proxy, an MCP gateway, model-extracted memory, an
orchestration UI. Each was argued and cut in the review.

## 6. The long bets

1. **Repo-owned proof beats vendor-owned judgment.** Portability, compliance, and the
   multi-vendor reality all push the same way. The stamp and the attestation are the bet.
2. **Deterministic honesty gates become required checks everywhere.** The flood of
   agent-authored pull requests forces it on maintainers first. weeder, SARIF-native, local and
   in CI, is the bet; the risk is native absorption, and the answer is running inside the loop
   and staying neutral.
3. **Neutral measurement becomes how teams buy.** Cost per correct on your own repo, per model
   and per tool, is a procurement number. copeca as every repo's eval harness is the bet.
4. **Friction telemetry is the self-improvement primitive.** Small, novel, and the only thing
   here nobody else is building. The bet with the best ratio of leverage to cost.
5. **The stamp as a micro-standard.** One page, one adopter needed. Cheap to lose.
6. **Local-first survives** because ownership is a requirement for a durable minority:
   regulated, self-hosted, and anyone who has been burned by a vendor's memory vanishing.

And the bet against: that pure code navigation stays scarce. As context grows and cheapens,
tilth's reader value erodes; its parser value does not. Reallocate accordingly.

## 7. How to start building toward it

In order, each step usable on its own:

1. **Adopt the standards on day one.** weeder emits SARIF. receipts are in-toto statements.
   friction events use the OpenTelemetry GenAI shape. No new schema where one exists.
2. **Ship the three unshipped beds.** tend2 to npm, petals and pleach public. The loop must
   be installable before anything else is credible.
3. **Build weeder** in the tilth workspace: static rules to SARIF first, `guard` via git hooks
   second, `bite` via pleach's separate-test-commit policy third. Its first wedge is a GitHub
   Action, "garden check", posting SARIF to code scanning for maintainers drowning in agent
   PRs. That is distribution the platform pays for.
4. **Harden the proof.** Verifier version in tend2 stamps; Tried as a machine-checked output
   of every pleach work order; publish the one-page tended-repository spec.
5. **Friction ledger MVP** in tend2: a hook script that records events, a reducer that ranks
   paths, hot spots in `tend2 next`.
6. **The stem as bundles plus lockfile wrapper**, with `doctor` proving hooks fire.
7. **copeca init**: scaffold a repo's own eval corpus from its history, so any team can run
   the yardstick on their code.
8. **Dogfood on the garden's own repos**, publish the numbers, and let the site tell the loop
   with real output.

Kill criteria travel with each step: weeder ships only under two percent false blocks on real
history; the friction ledger ships only if a hot spot it names is confirmed by the next
simplification loop; the stamp proposal is dropped if no adopter appears in a season.

## 8. What this scenario could get wrong

That platforms move faster into the inner loop than assumed, with local agents that carry
platform law offline. That context becomes so cheap that structural reading and even
friction telemetry matter less than raw model quality. That maintainers accept model review
as sufficient and never demand deterministic gates. That the compliance tailwind is weaker
than expected. Each of these would shrink a bet, not invalidate the shape: a repository that
carries its own intent, law, proof, and yardstick is a better repository under every one of
those futures too.
