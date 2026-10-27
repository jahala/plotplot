#!/usr/bin/env bash
# Requests for the owner's answer, and the answers: gate's contract, which hedge checks. The schema
# compiles; every valid fixture validates and every negative fixture is rejected and declared below;
# every request and answer in the vector validates; each digest case's bytes follow from its text
# by the page's rule and hash to its digest; the same and differ pairs agree with their names; and
# the vector covers every bound field. When a yes holds is computed by each implementation against
# the vector in its own tests. Run from the repository root, no arguments. Prints one line per
# assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/request.schema.json"
fixdir="contracts/fixtures/request"
vector="$fixdir/vector.json"

fail=0
scratch="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-request.XXXXXX")"
put_away() { if command -v trash >/dev/null 2>&1; then trash "$@" 2>/dev/null || true; fi; }
trap 'put_away "$scratch"' EXIT
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}
validates() {
  npx --no-install ajv validate -s "$schema" -d "$1" --spec=draft2020 -c ajv-formats >/dev/null 2>&1
}

[ -f contracts/request.md ]; assert $? "the page exists at contracts/request.md"
[ -f "$schema" ]; assert $? "the schema exists at $schema"
[ -f "$vector" ]; assert $? "the vector exists at $vector"
if [ "$fail" -ne 0 ]; then echo "request.test.sh: cannot continue without the page, the schema and the vector"; exit 1; fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema compiles as JSON Schema draft 2020-12"
  echo "request.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema compiles as JSON Schema draft 2020-12"

requests=0; answers=0
for f in "$fixdir"/valid-*.json; do
  [ -f "$f" ] || continue
  validates "$f"; assert $? "$(basename "$f") validates"
  case "$(jq -r .format "$f")" in
    gate-request/1) requests=$((requests + 1)) ;;
    gate-answer/1) answers=$((answers + 1)) ;;
  esac
done
[ "$requests" -ge 2 ]; assert $? "valid fixtures hold a request with an act and one without (found $requests requests)"
[ "$answers" -ge 2 ]; assert $? "valid fixtures hold an owner's answer and an answer by expiry (found $answers answers)"

declare_negative() {
  local file="$1" reason="$2"
  if [ ! -f "$file" ]; then assert 1 "$reason: fixture $file exists"; return; fi
  if validates "$file"; then assert 1 "$reason: $file is rejected"; else assert 0 "$reason: $file is rejected"; fi
}

declare_negative "$fixdir/invalid-format-version.json" "a format version this contract does not name"
declare_negative "$fixdir/invalid-no-expiry.json" "a request with no expiry"
declare_negative "$fixdir/invalid-options-name-no.json" "options that name no, which every request may be answered"
declare_negative "$fixdir/invalid-act-options-not-yes.json" "a request with an act whose options are not yes alone"
declare_negative "$fixdir/invalid-act-relative-cwd.json" "an act in a folder that is not absolute"
declare_negative "$fixdir/invalid-act-dot-dot-cwd.json" "an act in a folder with a .. segment"
declare_negative "$fixdir/invalid-act-trailing-slash-cwd.json" "an act in a folder with a trailing slash"
declare_negative "$fixdir/invalid-act-empty-argv.json" "an act with no words"
declare_negative "$fixdir/invalid-act-unknown-kind.json" "an act of a kind version 1 does not name"
declare_negative "$fixdir/invalid-empty-ask.json" "a request that asks nothing"
declare_negative "$fixdir/invalid-unknown-field.json" "a field the contract does not name"
declare_negative "$fixdir/invalid-asker-not-a-name.json" "an asker that is not a tool's name"
declare_negative "$fixdir/invalid-moment-not-utc.json" "a moment that is not UTC"
declare_negative "$fixdir/invalid-answer-request-not-sha256.json" "an answer that names no request digest"
declare_negative "$fixdir/invalid-expiry-answers-yes.json" "an answer by expiry that is not no"
declare_negative "$fixdir/invalid-answer-no-by.json" "an answer that does not say who gave it"

# every negative fixture in the folder is declared above, so none sits there untested
for file in "$fixdir"/invalid-*.json; do
  name=$(basename "$file")
  if grep -q "declare_negative \"\$fixdir/$name\"" "$0"; then
    assert 0 "$name is declared as a negative fixture"
  else
    assert 1 "$name is declared as a negative fixture"
  fi
done

# every request and answer the vector holds is one the schema accepts
n=0
while IFS= read -r doc; do
  n=$((n + 1))
  printf '%s\n' "$doc" > "$scratch/doc-$n.json"
  validates "$scratch/doc-$n.json"; assert $? "vector document $n ($(printf '%s' "$doc" | jq -r .format)) validates"
done < <(jq -c '
  [ (.digests[].text | fromjson),
    (.same[], .differ[] | .a, .b),
    (.holds[] | .request, .answer),
    (.holds_for_act[].records[] | .request, .answer) ] | .[] | select(. != null)' "$vector")
[ "$n" -ge 10 ]; assert $? "the vector holds requests and answers to validate (found $n)"

# the bytes of each digest case follow from its text, and hash to its digest
i=0
while IFS= read -r -d '' bytes && IFS= read -r -d '' digest; do
  i=$((i + 1))
  got="$(printf '%s' "$bytes" | shasum -a 256 | cut -d' ' -f1)"
  [ "$got" = "$digest" ]; assert $? "digest case $i: the sha256 of its bytes is its digest"
done < <(jq -j '.digests[] | .bytes, "\u0000", .digest, "\u0000", (select(.act_bytes) | .act_bytes, "\u0000", .act_digest, "\u0000")' "$vector")
[ "$i" -ge 3 ]; assert $? "the vector holds at least three digests to check (found $i)"

node --input-type=module - "$vector" <<'EOF' > "$scratch/rule.txt"
// The page's rule, written here a second time so the vector cannot drift from the page:
// the bound fields, keys sorted, no whitespace, strings as JSON.stringify writes them.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
const vector = JSON.parse(readFileSync(process.argv[2], "utf8"));
const canon = (v) => Array.isArray(v) ? `[${v.map(canon).join(",")}]`
  : typeof v === "object" ? `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canon(v[k])}`).join(",")}}`
  : JSON.stringify(v);
const bound = (r) => Object.fromEntries(vector.bound.filter((k) => k in r).map((k) => [k, r[k]]));
const digest = (r) => createHash("sha256").update(canon(bound(r)), "utf8").digest("hex");
const say = (ok, what) => console.log(`${ok ? "ok" : "not ok"} - ${what}`);
vector.digests.forEach((c, i) => {
  const r = JSON.parse(c.text);
  say(canon(bound(r)) === c.bytes, `digest case ${i + 1}: its bytes follow from its text by the page's rule`);
  if (r.act) say(canon(r.act) === c.act_bytes, `digest case ${i + 1}: its act bytes follow from its act`);
});
vector.same.forEach((p, i) => say(digest(p.a) === digest(p.b), `same pair ${i + 1}: ${p.because}`));
vector.differ.forEach((p, i) => say(digest(p.a) !== digest(p.b), `differ pair ${i + 1} (${p.field}): ${p.because}`));
EOF
while IFS= read -r line; do
  case "$line" in ok\ -*) echo "$line" ;; *) echo "$line"; fail=1 ;; esac
done < "$scratch/rule.txt"
grep -q '^ok - same pair' "$scratch/rule.txt"; assert $? "the rule's checks ran"

for field in asker ask why evidence options act; do
  jq -e --arg f "$field" '[.differ[] | select(.field == $f)] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "a differ pair changes the bound field $field"
done
jq -e '.bound == ["format","asker","ask","why","evidence","options","act"]' "$vector" >/dev/null 2>&1
assert $? "the vector's bound fields are the page's"
for field in asked_at expires_at waiting; do
  jq -e --arg f "$field" '[.same[] | select((.a[$f] // null) != (.b[$f] // null))] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "a same pair changes the unbound field $field"
done
for list in holds holds_for_act; do
  jq -e --arg l "$list" '[.[$l][].expect] | (index(true) != null) and (index(false) != null)' "$vector" >/dev/null 2>&1
  assert $? "the vector's $list cases hold at least one yes that holds and one that does not"
done

if [ "$fail" -eq 0 ]; then echo "request.test.sh: all assertions passed"; exit 0; fi
echo "request.test.sh: failures present"; exit 1
