#!/usr/bin/env bash
# Validates the proof record (contracts/delivery.md, "What a gate may trust") against
# contracts/proof.schema.json: the positive fixture under contracts/fixtures/proof/ validates,
# and each negative fixture is rejected for the reason its name gives (a failure recorded, no
# claim digest, a commit as the subject, output carried, an absolute path, nothing counted).
# Run from the repository root, no arguments. Prints one line per assertion. Exits 0 if every
# assertion passed, 1 otherwise.
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
schema="contracts/proof.schema.json"
fixdir="contracts/fixtures/proof"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$schema" ]; assert $? "the proof record schema exists at $schema"
[ -d "$fixdir" ]; assert $? "the fixture directory exists at $fixdir"
[ "$fail" -eq 0 ] || { echo "proof.test.sh: cannot continue without the schema and the fixtures"; exit 1; }

npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
assert $? "$schema compiles as a JSON Schema draft 2020-12 document"
[ "$fail" -eq 0 ] || { echo "proof.test.sh: cannot continue with an invalid schema"; exit 1; }

positives=0; negatives=0
for f in "$fixdir"/*.json; do
  base="$(basename "$f")"
  if npx --no-install ajv validate -s "$schema" -d "$f" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then valid=0; else valid=1; fi
  case "$base" in
    invalid-*) negatives=$((negatives + 1)); [ "$valid" -eq 1 ]; assert $? "$base is rejected" ;;
    *) positives=$((positives + 1)); [ "$valid" -eq 0 ]; assert $? "$base validates" ;;
  esac
done
[ "$positives" -ge 1 ]; assert $? "at least one positive fixture was read ($positives)"
[ "$negatives" -ge 6 ]; assert $? "every named way of being wrong has a fixture ($negatives)"

grep -q 'What a gate may trust' contracts/delivery.md
assert $? "contracts/delivery.md carries the section the schema serves"

if [ "$fail" -ne 0 ]; then echo "proof.test.sh: failures present"; exit 1; fi
echo "proof.test.sh: all assertions passed"
exit 0
