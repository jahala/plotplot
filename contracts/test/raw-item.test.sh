#!/usr/bin/env bash
# Validates every line of contracts/fixtures/raw-item/valid.jsonl against
# contracts/raw-item.schema.json, and confirms each negative fixture (no text, empty text, no
# date of fetching, an id that is not a sha256, an unknown field, a private item with a URL
# but no flag) is rejected. Run from the repository root, no arguments. Prints one line per
# assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/raw-item.schema.json"
fixdir="contracts/fixtures/raw-item"

fail=0
scratch="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-raw-item.XXXXXX")"
put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
trap 'put_away "$scratch"' EXIT
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then
    echo "ok - $*"
  else
    echo "not ok - $*"
    fail=1
  fi
}

[ -f "$schema" ]; assert $? "raw item schema exists at $schema"
[ -f "$fixdir/valid.jsonl" ]; assert $? "valid fixture exists at $fixdir/valid.jsonl"
if [ "$fail" -ne 0 ]; then
  echo "raw-item.test.sh: cannot continue without the schema and the valid fixture"
  exit 1
fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema itself compiles as a JSON Schema draft 2020-12 document"
  echo "raw-item.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema itself compiles as a JSON Schema draft 2020-12 document"

# JSON Lines: one item per line, each validated on its own.
lines=0
while IFS= read -r line || [ -n "$line" ]; do
  [ -z "$line" ] && continue
  lines=$((lines + 1))
  item="$scratch/valid-$lines.json"
  printf '%s\n' "$line" > "$item"
  npx --no-install ajv validate -s "$schema" -d "$item" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
  assert $? "valid.jsonl line $lines validates against the raw item schema"
done < "$fixdir/valid.jsonl"
[ "$lines" -ge 3 ]; assert $? "valid.jsonl holds at least a public, a private and a reply item (found $lines)"

declare_negative() {
  local file="$1" reason="$2"
  if [ ! -f "$file" ]; then
    assert 1 "$reason: fixture $file exists"
    return
  fi
  if npx --no-install ajv validate -s "$schema" -d "$file" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
    assert 1 "$reason: $file is rejected by the raw item schema"
  else
    assert 0 "$reason: $file is rejected by the raw item schema"
  fi
}

declare_negative "$fixdir/invalid-no-text.json" "an item with no text"
declare_negative "$fixdir/invalid-empty-text.json" "an item whose text is empty"
declare_negative "$fixdir/invalid-no-fetched-at.json" "an item with no time of fetching"
declare_negative "$fixdir/invalid-id-not-sha256.json" "an id that is not a sha256"
declare_negative "$fixdir/invalid-unknown-field.json" "a field the contract does not name"
declare_negative "$fixdir/invalid-no-private-flag.json" "an item that does not say whether it is private"

if [ "$fail" -eq 0 ]; then
  echo "raw-item.test.sh: all assertions passed"
  exit 0
else
  echo "raw-item.test.sh: failures present"
  exit 1
fi
