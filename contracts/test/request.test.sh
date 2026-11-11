#!/usr/bin/env bash
# Requests for the owner's answer, and the answers, version 2: the contract hedge keeps and checks.
# The schema compiles; every valid fixture validates and every negative fixture is rejected and
# declared below; every request, answer and keys file in the vector validates; each digest case's
# bytes follow from its text by the page's rule and hash to its digest; the same and differ pairs
# agree with their names; every key id is the sha256 of its 65-byte point; every signature case
# verifies or fails exactly as it says, and for the reason it names; the accepts, holds,
# holds_for_act and peek cases agree with the page's rules, written here a second time; and the
# vector and fixtures are what contracts/test/request-vector.mjs writes. Run from the repository
# root, no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

schema="contracts/request.schema.json"
fixdir="contracts/fixtures/request"
vector="$fixdir/vector.json"
generator="contracts/test/request-vector.mjs"

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
[ -f "$generator" ]; assert $? "the vector's generator exists at $generator"
if [ "$fail" -ne 0 ]; then echo "request.test.sh: cannot continue without the page, the schema, the vector and its generator"; exit 1; fi

if ! npx --no-install ajv compile -s "$schema" --spec=draft2020 -c ajv-formats >/dev/null 2>&1; then
  assert 1 "$schema compiles as JSON Schema draft 2020-12"
  echo "request.test.sh: cannot continue with an invalid schema"
  exit 1
fi
assert 0 "$schema compiles as JSON Schema draft 2020-12"

requests=0; answers=0; keyfiles=0
for f in "$fixdir"/valid-*.json; do
  [ -f "$f" ] || continue
  validates "$f"; assert $? "$(basename "$f") validates"
  case "$(jq -r .format "$f")" in
    gate-request/2) requests=$((requests + 1)) ;;
    gate-answer/2) answers=$((answers + 1)) ;;
    hedge-owner-keys/1) keyfiles=$((keyfiles + 1)) ;;
  esac
done
[ "$requests" -ge 2 ]; assert $? "valid fixtures hold at least two version 2 requests (found $requests)"
[ "$answers" -ge 2 ]; assert $? "valid fixtures hold at least two version 2 answers (found $answers)"
[ "$keyfiles" -ge 1 ]; assert $? "valid fixtures hold an owner's keys file (found $keyfiles)"
jq -s -e 'any(.[]; .format == "gate-request/2" and (.act | not) and (.options | length > 1)
  and .other and .no and .pick and .for and (.facts | length > 0) and (.rulings | length > 0))' \
  "$fixdir"/valid-*.json >/dev/null 2>&1
assert $? "a valid fixture is a question with options, other, no, pick, for, facts and rulings"
jq -s -e 'any(.[]; .format == "gate-request/2" and .act and (.options | map(.id)) == ["yes"])' \
  "$fixdir"/valid-*.json >/dev/null 2>&1
assert $? "a valid fixture is a request for an act with the single option yes"
jq -s -e 'any(.[]; .format == "gate-answer/2" and .by == "owner" and .signature)' \
  "$fixdir"/valid-*.json >/dev/null 2>&1
assert $? "a valid fixture is an owner's answer with a signature"
jq -s -e 'any(.[]; .format == "gate-answer/2" and .by == "expiry" and (.signature | not))' \
  "$fixdir"/valid-*.json >/dev/null 2>&1
assert $? "a valid fixture is an answer by expiry with no signature"

declare_negative() {
  local file="$1" reason="$2"
  if [ ! -f "$file" ]; then assert 1 "$reason: fixture $file exists"; return; fi
  if validates "$file"; then assert 1 "$reason: $file is rejected"; else assert 0 "$reason: $file is rejected"; fi
}

# a request
declare_negative "$fixdir/invalid-format-version-1.json" "a request in version 1, which version 2 replaces"
declare_negative "$fixdir/invalid-no-expiry.json" "a request with no expiry"
declare_negative "$fixdir/invalid-empty-ask.json" "a request that asks nothing"
declare_negative "$fixdir/invalid-asker-not-a-name.json" "an asker that is not a tool's name"
declare_negative "$fixdir/invalid-moment-not-utc.json" "a moment that is not UTC"
declare_negative "$fixdir/invalid-unknown-field.json" "a request with a field the contract does not name"
declare_negative "$fixdir/invalid-options-plain-text.json" "options written as plain text, as version 1 wrote them"
declare_negative "$fixdir/invalid-options-empty.json" "a request with no option"
declare_negative "$fixdir/invalid-option-id-no.json" "an option whose id is no, which every request may be answered"
declare_negative "$fixdir/invalid-option-id-not-lowercase.json" "an option id that is not lowercase"
declare_negative "$fixdir/invalid-option-no-does.json" "an option that does not say what the asker does"
declare_negative "$fixdir/invalid-option-unknown-field.json" "an option with a field the contract does not name"
declare_negative "$fixdir/invalid-pick-not-a-name.json" "a pick that cannot be an option's id"
declare_negative "$fixdir/invalid-fact-no-text.json" "a fact with a name and no text"
declare_negative "$fixdir/invalid-ruling-relation.json" "a ruling whose relation is neither holds nor clashes"
declare_negative "$fixdir/invalid-ruling-no-source.json" "a ruling that does not say where it comes from"
declare_negative "$fixdir/invalid-act-option-not-yes.json" "a request for an act whose one option is not yes"
declare_negative "$fixdir/invalid-act-two-options.json" "a request for an act with an option besides yes"
declare_negative "$fixdir/invalid-act-pick-not-yes.json" "a request for an act that picks anything but yes"
declare_negative "$fixdir/invalid-act-relative-cwd.json" "an act in a folder that is not absolute"
declare_negative "$fixdir/invalid-act-dot-dot-cwd.json" "an act in a folder with a .. segment"
declare_negative "$fixdir/invalid-act-trailing-slash-cwd.json" "an act in a folder with a trailing slash"
declare_negative "$fixdir/invalid-act-empty-argv.json" "an act with no words"
declare_negative "$fixdir/invalid-act-unknown-kind.json" "an act of a kind the contract does not name"
# an answer
declare_negative "$fixdir/invalid-answer-format-version-1.json" "an answer in version 1, which never holds again"
declare_negative "$fixdir/invalid-answer-request-not-sha256.json" "an answer that names no request digest"
declare_negative "$fixdir/invalid-answer-no-by.json" "an answer that does not say who gave it"
declare_negative "$fixdir/invalid-answer-unknown-field.json" "an answer with a field the contract does not name"
declare_negative "$fixdir/invalid-answer-choice-and-other.json" "an answer with both a choice and the owner's words"
declare_negative "$fixdir/invalid-answer-neither-choice-nor-other.json" "an answer with neither a choice nor the owner's words"
declare_negative "$fixdir/invalid-answer-choice-not-a-name.json" "a choice that cannot be an option's id"
declare_negative "$fixdir/invalid-answer-owner-no-signature.json" "an owner's answer without a signature"
declare_negative "$fixdir/invalid-expiry-answers-yes.json" "an answer by expiry that is not no"
declare_negative "$fixdir/invalid-expiry-in-words.json" "an answer by expiry in words"
declare_negative "$fixdir/invalid-expiry-signed.json" "an answer by expiry that carries a signature"
declare_negative "$fixdir/invalid-expiry-with-reason.json" "an answer by expiry that gives a reason"
declare_negative "$fixdir/invalid-signature-encoding.json" "a signature not in standard base64 with padding"
declare_negative "$fixdir/invalid-signature-not-der.json" "a signature that is not a DER sequence"
declare_negative "$fixdir/invalid-signature-alg.json" "a signature by an algorithm the contract does not name"
declare_negative "$fixdir/invalid-signature-key-not-sha256.json" "a signature whose key is not a key id"
declare_negative "$fixdir/invalid-signature-unknown-field.json" "a signature with a field the contract does not name"
# the owner's keys
declare_negative "$fixdir/invalid-keys-format-version.json" "a keys file in a version the contract does not name"
declare_negative "$fixdir/invalid-keys-empty.json" "a keys file with no key"
declare_negative "$fixdir/invalid-keys-point-compressed.json" "a key whose point is not the 65-byte uncompressed form"
declare_negative "$fixdir/invalid-keys-id-not-sha256.json" "a key whose id is not a sha256"
declare_negative "$fixdir/invalid-keys-no-added.json" "a key that does not say when it was added"
declare_negative "$fixdir/invalid-keys-unknown-field.json" "a key with a field the contract does not name"

# every negative fixture in the folder is declared above, so none sits there untested
for file in "$fixdir"/invalid-*.json; do
  [ -f "$file" ] || continue
  name=$(basename "$file")
  if grep -q "declare_negative \"\$fixdir/$name\"" "$0"; then
    assert 0 "$name is declared as a negative fixture"
  else
    assert 1 "$name is declared as a negative fixture"
  fi
done

# every request, answer and keys file the vector holds is one the schema accepts; a version 1
# document stays out, since it is in the vector only to show that it never holds
n=0
while IFS= read -r doc; do
  n=$((n + 1))
  printf '%s\n' "$doc" > "$scratch/doc-$n.json"
  validates "$scratch/doc-$n.json"; assert $? "vector document $n ($(printf '%s' "$doc" | jq -r .format)) validates"
done < <(jq -c '
  [ .keys,
    (.digests[].text | fromjson),
    (.same[], .differ[] | .a, .b),
    (.signatures[] | .answer + {signature: {alg: "ecdsa-p256-sha256", key: .key, value: .signature}}),
    (.accepts[] | .request, .answer),
    (.holds[], .peek[], .holds_for_act[].records[] | .request, .answer) ]
  | .[] | select(. != null) | select(.format != "gate-request/1" and .format != "gate-answer/1")' "$vector")
[ "$n" -ge 40 ]; assert $? "the vector holds requests, answers and keys to validate (found $n)"
jq -e '[.holds[], .holds_for_act[] | select(.. | objects | .format? | . == "gate-answer/1" or . == "gate-request/1") | .expect]
  | length > 0 and all(. == false)' "$vector" >/dev/null 2>&1
assert $? "the vector's version 1 records appear only in cases where no yes holds"

# the bytes of each digest case follow from its text, and hash to its digest
i=0
while IFS= read -r -d '' bytes && IFS= read -r -d '' digest; do
  i=$((i + 1))
  got="$(printf '%s' "$bytes" | shasum -a 256 | cut -d' ' -f1)"
  [ "$got" = "$digest" ]; assert $? "digest case $i: the sha256 of its bytes is its digest"
done < <(jq -j '.digests[] | .bytes, "\u0000", .digest, "\u0000", (select(.act_bytes) | .act_bytes, "\u0000", .act_digest, "\u0000")' "$vector")
[ "$i" -ge 4 ]; assert $? "the vector holds at least four digests to check (found $i)"

node --input-type=module - "$vector" "$fixdir" <<'EOF' > "$scratch/rule.txt"
// The page's rules, written here a second time so the vector cannot drift from the page. They
// share nothing with the generator: a slip in one is caught by the other.
import { createECDH, createHash, createPublicKey, verify } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
const vector = JSON.parse(readFileSync(process.argv[2], "utf8"));
const fixdir = process.argv[3];
const say = (ok, what) => console.log(`${ok ? "ok" : "not ok"} - ${what}`);

// the digest rule: keys sorted, no whitespace, strings as JSON.stringify writes them
const canon = (v) => Array.isArray(v) ? `[${v.map(canon).join(",")}]`
  : typeof v === "object" ? `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canon(v[k])}`).join(",")}}`
  : JSON.stringify(v);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const bound = (r) => Object.fromEntries(vector.bound.filter((k) => k in r).map((k) => [k, r[k]]));
const digest = (r) => sha(Buffer.from(canon(bound(r)), "utf8"));

// moments compared as instants: whole seconds, then the fraction digit by digit
const instant = (m) => {
  const x = /^(\d{4})-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)(?:\.(\d+))?Z$/.exec(m);
  if (!x) throw new Error(`not a moment: ${m}`);
  const s = Date.UTC(+x[1], +x[2] - 1, +x[3], +x[4], +x[5], +x[6]) / 1000;
  return [s, (x[7] ?? "").padEnd(40, "0")];
};
const before = (a, b) => { const [x, y] = [instant(a), instant(b)]; return x[0] < y[0] || (x[0] === y[0] && x[1] < y[1]); };

// a key: its 65-byte point, strictly in standard base64, on the curve
const strict64 = (s) => Buffer.from(s, "base64").toString("base64") === s;
const point = (b64) => {
  const p = Buffer.from(b64, "base64");
  if (!strict64(b64) || p.length !== 65 || p[0] !== 4) throw new Error("not a 65-byte uncompressed point");
  return p;
};
const publicKey = (b64) => { const p = point(b64); return createPublicKey({ key: { kty: "EC", crv: "P-256",
  x: p.subarray(1, 33).toString("base64url"), y: p.subarray(33).toString("base64url") }, format: "jwk" }); };
const ecdsa = (bytes, b64point, value) => strict64(value)
  && verify("sha256", Buffer.from(bytes, "utf8"), { key: publicKey(b64point), dsaEncoding: "der" }, Buffer.from(value, "base64"));
const n = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551n;
const sOf = (value) => { const d = Buffer.from(value, "base64"); const rl = d[3]; const sl = d[5 + rl];
  return BigInt("0x" + d.subarray(6 + rl, 6 + rl + sl).toString("hex")); };

// the signature rule: the key in the installed keys, the answer given before the key retired,
// and the signature checks over the answer's canonical bytes without its signature
const verdict = (answer, keys = vector.keys) => {
  const sig = answer.signature;
  if (!sig || sig.alg !== "ecdsa-p256-sha256") return "no signature";
  const { signature, ...rest } = answer;
  const key = keys.keys.find((k) => k.id === sig.key);
  if (!key) return "key";
  if (key.retired !== undefined && !before(answer.at, key.retired)) return "retired";
  return ecdsa(canon(rest), key.point, sig.value) ? "verifies" : "bytes";
};

// when hedge accepts a record: rules a schema cannot say
const accepts = ({ request: r, answer: a }) => {
  const ids = r.options.map((o) => o.id);
  if (new Set(ids).size !== ids.length) return false;
  if (r.pick !== undefined && !ids.includes(r.pick)) return false;
  if (!a) return true;
  if (a.request !== digest(r)) return false;
  if (a.choice !== undefined && a.choice !== "no" && !ids.includes(a.choice)) return false;
  if (a.other !== undefined && r.other === undefined) return false;
  if (a.by === "expiry") return !before(a.at, r.expires_at);
  return before(a.at, r.expires_at) && verdict(a) === "verifies";
};

// when a yes holds
const holds = ({ request: r, answer: a, used, now }) => !!a
  && r.format === "gate-request/2" && a.format === "gate-answer/2" && a.request === digest(r)
  && a.by === "owner" && a.choice === "yes" && before(a.at, r.expires_at) && before(now, r.expires_at)
  && verdict(a) === "verifies" && used === null;

// the digest cases, and the same and differ pairs
vector.digests.forEach((c, i) => {
  const r = JSON.parse(c.text);
  say(canon(bound(r)) === c.bytes, `digest case ${i + 1}: its bytes follow from its text by the page's rule`);
  if (r.act) say(canon(r.act) === c.act_bytes, `digest case ${i + 1}: its act bytes follow from its act`);
});
vector.same.forEach((p, i) => say(digest(p.a) === digest(p.b), `same pair ${i + 1}: ${p.because}`));
vector.differ.forEach((p, i) => say(digest(p.a) !== digest(p.b), `differ pair ${i + 1} (${p.field}): ${p.because}`));

// the keys: every id is the sha256 of its 65-byte point, and every test key's point follows from it
for (const k of vector.keys.keys) {
  let ok = false; try { publicKey(k.point); ok = sha(point(k.point)) === k.id; } catch { ok = false; }
  say(ok, `installed key ${k.id.slice(0, 12)}: a point on P-256, and its id is the sha256 of its 65 bytes`);
}
for (const t of vector.test_keys.keys) {
  const ecdh = createECDH("prime256v1"); ecdh.setPrivateKey(Buffer.from(t.d, "hex"));
  say(ecdh.getPublicKey().toString("base64") === t.point && sha(point(t.point)) === t.id,
    `test key ${t.name}: its point follows from its private scalar, and its id from its point`);
}
say(/test/i.test(vector.test_keys.note) && /nothing real/i.test(vector.test_keys.note),
  "the test keys are marked as test keys that sign nothing real");

// the signature cases: each verifies or fails as it says, and a failure fails for its reason alone
const halves = new Set();
vector.signatures.forEach((c, i) => {
  const what = `signature case ${i + 1}: ${c.because}`;
  const signed = { ...c.answer, signature: { alg: "ecdsa-p256-sha256", key: c.key, value: c.signature } };
  say(!("signature" in c.answer) && canon(c.answer) === c.bytes, `${what}: its bytes are the answer without its signature, by the rule`);
  say(sha(point(c.point)) === c.key, `${what}: its key id is the sha256 of its point`);
  const got = verdict(signed);
  say(got === (c.expect ? "verifies" : c.fails), `${what}: ${c.expect ? "verifies" : `fails on ${c.fails}`} (got ${got})`);
  const installed = vector.keys.keys.find((k) => k.id === c.key);
  if (c.expect) { halves.add(sOf(c.signature) > n / 2n ? "high" : "low"); say(installed?.point === c.point, `${what}: its point is the installed key's`); }
  if (c.fails === "bytes") say(installed?.point === c.point && ecdsa(c.signed_bytes, c.point, c.signature) && c.signed_bytes !== c.bytes,
    `${what}: the signature checks over the bytes it was made over, so only the bytes differ`);
  if (c.fails === "key") say(!installed && ecdsa(c.bytes, c.point, c.signature),
    `${what}: the signature checks against its own point, so only the key's absence fails it`);
  if (c.fails === "retired") say(installed?.retired !== undefined && ecdsa(c.bytes, c.point, c.signature),
    `${what}: the signature checks, so only the key's retirement fails it`);
});
say(halves.has("high") && halves.has("low"), "the signatures that verify include an s in the upper half and one in the lower");
for (const reason of ["bytes", "key", "retired"]) {
  say(vector.signatures.some((c) => c.fails === reason), `a signature case fails on ${reason}`);
}

// accepts, holds, holds for an act, and the peek sequences
vector.accepts.forEach((c, i) => say(accepts(c) === c.expect, `accepts case ${i + 1}: ${c.because}`));
vector.holds.forEach((c, i) => say(holds(c) === c.expect, `holds case ${i + 1}: ${c.because}`));
vector.holds_for_act.forEach((c, i) => {
  const got = c.records.some((rec) => rec.request.act && canon(rec.request.act) === canon(c.act) && holds({ ...rec, now: c.now }));
  say(got === c.expect, `holds_for_act case ${i + 1}: ${c.because}`);
});
vector.peek.forEach((c, i) => {
  let used = c.used;
  const ok = c.steps.every((s) => {
    const h = holds({ ...c, used });
    const records = s.run === "hedge holds" && h;
    if (records) used = c.now;
    return s.holds === h && s.records_use === records && (s.run === "hedge holds" || !s.records_use);
  });
  say(ok, `peek case ${i + 1}: ${c.because}`);
});

// the valid fixtures agree with the vector: answers name a fixture request and are accepted for it
const fixtures = readdirSync(fixdir).filter((f) => f.startsWith("valid-")).map((f) => [f, JSON.parse(readFileSync(`${fixdir}/${f}`, "utf8"))]);
const fixtureRequests = fixtures.filter(([, d]) => d.format === "gate-request/2").map(([, d]) => d);
for (const [f, d] of fixtures.filter(([, d]) => d.format === "gate-answer/2")) {
  const r = fixtureRequests.find((q) => digest(q) === d.request);
  say(!!r && accepts({ request: r, answer: d }), `${f}: names a fixture request, and hedge accepts it for that request`);
}
for (const [f, d] of fixtures.filter(([, d]) => d.format === "hedge-owner-keys/1")) {
  say(canon(d) === canon(vector.keys), `${f}: is the vector's keys file`);
}
EOF
rule_status=$?
while IFS= read -r line; do
  case "$line" in ok\ -*) echo "$line" ;; *) echo "$line"; fail=1 ;; esac
done < "$scratch/rule.txt"
[ "$rule_status" -eq 0 ]; assert $? "the rule's checks ran to the end (node exited $rule_status)"
grep -q '^ok - same pair' "$scratch/rule.txt"; assert $? "the rule's checks ran"

for field in asker ask why evidence options other no pick for facts rulings act; do
  jq -e --arg f "$field" '[.differ[] | select(.field == $f)] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "a differ pair changes the bound field $field"
done
jq -e '.bound == ["format","asker","ask","why","evidence","options","other","no","pick","for","facts","rulings","act"]' "$vector" >/dev/null 2>&1
assert $? "the vector's bound fields are the page's"
for field in asked_at expires_at waiting; do
  jq -e --arg f "$field" '[.same[] | select((.a[$f] // null) != (.b[$f] // null))] | length > 0' "$vector" >/dev/null 2>&1
  assert $? "a same pair changes the unbound field $field"
done
for list in accepts holds holds_for_act; do
  jq -e --arg l "$list" '[.[$l][].expect] | (index(true) != null) and (index(false) != null)' "$vector" >/dev/null 2>&1
  assert $? "the vector's $list cases hold at least one that passes and one that does not"
done
jq -e '[.holds[] | select(.used != null)] | length > 0' "$vector" >/dev/null 2>&1
assert $? "a holds case is a yes already spent"
jq -e '[.holds[] | select(.answer != null and .answer.at < .request.expires_at and .now >= .request.expires_at)] | length > 0' "$vector" >/dev/null 2>&1
assert $? "a holds case is a yes given in time and checked after the expiry"
jq -e '[.peek[].steps[] | select(.run == "hedge holds --peek") | .records_use] | length > 0 and all(. == false)' "$vector" >/dev/null 2>&1
assert $? "the peek cases run hedge holds --peek, and it never records a use"

node "$generator" --check > "$scratch/generator.txt" 2>&1
status=$?
[ "$status" -eq 0 ] || sed 's/^/  /' "$scratch/generator.txt"
assert $status "the vector and the fixtures are what $generator writes"

if [ "$fail" -eq 0 ]; then echo "request.test.sh: all assertions passed"; exit 0; fi
echo "request.test.sh: failures present"; exit 1
