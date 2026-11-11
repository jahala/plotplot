#!/usr/bin/env node
// Writes the vector of contracts/request.md, version 2, and every fixture beside it, into
// contracts/fixtures/request/. Run from anywhere:
//   node contracts/test/request-vector.mjs           write the files
//   node contracts/test/request-vector.mjs --check   write nothing; exit 1 if a file differs
// The test keys below are made from fixed text, so every run makes the same keys. ECDSA
// signatures are random, so a run keeps each signature already in the files when it still checks
// over the same bytes with the same key, and signs afresh only what changed. Two runs therefore
// write the same files. contracts/test/request.test.sh checks the files with rules of its own.
import { createECDH, createHash, createPrivateKey, createPublicKey, sign, verify } from "node:crypto";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const dir = join(dirname(fileURLToPath(import.meta.url)), "..", "fixtures", "request");
const checking = process.argv.includes("--check");

// The digest rule, as the page writes it: keys sorted by bytes, no whitespace, and each string
// with only the escapes the page names.
const str = (s) => `"${[...s].map((ch) => {
  const c = ch.codePointAt(0);
  if (ch === '"') return '\\"';
  if (ch === "\\") return "\\\\";
  const short = { 8: "\\b", 9: "\\t", 10: "\\n", 12: "\\f", 13: "\\r" }[c];
  if (short) return short;
  return c < 0x20 ? `\\u00${c.toString(16).padStart(2, "0")}` : ch;
}).join("")}"`;
const canon = (v) => Array.isArray(v) ? `[${v.map(canon).join(",")}]`
  : typeof v === "object" ? `{${Object.keys(v).sort().map((k) => `${str(k)}:${canon(v[k])}`).join(",")}}`
  : str(v);
const sha = (b) => createHash("sha256").update(b).digest("hex");
const BOUND = ["format", "asker", "ask", "why", "evidence", "options", "other", "no", "pick", "for", "facts", "rulings", "act"];
const boundOf = (r) => Object.fromEntries(BOUND.filter((k) => k in r).map((k) => [k, r[k]]));
const digest = (r) => sha(Buffer.from(canon(boundOf(r)), "utf8"));

// Ways an asker might write the same request: keys reversed at every level, indented, or on one
// line with spaces after each comma and colon, optionally with everything past ASCII escaped.
const reversed = (v) => Array.isArray(v) ? v.map(reversed)
  : typeof v === "object" && v !== null ? Object.fromEntries(Object.entries(v).reverse().map(([k, x]) => [k, reversed(x)]))
  : v;
const spaced = (v, ascii) => Array.isArray(v) ? `[${v.map((x) => spaced(x, ascii)).join(", ")}]`
  : typeof v === "object" ? `{${Object.entries(v).map(([k, x]) => `${spaced(k, ascii)}: ${spaced(x, ascii)}`).join(", ")}}`
  : ascii ? JSON.stringify(v).replace(/[\u007f-\uffff]/g, (c) => `\\u${c.charCodeAt(0).toString(16).padStart(4, "0")}`)
  : JSON.stringify(v);
const without = (o, ...keys) => Object.fromEntries(Object.entries(o).filter(([k]) => !keys.includes(k)));

// The test keys. Each private scalar is the sha256 of fixed text, so it is the same on every run.
const N = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551n;
const testKey = (name) => {
  const d = createHash("sha256").update(`plotplot request contract, test key ${name}, signs nothing real`).digest();
  const s = BigInt(`0x${d.toString("hex")}`);
  if (s === 0n || s >= N) throw new Error(`test key ${name}: the scalar is not below the group order`);
  const ecdh = createECDH("prime256v1");
  ecdh.setPrivateKey(d);
  const p = ecdh.getPublicKey();
  const jwk = { kty: "EC", crv: "P-256", x: p.subarray(1, 33).toString("base64url"), y: p.subarray(33).toString("base64url") };
  return {
    name, d: d.toString("hex"), point: p.toString("base64"), id: sha(p),
    priv: createPrivateKey({ key: { ...jwk, d: d.toString("base64url") }, format: "jwk" }),
    pub: createPublicKey({ key: jwk, format: "jwk" }),
  };
};
const A = testKey("A"); // installed, current
const B = testKey("B"); // installed, retired
const C = testKey("C"); // never installed
const KEYS = [A, B, C];

// Signatures kept from the files already written, by key and bytes.
const kept = new Map();
const remember = (keyId, bytes, value) => {
  const key = KEYS.find((k) => k.id === keyId);
  if (key && typeof value === "string" && checks(key, bytes, value)) {
    kept.set(`${keyId}\n${bytes}`, [...(kept.get(`${keyId}\n${bytes}`) ?? []), value]);
  }
};
const checks = (key, bytes, value) => {
  try { return verify("sha256", Buffer.from(bytes, "utf8"), { key: key.pub, dsaEncoding: "der" }, Buffer.from(value, "base64")); }
  catch { return false; }
};
const walk = (v) => {
  if (Array.isArray(v)) v.forEach((x) => walk(x));
  else if (v && typeof v === "object") {
    if (v.signature && typeof v.signature === "object") remember(v.signature.key, canon(without(v, "signature")), v.signature.value);
    if (typeof v.signature === "string" && v.key) remember(v.key, v.signed_bytes ?? v.bytes, v.signature);
    Object.values(v).forEach((x) => walk(x));
  }
};
if (existsSync(dir)) {
  for (const f of readdirSync(dir).filter((f) => f.endsWith(".json"))) {
    try { walk(JSON.parse(readFileSync(join(dir, f), "utf8"))); } catch { /* a file that does not parse keeps nothing */ }
  }
}
const highS = (value) => {
  const der = Buffer.from(value, "base64");
  const rl = der[3];
  const sl = der[5 + rl];
  return BigInt(`0x${der.subarray(6 + rl, 6 + rl + sl).toString("hex")}`) > N / 2n;
};
// half: "high" or "low" asks for a signature whose s lies in that half of the group order, since
// the Secure Enclave does not normalize s and a verifier must accept both.
const signBytes = (key, bytes, half) => {
  const fits = (v) => half === undefined || highS(v) === (half === "high");
  const old = (kept.get(`${key.id}\n${bytes}`) ?? []).find(fits);
  if (old) return old;
  for (;;) {
    const v = sign("sha256", Buffer.from(bytes, "utf8"), { key: key.priv, dsaEncoding: "der" }).toString("base64");
    if (fits(v)) { kept.set(`${key.id}\n${bytes}`, [v]); return v; }
  }
};
const ALG = "ecdsa-p256-sha256";
const signed = (answer, key, half) => ({ ...answer, signature: { alg: ALG, key: key.id, value: signBytes(key, canon(answer), half) } });

// Moments.
const ASKED = "2026-11-12T09:00:00Z";
const EXPIRES = "2026-11-19T09:00:00Z";
const B_ADDED = "2026-11-12T08:00:00Z";
const B_RETIRED = "2026-11-14T08:00:00Z";
const A_ADDED = "2026-11-14T08:00:00Z";
const AT = "2026-11-14T10:12:00Z";
const NOW = "2026-11-14T11:00:00Z";
const USED = "2026-11-14T10:30:00Z";

const keysFile = {
  format: "hedge-owner-keys/1",
  keys: [
    { id: B.id, point: B.point, added: B_ADDED, retired: B_RETIRED },
    { id: A.id, point: A.point, added: A_ADDED },
  ],
};

// The requests.
const site = { kind: "command", argv: ["npm", "run", "deploy"], cwd: "/Users/owner/repos/plotplot-ai" };
const actRequest = {
  format: "gate-request/2",
  asker: "transplant",
  ask: "Deploy plotplot.ai to its domain for the first time",
  why: "The first deploy to a domain is public in the owner's name.",
  evidence: ["jahala/plotplot-ai pull request 42, merged with every check green", "the preview at https://preview.plotplot.ai"],
  options: [{ id: "yes", label: "Deploy it now", does: "transplant runs npm run deploy once, in the site's folder." }],
  no: "Nothing is deployed. The preview stays up.",
  act: site,
  asked_at: ASKED,
  expires_at: EXPIRES,
  waiting: ["the launch post", "the domain's email"],
};
const ioRequest = {
  ...actRequest,
  ask: "Deploy plotplot.io to its domain for the first time",
  act: { ...site, cwd: "/Users/owner/repos/plotplot-io" },
};
const actInWords = { ...actRequest, other: "Say what you want instead, and transplant changes the deploy to match." };
const question = {
  format: "gate-request/2",
  asker: "pollen",
  ask: "Where should pollen post the launch notice?",
  why: "The notice goes out in the owner's name, and each place costs something different.",
  evidence: ["the draft notice", "last month's reach, by channel"],
  options: [
    { id: "newsletter", label: "The newsletter only", does: "pollen sends the notice to the newsletter's readers and nowhere else." },
    { id: "forum", label: "One forum thread", does: "pollen opens one thread in the forum the draft names and answers replies for a week." },
    { id: "paid", label: "A paid placement", does: "pollen books one placement and spends no more than the most it can cost." },
  ],
  other: "Say what you want instead, and pollen revises the plan from it.",
  no: "Nothing is posted. pollen keeps the draft.",
  pick: "newsletter",
  for: "plotplot-ai",
  facts: [
    { name: "Most it can cost", text: "1,000 kr, set aside before the first click" },
    { name: "Who sees it", text: "people outside the garden, in the owner's name" },
  ],
  rulings: [
    { text: "Public posts go out in the owner's name only after the owner's yes.", source: "compass, 2026-11-04", relation: "holds" },
    { text: "Spend nothing on ads before the product has users.", source: "compass, 2026-11-04", relation: "clashes" },
  ],
  asked_at: ASKED,
  expires_at: EXPIRES,
  waiting: ["the launch"],
};
const yesQuestion = {
  format: "gate-request/2",
  asker: "pollen",
  ask: "May pollen reply to comments on the launch notice?",
  why: "Replies go out in the owner's name.",
  evidence: [],
  options: [{ id: "yes", label: "Yes, reply", does: "pollen replies to each comment within a day." }],
  asked_at: ASKED,
  expires_at: EXPIRES,
};
const escapes = {
  format: "gate-request/2",
  asker: "gate",
  ask: "Say \"yes\" to C:\\path/a\tb\nc\rd\be\ff\u0001g\u001fh\u007fi \u00e9 \u2028 \u{1f331}?",
  why: "Urgent requests wait until the owner opens a terminal; a phone channel would reach them sooner.",
  evidence: [],
  options: [{ id: "yes", label: "Yes \u00e9 \u{1f331}", does: "Nothing; this request is the escaping rule's test." }],
  asked_at: ASKED,
  expires_at: EXPIRES,
};
// Version 1 records, as hedge wrote them before version 2: they stay as records and never hold.
const v1Request = {
  format: "gate-request/1", asker: "transplant", ask: actRequest.ask, why: actRequest.why, evidence: actRequest.evidence,
  options: ["yes"], act: site, asked_at: "2026-10-27T09:00:00Z", expires_at: "2026-11-20T09:00:00Z", waiting: actRequest.waiting,
};
const v1Answer = {
  format: "gate-answer/1", request: "8abba80c73b96be18ee0cb306bdaf32c664147359b3e2a2b988b813f18445870",
  answer: "yes", by: "owner", at: "2026-10-27T10:12:00Z",
};

// The answers.
const answer = (request, fields, at = AT) => ({ format: "gate-answer/2", request: digest(request), by: "owner", at, ...fields });
const yesBare = answer(actRequest, { choice: "yes" });
const noBare = answer(actRequest, { choice: "no" });
const yesA = signed(yesBare, A, "high");
const noA = signed(noBare, A);
const choiceA = signed(answer(question, { choice: "newsletter", reason: "Start small. The forum can wait until the launch is out \u{1f331}" }), A);
const wordsBare = answer(question, { other: "Post it in the forum, but only after the newsletter has gone out." });
const wordsA = signed(wordsBare, A, "low");
const expiryNo = { format: "gate-answer/2", request: digest(question), by: "expiry", at: EXPIRES, choice: "no" };
const yesQuestionA = signed(answer(yesQuestion, { choice: "yes" }), A);
const yesB = signed(answer(actRequest, { choice: "yes" }, "2026-11-13T10:12:00Z"), B);
const yesBLate = signed(answer(actRequest, { choice: "yes" }, "2026-11-15T10:12:00Z"), B);
const yesC = signed(yesBare, C);
const forged = { ...yesBare, signature: { ...yesQuestionA.signature } };

// The signature cases.
const sigCase = (because, signedAnswer, expect, extra = {}) => {
  const bare = without(signedAnswer, "signature");
  const key = KEYS.find((k) => k.id === signedAnswer.signature.key);
  return { because, answer: bare, bytes: canon(bare), key: key.id, point: key.point, signature: signedAnswer.signature.value, expect, ...extra };
};
const signatures = [
  sigCase("the owner's yes to a request for an act, signed by the current key; its s is in the upper half, which a verifier accepts", yesA, true),
  sigCase("the owner's choice of an option, with a reason past ASCII, which the bytes hold as UTF-8", choiceA, true),
  sigCase("the owner's own words, with no choice; its s is in the lower half", wordsA, true),
  sigCase("a retired key still verifies an answer it signed before it retired", yesB, true),
  sigCase("the signature was made over the owner's no, and the answer says yes", { ...yesBare, signature: noA.signature }, false,
    { fails: "bytes", signed_bytes: canon(noBare) }),
  sigCase("a key that is not in the installed keys, though its signature checks against its own point", yesC, false, { fails: "key" }),
  sigCase("an answer dated after its key retired", yesBLate, false, { fails: "retired" }),
];

// The accepts cases: rules a schema cannot say, which hedge applies before it records a request
// or an answer.
const accepts = [
  { because: "a question with three options, the owner's words allowed, and a pick among them", request: question, expect: true },
  { because: "two options with one id", request: { ...question, options: [...question.options, { id: "forum", label: "Two forum threads", does: "pollen opens two threads." }] }, expect: false },
  { because: "a pick that names no option", request: { ...question, pick: "email" }, expect: false },
  { because: "the owner chose an option", request: question, answer: choiceA, expect: true },
  { because: "the owner said no, which every request may be answered", request: question, answer: signed(answer(question, { choice: "no" }), A), expect: true },
  { because: "a choice that is not one of the request's options", request: question, answer: signed(answer(question, { choice: "email" }), A), expect: false },
  { because: "the owner's own words, on a request that allows them", request: question, answer: wordsA, expect: true },
  { because: "the owner's own words, on a request that does not allow them", request: actRequest, answer: signed(answer(actRequest, { other: wordsBare.other }), A), expect: false },
  { because: "the owner's own words on a request for an act are recorded, though they never make the act hold", request: actInWords, answer: signed(answer(actInWords, { other: "Deploy it after the launch post instead." }), A), expect: true },
  { because: "the no hedge records at the expiry", request: question, answer: expiryNo, expect: true },
  { because: "an answer by expiry dated before the expiry is no answer", request: question, answer: { ...expiryNo, at: "2026-11-18T09:00:00Z" }, expect: false },
  { because: "an owner's answer at the moment of expiry is too late to record", request: question, answer: signed(answer(question, { choice: "newsletter" }, EXPIRES), A), expect: false },
  { because: "an owner's answer whose signature does not verify is never written", request: question, answer: { ...choiceA, signature: { ...choiceA.signature, value: wordsA.signature.value } }, expect: false },
  { because: "an answer that names another request's digest", request: question, answer: { ...choiceA, request: digest(actRequest) }, expect: false },
];

// The holds cases: whether the owner's yes holds for a request, at the moment now, given the use
// hedge has recorded of it.
const hold = (because, request, ans, expect, extra = {}) => ({ because, request, answer: ans, used: null, now: NOW, expect, ...extra });
const holds = [
  hold("the owner said yes before the expiry, signed, and it is unused", actRequest, yesA, true),
  hold("no answer is not a yes, even before the expiry", actRequest, null, false),
  hold("the owner said no", actRequest, noA, false),
  hold("hedge answered no at the expiry", question, expiryNo, false, { now: EXPIRES }),
  hold("a yes recorded for one request does not hold for a request that differs in its folder", ioRequest, yesA, false),
  hold("a yes given at the moment of expiry is too late", actRequest, signed(answer(actRequest, { choice: "yes" }, EXPIRES), A), false, { now: EXPIRES }),
  hold("moments are compared as instants, not as text: half a second after the expiry is after it", actRequest,
    signed(answer(actRequest, { choice: "yes" }, "2026-11-19T09:00:00.500Z"), A), false, { now: "2026-11-19T09:00:01Z" }),
  hold("moments are compared as instants: a fraction of a second before the expiry is before it", actRequest,
    signed(answer(actRequest, { choice: "yes" }, "2026-11-19T08:59:59.999Z"), A), true, { now: "2026-11-19T08:59:59.9995Z" }),
  hold("an answer that is one of the options but not yes does not hold", question, choiceA, false),
  hold("the owner's own words never hold, even on a request that allows them", actInWords,
    signed(answer(actInWords, { other: "Deploy it after the launch post instead." }), A), false),
  hold("a spent yes: hedge has recorded its use", actRequest, yesA, false, { used: USED }),
  hold("a yes given in time lapses with the request: checked at the expiry, it no longer holds", actRequest, yesA, false, { now: EXPIRES }),
  hold("a forged yes: the signature of the owner's yes to another request, copied", actRequest, forged, false),
  hold("a yes signed by a key that is not installed", actRequest, yesC, false),
  hold("a yes signed by a retired key after it retired", actRequest, yesBLate, false),
  hold("a yes signed by a retired key before it retired still holds", actRequest, yesB, true),
  hold("a question whose one option is yes holds for the request", yesQuestion, yesQuestionA, true),
  hold("a version 1 yes never holds again, though the owner gave it in time", v1Request, v1Answer, false, { now: "2026-10-28T09:00:00Z" }),
];

// The holds_for_act cases: whether some recorded yes holds for the act seen.
const record = (request, ans, used = null) => ({ request, answer: ans, used });
const holdsForAct = [
  { because: "the act seen is the act the owner said yes to, its keys in another order", act: reversed(site), now: NOW, records: [record(actRequest, yesA)], expect: true },
  { because: "one more word in the command seen", act: { ...site, argv: [...site.argv, "--force"] }, now: NOW, records: [record(actRequest, yesA)], expect: false },
  { because: "a yes for one folder does not hold in another, even beside a request for that folder that has no answer", act: ioRequest.act, now: NOW,
    records: [record(actRequest, yesA), record(ioRequest, null)], expect: false },
  { because: "the act was refused", act: site, now: NOW, records: [record(actRequest, noA)], expect: false },
  { because: "a question with no act holds for no act", act: site, now: NOW, records: [record(yesQuestion, yesQuestionA)], expect: false },
  { because: "the yes was spent by its first use", act: site, now: NOW, records: [record(actRequest, yesA, USED)], expect: false },
  { because: "a version 1 yes for the same act never holds again", act: site, now: "2026-10-28T09:00:00Z", records: [record(v1Request, v1Answer)], expect: false },
];

// The peek cases: hedge holds --peek, then hedge holds, in order, on one yes.
const peek = [
  { because: "--peek twice on an unused yes holds both times and records no use; hedge holds then passes once and spends it; --peek after that says spent",
    request: actRequest, answer: yesA, used: null, now: NOW, steps: [
      { run: "hedge holds --peek", holds: true, records_use: false },
      { run: "hedge holds --peek", holds: true, records_use: false },
      { run: "hedge holds", holds: true, records_use: true },
      { run: "hedge holds --peek", holds: false, records_use: false },
      { run: "hedge holds", holds: false, records_use: false },
    ] },
  { because: "with no answer, neither command holds and neither records a use",
    request: actRequest, answer: null, used: null, now: NOW, steps: [
      { run: "hedge holds --peek", holds: false, records_use: false },
      { run: "hedge holds", holds: false, records_use: false },
    ] },
  { because: "a yes that lapsed with its request does not hold for --peek either",
    request: actRequest, answer: yesA, used: null, now: EXPIRES, steps: [
      { run: "hedge holds --peek", holds: false, records_use: false },
    ] },
];

// The digest cases, and the same and differ pairs.
const digestCase = (because, text) => {
  const r = JSON.parse(text);
  const c = { because, text, bytes: canon(boundOf(r)), digest: digest(r) };
  if (r.act) { c.act_bytes = canon(r.act); c.act_digest = sha(Buffer.from(c.act_bytes, "utf8")); }
  return c;
};
const digests = [
  digestCase("a request for an act, its keys reversed at every level, spaced and indented; the unbound fields stay out of the bytes",
    JSON.stringify(reversed(actRequest), null, 4)),
  digestCase("a question with every optional field, written on one line", spaced(question, false)),
  digestCase("every character the escaping rule names: a quote, a backslash, a slash, the five short escapes, other control characters as lowercase \\u00xx, and DEL, an accent, U+2028 and a seedling written as themselves; the text writes everything past ASCII as an escape, and the bytes hold it as itself",
    spaced(escapes, true)),
];
const opt = (r, i, f) => ({ ...r, options: r.options.map((o, j) => (j === i ? f(o) : o)) });
const same = [
  { because: "the expiry is not bound: asking again with a later deadline is the same request", a: actRequest, b: { ...actRequest, expires_at: "2026-11-26T09:00:00Z" } },
  { because: "the moment of asking is not bound", a: actRequest, b: { ...actRequest, asked_at: "2026-11-12T10:30:00Z" } },
  { because: "what waits is not bound: it ranks the request, it does not change what is asked", a: actRequest, b: without(actRequest, "waiting") },
  { because: "key order, at every level, is not part of the request", a: question, b: reversed(question) },
];
const differ = [
  { field: "asker", because: "another tool asking the same thing is another request", a: actRequest, b: { ...actRequest, asker: "pollen" } },
  { field: "ask", because: "one character of what is asked", a: actRequest, b: { ...actRequest, ask: `${actRequest.ask}.` } },
  { field: "ask", because: "no Unicode normalization: a composed and a decomposed accent are different text",
    a: { ...question, ask: "Post the launch notice at the caf\u00e9?" }, b: { ...question, ask: "Post the launch notice at the cafe\u0301?" } },
  { field: "why", because: "the reason the owner read is part of what they said yes to", a: actRequest, b: { ...actRequest, why: "The first deploy is public." } },
  { field: "evidence", because: "evidence added after the answer is a new request", a: actRequest, b: { ...actRequest, evidence: [...actRequest.evidence, "the launch post's draft"] } },
  { field: "evidence", because: "the order of the evidence is kept", a: actRequest, b: { ...actRequest, evidence: [...actRequest.evidence].reverse() } },
  { field: "options", because: "a question with another option", a: question,
    b: { ...question, options: [...question.options, { id: "both", label: "The newsletter and the forum", does: "pollen does both, the newsletter first." }] } },
  { field: "options", because: "the order of the options is kept", a: question, b: { ...question, options: [...question.options].reverse() } },
  { field: "options", because: "an option's label is what the owner read", a: question, b: opt(question, 0, (o) => ({ ...o, label: "Only the newsletter" })) },
  { field: "options", because: "what an option does is what the owner chose", a: actRequest, b: opt(actRequest, 0, (o) => ({ ...o, does: "transplant runs npm run deploy." })) },
  { field: "options", because: "an option's id is what the answer names", a: question, b: { ...opt(question, 0, (o) => ({ ...o, id: "letter" })), pick: "letter" } },
  { field: "other", because: "a request that lets the owner answer in their own words and the same request that does not", a: actRequest, b: actInWords },
  { field: "other", because: "what the asker does with the owner's words", a: question, b: { ...question, other: "Say what you want instead." } },
  { field: "no", because: "what the asker does after a no", a: actRequest, b: { ...actRequest, no: "Nothing is deployed." } },
  { field: "no", because: "a request that says what follows a no and the same request that does not", a: actRequest, b: without(actRequest, "no") },
  { field: "pick", because: "the option the asker recommends", a: question, b: { ...question, pick: "forum" } },
  { field: "pick", because: "a recommendation and none", a: question, b: without(question, "pick") },
  { field: "for", because: "the project the ask serves", a: question, b: { ...question, for: "plotplot" } },
  { field: "facts", because: "one fact's text", a: question, b: { ...question, facts: [{ ...question.facts[0], text: "2,000 kr" }, question.facts[1]] } },
  { field: "facts", because: "the order of the facts is kept", a: question, b: { ...question, facts: [...question.facts].reverse() } },
  { field: "rulings", because: "a ruling that holds and the same ruling said to clash", a: question,
    b: { ...question, rulings: [{ ...question.rulings[0], relation: "clashes" }, question.rulings[1]] } },
  { field: "rulings", because: "where a ruling comes from", a: question,
    b: { ...question, rulings: [{ ...question.rulings[0], source: "compass, 2026-11-05" }, question.rulings[1]] } },
  { field: "act", because: "one word of the command", a: actRequest, b: { ...actRequest, act: { ...site, argv: ["npm", "run", "deploy:prod"] } } },
  { field: "act", because: "the same text split into other words is another command", a: actRequest, b: { ...actRequest, act: { ...site, argv: ["npm", "run deploy"] } } },
  { field: "act", because: "the same command in another folder", a: actRequest, b: { ...actRequest, act: { ...site, cwd: "/Users/owner/repos/plotplot-io" } } },
  { field: "act", because: "a request with an act and the same request without one", a: actRequest, b: without(actRequest, "act") },
];

const vector = {
  note: "The rules of contracts/request.md, version 2: the exact bytes a digest is computed over, the requests whose digests must match or differ, the exact bytes an answer's signature is made over and the signatures that verify or fail, the records hedge accepts, and when a yes holds. Every implementation reproduces each bytes, digest and verdict from its inputs, and each expect. Written by contracts/test/request-vector.mjs.",
  bound: BOUND,
  test_keys: {
    note: "TEST KEYS, published here so anyone can check the signatures below. They sign nothing real, and no keys file outside a test may list them. Each d is the private scalar in hex, the sha256 of the text \"plotplot request contract, test key <name>, signs nothing real\". A and B are installed in the keys below, B retired; C is in no keys file.",
    keys: KEYS.map(({ name, d, point, id }) => ({ name, d, point, id })),
  },
  keys: keysFile,
  digests,
  same,
  differ,
  signatures,
  accepts,
  holds,
  holds_for_act: holdsForAct,
  peek,
};

// The fixtures. Each negative fixture is a valid one with one thing wrong.
const signatureOf = yesA.signature;
const rawSignature = (() => {
  const der = Buffer.from(signatureOf.value, "base64");
  const rl = der[3];
  const r = der.subarray(4, 4 + rl);
  const s = der.subarray(6 + rl, 6 + rl + der[5 + rl]);
  const fixed = (x) => Buffer.concat([Buffer.alloc(32), x.subarray(x.length - Math.min(32, x.length))]).subarray(-32);
  const raw = Buffer.concat([fixed(r), fixed(s)]).toString("base64");
  if (raw.startsWith("M")) throw new Error("the raw signature happens to start as DER does; pick another");
  return raw;
})();
const compressed = (() => {
  const p = Buffer.from(A.point, "base64");
  return Buffer.concat([Buffer.from([2 + (p[64] & 1)]), p.subarray(1, 33)]).toString("base64");
})();
const keyOne = (f) => ({ ...keysFile, keys: [f(keysFile.keys[0]), keysFile.keys[1]] });
const sigWith = (f) => ({ ...yesA, signature: f(yesA.signature) });
const fact = (f) => ({ ...question, facts: [f(question.facts[0]), question.facts[1]] });
const ruling = (f) => ({ ...question, rulings: [f(question.rulings[0]), question.rulings[1]] });
const actWith = (f) => ({ ...actRequest, act: f(site) });

const fixtures = {
  "valid-request-question.json": question,
  "valid-request-act.json": actRequest,
  "valid-answer-owner.json": yesA,
  "valid-answer-owner-words.json": wordsA,
  "valid-answer-expiry.json": expiryNo,
  "valid-owner-keys.json": keysFile,

  "invalid-format-version-1.json": { ...actRequest, format: "gate-request/1" },
  "invalid-no-expiry.json": without(actRequest, "expires_at"),
  "invalid-empty-ask.json": { ...actRequest, ask: "" },
  "invalid-asker-not-a-name.json": { ...actRequest, asker: "Transplant Bot" },
  "invalid-moment-not-utc.json": { ...actRequest, expires_at: "2026-11-19T10:00:00+01:00" },
  "invalid-unknown-field.json": { ...actRequest, priority: "high" },
  "invalid-options-plain-text.json": { ...question, options: question.options.map((o) => o.id) },
  "invalid-options-empty.json": { ...question, options: [] },
  "invalid-option-id-no.json": opt(question, 2, (o) => ({ ...o, id: "no" })),
  "invalid-option-id-not-lowercase.json": opt(question, 1, (o) => ({ ...o, id: "Forum" })),
  "invalid-option-no-does.json": opt(question, 1, (o) => without(o, "does")),
  "invalid-option-unknown-field.json": opt(question, 2, (o) => ({ ...o, cost: "1,000 kr" })),
  "invalid-pick-not-a-name.json": { ...question, pick: "The newsletter only" },
  "invalid-fact-no-text.json": fact((f) => without(f, "text")),
  "invalid-ruling-relation.json": ruling((r) => ({ ...r, relation: "supports" })),
  "invalid-ruling-no-source.json": ruling((r) => without(r, "source")),
  "invalid-act-option-not-yes.json": opt(actRequest, 0, (o) => ({ ...o, id: "deploy" })),
  "invalid-act-two-options.json": { ...actRequest, options: [...actRequest.options, { id: "later", label: "After the launch post", does: "transplant waits for the launch post, then deploys." }] },
  "invalid-act-pick-not-yes.json": { ...actRequest, pick: "later" },
  "invalid-act-relative-cwd.json": actWith((a) => ({ ...a, cwd: "repos/plotplot-ai" })),
  "invalid-act-dot-dot-cwd.json": actWith((a) => ({ ...a, cwd: "/Users/owner/repos/../repos/plotplot-ai" })),
  "invalid-act-trailing-slash-cwd.json": actWith((a) => ({ ...a, cwd: "/Users/owner/repos/plotplot-ai/" })),
  "invalid-act-empty-argv.json": actWith((a) => ({ ...a, argv: [] })),
  "invalid-act-unknown-kind.json": actWith((a) => ({ ...a, kind: "spend" })),

  "invalid-answer-format-version-1.json": { ...yesA, format: "gate-answer/1" },
  "invalid-answer-request-not-sha256.json": { ...yesA, request: "deploy-42" },
  "invalid-answer-no-by.json": without(yesA, "by"),
  "invalid-answer-unknown-field.json": { ...yesA, via: "the question window" },
  "invalid-answer-choice-and-other.json": { ...wordsA, choice: "forum" },
  "invalid-answer-neither-choice-nor-other.json": without(yesA, "choice"),
  "invalid-answer-choice-not-a-name.json": { ...yesA, choice: "Yes" },
  "invalid-answer-owner-no-signature.json": yesBare,
  "invalid-expiry-answers-yes.json": { ...expiryNo, choice: "yes" },
  "invalid-expiry-in-words.json": { ...without(expiryNo, "choice"), other: "No answer came." },
  "invalid-expiry-signed.json": { ...expiryNo, signature: signatureOf },
  "invalid-expiry-with-reason.json": { ...expiryNo, reason: "The owner did not answer in time." },
  "invalid-signature-encoding.json": sigWith((s) => ({ ...s, value: Buffer.from(s.value, "base64").toString("base64url") })),
  "invalid-signature-not-der.json": sigWith((s) => ({ ...s, value: rawSignature })),
  "invalid-signature-alg.json": sigWith((s) => ({ ...s, alg: "ecdsa-p384-sha384" })),
  "invalid-signature-key-not-sha256.json": sigWith((s) => ({ ...s, key: "owner" })),
  "invalid-signature-unknown-field.json": sigWith((s) => ({ ...s, made: AT })),

  "invalid-keys-format-version.json": { ...keysFile, format: "hedge-owner-keys/2" },
  "invalid-keys-empty.json": { ...keysFile, keys: [] },
  "invalid-keys-point-compressed.json": keyOne((k) => ({ ...k, point: compressed })),
  "invalid-keys-id-not-sha256.json": keyOne((k) => ({ ...k, id: "B" })),
  "invalid-keys-no-added.json": keyOne((k) => without(k, "added")),
  "invalid-keys-unknown-field.json": keyOne((k) => ({ ...k, label: "the owner's laptop" })),
};

const files = { "vector.json": `${JSON.stringify(vector, null, 1)}\n` };
for (const [name, doc] of Object.entries(fixtures)) files[name] = `${JSON.stringify(doc, null, 2)}\n`;

let drift = 0;
const stale = readdirSync(dir).filter((f) => f.endsWith(".json") && !(f in files));
for (const f of stale) { console.log(`stale: ${join(dir, f)} is not one this generator writes; move it to the Trash`); drift = 1; }
for (const [name, text] of Object.entries(files)) {
  const path = join(dir, name);
  const now = existsSync(path) ? readFileSync(path, "utf8") : null;
  if (now === text) continue;
  if (checking) { console.log(`differs: ${path}`); drift = 1; } else { writeFileSync(path, text); console.log(`wrote ${path}`); }
}
process.exit(drift);
