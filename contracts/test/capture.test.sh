#!/usr/bin/env bash
# The capture contracts mull owns: protocol, adapter, document, observation, subject, reaction
# and capture-plan. For each, the schema compiles; every valid fixture validates (a .jsonl
# fixture line by line); every invalid fixture is rejected. Ids are recomputed by mull's own
# test against the rule each page states. Run from the repository root, no arguments. Prints one
# line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

names="protocol adapter document observation subject reaction capture-plan"
fail=0
scratch="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-capture.XXXXXX")"
put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
trap 'put_away "$scratch"' EXIT
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
validates() {
  npx --no-install ajv validate -s "$1" -d "$2" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
}

for name in $names; do
  schema="contracts/$name.schema.json"
  page="contracts/$name.md"
  fixdir="contracts/fixtures/$name"
  [ -f "$page" ]; assert $? "$name: the page exists at $page"
  [ -f "$schema" ]; assert $? "$name: the schema exists at $schema"
  [ -f "$schema" ] || continue
  npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
  assert $? "$name: the schema compiles as JSON Schema draft 2020-12"

  valid=0
  for f in "$fixdir"/valid*.json; do
    [ -f "$f" ] || continue
    valid=$((valid + 1))
    validates "$schema" "$f"; assert $? "$name: $(basename "$f") validates"
  done
  for f in "$fixdir"/valid*.jsonl; do
    [ -f "$f" ] || continue
    n=0
    while IFS= read -r line || [ -n "$line" ]; do
      [ -z "$line" ] && continue
      n=$((n + 1)); valid=$((valid + 1))
      printf '%s\n' "$line" > "$scratch/line.json"
      validates "$schema" "$scratch/line.json"; assert $? "$name: $(basename "$f") line $n validates"
    done < "$f"
  done
  [ "$valid" -ge 1 ]; assert $? "$name: at least one valid fixture (found $valid)"

  invalid=0
  for f in "$fixdir"/invalid-*.json; do
    [ -f "$f" ] || continue
    invalid=$((invalid + 1))
    if validates "$schema" "$f"; then
      assert 1 "$name: $(basename "$f") is rejected"
    else
      assert 0 "$name: $(basename "$f") is rejected"
    fi
  done
  [ "$invalid" -ge 2 ]; assert $? "$name: at least two invalid fixtures (found $invalid)"
done

if [ "$fail" -eq 0 ]; then echo "capture.test.sh: all assertions passed"; exit 0; fi
echo "capture.test.sh: failures present"; exit 1
