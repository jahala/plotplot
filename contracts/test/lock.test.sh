#!/usr/bin/env bash
# Validates contracts/fixtures/garden.lock against contracts/lock.schema.json (parsing the
# TOML fixture into JSON first, since the schema describes the parsed value), and proves
# the fit runner refuses a pinned artifact whose checksum does not match. Run from the
# repository root, no arguments. Prints one line per assertion. Exits 0 if every assertion
# passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/lock.schema.json"
fixture="contracts/fixtures/garden.lock"
lib="scripts/fit/lib.sh"

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

[ -f "$schema" ]; assert $? "lock schema exists at $schema"
[ -f "$fixture" ]; assert $? "garden.lock fixture exists at $fixture"
[ -f "$lib" ]; assert $? "the fit runner exists at $lib"

if [ "$fail" -ne 0 ]; then
  echo "lock.test.sh: cannot continue without the schema, the fixture and the fit runner"
  exit 1
fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema itself compiles as a JSON Schema draft 2020-12 document"
  echo "lock.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema itself compiles as a JSON Schema draft 2020-12 document"

work="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-lock-test.XXXXXX")"
cleanup() { trash "$work" >/dev/null 2>&1 || true; }
trap cleanup EXIT

parsed_json="$work/garden.lock.json"
if node -e '
  const {parse} = require("smol-toml");
  const fs = require("fs");
  const doc = parse(fs.readFileSync(process.argv[1], "utf8"));
  fs.writeFileSync(process.argv[2], JSON.stringify(doc));
' "$fixture" "$parsed_json" 2>"$work/parse-error.txt"; then
  assert 0 "$fixture parses as TOML"
else
  assert 1 "$fixture parses as TOML"
  sed 's/^/    /' "$work/parse-error.txt"
fi

if [ -f "$parsed_json" ]; then
  if npx --no-install ajv validate -s "$schema" -d "$parsed_json" --spec=draft2020 -c ajv-formats >"$work/validate-out.txt" 2>&1; then
    assert 0 "the parsed garden.lock fixture validates against $schema"
  else
    assert 1 "the parsed garden.lock fixture validates against $schema"
    sed 's/^/    /' "$work/validate-out.txt"
  fi
else
  assert 1 "the parsed garden.lock fixture validates against $schema"
fi

# --- the fit runner refuses a checksum mismatch, driven by a garden.lock-shaped fixture --
# shellcheck source=/dev/null
source "$lib"

pkg_dir="$work/pkg"
mkdir -p "$pkg_dir"
marker_file="$work/ran.marker"
cat >"$pkg_dir/lockjudge" <<EOF
#!/usr/bin/env bash
echo "ran" > "$marker_file"
exit 0
EOF
chmod +x "$pkg_dir/lockjudge"
artifact="$work/lockjudge.tar.gz"
tar -czf "$artifact" -C "$pkg_dir" lockjudge
real_sha="$(fit_sha256 "$artifact")"
wrong_sha="$(printf '%s' "$real_sha" | tr '0-9a-f' '1-9a-f0')"

mismatched_lock="$work/mismatched.lock"
cat >"$mismatched_lock" <<EOF
season = "2026.09"

[judges.lockjudge]
version = "1.0.0"

[judges.lockjudge.platforms."fixture-test-platform"]
url = "file://$artifact"
sha256 = "$wrong_sha"
EOF

put_away "$marker_file"
mismatch_output="$(fit_run "$mismatched_lock" lockjudge fixture-test-platform lockjudge -- 2>&1)"
mismatch_status=$?

[ "$mismatch_status" -ne 0 ]; assert $? "the fit runner exits non-zero for a garden.lock entry whose sha256 does not match its artifact"
[ ! -f "$marker_file" ]; assert $? "the fit runner never ran the mismatched artifact"
case "$mismatch_output" in
  *"checksum mismatch"*) assert 0 "the refusal names the checksum mismatch" ;;
  *) assert 1 "the refusal names the checksum mismatch (output: $mismatch_output)" ;;
esac

# and the matching-checksum case actually runs, so the mismatch case above is a real
# negative and not a lock file the fit runner could never satisfy at all
correct_lock="$work/correct.lock"
cat >"$correct_lock" <<EOF
season = "2026.09"

[judges.lockjudge]
version = "1.0.0"

[judges.lockjudge.platforms."fixture-test-platform"]
url = "file://$artifact"
sha256 = "$real_sha"
EOF
put_away "$marker_file"
fit_run "$correct_lock" lockjudge fixture-test-platform lockjudge -- >/dev/null 2>&1
correct_status=$?
[ "$correct_status" -eq 0 ] && [ -f "$marker_file" ]
assert $? "the same artifact with its real sha256 runs cleanly (control for the mismatch case)"

# A git-pinned judge must name a full commit sha; a short or symbolic rev is refused.
bad_lock="$work/bad-git.lock.json"
cat >"$bad_lock" <<'JSON'
{"season":"2026.09","judges":{"pollen":{"version":"0.1.0","git":{"url":"https://github.com/jahala/pollen.git","rev":"main"}}}}
JSON
if npx --no-install ajv validate -s "$schema" -d "$bad_lock" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "the lock schema refuses a git judge whose rev is not a 40-hex commit sha (it accepted 'main')"
else
  assert 0 "the lock schema refuses a git judge whose rev is not a 40-hex commit sha"
fi
good_lock="$work/good-git.lock.json"
cat >"$good_lock" <<'JSON'
{"season":"2026.09","judges":{"pollen":{"version":"0.1.0","git":{"url":"https://github.com/jahala/pollen.git","rev":"ffb905bf6b3a01335b85f8202be3c250bd0de9e6"}}}}
JSON
if npx --no-install ajv validate -s "$schema" -d "$good_lock" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 0 "the lock schema accepts a git judge pinned to a full commit sha"
else
  assert 1 "the lock schema accepts a git judge pinned to a full commit sha"
fi

if [ "$fail" -eq 0 ]; then
  echo "lock.test.sh: all assertions passed"
  exit 0
else
  echo "lock.test.sh: failures present"
  exit 1
fi
