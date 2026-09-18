#!/usr/bin/env bash
# The handback rule of the runner seam (contracts/runner.md, "The handback is complete when
# stop is reported"): a stop means the worker's final message is readable in full. The bed
# holds its side with its own test, which drives a fake worker whose final message lands
# after the stop signal and asserts that read returns that final message. This test runs
# that file from the pinned checkout, PLOTPLOT_UMBEL_SRC, under the bed's own runner, so the
# rule is re-proven at every revision the umbrella pins. Run from the repository root, no
# arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise,
# 3 when the checkout is not named, carries no installed dependencies, or bun or tmux is
# absent (reason on stderr): a rule that could not be run is never reported as held.
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
cannot() { echo "handback.test.sh: $*" >&2; exit 3; }

umbel="${PLOTPLOT_UMBEL_SRC:-}"
[ -n "$umbel" ] && [ -d "$umbel" ] || cannot "PLOTPLOT_UMBEL_SRC is unset or not a directory; the rule cannot be run"
for tool in bun tmux; do
  command -v "$tool" >/dev/null 2>&1 || cannot "$tool is not installed; umbel's own test cannot run here"
done
[ -d "$umbel/node_modules" ] || cannot "$umbel carries no node_modules; run bun install in the pinned checkout first"
echo "umbel: $umbel at $(git -C "$umbel" rev-parse --short HEAD 2>/dev/null || echo unknown)"

grep -q 'The handback is complete when stop is reported' contracts/runner.md
assert $? "contracts/runner.md states the handback rule"

held="test/integration/read-at-stop.test.ts"
[ -f "$umbel/$held" ]; assert $? "umbel carries its side of the rule at $held"
[ "$fail" -eq 0 ] || { echo "handback.test.sh: failures present"; exit 1; }

grep -q 'final message is readable in full' "$umbel/$held"
assert $? "$held states the rule it holds: a stop means the final message is readable in full"

log="$(mktemp "${TMPDIR:-/tmp}/plotplot-handback.XXXXXX")"
(cd "$umbel" && bun test "$held") > "$log" 2>&1
status=$?
passed="$(grep -E '^ *[0-9]+ pass' "$log" | tail -1 | tr -dc '0-9')"
assert "$status" "umbel's read-at-stop test passes at the pinned revision (${passed:-0} passing)"
[ "$status" -eq 0 ] || tail -n 15 "$log" | sed 's/^/# /'
[ -n "$passed" ] && [ "$passed" -ge 1 ]
assert $? "the run counted at least one passing test, so a file that ran nothing does not read as held"
trash "$log" 2>/dev/null || true

if [ "$fail" -ne 0 ]; then echo "handback.test.sh: failures present"; exit 1; fi
echo "handback.test.sh: all assertions passed"
exit 0
