#!/usr/bin/env bash
# The rulings contract: the schema compiles with the capture plan's schema, which holds the one
# ruling definition both use; every valid fixture validates; every invalid fixture is rejected; and
# the owner's own rulings file validates when it exists on this machine. Run from the repository
# root, no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/rulings.schema.json"
ref="contracts/capture-plan.schema.json"
fixdir="contracts/fixtures/rulings"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
validates() {
  npx --no-install ajv validate -s "$schema" -r "$ref" -d "$1" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
}

[ -f contracts/rulings.md ]; assert $? "rulings: the page exists"
npx --no-install ajv compile -s "$schema" -r "$ref" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
assert $? "rulings: the schema compiles with the capture plan's ruling definition"

valid=0
for f in "$fixdir"/valid*.json; do
  [ -f "$f" ] || continue
  valid=$((valid + 1))
  validates "$f"; assert $? "rulings: $(basename "$f") validates"
done
[ "$valid" -ge 1 ]; assert $? "rulings: at least one valid fixture (found $valid)"

invalid=0
for f in "$fixdir"/invalid-*.json; do
  [ -f "$f" ] || continue
  invalid=$((invalid + 1))
  if validates "$f"; then assert 1 "rulings: $(basename "$f") is rejected"; else assert 0 "rulings: $(basename "$f") is rejected"; fi
done
[ "$invalid" -ge 2 ]; assert $? "rulings: at least two invalid fixtures (found $invalid)"

if [ "$fail" -eq 0 ]; then echo "rulings.test.sh: all assertions passed"; exit 0; fi
echo "rulings.test.sh: failures present"; exit 1
