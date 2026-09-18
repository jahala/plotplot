#!/usr/bin/env bash
# Compares the plan schema text tend2 vendors with pleach's canonical text, byte for byte,
# each read at the commit contracts/pins.json pins. pins.test.sh compares version numbers,
# and a version number cannot see a text that changes under it: at v1.1.5 the adapter seam's
# wait gained idleMs in pleach while tend2's vendored copy kept the old line (tend 226). Both
# files come from the platform, or from PLOTPLOT_PIN_PLAN_SCHEMA_FILE and
# PLOTPLOT_PIN_PLAN_SCHEMA_VENDORED_FILE on a machine without gh. Run from the repository
# root, no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 on
# a difference, 3 when a pinned file could not be read.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

test_name="plan-text.test.sh"
# shellcheck source=contracts/test/lib/pinned.sh
. contracts/test/lib/pinned.sh

fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}

for key in plan_schema plan_schema_vendored; do
  node -e "process.exit(require('./contracts/pins.json').$key ? 0 : 1)" 2>/dev/null
  assert $? "contracts/pins.json pins $key"
done
[ "$fail" -eq 0 ] || { echo "$test_name: cannot continue without both pins"; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-plan-text.XXXXXX")"
canonical="$work/canonical.md"
vendored="$work/vendored.md"
if ! read_pinned PLAN_SCHEMA > "$canonical" || ! read_pinned PLAN_SCHEMA_VENDORED > "$vendored"; then
  echo "unevaluable - a pinned plan schema text could not be read; see stderr"
  echo "$test_name: a pinned text could not be read"
  trash "$work" 2>/dev/null || true
  exit 3
fi

[ -s "$canonical" ]; assert $? "pleach's canonical text is not empty at its pinned commit"
[ -s "$vendored" ]; assert $? "tend2's vendored text is not empty at its pinned commit"
if cmp -s "$canonical" "$vendored"; then
  assert 0 "tend2's vendored plan schema is byte-identical to pleach's canonical text"
else
  assert 1 "tend2's vendored plan schema is byte-identical to pleach's canonical text"
  diff "$vendored" "$canonical" | head -12 | sed 's/^/# /'
fi

trash "$work" 2>/dev/null || true
if [ "$fail" -ne 0 ]; then echo "$test_name: failures present"; exit 1; fi
echo "$test_name: all assertions passed"
exit 0
