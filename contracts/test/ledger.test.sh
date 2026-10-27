#!/usr/bin/env bash
# The spending ledger, well's contract. The rule lives in two runtimes
# (cultivate's ledger.py and the owner's judge runner), so its home is the test vector each
# implementation must pass; those runs happen in each implementation. Here: the schema compiles,
# the vector parses, and the vector exercises every event and the cap. Run from the repository
# root, no arguments. One line per assertion; exit 0 if all passed, 1 otherwise.
set -u
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1
schema="contracts/ledger.schema.json"
vector="contracts/fixtures/ledger/vector.json"
fail=0
assert() { if [ "$1" -eq 0 ]; then echo "ok - $2"; else echo "not ok - $2"; fail=1; fi; }

[ -f contracts/ledger.md ]; assert $? "the page exists"
npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
assert $? "$schema compiles as JSON Schema draft 2020-12"
jq -e '.steps | length > 0' "$vector" >/dev/null 2>&1; assert $? "the vector parses and has steps"
for ev in reserve settle release; do
  jq -e --arg ev "$ev" '[.steps[] | select(.event == $ev)] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "the vector exercises a $ev"
done
jq -e '[.steps[] | select(.expect | tostring | test("refus|ceiling|cap|block"; "i"))] | length > 0' "$vector" >/dev/null 2>&1
assert $? "the vector holds at least one refusal"
jq -e '[.steps[].tool] | unique | length >= 2' "$vector" >/dev/null 2>&1
assert $? "the vector has two spenders sharing one budget"
jq -e '[.across_months.cases[] | select(.expect == "refuse:unsettled")] | length > 0' "$vector" >/dev/null 2>&1
assert $? "the vector holds an old reservation that still blocks its request"
for reason in cap-day cap-week; do
  jq -e --arg r "refuse:$reason" '[.pacing.cases[] | select(.expect == $r)] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "the vector's pacing refuses a reserve with $reason"
done
lock="contracts/fixtures/ledger/lock.json"
jq -e '.cases | length > 0 and all(.[]; has("lock_age_s") and has("breaker_age_s") and has("expect") and has("after"))' "$lock" >/dev/null 2>&1
assert $? "the lock cases parse and each names its setup, outcome and what remains"
jq -e '[.cases[] | select(.breaker_age_s == 0 and .expect == "not written")] | length > 0' "$lock" >/dev/null 2>&1
assert $? "the lock cases hold a stale lock left alone while another waiter holds the breaker"

if [ "$fail" -eq 0 ]; then echo "ledger.test.sh: all assertions passed"; exit 0; fi
echo "ledger.test.sh: failures present"; exit 1
