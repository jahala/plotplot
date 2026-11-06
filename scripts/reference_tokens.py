#!/usr/bin/env python3
"""The reference pages carry their own copy of the brand tokens. This check holds that copy to
.brand/tokens.css: every --pp-* a reference defines has the token's value in the same theme, and
every var(--pp-*) a reference uses is defined in its light :root, so no dot or rule renders blank.
tokens.css carries dark overrides for colours only, so a reference's dark block is held to it for
colours; its dark shadows have no token yet (components.md, "Shadows").

Usage: reference_tokens.py <brand dir>   (exit 0 clean, 1 with one line per fault)
"""
import re
import sys
from pathlib import Path

DEF = re.compile(r"(--pp-[a-z0-9-]+)\s*:\s*([^;]+);")
USE = re.compile(r"var\(\s*(--pp-[a-z0-9-]+)")


def block(css: str, selector: str) -> str:
    """The body of the first rule whose selector is exactly `selector`, braces balanced."""
    m = re.search(r"(^|[\s}])" + re.escape(selector) + r"\s*\{", css)
    if not m:
        return ""
    i, depth = m.end(), 1
    start = i
    while i < len(css) and depth:
        depth += {"{": 1, "}": -1}.get(css[i], 0)
        i += 1
    return css[start : i - 1]


def defs(body: str) -> dict:
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    return {k: re.sub(r"\s*,\s*", ",", " ".join(v.split())).lower() for k, v in DEF.findall(body)}


def themes(css: str) -> tuple:
    return defs(block(css, ":root")), defs(block(css, '[data-theme="dark"]'))


def check(brand: Path) -> list:
    t_light, t_dark = themes((brand / "tokens.css").read_text())
    faults = []
    for page in sorted((brand / "reference").glob("*.html")):
        text = page.read_text()
        light, dark = themes(text)
        name = page.relative_to(brand.parent)
        colours = {k: v for k, v in dark.items() if v.startswith("#")}
        for theme, mine, theirs in (("light", light, t_light), ("dark", colours, {**t_light, **t_dark})):
            for var, value in mine.items():
                if var not in theirs:
                    faults.append(f"{name}: {theme} defines {var}, which tokens.css does not")
                elif value != theirs[var]:
                    faults.append(f"{name}: {theme} {var} is {value}, tokens.css says {theirs[var]}")
        for var in sorted(set(USE.findall(text)) - set(light)):
            faults.append(f"{name}: uses var({var}) but its :root never defines it")
        for var in sorted((set(USE.findall(text)) & set(t_dark)) - set(dark)):
            faults.append(f"{name}: uses var({var}), which tokens.css changes in dark, but its dark block never sets it")
    return faults


if __name__ == "__main__":
    found = check(Path(sys.argv[1]))
    print("\n".join(found) if found else "references match tokens.css")
    sys.exit(1 if found else 0)
