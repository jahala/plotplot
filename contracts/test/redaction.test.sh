#!/usr/bin/env bash
# Feeds every row of contracts/fixtures/redaction/cases.jsonl to the stem's redact face and
# (rows stored base64 at rest, so no scanner mistakes the table for a leak) and compares
# standard output to the row's decoded `out` byte for byte and the tally's total to the
# row's `count` (contracts/redaction.md). The face under test is PLOTPLOT_REDACT_BIN, else
# `plotplot` on PATH; either may be a command of several words. Which ran is printed first,
# so a verdict is attributable. When no stem with a redact face is available the test cannot
# judge and says so with exit 3: an unverifiable contract is never reported as agreement.
# Run from the repository root, no arguments. Prints one line per assertion. Exits 0 if
# every assertion passed, 1 otherwise, 3 when the face is absent (reason on stderr).
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
cases="contracts/fixtures/redaction/cases.jsonl"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$cases" ]; assert $? "the fixture table exists at $cases"
[ "$fail" -eq 0 ] || exit 1

read -r -a bin <<< "${PLOTPLOT_REDACT_BIN:-plotplot}"
if ! command -v "${bin[0]}" >/dev/null 2>&1; then
  echo "redaction.test.sh: ${bin[0]} is not on PATH and PLOTPLOT_REDACT_BIN is unset; the face cannot be tested" >&2
  exit 3
fi
if ! printf 'probe' | "${bin[@]}" redact >/dev/null 2>&1; then
  echo "redaction.test.sh: '${bin[*]} redact' does not run at $("${bin[@]}" --version 2>/dev/null | head -1); the stem has no redact face yet (jahala/plotplot 52)" >&2
  exit 3
fi
echo "redact face: ${bin[*]} ($("${bin[@]}" --version 2>/dev/null | head -1))"

rows=$(node -e 'const fs=require("fs");const lines=fs.readFileSync(process.argv[1],"utf8").split("\n").filter(Boolean);console.log(lines.length)' "$cases")
i=0
while [ "$i" -lt "$rows" ]; do
  id=$(node -e 'const r=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8").split("\n").filter(Boolean)[+process.argv[2]]);process.stdout.write(r.id)' "$cases" "$i")
  want=$(node -e 'const r=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8").split("\n").filter(Boolean)[+process.argv[2]]);process.stdout.write(Buffer.from(r.out,"base64").toString("utf8"))' "$cases" "$i")
  count=$(node -e 'const r=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8").split("\n").filter(Boolean)[+process.argv[2]]);process.stdout.write(String(r.count))' "$cases" "$i")
  got=$(node -e 'const r=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8").split("\n").filter(Boolean)[+process.argv[2]]);process.stdout.write(Buffer.from(r.in,"base64").toString("utf8"))' "$cases" "$i" | "${bin[@]}" redact --tally 2>"/tmp/redaction-tally.$$")
  status=$?
  total=$(node -e 'try{const t=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));process.stdout.write(String(t.total))}catch{process.stdout.write("?")}' "/tmp/redaction-tally.$$")
  [ "$status" -eq 0 ] && [ "$got" = "$want" ]; assert $? "$id: output matches the fixture" "exit $status"
  [ "$total" = "$count" ]; assert $? "$id: tally total is $count" "got $total"
  i=$((i+1))
done
rm -f "/tmp/redaction-tally.$$" 2>/dev/null
exit "$fail"
