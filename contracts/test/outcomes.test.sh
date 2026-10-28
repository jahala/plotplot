#!/usr/bin/env bash
# Validates every contracts/fixtures/outcomes/valid-*.json against contracts/outcomes.schema.json, and
# confirms each negative fixture (a new format version, an outcome outside the 14 steps, a need's
# place without its step, a negative count, a malformed need id) is rejected. Run from the repository root, no arguments. Prints one line per assertion. Exits 0
# if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/outcomes.schema.json"
fixdir="contracts/fixtures/outcomes"

fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then
    echo "ok - $*"
  else
    echo "not ok - $*"
    fail=1
  fi
}

[ -f "$schema" ]; assert $? "outcomes schema exists at $schema"
if [ "$fail" -ne 0 ]; then
  echo "outcomes.test.sh: cannot continue without the schema"
  exit 1
fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema itself compiles as a JSON Schema draft 2020-12 document"
  echo "outcomes.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema itself compiles as a JSON Schema draft 2020-12 document"

valid=0
for f in "$fixdir"/valid-*.json; do
  [ -f "$f" ] || continue
  valid=$((valid + 1))
  npx --no-install ajv validate -s "$schema" -d "$f" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
  assert $? "$(basename "$f") validates against the outcomes schema"
done
[ "$valid" -ge 1 ]; assert $? "at least one valid outcomes fixture is present (found $valid)"

declare_negative() {
  local file="$1" reason="$2"
  if [ ! -f "$file" ]; then
    assert 1 "$reason: fixture $file exists"
    return
  fi
  if npx --no-install ajv validate -s "$schema" -d "$file" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
    assert 1 "$reason: $file is rejected by the outcomes schema"
  else
    assert 0 "$reason: $file is rejected by the outcomes schema"
  fi
}

declare_negative "$fixdir/invalid-format-version.json" "a format version this contract does not name"
declare_negative "$fixdir/invalid-outcome-outside-steps.json" "an outcome id outside the 14 universal steps"
declare_negative "$fixdir/invalid-place-without-step.json" "a need placed on an outcome without its step"
declare_negative "$fixdir/invalid-negative-count.json" "a negative count of people"
declare_negative "$fixdir/invalid-need-id.json" "a need id that is not need: and 16 hex characters"

# every negative fixture in the folder is declared above, so none sits there untested
for file in "$fixdir"/invalid-*.json; do
  name=$(basename "$file")
  if grep -q "declare_negative \"\$fixdir/$name\"" "$0"; then
    assert 0 "$name is declared as a negative fixture"
  else
    assert 1 "$name is declared as a negative fixture"
  fi
done

if [ "$fail" -eq 0 ]; then
  echo "outcomes.test.sh: all assertions passed"
  exit 0
else
  echo "outcomes.test.sh: failures present"
  exit 1
fi
