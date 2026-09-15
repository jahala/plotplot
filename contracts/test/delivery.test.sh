#!/usr/bin/env bash
# contracts/delivery.md is cited by a test in tend2 and a test in pleach at their pinned
# versions, and each test holds its side. For each bed this test finds the test files that
# cite the contract by path, asserts there is at least one, asserts the citing tests cover
# the bed's rules (tend2: the payload pin covers the checks section only; pleach: scratch is
# never collected and whole-map artefacts are re-earned at land), and runs those files with
# the bed's own runner (vitest for tend2, bun test for pleach), which must exit 0.
#
# The checkouts under test: PLOTPLOT_TEND2_SRC and PLOTPLOT_PLEACH_SRC, each a checkout of
# the bed at the version the umbrella pins. Which revision ran is printed first, so a
# verdict is attributable. Run from the repository root, no arguments. Prints one line per
# assertion. Exits 0 if every assertion passed, 1 otherwise, 3 when a checkout is not
# named or does not exist (reason on stderr).
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
contract="contracts/delivery.md"
fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$contract" ]; assert $? "the contract exists at $contract"
[ "$fail" -eq 0 ] || exit 1

for var in PLOTPLOT_TEND2_SRC PLOTPLOT_PLEACH_SRC; do
  if [ -z "${!var:-}" ] || [ ! -d "${!var}" ]; then
    echo "delivery.test.sh: $var is unset or not a directory; the bed's tests cannot be read" >&2
    exit 3
  fi
done

# citing_tests SRC: prints the test files under SRC that name the contract by path.
citing_tests() {
  grep -rl --include='*.test.ts' --include='*.test.mjs' --include='*.test.js' --include='*.test.sh' "$contract" "$1/test" "$1/src" "$1/tests" 2>/dev/null || true
}
# covers FILES... PATTERN LABEL: at least one of the files mentions the pattern.
covers() {
  local pattern="$1" label="$2"; shift 2
  local hit=1
  for f in "$@"; do grep -q -E -- "$pattern" "$f" 2>/dev/null && hit=0; done
  assert "$hit" "$label"
}

check_bed() {
  local bed="$1" src="$2" runner="$3"; shift 3
  local rev files
  rev="$(git -C "$src" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  echo "$bed: $src at $rev"
  files="$(citing_tests "$src")"
  [ -n "$files" ]; assert $? "$bed: at least one test cites $contract"
  if [ -z "$files" ]; then
    for label in "$@"; do assert 1 "$bed: a citing test covers: $label"; done
    assert 1 "$bed: the citing tests pass under its runner ($runner)"
    return
  fi
  # shellcheck disable=SC2086
  set -- "$@"
  local f_list=()
  while IFS= read -r f; do f_list+=("$f"); done <<< "$files"
  for spec in "$@"; do
    covers "${spec%%|*}" "$bed: a citing test covers: ${spec#*|}" "${f_list[@]}"
  done
  local rel=()
  for f in "${f_list[@]}"; do rel+=("${f#"$src/"}"); done
  (cd "$src" && $runner "${rel[@]}") > /tmp/delivery-test-run.$$ 2>&1
  local status=$?
  [ "$status" -eq 0 ]; assert $? "$bed: the citing tests pass under its runner ($runner)" "exit $status: $(tail -n 3 /tmp/delivery-test-run.$$ | tr '\n' ' ')"
  rm -f /tmp/delivery-test-run.$$ 2>/dev/null
}

check_bed tend2 "$PLOTPLOT_TEND2_SRC" "npx --no-install vitest run -c vitest.config.ts" \
  "expect-payload|checks section|checksSection|payload pin|the pin covers the checks section only"
check_bed pleach "$PLOTPLOT_PLEACH_SRC" "bun test" \
  "loop-scratch|scratch is never collected" \
  "re-earned|whole-map|whole map|whole-map artefacts are re-earned at land"
exit "$fail"
