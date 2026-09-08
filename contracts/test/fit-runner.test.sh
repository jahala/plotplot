#!/usr/bin/env bash
# Proves scripts/fit/lib.sh end to end on a local fixture artifact: builds a tarball with
# a known checksum in this test, fetches and runs it through the fit runner, checks the
# check ran with a PATH holding no other garden tool, then proves a second run with a
# wrong checksum refuses to run anything. Run from the repository root, no arguments.
# Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
lib="$repo_root/scripts/fit/lib.sh"

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

[ -f "$lib" ]; assert $? "scripts/fit/lib.sh exists"
if [ "$fail" -ne 0 ]; then
  echo "fit-runner.test.sh: cannot continue without scripts/fit/lib.sh"
  exit 1
fi

# shellcheck source=/dev/null
source "$lib"

work="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit-runner-test.XXXXXX")"
cleanup() { trash "$work" >/dev/null 2>&1 || true; }
trap cleanup EXIT

# --- build a fixture artifact with a known-good checksum -----------------------------
pkg_dir="$work/pkg"
mkdir -p "$pkg_dir"
marker_file="$work/ran.marker"
cat >"$pkg_dir/fakejudge" <<EOF
#!/usr/bin/env bash
echo "ran" > "$marker_file"
echo "fakejudge-output: \$*"
if command -v other-garden-tool >/dev/null 2>&1; then
  echo "LEAK: other-garden-tool is reachable on PATH" >&2
  exit 9
fi
exit 0
EOF
chmod +x "$pkg_dir/fakejudge"

artifact="$work/artifact.tar.gz"
tar -czf "$artifact" -C "$pkg_dir" fakejudge
good_sha="$(fit_sha256 "$artifact")"

good_lock="$work/good.lock"
cat >"$good_lock" <<EOF
season = "2026.09"

[judges.fakejudge]
version = "9.9.9"

[judges.fakejudge.platforms."fixture-test-platform"]
url = "file://$artifact"
sha256 = "$good_sha"
EOF

# --- plant a decoy "other garden tool" earlier on the ambient PATH -------------------
decoy_dir="$work/decoy"
mkdir -p "$decoy_dir"
cat >"$decoy_dir/other-garden-tool" <<'EOF'
#!/usr/bin/env bash
echo "I am a decoy garden tool that must not be reachable from inside the fit runner"
EOF
chmod +x "$decoy_dir/other-garden-tool"
export PATH="$decoy_dir:$PATH"

scratch_snapshot() {
  find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'plotplot-fit.*' 2>/dev/null | sort
}

# --- good run: correct checksum, must run the judge with a clean PATH ----------------
put_away "$marker_file"
before_good="$(scratch_snapshot)"
good_output="$(fit_run "$good_lock" fakejudge fixture-test-platform fakejudge -- --probe 2>&1)"
good_status=$?

[ "$good_status" -eq 0 ]; assert $? "fit_run exits 0 on a checksum that matches (got $good_status)"
[ -f "$marker_file" ]; assert $? "fit_run actually executed the fetched judge on a good checksum"
case "$good_output" in
  *"fakejudge-output: --probe"*) assert 0 "the judge received its check arguments" ;;
  *) assert 1 "the judge received its check arguments (output: $good_output)" ;;
esac
case "$good_output" in
  *"LEAK"*) assert 1 "no other garden tool is reachable on PATH inside the fit runner" ;;
  *) assert 0 "no other garden tool is reachable on PATH inside the fit runner" ;;
esac

after_good="$(scratch_snapshot)"
[ "$before_good" = "$after_good" ]; assert $? "the fit runner leaves no new scratch directory behind after a good run"

# --- bad run: wrong checksum, must refuse and never execute the judge ----------------
put_away "$marker_file"
bad_lock="$work/bad.lock"
wrong_sha="$(printf '%s' "$good_sha" | tr '0-9a-f' '1-9a-f0')" # a different, still 64-hex value
cat >"$bad_lock" <<EOF
season = "2026.09"

[judges.fakejudge]
version = "9.9.9"

[judges.fakejudge.platforms."fixture-test-platform"]
url = "file://$artifact"
sha256 = "$wrong_sha"
EOF

before_bad="$(scratch_snapshot)"
bad_output="$(fit_run "$bad_lock" fakejudge fixture-test-platform fakejudge -- --probe 2>&1)"
bad_status=$?

[ "$bad_status" -ne 0 ]; assert $? "fit_run exits non-zero when the checksum does not match (got $bad_status)"
[ ! -f "$marker_file" ]; assert $? "fit_run never executed the judge when the checksum did not match"
case "$bad_output" in
  *"checksum mismatch"*) assert 0 "the failure names the checksum mismatch" ;;
  *) assert 1 "the failure names the checksum mismatch (output: $bad_output)" ;;
esac

after_bad="$(scratch_snapshot)"
[ "$before_bad" = "$after_bad" ]; assert $? "the fit runner leaves no new scratch directory behind after a refused run"

if [ "$fail" -eq 0 ]; then
  echo "fit-runner.test.sh: all assertions passed"
  exit 0
else
  echo "fit-runner.test.sh: failures present"
  exit 1
fi
