#!/usr/bin/env python3
"""A public repository names none of the owner's private projects (contracts/private-names.json).

Usage:
  public_names.py <repo root>                  every tracked file
  public_names.py <repo root> --range A..B     also every commit message in the range
  public_names.py --hash <name>                the hash to add a name to the list
Exit 0 clean, 1 with one line per hit. A hit names the file and line, never the word.
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

LIST = Path(__file__).resolve().parent.parent / "contracts" / "private-names.json"
WORD = re.compile(r"[^a-z0-9-]+")


def fnv1a(word: str) -> str:
    h = 0xCBF29CE484222325
    for b in word.encode():
        h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def hits(text: str, hashes: set) -> list:
    """Line numbers whose words include a private name."""
    return [n for n, line in enumerate(text.splitlines(), 1)
            if any(w and fnv1a(w) in hashes for w in WORD.split(line.lower()))]


def scan(root: Path, rng: str | None, hashes: set) -> list:
    out = []
    files = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], capture_output=True, text=True, check=True).stdout.split("\0")
    for f in filter(None, files):
        p = root / f
        if f == "contracts/private-names.json" or not p.is_file():
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        out += [f"{f}:{n}: names a private project" for n in hits(text, hashes)]
    if rng:
        log = subprocess.run(["git", "-C", str(root), "log", "--format=%H%x00%B%x01", rng], capture_output=True, text=True, check=True).stdout
        for entry in filter(str.strip, log.split("\x01")):
            sha, _, body = entry.strip().partition("\0")
            if hits(body, hashes):
                out.append(f"commit {sha[:12]}: its message names a private project")
    return out


if __name__ == "__main__":
    if sys.argv[1:2] == ["--hash"]:
        print(fnv1a(sys.argv[2].lower()))
        sys.exit(0)
    root = Path(sys.argv[1])
    rng = sys.argv[3] if sys.argv[2:3] == ["--range"] else None
    found = scan(root, rng, set(json.loads(LIST.read_text())["hashes"]))
    print("\n".join(found) if found else "no private project is named")
    sys.exit(1 if found else 0)
