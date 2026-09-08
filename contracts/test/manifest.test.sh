#!/usr/bin/env bash
# Validates every positive contracts/fixtures/manifest/*.garden.json against
# contracts/manifest.schema.json, and confirms each negative fixture (missing kind,
# undeclared context cost, no metric command) is rejected. Run from the repository root,
# no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1
# otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/manifest.schema.json"
fixdir="contracts/fixtures/manifest"

fail=0

put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then
    echo "ok - $*"
  else
    echo "not ok - $*"
    fail=1
  fi
}

[ -f "$schema" ]; assert $? "manifest schema exists at $schema"
[ -d "$fixdir" ]; assert $? "manifest fixture directory exists at $fixdir"

if [ "$fail" -ne 0 ]; then
  echo "manifest.test.sh: cannot continue without the schema and the fixture directory"
  exit 1
fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema itself compiles as a JSON Schema draft 2020-12 document"
  echo "manifest.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema itself compiles as a JSON Schema draft 2020-12 document"

positive_count=0
for f in "$fixdir"/*.garden.json; do
  base="$(basename "$f")"
  case "$base" in
    invalid-*) continue ;;
  esac
  positive_count=$((positive_count + 1))
  if npx --no-install ajv validate -s "$schema" -d "$f" --spec=draft2020 -c ajv-formats >/tmp/manifest-test-out.$$ 2>&1; then
    assert 0 "$base validates against the manifest schema"
  else
    assert 1 "$base validates against the manifest schema"
    sed 's/^/    /' /tmp/manifest-test-out.$$
  fi
  put_away /tmp/manifest-test-out.$$
done

expected_beds="tilth tend2 petals pleach umbel copeca pollen weeder"
for bed in $expected_beds; do
  if [ -f "$fixdir/$bed.garden.json" ]; then
    assert 0 "a positive fixture exists for bed '$bed'"
  else
    assert 1 "a positive fixture exists for bed '$bed'"
  fi
done

if [ "$positive_count" -ge 8 ]; then
  assert 0 "at least one positive fixture per named bed is present (found $positive_count)"
else
  assert 1 "at least one positive fixture per named bed is present (found $positive_count, want >= 8)"
fi

declare_negative() {
  local file="$1" reason="$2"
  if [ ! -f "$file" ]; then
    assert 1 "$reason: fixture $file exists"
    return
  fi
  if npx --no-install ajv validate -s "$schema" -d "$file" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
    assert 1 "$reason: $file is rejected by the manifest schema"
  else
    assert 0 "$reason: $file is rejected by the manifest schema"
  fi
}

declare_negative "$fixdir/invalid-missing-kind.garden.json" "a missing kind"
declare_negative "$fixdir/invalid-git-no-rev.garden.json" "a git install without a pinned commit"
declare_negative "$fixdir/invalid-smoke-shell-string.garden.json" "a smoke command written as a shell string"
declare_negative "$fixdir/invalid-repository-as-bed.garden.json" "a bed manifest missing install, faces, check, metric, context and coverage"
declare_negative "$fixdir/invalid-undeclared-context.garden.json" "an undeclared context cost"
declare_negative "$fixdir/invalid-no-metric.garden.json" "no metric command"

if [ "$fail" -eq 0 ]; then
  echo "manifest.test.sh: all assertions passed"
  exit 0
else
  echo "manifest.test.sh: failures present"
  exit 1
fi
