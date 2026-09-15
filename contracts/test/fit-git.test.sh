#!/usr/bin/env bash
# The fit runner resolves a judge pinned by git rev (jahala/plotplot 60): fit_lock_lookup
# returns the git url and rev for each such judge in contracts/fixtures/garden.lock (tend2, pleach, umbel), lib.sh
# defines fit_clone, and fit_clone on a local fixture repository checks out exactly the rev,
# refusing when the clone's HEAD is not the rev asked for. Run from the repository root, no
# arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
lib="scripts/fit/lib.sh"
fixture="contracts/fixtures/garden.lock"
fail=0
put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
[ -f "$lib" ]; assert $? "the fit runner exists at $lib"
[ -f "$fixture" ]; assert $? "the lock fixture exists at $fixture"
[ "$fail" -eq 0 ] || exit 1

# shellcheck source=/dev/null
. "$lib"

for judge in tend2 pleach umbel; do
  resolved="$(fit_lock_lookup "$fixture" "$judge" 2>/dev/null || true)"
  url="$(node -e 'try{const j=JSON.parse(process.argv[1]||"{}");process.stdout.write((j.git&&j.git.url)||"")}catch{}' "$resolved")"
  rev="$(node -e 'try{const j=JSON.parse(process.argv[1]||"{}");process.stdout.write((j.git&&j.git.rev)||"")}catch{}' "$resolved")"
  [ -n "$url" ] && [ -n "$rev" ]; assert $? "fit_lock_lookup returns git url and rev for the $judge judge" "got: ${resolved:-nothing}"
done

declare -F fit_clone >/dev/null 2>&1; assert $? "lib.sh defines fit_clone <url> <rev> <dest>"
if ! declare -F fit_clone >/dev/null 2>&1; then
  assert 1 "fit_clone checks out exactly the rev asked for on a local fixture repository"
  assert 1 "fit_clone refuses when the clone's HEAD is not the rev asked for"
  exit "$fail"
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit-git.XXXXXX")"
src="$work/src"; mkdir -p "$src"
git -C "$src" init -q -b master && git -C "$src" -c user.name=fixture -c user.email=fixture@plotplot.local commit -q --allow-empty -m "one" && git -C "$src" -c user.name=fixture -c user.email=fixture@plotplot.local commit -q --allow-empty -m "two"
first="$(git -C "$src" rev-parse HEAD~1)"; head_rev="$(git -C "$src" rev-parse HEAD)"
fit_clone "file://$src" "$first" "$work/clone-first" >/dev/null 2>&1
[ "$(git -C "$work/clone-first" rev-parse HEAD 2>/dev/null)" = "$first" ]; assert $? "fit_clone checks out exactly the rev asked for on a local fixture repository"
fit_clone "file://$src" "0000000000000000000000000000000000000000" "$work/clone-bad" >/dev/null 2>&1
[ $? -ne 0 ] && [ ! -d "$work/clone-bad/.git" ]; assert $? "fit_clone refuses when the clone's HEAD is not the rev asked for" "head was $head_rev"
put_away "$work"
exit "$fail"
