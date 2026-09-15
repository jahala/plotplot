#!/usr/bin/env bash
# Runs every contracts/test/*.test.sh, prints one line per test with its outcome, and exits
# 1 if a test that is not listed in contracts/test/EXPECTED_RED failed. A listed test still
# runs and is reported (red or unevaluable, exit 3); a listed test that passes is reported
# as "green, remove its EXPECTED_RED line". Tests that need a checkout or a tool they cannot
# find exit 3 and are reported as unevaluable, never as green. Run from the repository root.
set -u
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1
expected="contracts/test/EXPECTED_RED"
fail=0
for t in contracts/test/*.test.sh; do
  name="$(basename "$t")"
  listed=0
  grep -q -E "^${name//./\\.}[[:space:]]" "$expected" 2>/dev/null && listed=1
  log="$(mktemp "${TMPDIR:-/tmp}/plotplot-contracts-test.XXXXXX")"
  bash "$t" > "$log" 2>&1
  status=$?
  case "$status" in
    0) if [ "$listed" -eq 1 ]; then echo "$name: green, remove its EXPECTED_RED line"; fail=1; else echo "$name: ok"; fi ;;
    3) echo "$name: unevaluable ($(grep -m1 -E '^[a-z-]+\.test\.sh:' "$log" | cut -d: -f2- | sed 's/^ //' | cut -c1-110))"; [ "$listed" -eq 1 ] || fail=1 ;;
    *) n="$(grep -c '^not ok' "$log")"; if [ "$listed" -eq 1 ]; then echo "$name: red as recorded ($n not ok)"; else echo "$name: FAILED ($n not ok, exit $status)"; grep '^not ok' "$log" | head -5 | sed 's/^/  /'; fail=1; fi ;;
  esac
  if command -v trash >/dev/null 2>&1; then trash "$log" 2>/dev/null || true; fi
done
exit "$fail"
