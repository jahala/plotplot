#!/usr/bin/env bash
# The states rule of contracts/delivery.md: a conductor's verdict never reports a non-progress
# state without a reason and a next step. Runs the pleach under test on the fixture plan
# contracts/fixtures/verdict/plan.json, one command node whose command is `false`, inside a
# throwaway git repository, then reads the run journal and asserts that the node's verdict
# exists, is not closed, and carries non-empty `reason` and `nextStep`.
#
# The conductor under test: PLOTPLOT_PLEACH_BIN, else `pleach` on PATH; it may be a command
# of several words (for example "bun /tmp/pleach-pinned/src/main.ts"). Which ran is printed
# first, so a verdict is attributable. Run from the repository root, no arguments. Prints one
# line per assertion. Exits 0 if every assertion passed, 1 otherwise, 3 when no conductor is
# available (reason on stderr).
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
plan="contracts/fixtures/verdict/plan.json"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
[ -f "$plan" ]; assert $? "the fixture plan exists at $plan"
[ "$fail" -eq 0 ] || exit 1

read -r -a pleach <<< "${PLOTPLOT_PLEACH_BIN:-pleach}"
if ! command -v "${pleach[0]}" >/dev/null 2>&1; then
  echo "verdict.test.sh: ${pleach[0]} is not on PATH and PLOTPLOT_PLEACH_BIN is unset; no conductor to test" >&2
  exit 3
fi
echo "conductor: ${pleach[*]} ($("${pleach[@]}" --version 2>/dev/null | head -1))"

scratch="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-verdict.XXXXXX")"
git -C "$scratch" init -q -b master && git -C "$scratch" -c user.name=fixture -c user.email=fixture@plotplot.local commit -q --allow-empty -m "fixture root"
cp "$plan" "$scratch/plan.json"
journal="$scratch/journal.jsonl"
"${pleach[@]}" run "$scratch/plan.json" --repo-root "$scratch" --journal "$journal" --max-concurrency 1 --timeout-ms 120000 > "$scratch/run.log" 2>&1
run_status=$?
[ -f "$journal" ]; assert $? "the run wrote a journal" "exit $run_status: $(tail -c 300 "$scratch/run.log" | tr '\n' ' ')"
if [ ! -f "$journal" ]; then put_away "$scratch"; exit 1; fi

verdict="$(node -e '
const fs = require("fs");
const lines = fs.readFileSync(process.argv[1], "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l));
const v = lines.filter((e) => e.event === "verdict" && e.node === "verdict.c1").pop();
process.stdout.write(v ? JSON.stringify(v) : "");
' "$journal")"
[ -n "$verdict" ]; assert $? "the journal carries a verdict for verdict.c1"
status="$(node -e 'const v=JSON.parse(process.argv[1]||"{}");process.stdout.write(v.status||"")' "$verdict")"
[ -n "$status" ] && [ "$status" != "closed" ]; assert $? "the verdict is a non-progress state" "status '$status'"
reason="$(node -e 'const v=JSON.parse(process.argv[1]||"{}");process.stdout.write(typeof v.reason==="string"?v.reason:"")' "$verdict")"
next="$(node -e 'const v=JSON.parse(process.argv[1]||"{}");process.stdout.write(typeof v.nextStep==="string"?v.nextStep:"")' "$verdict")"
[ -n "$reason" ]; assert $? "the verdict carries a non-empty reason" "keys: $(node -e 'process.stdout.write(Object.keys(JSON.parse(process.argv[1]||"{}")).join(","))' "$verdict")"
[ -n "$next" ]; assert $? "the verdict carries a non-empty nextStep"
put_away "$scratch"
exit "$fail"
