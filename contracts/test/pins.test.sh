#!/usr/bin/env bash
# Compares the loop format version tend2 ships and the plan schema version pleach ships
# against the versions contracts/pins.json declares. Reads tend2 and pleach from their own
# checkouts on this machine (paths named in docs/prompts/contracts-build-2026-09.md); if a
# checkout is not present at its expected path, that comparison fails loudly rather than
# being skipped, since a silent skip would read as agreement. Run from the repository
# root, no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1
# otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

pins="contracts/pins.json"
tend2_format_md="/Users/jahala/conductor/workspaces/feature-map/missoula/FORMAT.md"
pleach_plan_schema_md="/Users/jahala/conductor/workspaces/pleach-v1/cayenne/docs/contract/plan-schema.md"

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

[ -f "$pins" ]; assert $? "$pins exists"
if [ "$fail" -ne 0 ]; then
  echo "pins.test.sh: cannot continue without $pins"
  exit 1
fi

pinned_loop_format="$(node -e 'console.log(require("./contracts/pins.json").loop_format.version)' 2>/dev/null)"
pinned_plan_schema="$(node -e 'console.log(require("./contracts/pins.json").plan_schema.version)' 2>/dev/null)"

[ -n "$pinned_loop_format" ]; assert $? "$pins names a loop_format.version"
[ -n "$pinned_plan_schema" ]; assert $? "$pins names a plan_schema.version"

# --- tend2's loop format version -------------------------------------------------------
if [ -f "$tend2_format_md" ]; then
  assert 0 "tend2's FORMAT.md is readable at $tend2_format_md"
  actual_loop_format="$(grep -oE 'contract, version [0-9]+' "$tend2_format_md" | head -1 | grep -oE '[0-9]+$')"
  if [ -n "$actual_loop_format" ]; then
    assert 0 "tend2's FORMAT.md states a loop format version ($actual_loop_format)"
  else
    assert 1 "tend2's FORMAT.md states a loop format version"
  fi
  if [ "$actual_loop_format" = "$pinned_loop_format" ]; then
    assert 0 "the pinned loop format version ($pinned_loop_format) matches tend2's FORMAT.md ($actual_loop_format)"
  else
    assert 1 "the pinned loop format version ($pinned_loop_format) matches tend2's FORMAT.md (got $actual_loop_format)"
  fi
else
  assert 1 "tend2's FORMAT.md is readable at $tend2_format_md (checkout not found on this machine)"
fi

# --- pleach's plan schema version -------------------------------------------------------
if [ -f "$pleach_plan_schema_md" ]; then
  assert 0 "pleach's plan-schema.md is readable at $pleach_plan_schema_md"
  actual_plan_schema="$(grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' "$pleach_plan_schema_md" | head -1 | sed 's/^v//')"
  if [ -n "$actual_plan_schema" ]; then
    assert 0 "pleach's plan-schema.md states a plan schema version ($actual_plan_schema)"
  else
    assert 1 "pleach's plan-schema.md states a plan schema version"
  fi
  if [ "$actual_plan_schema" = "$pinned_plan_schema" ]; then
    assert 0 "the pinned plan schema version ($pinned_plan_schema) matches pleach's plan-schema.md ($actual_plan_schema)"
  else
    assert 1 "the pinned plan schema version ($pinned_plan_schema) matches pleach's plan-schema.md (got $actual_plan_schema)"
  fi
else
  assert 1 "pleach's plan-schema.md is readable at $pleach_plan_schema_md (checkout not found on this machine)"
fi

if [ "$fail" -eq 0 ]; then
  echo "pins.test.sh: all assertions passed"
  exit 0
else
  echo "pins.test.sh: failures present"
  exit 1
fi
