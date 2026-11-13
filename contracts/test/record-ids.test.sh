#!/usr/bin/env bash
# Recomputes the id of every line in each record contract's valid fixtures from the rule its
# contract writes (contracts/<name>.md, the `id` row), and fails on any line whose id does not
# match. A fixture with a made-up id passes a schema check, since an id is only a 64-hex string
# there, so this is the check that holds the rule. Run from the repository root, no arguments.
set -u
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1
python3 - <<'PY'
import hashlib, json, sys
from pathlib import Path

def h(parts):
    return hashlib.sha256("\n".join(parts).encode()).hexdigest()

def as_json(v):
    # as JSON.stringify writes it: a number without a trailing .0, a string with its quotes
    if isinstance(v, float) and v.is_integer():
        return str(int(v))
    return json.dumps(v, ensure_ascii=False)

rules = {
    # contracts/document.md: sha256 of document, source, url (or locator) and content, joined by newlines
    "document": lambda d: h(["document", d["source"], d.get("url") or d.get("locator"), d["content"]]),
    # contracts/raw-item.md: sha256 of source, url (or locator) and text, joined by newlines
    "raw-item": lambda d: h([d["source"], d.get("url") or d.get("locator"), d["text"]]),
    # contracts/change.md: change, source, target, field, op, value ("" when null), said_at
    "change": lambda d: h(["change", d["source"], d["target"], d["field"], d["op"], "" if d.get("value") is None else (d["value"] if isinstance(d["value"], str) else json.dumps(d["value"])), d["said_at"]]),
    # contracts/observation.md: observation, source, subject, metric, time key (at, or from/to), and value as JSON writes it
    "observation": lambda d: h(["observation", d["source"], d["subject"], d["metric"], d["at"] if "at" in d else f'{d["period"]["from"]}/{d["period"]["to"]}', as_json(d["value"])]),
    # contracts/reaction.md: reaction, source, target, kind, author
    "reaction": lambda d: h(["reaction", d["source"], d["target"], d["kind"], d["author"]]),
    # contracts/subject.md: subject, a newline, and the first identifier as scheme:value
    "subject": lambda d: h(["subject", f'{d["identifiers"][0]["scheme"]}:{d["identifiers"][0]["value"]}']),
}

fail = 0
for name, rule in rules.items():
    path = Path(f"contracts/fixtures/{name}/valid.jsonl")
    if not path.exists():
        print(f"not ok - {path} exists"); fail = 1; continue
    for n, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        d = json.loads(line)
        try:
            want = rule(d)
        except (KeyError, IndexError, TypeError) as e:
            print(f"not ok - {name} line {n}: the id rule cannot be computed ({e!r})"); fail = 1; continue
        if d.get("id") == want:
            print(f"ok - {name} line {n}: id follows contracts/{name}.md")
        else:
            print(f"not ok - {name} line {n}: id {d.get('id')} is not the rule's {want}"); fail = 1
sys.exit(fail)
PY
