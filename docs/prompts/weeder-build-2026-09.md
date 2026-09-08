# weeder — build brief

Renamed from weed on 2026-09-06; the tool, its rules and its numbers are unchanged.

Brief for the agent building weeder. Self-contained: you have not seen any prior conversation.
Read in this order before writing code: `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/building-the-garden.md`
(the law every bed agent follows), `~/conductor/workspaces/plotplot-ai/cape-town-v1/docs/tend2/weeder.tend2.html`
(your loop on the umbrella map: the fit checks the umbrella will verify), then this brief.
Where this brief and the loop differ, the loop wins; where either differs from the brief in
`building-the-garden.md`, that brief wins. Revised 2026-09-05 after the architecture review
and the settlement with tend2; the revisions are listed at the end.

## What weeder is

weeder is the judge of the diff. A static binary that reads what an agent produced and refuses
the growth that should not be there: deleted or weakened tests, skips, stubs, swallowed
errors, secrets, guardrail edits, dependency-direction violations. It runs in milliseconds
with zero tokens. Agents produce; weeder decides.

Three faces on one core:

- `weeder check` judges a **diff** and may block. Pre-commit, Stop hook, pleach's smoke gate, CI.
- `weeder scan` judges the **tree** and never blocks. Repo-state hygiene: a cited path or symbol
  that no longer exists, dead exports, TODO age, dependency lag. tend2's hygiene loop is its
  first consumer.
- `weeder guard` is the **law** in git: hooks installed through `core.hooksPath` that run
  `check` at commit and push and refuse history rewrites and non-fast-forward pushes to
  protected branches.

Plus `weeder bite`, the dynamic rule, once pleach commits the test phase separately: apply the
test commit alone, the test must fail; apply the implementation, it must pass.

Every finding is **SARIF 2.1.0**. There is no findings schema of our own; GitHub code
scanning, every editor, and every CI already read SARIF.

## Posture and guardrails

- Execute the work yourself; no subagents. Report back over pollen to `cape-town` (the
  umbrella) at each milestone named in §8, and in your final message: what was built, test
  output, calibration numbers, the fit checks you believe pass and the command proving each,
  and the decisions waiting for the owner.
- The failing test is the spec. Every rule lands as a failing fixture first, in every listed
  language. Tests drive the real binary against real git repositories in temp dirs. No mocks,
  no theater tests.
- Delete with `trash`, never `rm`. Never `git reset`. Never touch the git stash. Commit on a
  branch in the tilth workspace as you go; do not push, do not open PRs, do not publish, do
  not create repositories. Those wait for the owner's go.
- No TODOs, stubs, or fallbacks in committed code; weeder would flag them. No time estimates.
  Nothing silently dropped: what you skipped or could not verify goes in the report, named.
- Voice in every user-facing string: calm, precise, literate, a little wit. Sentence case,
  product names lowercase, no exclamation marks. A finding states what was found, why it
  matters, and the next action, in that order. Never: supercharge, unlock, 10x, magic,
  synergy, revolutionary, game-changing, cutting-edge, seamless, effortless, next-gen,
  AI-powered.
- Rules detect shapes, never vendor or product names.

## Where it lives, and why

**Its own repository**, the `weeder` repo (builder workspace
`~/conductor/workspaces/weeder/bandung`), as a Rust crate with its own binary `weeder`, its own
identity, and a release pipeline copied from tilth's (multi-platform binaries, crates.io, an
npm wrapper). It depends on a `tilth-core` library crate for tree-sitter outlines, symbol
search, `is_test_file`, and dependency analysis, as a git dependency on the tilth repository
until that crate is published. Reasons, all from the review: hooks must be static binaries
because Claude Code is a native binary and Codex is Rust, so a hook needing Node silently
never fires on a common machine; tilth already owns structural reading in fourteen languages;
a library dependency is not shared state, so weeder still passes the standalone rule; and
separate repositories keep that rule honest. (Decided 2026-09-05: the earlier draft placed
weeder inside the tilth Cargo workspace; the owner created a separate repository, which the
review had preferred anyway.)

If `tilth-core` does not yet exist as a crate when you start, your first milestone is the
extraction proposal: a branch in the tilth workspace that moves the parsing modules
(`src/read/outline/`, `src/search/symbol.rs`, `src/search/deps.rs`, `src/types.rs::is_test_file`,
`src/search/blast.rs`) behind a `tilth-core` crate with tilth's own binary consuming it and
tilth's suite still green. Report over pollen before building weeder on top; the tilth agent
owns that landing.

## Shape

**Core** (`weeder/src/core/`): pure. `parse_diff → Hunks`, `classify_file(path, content) →
{kind: test | prod | config | manifest | generated | guardrail, lang}`, `rules: (Hunks |
Tree, Context) → Vec<Finding>`, `sarif::render(findings) → Log`. No I/O in core; failure in
return types.

**Seams** (`weeder/src/seams/`): git (diff, file at HEAD and in tree, refs, hooks path), exec
(run a test command with a timeout), fs. Injected; never imported by core.

**Faces** (`weeder/src/faces/`):

```
weeder check [--base <ref>] [--staged] [--scope <glob>...] [--strict] [--format sarif|table]
weeder scan  [--rules <ids>] [--format sarif|table]
weeder guard install | status | uninstall          git hooks via core.hooksPath
weeder bite  --test "<command>" [--base <ref>]      Part 5
weeder hook  <claude|gemini|codex>                  harness hook JSON on stdin
weeder rules                                        the catalogue with severities and faces
```

`check` default: index plus working tree against HEAD, which is what pleach's smoke gate sees
after it stages the worker's files (`~/conductor/repos/pleach/src/loop/run-node.ts` lines
190 to 210: `accept.smoke` runs in the node worktree, exit-code gated, no arguments passed).
`--base` compares against a ref for CI. `--staged` is the pre-commit face.

**Exit codes:** 0 clean or warnings only; 2 at least one block-level result; 3 weeder could not
run. A gate that cannot run fails closed: 3 is a failure to pleach and the message says so.
`scan` exits 0 or 3 only.

**`--strict`** (defined 2026-09-05 with the builder): suppressions are reported but not
honoured, and a file a rule cannot evaluate fails closed with exit 3 instead of being skipped.
Strict never escalates a warn-level result to block; severity belongs to the rule, so
warnings stay a message for the human at the pull request.

**Output:** SARIF 2.1.0 by default when stdout is not a terminal; a table when it is.
`--format` overrides. Each result carries rule id, level, location, message, and where a fix
is mechanical, a `fixes` entry. Document the SARIF conventions in `docs/sarif.md`.

**Config:** `weeder.toml` at the repo root, optional. Per rule: `off | warn | block` for check
rules, `off | on` for scan rules; a `[scope]` table of allowed globs; a `[deps]` table of
allowed import directions; thresholds for TODO age and dependency lag. Defaults ship in the
binary; `weeder rules` prints them.

**Suppression:** commit trailer `Weeder-allow: T2 <reason>` or inline `weeder-allow T2: <reason>`.
Suppressions are results too, at note level, so a reviewer sees them and the pile is visible.

## Rule catalogue, v1

Languages for v1: TypeScript/JavaScript (vitest, jest, node:test), Python (pytest,
unittest), Rust (cargo test), Go (go test). Every rule: one adversarial fixture where it must
fire and one neighbour where it must stay silent, per language.

**check rules** (diff):

| Id | Finding | Default | Detection |
|---|---|---|---|
| T1 | test deleted: a test file removed, or a test case removed from a changed test file | block | file classification plus case-count diff |
| T2 | assertions dropped in a changed test file with no allowance | block | assertion count HEAD vs tree |
| T3 | skip or focus added (`.skip`, `.only`, `xit`, `@pytest.mark.skip/xfail`, `#[ignore]`, `t.Skip(`) | block | added-line patterns in test files |
| T4 | tolerance widened: a numeric literal in an approximate assertion or timeout increased | warn | paired removed/added lines, numeric extraction |
| T5 | expected values regenerated: snapshot, golden, or fixture files changed alongside production code | warn | path classification plus prod change present |
| T6 | error assertion weakened: specific message became "any error" | warn | paired lines per language |
| T7 | test uncollected by rename | block | rename detection plus naming conventions |
| M1 | mock over the unit under change: a test in the diff mocks a module whose production file is also in the diff | warn | mock specifiers resolved against diff paths; tend2's `src/verify/mock.ts` is prior art for TS/JS, read it, do not port it |
| S1 | stub or TODO in production code: `TODO`, `FIXME`, `unimplemented!`, `todo!`, `NotImplementedError`, `throw new Error('not implemented')`, a body that is only `pass` or `return null` | block | added lines in prod files |
| S2 | swallowed error: empty catch, bare except, `except Exception: pass`, `.catch(() => {})`, `_ = err`, `unwrap_or_default()` on a fallible call in new code | warn | added lines, language-aware |
| S3 | debug leftovers in production code | warn | added lines in prod files |
| D1 | dependency manifest changed | warn; block when a scope excludes it | path classification |
| D2 | dependency direction violated: an import crosses a boundary `[deps]` forbids (pleach's own doctrine: core imports nothing from seams or faces) | block | tilth-core dependency analysis on changed files |
| X1 | secret-looking string added | block | prefix list plus entropy on assignments to key-like names |
| X2 | out of scope: a file touched that no scope glob allows | block when scope given | path match; tilth-core callers to name the blast radius |
| C1 | guardrail edited, constitution tier: `.claude/settings*.json`, `.gemini/settings.json`, `.codex/*`, `weeder.toml`, `.githooks/*`, the hard-limits section of `AGENTS.md` or `CLAUDE.md` (ruled 2026-09-06 after calibration showed 42 of 71 blocks were legitimate workflow edits) | block | path classification |
| C3 | CI workflow edited: `.github/workflows/*` and equivalents; a repo promotes these to block under `[guardrails] paths` in `weeder.toml` | warn | path classification |
| C2 | ignore broadened to hide source or tests | warn | added patterns matched against source globs |
| G1 | conflict markers | block | line-start markers |
| G2 | large or binary file added | warn | size and binary detection |

**scan rules** (tree, never block; settled with tend2 on 2026-09-05):

| Id | Finding | Detection |
|---|---|---|
| R1 | a path, command, flag, or symbol cited in `CLAUDE.md`, `AGENTS.md`, `README.md`, or `docs/**/*.md` no longer exists | markdown code spans and paths resolved against the tree and tilth-core symbols |
| R2 | dead export: a public symbol with no reference anywhere in the repo | tilth-core callers |
| R3 | TODO older than the configured age | `git blame` on TODO lines |
| R4 | dependency lag: a manifest pin more than the configured distance behind the registry's latest, checked offline against a committed snapshot when no network | manifest parse plus snapshot |

Keep-your-place applies to scan rules: one that names no hot spot confirmed by a later loop
in a season is demoted to off by default.

## Faces and integration

**pleach.** `examples/pleach/plan.json` with `"accept": { "smoke": "weeder check --strict" }`;
validate it with `pleach validate`. Document in `docs/pleach.md` that weeder sees the staged
files plus the tree against HEAD, so no base argument is needed.

**guard.** `weeder guard install` writes `.githooks/pre-commit` (`weeder check --staged --strict`),
`.githooks/pre-push` (`weeder check --strict --base <upstream>` plus refusal of non-fast-forward
pushes to protected branches from `weeder.toml`), and `.githooks/pre-rebase` (refuse rewriting
protected history), then sets `core.hooksPath`. `guard status` proves each hook is live.
The only bypass, `--no-verify`, is denied by the harness hook; the bundle the stem generates
carries that denial.

**hooks.** `weeder hook claude` handles `PreToolUse` for `Bash` when the command contains
`git commit` (check `--staged`, deny with the SARIF results rendered as the reason) and
`Stop` (check the working tree; block the stop with the findings). `weeder hook gemini` handles
`BeforeTool` the same way. `weeder hook codex`: verify what Codex's hook events can deny now,
implement what is possible, record what is not. Gotcha: Claude Code reads hooks from
`settings.json`, never from `~/.claude.json`. Prove each hook with a real harness run.

**CI.** `examples/ci/github.yml`: `weeder check --base origin/${{ github.base_ref }} --strict
--format sarif > weeder.sarif` uploaded with `github/codeql-action/upload-sarif`, so results are
code-scanning alerts. This Action is weeder's first distribution: maintainers drowning in
agent-authored pull requests.

**tend2 seam.** `docs/tend2-seam.md`: how tend2's hygiene loop cites `weeder scan --format
sarif` output as evidence, and how tend2's verifier could call `weeder check` for its mock-only
refusal instead of carrying `mock.ts`. Proposal only; do not edit tend2.

## Calibration and the kill criterion

1. `scripts/calibrate.rs` (or a `cargo xtask`) runs `weeder check --base <parent> --strict`
   over the last **200 merged commits** on each of: `~/CascadeProjects/tilth`,
   `~/conductor/repos/pleach`, `~/conductor/workspaces/feature-map/missoula`,
   `~/conductor/workspaces/copeca/cancun`, `~/conductor/repos/rctrl`. Results to
   `docs/calibration-2026-09.md`: per repo, commits, blocked, warned, and every blocked commit
   classified true positive, acceptable, or false positive.
2. **Ship bar:** block-level false positives under 2 percent of the pooled merged commits,
   with per-repo counts shown; a repo with fewer than 200 merged commits contributes what it
   has and is reported, not judged alone. (`~/conductor/repos/rctrl` is umbel's repository.)
   **Kill bar:** reaching it requires disabling T1, T2, T3, or S1. Then the deterministic core
   is too thin to be a gate; write that verdict first in the calibration file and stop.
3. `fixtures/adversarial/`: one minimal repo per rule per language where the rule must fire,
   and a neighbour where it must not.

## How we know it is good (added 2026-09-05)

weeder is deterministic by design, and "good" for a deterministic gate has six measurable
parts. Each is a check on weeder's umbrella loop with an evidence script.

1. **Determinism is tested, not claimed.** Same diff, same tree, same config produce
   byte-identical SARIF after run timestamps are stripped, across three runs, on every fixture
   and the calibration corpus. Results are sorted by file, line, and rule. No dependence on
   locale, time, environment, or hash-map iteration order.
2. **Recall, not only precision.** Calibration measures false blocks on real merged commits.
   Its mirror is mutation injection: take real commits from the same corpus, inject each
   anti-pattern mechanically (delete a test case, add a skip, drop an assertion, add a TODO
   body, swallow an error, add a secret, edit a workflow file), and weeder must fire on exactly
   that. Bars: block-level rules T1, T3, S1, X1, C1, G1 at or above 95 percent recall per
   language; warn-level rules reported, aiming at 80. Both numbers live in
   `docs/calibration-2026-09.md`.
3. **Never panic, fail closed.** Malformed diffs, binaries, non-UTF-8, CRLF, renames,
   symlinks, submodules, merge and empty commits, huge files: exit 0, 2, or 3 with a reason,
   never a panic. A property or fuzz test on the diff parser is the evidence. A panic in a hook
   is a hole.
4. **Latency budget.** `weeder check --staged` under 200 ms on a typical diff in the fixture
   repos and under 2 s worst case, measured in CI with a regression failure. A slow hook gets
   disabled by its users.
5. **The builder does not grade its own calibration.** The classification of blocked commits
   is the number weeder ships on. An independent party, a pleach audit node on a different
   provider or the owner, re-grades a random sample of at least twenty blocked commits and
   twenty injected cases; agreement at or above 90 percent, or the classification is untrusted
   and redone.
6. **Watch for routing around it.** Suppression rate, `Weeder-allow` trailers per hundred
   commits on the garden repos since guard was installed, reported beside precision. Rising
   suppressions with flat true positives mean the gate is gamed or mis-tuned.

## Part 5 — `weeder bite`

Only after calibration passes. Premise: pleach commits the test phase separately (its
umbrella loop carries that check). Given `--test "<command>"`: in a temp worktree, check out
the test commit on the base, run, it must fail; apply the implementation, run, it must pass.
A test that passes without the change is theater, reported as block-level `B1` with the test
names that passed. Kill: if fewer than 90 percent of pleach's phased nodes in the proof repo
yield a clean test commit, report it and leave bite unshipped.

## Belonging to the garden

The umbrella verifies weeder against the fit contract from `docs/building-the-garden.md` §3
(F1 to F6) with `scripts/fit/weeder.sh`. To pass it, ship: `garden.json` (kind: gate and hook;
zero declared context cost; metric command: calibration precision; check command: `weeder
check --format sarif`); `SKILL.md` with a one-sentence description; `docs/tend2/` inside the
crate with your module loops; `.brand/products/weeder/` in the pleach shape with the name and
accent marked `[flagged]` (six accents are taken; pollen has claimed a seventh; measure
candidates with the umbrella's `scripts/palette_contrast.py`); the garden footer on any page.
LICENSE is MIT on the branch, flagged for confirmation. A landing page is a later loop.

## Milestones to report over pollen

1. `tilth-core` extraction proposal branch green, or confirmation the crate exists.
2. check rules T1, T3, S1, X1, C1, G1 firing on fixtures with SARIF validating.
3. Full check catalogue plus calibration verdict.
4. guard installed and proven in a fixture repo; hooks proven on two harnesses.
5. scan rules R1 to R4 with tend2 as consumer.
6. bite, or its kill report.

## Acceptance

- Every rule has a failing-first fixture in every listed language; `cargo test` green in the
  workspace; tilth's own suite still green.
- SARIF output validates against the 2.1.0 schema on every fixture.
- `weeder check` on the tilth workspace tree at the end reports nothing at block level.
- pleach example plan validates; guard proven; hooks proven by real harness runs recorded in
  `docs/proof-2026-09.md`.
- Calibration file written with the verdict in its first sentence.
- `garden.json`, `SKILL.md`, the crate's own map, and the brand layer present; flagged items
  marked.
- Nothing pushed, published, or created on GitHub.

## Prior art, checked 2026-09-05 (compose with it, do not rebuild it)

Most of weeder's individual rules exist somewhere as a single-language or single-concern tool.
weeder's unclaimed ground is the honesty of the **diff against tests and guardrails**, the cheap
effectiveness proof (`bite`), the placement at the agent's Stop and at pleach's node, and the
calibration discipline. Where a rule below has a mature home, weeder either reuses it or keeps
its own only because the cross-language, diff-time framing needs it.

| weeder rule or face | Existing open-source tools | Consequence for weeder |
|---|---|---|
| X1 secrets | gitleaks, trufflehog, detect-secrets; agent-guard wraps gitleaks at the agent's tool boundary and in git hooks | keep a small prefix-and-entropy rule for the hook path; recommend gitleaks for depth; never claim to replace it |
| S2 swallowed errors, S3 debug leftovers, R2 dead exports | aislop (597 stars, MIT, 50+ deterministic rules over 10 languages, SARIF, hooks for Claude Code, Cursor, Gemini, a GitHub Action, a 0 to 100 score); knip, ts-prune, vulture, deadcode per language | these are aislop's home ground; keep S2 and S3 only as warn-level, and only where aislop is absent; R2 is a candidate to drop |
| D2 dependency direction | OpenLore `check_architecture` (297 stars, cross-language, from `.openlore/architecture.json`); dependency-cruiser, import-linter, ArchUnit | keep only if tilth-core makes it nearly free; otherwise recommend OpenLore or the per-language tool |
| X2 blast radius | OpenLore `change_impact_certificate` and `certify_public_surface` | weeder's X2 is scope enforcement from a plan, which OpenLore does not do; the blast-radius text can come from tilth-core |
| R4 dependency lag | Renovate, Dependabot | drop from scan unless tend2 asks for it as a hygiene signal |
| G1 conflict markers | pre-commit `check-merge-conflict` | trivial; keep, it is one line |
| T3 skips and focus | ESLint `jest/no-disabled-tests`, `no-focused-tests`; ruff PT rules | those flag any skip in a file; weeder flags an **added** skip in the diff, a different question; keep |
| T1, T2, T4, T5, T6, T7 test tampering | none found as a deterministic gate; GitHub's own review guidance says to watch for it by eye; PyTorch's playbook checks it manually; pyor.review sells a model-based reviewer | weeder's core; this is the unclaimed ground |
| C1 guardrail edits, X2 scope from a plan | none found | weeder's core |
| `bite` | mutation testing (Stryker, mutmut, cargo-mutants) as the heavyweight cousin | weeder's cheap revert-and-run is distinct; keep |
| `guard`, hooks, SARIF, Action | husky, lefthook, pre-commit framework; aislop and OpenLore both ship hooks, SARIF and Actions | plumbing is commodity; weeder's version exists so the honesty rules run at commit and at Stop, not for its own sake |
| PR-level slop triage | anti-slop (813 stars, AGPL): 34 checks on branch, size, title, description, commits, contributor history; language-agnostic; auto-closes | a different layer, metadata not diff content; complementary; recommend it to maintainers alongside weeder |
| Shipmoor | a CLI with a pre-commit hook for "code-integrity findings", org account, source not found | commercial, unverified; note and move on |

## Revisions on 2026-09-05

- Runtime changed from TypeScript/Node to a Rust crate in the tilth workspace on `tilth-core`;
  hooks must be static binaries.
- Output changed from a garden findings schema to SARIF 2.1.0.
- `scan` face added with rules R1 to R4, non-blocking, tend2 first consumer.
- `guard` face added; it replaces the earlier hedge bed.
- D2 dependency-direction rule added.
- `bite` premised on pleach's separate test commit instead of hunk splitting.
- Reading order and reporting over pollen added.
