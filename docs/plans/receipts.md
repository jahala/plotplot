# Receipts — plan

What produced a change and what verified it, as a signed statement attached to the commit.
Provenance is compliance now (the EU AI Act and Cyber Resilience Act obligations arriving
2026 and 2027) and trust always: a receipt outlives the vendor, the session and the model
that produced the work.

Status: plan, 2026-09-06. The direction memo parked receipts until a second reader existed.
The owner asked for the plan; this plan keeps the gate honest by splitting a cheap
hash-verified v0, worth building with the stem, from signing, which waits for a reader.

## 1. What a receipt says

For one commit (or one merged pull request), in machine-readable form:

- **who produced it**: the harness and its version, the model or models, whether a human
  drove the session or a conductor did, the pleach plan and node if any, the principal in
  tend2's vocabulary (`cli`, `ci:<run-id>`, `judge:<provider>`, `human:<name>` only when a
  person signs);
- **what it touched**: the files changed, with the tree hash;
- **what verified it**: weeder's result summary (block and warn counts, the SARIF log's digest),
  tend2 stamps written in this change (check ids and their `@sha`), pleach gates passed,
  test commands run with exit codes;
- **what it cost**: tokens where the harness reported them, wall time, tool-call counts by
  tool name; never an estimate, `null` when unknown;
- **friction**: the digest of the session's friction summary, if the ledger exists;
- **producer**: `plotplot@<version>`, and the statement's own creation time.

Never in a receipt: prompt text, tool output, file contents, secrets, absolute home paths, a
person's name unless they signed.

## 2. Standards it stands on

- **in-toto Attestation Framework, Statement v1**: `_type`
  `https://in-toto.io/Statement/v1`, `subject[]` with `name` and a `digest` set,
  `predicateType`, `predicate`. The DigestSet supports `gitCommit` and `gitTree`, so a commit
  is a first-class subject without inventing anything.
- **DSSE** as the envelope, **Sigstore** for keyless signing and the transparency log:
  `cosign attest-blob` and `cosign verify-blob-attestation` with a bundle, keyless via OIDC
  in CI (GitHub Actions identity) or a browser flow locally.
- **git notes** as the storage that travels with the repository:
  `refs/notes/plotplot/receipts`, one note per commit, pushed explicitly.
- **GitHub artifact attestations** (`actions/attest`, `gh attestation verify`) as the
  optional mirror for repositories on GitHub, attesting the merge commit's patch digest.
- **SLSA Provenance v1** is not the predicate: it describes builds, not agent work. It is
  reused only if a CI build of a release artifact wants provenance too.
- gitsign signs commits keylessly but has no attestation command (checked 2026-09-06), so
  commit signing and receipts stay separate layers: gitsign says who committed, the receipt
  says what produced and verified the change.

## 3. The predicate

`predicateType: https://plotplot.ai/receipt/v1`. Schema in contracts as JSON Schema.

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {"name": "github.com/jahala/weeder", "digest": {"gitCommit": "3f2a…", "gitTree": "9c1d…"}}
  ],
  "predicateType": "https://plotplot.ai/receipt/v1",
  "predicate": {
    "producer": {"name": "plotplot", "version": "0.1.0"},
    "createdAt": "2026-09-06T15:40:12Z",
    "harness": {"name": "claude-code", "version": "2.1.261"},
    "models": ["claude-opus-5"],
    "principal": "cli",
    "sessions": ["4f0c…"],
    "conductor": {"tool": "pleach", "plan": "plans/rules-block.plan.json", "node": "rules-block"},
    "changed": ["src/rules/t1.rs", "tests/rules_t1.rs"],
    "verification": {
      "weeder": {"sarif": {"sha256": "…"}, "block": 0, "warn": 2},
      "tend2": [{"check": "rules-block:c3", "sha": "9853abb"}],
      "pleach": {"gates": ["markers", "smoke", "audit"], "auditProvider": "openai"},
      "commands": [{"command": "cargo test --workspace", "exit": 0}]
    },
    "cost": {"inputTokens": null, "outputTokens": null, "wallSeconds": 812, "toolCalls": {"Bash": 41, "Edit": 12, "tilth_search": 9}},
    "friction": {"summary": {"sha256": "…"}}
  }
}
```

The subject names the repository by its canonical URL and the commit by its own hash, so
verification needs nothing but the repository.

## 4. Lifecycle

**Draft at session end.** A `SessionEnd` hook (Claude Code's 1.5-second budget applies)
writes an unsigned draft from counters the friction emitter and the hooks already kept:
`.plotplot/receipts/drafts/<session_id>.json`. Fast, append-only, no network.

**Seal at commit.** `plotplot receipt seal`, run from the pre-commit git hook the stem
installs beside `weeder guard`, gathers the drafts of the sessions that touched the staged
files, folds in weeder's SARIF digest and any tend2 stamps in the diff, builds the Statement
with the commit-to-be's tree hash, and attaches it after the commit as a note on
`refs/notes/plotplot/receipts` (a `post-commit` hook writes the note once the commit hash
exists). Unsigned in v0: the note carries the statement and its sha256.

**Sign in CI.** On push, a workflow runs `plotplot receipt sign` for each new commit: wraps
the statement in DSSE via `cosign attest-blob --predicate … --type https://plotplot.ai/receipt/v1 --bundle`,
keyless with the Actions identity, and replaces the note with the bundle. Optionally mirrors
it as a GitHub attestation on the merge commit's patch.

**Verify anywhere.** `plotplot receipt verify <commit>` re-derives the subject digest from
git, checks the note's hash in v0 or the DSSE signature and Rekor entry in v1, and prints the
predicate. `plotplot receipt verify --range origin/main..HEAD` is the PR check: every commit
carries a receipt, or the check fails.

**Fetch and push.** Notes travel only when asked: the stem's `init` adds
`refs/notes/plotplot/receipts` to the fetch and push refspecs so clones carry them.

## 5. Faces

```
plotplot receipt draft            from a SessionEnd payload on stdin (hook face)
plotplot receipt seal             pre-commit / post-commit hook face
plotplot receipt sign <commit>    CI, keyless via Sigstore
plotplot receipt verify <commit|--range a..b> [--require-signed]
plotplot receipt show <commit>    the predicate, for a human
```

All faces of the stem binary; no separate tool.

## 6. What each bed contributes

| Bed | Contributes | Through |
|---|---|---|
| weeder | SARIF digest and counts for the change | `weeder check --format sarif` at seal time |
| tend2 | stamps in the diff, principal vocabulary | reading the loop files in the diff |
| pleach | plan, node, gates passed, audit provider, worker minutes | its journal; receipts fix the gap its handback exposed (no token counts) by carrying umbel's `telemetry.tokens` when present |
| umbel | tokens, context percentage, compaction flag per worker | the unit result |
| friction ledger | the session summary digest | `.plotplot/friction/` |
| stem | the faces, the hooks, the note plumbing, the schema | this plan |

## 7. Trust model, honestly

- v0 (unsigned note with a hash) proves integrity from the note's own hash only within the
  repository; it is a record, not a proof. It is still worth having: it is what a reviewer
  reads, and it is what the PR check requires.
- v1 (DSSE plus Sigstore) proves who signed and when, to anyone with the repository and the
  public log. The signer is the CI identity, so it proves "this passed through this
  workflow", which is the provenance claim compliance asks for.
- A receipt records what the harness reported. A harness that lies about its model is not
  caught by a receipt. That is a limit, stated, not a hole to paper over.
- Principal is a claim, never proof, exactly as tend2's stamp rule says; the signature is the
  proof of who sealed, and only that.

## 8. Checks (on the umbrella's `receipts` loop)

- the predicate schema in contracts validates the example above and rejects a predicate
  carrying prompt text, tool output, or an absolute home path (negative fixtures);
- `receipt draft` fed each harness's `SessionEnd` payload writes a draft under 200 ms with
  counts from the session state and nothing else;
- `receipt seal` on a fixture repository attaches a note whose subject digest equals the
  commit's `gitCommit` digest recomputed independently, and a second seal is idempotent;
- `receipt verify --range` fails on a range with one receipt missing and passes when every
  commit has one;
- `receipt sign` in a fixture workflow produces a DSSE bundle that
  `cosign verify-blob-attestation` accepts with the workflow's identity, and a tampered
  predicate fails;
- notes travel: a fresh clone after `plotplot init` fetches the receipts ref;
- the human check: the owner reads one real receipt and confirms it says what a reviewer
  would want to know and nothing they would not want recorded.

## 9. Activation gate and kill criteria

v0 (draft, seal, verify, notes) is built with the stem: it is small and it is the record.
v1 (signing, the PR check, GitHub mirror) is built when one of these exists: a second person
reading receipts, a CI policy that requires them, or a compliance ask. Kill for v0: if the
note per commit is not read by anyone in a season and no check consumes it, remove the
post-commit hook and keep the faces. Kill for v1: if keyless signing cannot run in the
garden's CI without a stored credential, do not ship signing; a stored key is the failure
mode Sigstore exists to avoid.

## 10. Order of work

1. Predicate schema and fixtures in contracts.
2. `draft` and `seal` faces in the stem, with the notes plumbing and `init` refspecs.
3. `verify` and `show`.
4. The PR check in the proof repository's workflow.
5. `sign` and the GitHub mirror, on the activation gate.

## 11. Decisions for the owner

- Whether receipts are drafted by default in every planted repository, or opt-in per repo.
- Whether principal `human:<name>` ever appears automatically (recommended: never; only on
  explicit signing, per tend2's rule).
- When v1 signing is worth its CI cost.
