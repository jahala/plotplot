# Engineering the umbrella: how work is done and proven here

Written 2026-09-24 from the project notes this page replaces. Recipes name real files.

## Landing

- Every change lands by pull request. Branch, push, `gh pr create`, `gh pr checks <n> --watch`
  gated on its exit code, `gh pr merge <n> --merge`. A ruleset on `master` requires the
  `garden` check; a direct push is refused. Never merge red; never chain a fallback after a
  failed check.
- Run `npm test` before any landing that touches `contracts/`; it is not in CI. It runs every
  `contracts/test/*.test.sh` through `scripts/contracts-test.sh`: a red that
  `contracts/test/EXPECTED_RED` records does not fail it, any other red does, and a recorded
  test that turns green fails it until its line is removed.
- Tests that read a bed take `PLOTPLOT_TEND2_SRC`, `PLOTPLOT_PLEACH_SRC` and
  `PLOTPLOT_UMBEL_SRC` (checkouts at the pinned revisions) and `PLOTPLOT_TEND2_BIN`,
  `PLOTPLOT_PLEACH_BIN`, `PLOTPLOT_REDACT_BIN` (the tools by path). Set all six in the same
  command as `npm test`; shell state does not persist between commands. Without a checkout a
  test says unevaluable, never green.
- Commit bodies explain the why; they are the reasoning trail.

## Pinned tools, never the machine's links

`pleach` and `tend2` on PATH point into other agents' live checkouts. Conduct with a pinned
clone (`bun <clone>/src/main.ts`), verify and lint with a pinned tend2 build
(`node <clone>/dist/cli.js`; `tend2 verify` misreports under bun, tend 228). Re-pin after
each tool landing. Move `contracts/pins.json` in a pull request when a pinned revision moves.

## Recipes

- **Add a contract check.** Copy `contracts/test/runner.test.sh` (a table read from a bed's
  source) or `contracts/test/proof.test.sh` (a schema with positive and negative fixtures):
  one line per assertion, exit 0, 1 or 3, `unevaluable` with the claim's name when it cannot
  run. Add the check to `docs/tend2/contracts.tend2.html`, red first, and its line to
  `EXPECTED_RED` naming the bed's issue. Stamp with
  `node <tend2>/dist/cli.js verify docs/tend2/contracts.tend2.html --check N --force --runner 'bash {evidence}'`.
- **Add a fit claim for a bed.** Copy a `check_f4`-shaped function in `scripts/fit/weeder.sh`;
  compose `scripts/fit/lib.sh` (`fit_lock_lookup`, `fit_fetch`, `fit_verify_sha256`,
  `fit_clone`); print `ok F<n> ...` per assertion.
- **Move a bed's pin.** Download the release's `.sha256` files, update `garden.lock` and
  `contracts/fixtures/garden.lock` by digest, run `plotplot lock verify` and the bed's fit
  script, record the result on the bed's loop, land by pull request (the weeder 0.2.4 pin,
  2026-09-18, is the worked example).
- **Conduct a node here.** A hand-written `docs/dogfood/<loop>/plan.json`, one phased node
  closing one check; setup builds the stem and runs `plotplot lock verify`; smoke
  `weeder check --strict`; audit `tend2 verify ... --audit-egress` on a second provider. A test
  that is red by design cannot pass the green gate: land its script from the quarantine branch
  and say so on the loop.
- **Record a fault in a bed.** File it on the bed's repository with evidence; the umbrella
  never patches a bed. Where the fault is a seam, write the contract page and its red test
  first.

## Evidence

Evidence scripts compose `scripts/fit/lib.sh`, print one line per claim, exit 0, 1 or 3, and
answer `unevaluable` for a claim they do not carry. A claim whose script cannot fail is
theater; every corrected claim is shown failing once (sixteen were found in one repository on
2026-09-18). Fixture rows that look like credentials are base64 at rest and decoded in the
test; print row ids, never values.

## Lessons that hardened

- A chain that commits on `;` after a red run has merged red twice; gate with `&&` on the
  exit code, never on grep of the output.
- A history-wide git search with no bound ran ten minutes and had to be killed; bound every
  scan or run it detached with a log.
- `trash` on a full volume frees nothing; caches replace in place, and the owner empties the
  Trash.
- A monitor or a background task dies with the session; long work runs as a detached script
  with a progress log.
