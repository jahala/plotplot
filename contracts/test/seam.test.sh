#!/usr/bin/env bash
# The seam between tend2's emit-plan and pleach: a plan tend2 writes from a map is a plan
# pleach runs. Copies the fixture map contracts/fixtures/seam/ into a scratch directory
# beside the renderer `tend2 init` writes, closes with `tend2 verify` every loop whose
# evidence the fixture carries (so a fully verified loop is stamped by the verifier, never
# by hand), emits a plan with a cast file naming every loop and a setup of one argv, then
# validates the plan against the schema the pleach under test prints and with
# `pleach validate`. contracts/test/lib/seam-plan.mjs then asserts, from the map's own
# checks, what the plan must carry: a phased node per open code check of an all-code loop
# (jahala/tend 163) closed by a command node (jahala/tend 158), one prompt node for a mixed
# loop, no node for a verified loop and a preflight line naming it, the cast on every
# worker node, a setup of one argv on every node, and node ids that resolve to loop ids
# and check ordinals per contracts/identifiers.md.
#
# The tools under test: PLOTPLOT_PLEACH_BIN, else `pleach` on PATH; PLOTPLOT_TEND2_BIN,
# else `tend2` on PATH. Either may be a command of several words (for example
# PLOTPLOT_PLEACH_BIN="bun /tmp/pleach-pinned/src/main.ts"). Which of each ran is printed
# on stdout first, so a verdict is attributable.
#
# Run from the repository root, no arguments. Prints one line per assertion. Exits 0 if
# every assertion passed, 1 otherwise, 3 when a tool it needs is absent (reason on stderr).
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

fixture="contracts/fixtures/seam"
helper="contracts/test/lib/seam-plan.mjs"
runner='bash {evidence}'
setup="bash -lc 'test -d docs/tend2 && test -f docs/tend2/loop.js'"

fail=0

put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
assert() {
  local status="$1" message="$2" reason="${3:-}"
  if [ "$status" -eq 0 ]; then
    echo "ok - $message"
  else
    echo "not ok - $message${reason:+ ($reason)}"
    fail=1
  fi
}
indent() { sed 's/^/    /' "$1"; }

# provenance WORD...: the resolved path of the last word that names a file, and the git
# commit of the checkout it lives in, if any.
provenance() {
  local word file=""
  for word in "$@"; do
    if [ -f "$word" ]; then file="$word"; elif command -v "$word" >/dev/null 2>&1; then
      case "$(command -v "$word")" in /*) file="$(command -v "$word")" ;; esac
    fi
  done
  [ -n "$file" ] || { echo "no file resolved"; return; }
  local real sha
  real="$(node -e 'console.log(require("fs").realpathSync(process.argv[1]))' "$file")"
  if sha="$(git -C "$(dirname "$real")" rev-parse HEAD 2>/dev/null)"; then
    echo "$real, git $sha"
  else
    echo "$real, not a git checkout"
  fi
}

if [ -n "${PLOTPLOT_PLEACH_BIN:-}" ]; then read -r -a pleach <<< "$PLOTPLOT_PLEACH_BIN"; else pleach=(pleach); fi
if [ -n "${PLOTPLOT_TEND2_BIN:-}" ]; then read -r -a tend2 <<< "$PLOTPLOT_TEND2_BIN"; else tend2=(tend2); fi

if ! pleach_version="$("${pleach[@]}" --version 2>/dev/null)"; then
  echo "seam.test.sh: pleach does not run as '${pleach[*]}'; set PLOTPLOT_PLEACH_BIN or put pleach on PATH" >&2
  exit 3
fi
if ! "${tend2[@]}" --help 2>&1 | grep -q 'emit-plan'; then
  echo "seam.test.sh: tend2 with emit-plan does not run as '${tend2[*]}'; set PLOTPLOT_TEND2_BIN or put tend2 on PATH" >&2
  exit 3
fi
if ! npx --no-install ajv help >/dev/null 2>&1; then
  echo "seam.test.sh: ajv-cli is not installed; run npm install" >&2
  exit 3
fi
tend2_version="$("${tend2[@]}" --version 2>/dev/null | head -1)"

echo "# pleach: ${pleach[*]} --version: $pleach_version ($(provenance "${pleach[@]}"))"
echo "# tend2: ${tend2[*]} --version: ${tend2_version:-none printed} ($(provenance "${tend2[@]}"))"

# The verifier a plan's gates invoke must resolve outside the worktree, so it is the tend2
# under test by absolute path.
if [ "${#tend2[@]}" -eq 1 ]; then verify_bin="$(command -v "${tend2[0]}")"; else verify_bin="${tend2[*]}"; fi

[ -d "$fixture/docs/tend2" ] && [ -f "$fixture/cast.json" ]
assert $? "the seam fixture map and its cast file exist at $fixture"
[ "$fail" -eq 0 ] || { echo "seam.test.sh: cannot continue without the fixture"; exit 1; }

scratch="$(mktemp -d)"
trap 'put_away "$scratch"' EXIT
map="$scratch/docs/tend2"

# tend2 init writes the renderer and its own project loop; the fixture supplies the loops.
(cd "$scratch" && "${tend2[@]}" init >"$scratch/init.log" 2>&1)
init_status=$?
[ "$init_status" -eq 0 ] && [ -f "$map/loop.js" ] && [ -f "$map/loop.css" ]
assert $? "tend2 init scaffolds a map with its renderer in the scratch directory" "exit $init_status"
for page in "$map"/*.tend2.html; do [ -e "$page" ] && put_away "$page"; done
cp -R "$fixture/." "$scratch/"

stamped_ok=0
for loop in $(node "$helper" evidence-complete "$map" "$scratch"); do
  if "${tend2[@]}" verify "$map/$loop.tend2.html" --repo-root "$scratch" --runner "$runner" >"$scratch/verify.log" 2>&1 \
    && ! grep -q '^- \[[ !~]\]' "$map/$loop.tend2.html"; then
    assert 0 "tend2 verify closes $loop, whose evidence the fixture carries"
  else
    assert 1 "tend2 verify closes $loop, whose evidence the fixture carries"
    indent "$scratch/verify.log"
  fi
  stamped_ok=1
done
[ "$stamped_ok" -eq 1 ]; assert $? "the fixture map holds a loop the verifier can close"

if "${tend2[@]}" lint "$map"/*.tend2.html >"$scratch/lint.log" 2>&1; then
  assert 0 "every fixture loop lints clean under tend2 lint"
else
  assert 1 "every fixture loop lints clean under tend2 lint"
  indent "$scratch/lint.log"
fi

plan="$scratch/plan.json"
"${tend2[@]}" emit-plan "$map" --repo-root "$scratch" --out "$plan" --cast-file "$scratch/cast.json" \
  --verify-bin "$verify_bin" --runner "$runner" --setup "$setup" >"$scratch/emit.log" 2>&1
emit_status=$?
[ "$emit_status" -eq 0 ] && [ -f "$plan" ]
assert $? "tend2 emit-plan writes a plan from the fixture map" "exit $emit_status: $(tr '\n' ' ' < "$scratch/emit.log")"
[ -f "$plan" ] || { echo "seam.test.sh: failures present"; exit 1; }

schema="$scratch/plan.schema.json"
if "${pleach[@]}" schema >"$schema" 2>"$scratch/schema.err" \
  && npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 0 "pleach schema prints a plan schema that compiles as JSON Schema draft 2020-12"
  if npx --no-install ajv validate -s "$schema" -d "$plan" --spec=draft2020 -c ajv-formats --errors=json >"$scratch/ajv.log" 2>&1; then
    assert 0 "the plan validates against the plan schema pleach prints"
  else
    assert 1 "the plan validates against the plan schema pleach prints" "$(node "$helper" schema-errors "$scratch/ajv.log")"
  fi
else
  assert 1 "pleach schema prints a plan schema that compiles as JSON Schema draft 2020-12" "$(tr '\n' ' ' < "$scratch/schema.err")"
  assert 1 "the plan validates against the plan schema pleach prints" "no schema to validate against"
fi

"${pleach[@]}" validate "$plan" >"$scratch/validate.log" 2>&1
validate_status=$?
if [ "$validate_status" -eq 0 ] && grep -q '"valid":true' "$scratch/validate.log"; then
  assert 0 "pleach validate accepts the plan: exit 0 and \"valid\":true"
else
  assert 1 "pleach validate accepts the plan: exit 0 and \"valid\":true" "exit $validate_status"
  indent "$scratch/validate.log"
fi

node "$helper" assert "$plan" "$map" "$scratch" "$scratch/cast.json" "$runner" "$scratch/emit.log"
[ $? -eq 0 ] || fail=1

if [ "$fail" -eq 0 ]; then
  echo "seam.test.sh: all assertions passed"
  exit 0
else
  echo "seam.test.sh: failures present"
  exit 1
fi
