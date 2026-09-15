#!/usr/bin/env bash
# The runner seam (contracts/runner.md): umbel's wait reasons and exit codes are the table
# pleach classifies by. Reads umbel's WAIT_EXIT_CODES table and pleach's worker reason union
# and classification at their pinned checkouts, PLOTPLOT_UMBEL_SRC and PLOTPLOT_PLEACH_SRC,
# and asserts against contracts/fixtures/runner/reasons.jsonl: every fixture reason is in
# umbel's table with the fixture's exit code, umbel names no reason the fixture lacks, pleach's
# union names every fixture reason, and pleach classifies each as the fixture says. Which
# revisions ran is printed first. Run from the repository root, no arguments. Prints one line
# per assertion. Exits 0 if every assertion passed, 1 otherwise, 3 when a checkout is not
# named or does not exist (reason on stderr).
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
fixture="contracts/fixtures/runner/reasons.jsonl"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$fixture" ]; assert $? "the fixture table exists at $fixture"
[ "$fail" -eq 0 ] || exit 1
for var in PLOTPLOT_UMBEL_SRC PLOTPLOT_PLEACH_SRC; do
  if [ -z "${!var:-}" ] || [ ! -d "${!var}" ]; then
    echo "runner.test.sh: $var is unset or not a directory; the seam cannot be read" >&2; exit 3
  fi
done
umbel="$PLOTPLOT_UMBEL_SRC"; pleach="$PLOTPLOT_PLEACH_SRC"
echo "umbel: $umbel at $(git -C "$umbel" rev-parse --short HEAD 2>/dev/null || echo unknown); pleach: $pleach at $(git -C "$pleach" rev-parse --short HEAD 2>/dev/null || echo unknown)"

# umbel's table: reason -> exit code, from the WAIT_EXIT_CODES object literal.
umbel_table="$(node -e '
const fs=require("fs");const src=fs.readFileSync(process.argv[1],"utf8");
const m=src.match(/WAIT_EXIT_CODES[^=]*=\s*\{([\s\S]*?)\n\};/);if(!m){process.exit(0)}
const out={};for(const line of m[1].split("\n")){const r=line.match(/^\s*\x27?([a-z-]+)\x27?\s*:\s*(\d+)/);if(r)out[r[1]]=+r[2]}
process.stdout.write(JSON.stringify(out))' "$umbel/src/faces/cli.ts" 2>/dev/null)"
[ -n "$umbel_table" ] && [ "$umbel_table" != "{}" ]; assert $? "umbel's WAIT_EXIT_CODES table is readable at src/faces/cli.ts"

# pleach's union and classification.
pleach_union="$(node -e '
const fs=require("fs");const src=fs.readFileSync(process.argv[1],"utf8");
const m=src.match(/reason\?:\s*([^;]+);/);if(!m){process.exit(0)}
process.stdout.write(JSON.stringify([...m[1].matchAll(/\x27([a-z-]+)\x27/g)].map(x=>x[1])))' "$pleach/src/loop/deps.ts" 2>/dev/null)"
[ -n "$pleach_union" ]; assert $? "pleach's worker reason union is readable at src/loop/deps.ts"
pleach_classify="$(node -e '
const fs=require("fs");const src=fs.readFileSync(process.argv[1],"utf8");
const m=src.match(/function classifyWorkerReason[\s\S]*?\n\}/);if(!m){process.exit(0)}
const out={};let pending=[];
for(const line of m[0].split("\n")){const c=line.match(/case \x27([a-z-]+)\x27/);if(c){pending.push(c[1]);continue}
const r=line.match(/return \x27([a-z]+)\x27/);if(r){for(const p of pending)out[p]=r[1];pending=[]}}
process.stdout.write(JSON.stringify(out))' "$pleach/src/core/classify.ts" 2>/dev/null)"
[ -n "$pleach_classify" ] && [ "$pleach_classify" != "{}" ]; assert $? "pleach's classifyWorkerReason table is readable at src/core/classify.ts"

rows=$(wc -l < "$fixture" | tr -d ' ')
i=0
while [ "$i" -lt "$rows" ]; do
  row="$(sed -n "$((i+1))p" "$fixture")"
  reason="$(node -e 'process.stdout.write(JSON.parse(process.argv[1]).reason)' "$row")"
  code="$(node -e 'process.stdout.write(String(JSON.parse(process.argv[1]).exitCode))' "$row")"
  klass="$(node -e 'process.stdout.write(JSON.parse(process.argv[1]).classification)' "$row")"
  got_code="$(node -e 'const t=JSON.parse(process.argv[1]||"{}");process.stdout.write(t[process.argv[2]]===undefined?"":String(t[process.argv[2]]))' "$umbel_table" "$reason")"
  [ "$got_code" = "$code" ]; assert $? "umbel returns exit $code for $reason" "got '${got_code:-absent}'"
  node -e 'process.exit(JSON.parse(process.argv[1]||"[]").includes(process.argv[2])?0:1)' "$pleach_union" "$reason"; assert $? "pleach's reason union names $reason"
  got_klass="$(node -e 'const t=JSON.parse(process.argv[1]||"{}");process.stdout.write(t[process.argv[2]]||"")' "$pleach_classify" "$reason")"
  [ "$got_klass" = "$klass" ]; assert $? "pleach classifies $reason as $klass" "got '${got_klass:-default (terminal)}'"
  i=$((i+1))
done
extra="$(node -e '
const fs=require("fs");const t=JSON.parse(process.argv[1]||"{}");
const rows=fs.readFileSync(process.argv[2],"utf8").split("\n").filter(Boolean).map(l=>JSON.parse(l).reason);
process.stdout.write(Object.keys(t).filter(k=>!rows.includes(k)).join(","))' "$umbel_table" "$fixture")"
[ -z "$extra" ]; assert $? "umbel names no reason the table lacks" "extra: $extra"
exit "$fail"
