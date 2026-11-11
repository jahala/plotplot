#!/usr/bin/env bash
# The public umbrella names none of the owner's private projects (contracts/private-names.json,
# scripts/public_names.py). Checks the matcher on a planted name that is not in the list, so this
# file names nothing private, then checks the umbrella's own tracked files. Run from the
# repository root, no arguments. Prints one line per assertion. Exits 0 if every assertion
# passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}

match() {
  python3 -c '
import sys; sys.path.insert(0, "scripts")
from public_names import fnv1a, hits
print(hits(sys.argv[1], {fnv1a("zz-planted")}))' "$1"
}

[ "$(match $'one\nsee zz-planted here\nthree')" = "[2]" ]; assert $? "a planted name is found on its line"
[ "$(match 'ZZ-Planted, in capitals')" = "[1]" ]; assert $? "case does not hide a name"
[ "$(match 'zz-plantedx and xzz-planted')" = "[]" ]; assert $? "a longer word is not a name"
[ "$(match 'path/zz-planted/file.md')" = "[1]" ]; assert $? "a name inside a path is found"

out="$(python3 scripts/public_names.py . 2>&1)"; status=$?
[ "$status" -eq 0 ]; assert $? "the umbrella's tracked files name no private project${out:+ ($out)}"

python3 -c 'import json; d = json.load(open("contracts/private-names.json")); assert d["hashes"] and all(len(h) == 16 for h in d["hashes"])'
assert $? "the list holds 64-bit hashes only"

exit "$fail"
