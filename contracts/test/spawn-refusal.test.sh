#!/usr/bin/env bash
# The spawn side of the runner seam (contracts/runner.md, "A spawn that needs a person is
# blocked, and says so by its exit code"). Reads umbel's SPAWN_EXIT_CODES table at the pinned
# checkout, PLOTPLOT_UMBEL_SRC, and asserts against
# contracts/fixtures/runner/spawn-refusals.jsonl: every fixture reason is in umbel's table
# with the fixture's exit code, and umbel names no refusal the fixture lacks. The conductor's
# classification is held by the conductor's own test. Run from the repository root, no
# arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise,
# 3 when the checkout is not named or does not exist (reason on stderr).
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
fixture="contracts/fixtures/runner/spawn-refusals.jsonl"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$fixture" ]; assert $? "the fixture table exists at $fixture"
[ "$fail" -eq 0 ] || exit 1
umbel="${PLOTPLOT_UMBEL_SRC:-}"
if [ -z "$umbel" ] || [ ! -d "$umbel" ]; then
  echo "spawn-refusal.test.sh: PLOTPLOT_UMBEL_SRC is unset or not a directory; the seam cannot be read" >&2; exit 3
fi
echo "umbel: $umbel at $(git -C "$umbel" rev-parse --short HEAD 2>/dev/null || echo unknown)"

# umbel's table: reason -> exit code, from the SPAWN_EXIT_CODES object literal.
table="$(node -e '
const fs=require("fs");const src=fs.readFileSync(process.argv[1],"utf8");
const m=src.match(/SPAWN_EXIT_CODES[^=]*=\s*\{([\s\S]*?)\n\};/);if(!m){process.exit(0)}
const out={};for(const line of m[1].split("\n")){const r=line.match(/^\s*[\x27"]?([a-z-]+)[\x27"]?\s*:\s*(\d+)/);if(r)out[r[1]]=+r[2]}
process.stdout.write(JSON.stringify(out))' "$umbel/src/faces/cli.ts" 2>/dev/null)"
[ -n "$table" ] && [ "$table" != "{}" ]; assert $? "umbel's SPAWN_EXIT_CODES table is readable at src/faces/cli.ts"
[ -n "$table" ] || table='{}'

while IFS= read -r row; do
  [ -n "$row" ] || continue
  reason="$(printf '%s' "$row" | node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(0,"utf8")).reason)')"
  code="$(printf '%s' "$row" | node -e 'process.stdout.write(String(JSON.parse(require("fs").readFileSync(0,"utf8")).exitCode))')"
  got="$(node -e 'const t=JSON.parse(process.argv[1]||"{}");const v=t[process.argv[2]];process.stdout.write(v===undefined?"":String(v))' "$table" "$reason")"
  [ "$got" = "$code" ]; assert $? "umbel refuses a spawn for $reason with exit $code" "got '${got:-none: not in the table}'"
done < "$fixture"

extra="$(node -e '
const fs=require("fs");const t=JSON.parse(process.argv[1]||"{}");
const rows=fs.readFileSync(process.argv[2],"utf8").split("\n").filter(Boolean).map(l=>JSON.parse(l).reason);
process.stdout.write(Object.keys(t).filter(k=>!rows.includes(k)).join(","))' "$table" "$fixture")"
[ -z "$extra" ]; assert $? "umbel names no spawn refusal the table lacks" "extra: $extra"
exit "$fail"
