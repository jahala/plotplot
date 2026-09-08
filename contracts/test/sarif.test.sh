#!/usr/bin/env bash
# Validates fixture SARIF logs against the official SARIF 2.1.0 JSON schema, vendored at
# contracts/vendor/sarif-schema-2.1.0.json with its sha256 checked against contracts/pins.json
# on every run. One fixture exists today: weeder's `check --format sarif`. petals and
# tend2 do not emit SARIF yet (docs/tend2/contracts.tend2.html Tried, 2026-09-08), so their
# fixtures are recorded as absent rather than invented. Run from the repository root, no
# arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/vendor/sarif-schema-2.1.0.json"
pins="contracts/pins.json"
validator="contracts/test/lib/validate-sarif.mjs"
weeder_fixture="contracts/fixtures/sarif/weeder-check.sarif.json"

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

[ -f "$schema" ]; assert $? "the vendored SARIF 2.1.0 schema exists at $schema"
[ -f "$pins" ]; assert $? "$pins exists"
[ -f "$validator" ]; assert $? "the SARIF validator exists at $validator"
[ -f "$weeder_fixture" ]; assert $? "weeder's SARIF fixture exists at $weeder_fixture"

if [ "$fail" -ne 0 ]; then
  echo "sarif.test.sh: cannot continue without the schema, the pins, the validator and the fixture"
  exit 1
fi

pinned_sha="$(node -e 'console.log(require("./contracts/pins.json").sarif_schema.sha256)' 2>/dev/null)"
actual_sha="$(shasum -a 256 "$schema" | awk '{print $1}')"
[ -n "$pinned_sha" ]; assert $? "contracts/pins.json names a sarif_schema.sha256"
[ "$pinned_sha" = "$actual_sha" ]; assert $? "the vendored schema's sha256 ($actual_sha) matches the pin ($pinned_sha)"

if node "$validator" "$weeder_fixture" >/tmp/sarif-test-weeder-out.$$ 2>&1; then
  assert 0 "weeder's check --format sarif fixture validates against the official SARIF 2.1.0 schema"
else
  assert 1 "weeder's check --format sarif fixture validates against the official SARIF 2.1.0 schema"
  sed 's/^/    /' /tmp/sarif-test-weeder-out.$$
fi
trash /tmp/sarif-test-weeder-out.$$ >/dev/null 2>&1 || true

# The validator itself must be able to fail: a log missing "runs" is not SARIF.
tmp_broken="$(mktemp "${TMPDIR:-/tmp}/plotplot-sarif-broken.XXXXXX.json")"
echo '{"$schema":"https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json","version":"2.1.0"}' >"$tmp_broken"
if node "$validator" "$tmp_broken" >/dev/null 2>&1; then
  assert 1 "the validator rejects a log with no runs array (it passed instead)"
else
  assert 0 "the validator rejects a log with no runs array"
fi
trash "$tmp_broken" >/dev/null 2>&1 || true

# The check names three beds with a check face. A bed with no SARIF fixture is a failure of
# the claim, never a skip: a green run over two of three beds would stamp an empty room.
for bed in petals tend2; do
  fixture="contracts/fixtures/sarif/${bed}-check.sarif.json"
  if [ ! -f "$fixture" ]; then
    assert 1 "$bed emits no SARIF yet, so $fixture does not exist and the claim is not proven for $bed"
  elif node "$validator" "$fixture" >/dev/null 2>&1; then
    assert 0 "$bed fixture validates against SARIF 2.1.0"
  else
    assert 1 "$bed fixture does not validate against SARIF 2.1.0"
  fi
done

if [ "$fail" -eq 0 ]; then
  echo "sarif.test.sh: all assertions passed"
  exit 0
else
  echo "sarif.test.sh: failures present"
  exit 1
fi
