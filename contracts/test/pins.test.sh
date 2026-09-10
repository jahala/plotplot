#!/usr/bin/env bash
# Compares the loop format version tend2 ships and the plan schema version pleach ships
# against the versions contracts/pins.json declares. Each pin names the repository, the
# file and the commit it was read from; this test fetches that file at that commit from
# the platform (gh, authenticated, since two of the beds are private today). A local
# checkout may be named instead through PLOTPLOT_PIN_<NAME>_FILE, for a machine without
# gh. When neither is available the test cannot judge and says so with exit 3: an
# unverifiable pin is never reported as agreement. Run from the repository root, no
# arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 on a
# mismatch, 3 when it could not read a pinned file.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

pins="contracts/pins.json"
fail=0
unevaluable=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}

[ -f "$pins" ]; assert $? "$pins exists"
[ "$fail" -eq 0 ] || { echo "pins.test.sh: cannot continue without $pins"; exit 1; }

# read_pinned NAME: prints the pinned file's text, from an override path or the platform.
read_pinned() {
  local name="$1" override_var="PLOTPLOT_PIN_$1_FILE" repo file ref
  repo="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].repo)")"
  file="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].file)")"
  ref="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].ref)")"
  if [ -n "${!override_var:-}" ]; then
    [ -f "${!override_var}" ] && cat "${!override_var}" && return 0
    echo "pins.test.sh: $override_var names a file that does not exist: ${!override_var}" >&2; return 3
  fi
  if ! command -v gh >/dev/null 2>&1; then
    echo "pins.test.sh: gh is not installed and $override_var is unset; $name cannot be verified" >&2; return 3
  fi
  local body
  if ! body="$(gh api "repos/$repo/contents/$file?ref=$ref" --jq .content 2>/dev/null)"; then
    echo "pins.test.sh: gh could not read $repo:$file at $ref (not authenticated, or the ref is gone); $name cannot be verified" >&2; return 3
  fi
  printf '%s' "$body" | base64 -d 2>/dev/null || printf '%s' "$body" | base64 -D
}

check_pin() {
  local key="$1" label="$2"
  local pinned pattern text actual
  pinned="$(node -e "console.log(require('./contracts/pins.json').$key.version)")"
  pattern="$(node -e "console.log(require('./contracts/pins.json').$key.pattern)")"
  [ -n "$pinned" ]; assert $? "$pins names a $key.version"
  if text="$(read_pinned "$(echo "$key" | tr a-z A-Z)")"; then
    actual="$(printf '%s\n' "$text" | grep -oE "$pattern" | head -1 | sed -E "s/$pattern/\\1/")"
    if [ -n "$actual" ]; then assert 0 "$label states a version ($actual) at its pinned commit"; else assert 1 "$label states a version at its pinned commit"; fi
    if [ "$actual" = "$pinned" ]; then assert 0 "the pinned $key ($pinned) matches $label ($actual)"; else assert 1 "the pinned $key ($pinned) matches $label (got $actual)"; fi
  else
    unevaluable=1
    echo "unevaluable - $label could not be read; see stderr"
  fi
}

check_pin loop_format "tend2's FORMAT.md"
check_pin plan_schema "pleach's plan-schema.md"

if [ "$fail" -ne 0 ]; then echo "pins.test.sh: failures present"; exit 1; fi
if [ "$unevaluable" -ne 0 ]; then echo "pins.test.sh: a pin could not be verified"; exit 3; fi
echo "pins.test.sh: all assertions passed"
exit 0
