#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# export-tokens.sh: derive the brand's machine views from its markdown
# ============================================================================
# Usage: export-tokens.sh [<brand-dir>] [--check]
#
# Builds ONE token list from .brand/*.md and writes it twice:
#   <brand-dir>/tokens.css   CSS custom properties
#   <brand-dir>/tokens.json  W3C DTCG groups of { $value, $type }
#
# Where each token comes from:
#   colour    the --pp-*: #hex declarations in colors.md's own CSS blocks
#             (:root and [data-theme="dark"]); each hex must also sit in a
#             colors.md table row
#   font      --font-<role> in typography.md's CSS block, as --pp-font-<role>
#   space, radius, stroke, shadow, motion
#             the token and value columns of their tables, found by header
#   layout    container max width, gutter and breakpoints, only as layout.md
#             states them
# A value that is prose rather than a CSS value is skipped, never guessed.
#
# --check derives into a scratch folder, compares it with the committed files
# token by token, writes nothing into the brand folder, prints one DIFF line
# per difference and exits 1 if there is any.
#
# Exit codes: 0 derived (or, with --check, the tokens match); 1 with --check,
# the committed tokens differ from the markdown; 2 the markdown cannot be
# derived (a FATAL line says why) or the command line is wrong.
# ============================================================================

BRAND_DIR=".brand"
CHECK=0
SEEN_DIR=0
for arg in "$@"; do
  case "$arg" in
    --check) CHECK=1 ;;
    -*) echo "Unknown option: $arg" >&2; echo "Usage: export-tokens.sh [<brand-dir>] [--check]" >&2; exit 2 ;;
    *)
      [[ "$SEEN_DIR" -eq 0 ]] || { echo "Usage: export-tokens.sh [<brand-dir>] [--check]" >&2; exit 2; }
      BRAND_DIR="$arg"; SEEN_DIR=1 ;;
  esac
done

for f in colors.md typography.md layout.md components.md identity.md; do
  [[ -f "$BRAND_DIR/$f" ]] || { echo "Missing: $BRAND_DIR/$f" >&2; exit 2; }
done

# Work files live in a folder this run creates and removes on any exit.
SCRATCH=$(mktemp -d)
trap 'rm -rf "$SCRATCH"' EXIT

PY=$(cat <<'PYEOF'
import json
import os
import re
import sys
from collections import namedtuple

EM_DASH = "—"
PREFIX = "--pp-"


class Fail(Exception):
    pass


# One token in the model. group is the JSON group, key the name inside it.
Token = namedtuple("Token", "section css value type group key")


def read(brand_dir, name):
    with open(os.path.join(brand_dir, name), encoding="utf-8") as f:
        return f.read()


# ── reading markdown ───────────────────────────────────────────────────────

def strip_comments(css):
    return re.sub(r"/\*.*?\*/", "", css, flags=re.S)


def fenced_blocks(md):
    return re.findall(r"```[^\n]*\n(.*?)```", md, flags=re.S)


def split_row(line):
    cells = line.strip().strip("|").split("|")
    return [c.strip().replace("`", "").strip() for c in cells]


def is_separator(cells):
    return all(re.fullmatch(r":?-+:?", c) for c in cells if c) and any(cells)


def tables_under(md, heading_prefix):
    """Every table under a heading that starts with heading_prefix, as
    (header cells, data rows). Code fences are skipped."""
    out = []
    active = False
    in_fence = False
    block = []

    def flush():
        if len(block) >= 2:
            header = split_row(block[0])
            rest = [split_row(x) for x in block[1:]]
            rows = [r for r in rest if not is_separator(r)]
            out.append((header, rows))
        del block[:]

    for line in md.split("\n"):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            flush()
            continue
        if in_fence:
            continue
        m = re.match(r"^#{1,6}\s+(.*?)\s*$", line)
        if m:
            flush()
            active = m.group(1).lower().startswith(heading_prefix.lower())
            continue
        if active and line.lstrip().startswith("|"):
            block.append(line)
        else:
            flush()
    flush()
    return out


def token_value_rows(md, heading_prefix):
    """(token, value) pairs found by the Token and Value header names."""
    pairs = []
    for header, rows in tables_under(md, heading_prefix):
        names = [h.lower() for h in header]
        tcol = next((i for i, h in enumerate(names) if h in ("token", "name")), None)
        vcol = names.index("value") if "value" in names else None
        if tcol is None or vcol is None:
            continue
        for r in rows:
            if len(r) <= max(tcol, vcol):
                continue
            tok, val = r[tcol], r[vcol]
            if re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9-]*", tok) and val:
                pairs.append((tok, val))
    return pairs


# ── value rules: a value is a CSS value or it is skipped ───────────────────

LENGTH = r"-?(?:\d+\.?\d*|\.\d+)(?:px|rem|em|vw|vh|ch|%)?"


def as_length(val):
    return val if re.fullmatch(LENGTH, val) else None


def as_radius(val):
    # "999px / 50%" lists alternatives; the first one is the token's value.
    first = val.split(" / ")[0].strip()
    return as_length(first)


def as_stroke(val):
    m = re.fullmatch(r"(" + LENGTH + r")(?:\s+(?:solid|dashed|dotted|double|none))?", val)
    return m.group(1) if m else None


def as_shadow(val):
    rest = re.sub(r"rgba?\(\s*[\d.\s,%/]+\)", "", val)
    rest = re.sub(r"#[0-9A-Fa-f]{3,8}\b", "", rest)
    rest = re.sub(r"\binset\b", "", rest)
    rest = re.sub(r"(?<=\d)(?:px|rem|em)\b", "", rest)
    if re.fullmatch(r"[\d.,\s-]+", rest) and re.search(r"\d", rest):
        return " ".join(val.split())
    return None


EASE = re.compile(r"cubic-bezier\(\s*-?[\d.]+\s*(?:,\s*-?[\d.]+\s*){3}\)|ease|ease-in|ease-out|ease-in-out|linear")
DURATION = re.compile(r"(?:\d+\.?\d*|\.\d+)(?:ms|s)")


def as_motion(val):
    """(value, DTCG type) or None."""
    if EASE.fullmatch(val):
        return val, "cubicBezier"
    if DURATION.fullmatch(val):
        return val, "duration"
    return None


# ── deriving the token list ────────────────────────────────────────────────

def brand_name(identity_md):
    for line in identity_md.split("\n"):
        if line.lstrip().startswith("|") and "Brand Name" in line:
            cells = split_row(line)
            if len(cells) >= 2 and cells[1]:
                return cells[1]
    return "brand"


def css_blocks(md, selector):
    """Bodies of the CSS rules with this selector, inside code fences."""
    bodies = []
    for block in fenced_blocks(md):
        text = strip_comments(block)
        for m in re.finditer(selector + r"\s*\{(.*?)\}", text, flags=re.S):
            bodies.append(m.group(1))
    return bodies


def colour_decls(bodies, where):
    out = []
    seen = {}
    for body in bodies:
        for name, hexv in re.findall(r"--pp-([A-Za-z0-9_-]+)\s*:\s*(#[0-9A-Fa-f]{6})\s*;", body):
            if name in seen and seen[name].lower() != hexv.lower():
                raise Fail("%s declares %s twice with different values (%s, %s)" % (where, PREFIX + name, seen[name], hexv))
            if name not in seen:
                out.append((name, hexv))
            seen[name] = hexv
    return out


def derive_tokens(brand_dir):
    colors_md = read(brand_dir, "colors.md")
    typog_md = read(brand_dir, "typography.md")
    layout_md = read(brand_dir, "layout.md")
    comps_md = read(brand_dir, "components.md")
    identity_md = read(brand_dir, "identity.md")

    root = []
    dark = []

    # 1. colour: the brand's own CSS blocks name the tokens
    light = colour_decls(css_blocks(colors_md, r":root"), "colors.md :root")
    night = colour_decls(css_blocks(colors_md, r"\[data-theme\s*=\s*\"dark\"\]"), 'colors.md [data-theme="dark"]')
    if not light:
        raise Fail("colors.md has no --pp-*: #hex declarations in a CSS :root block")
    table_hexes = set()
    for line in colors_md.split("\n"):
        if line.lstrip().startswith("|"):
            table_hexes.update(h.lower() for h in re.findall(r"#[0-9A-Fa-f]{6}", line))
    for block, decls in (("", light), ("[data-theme=\"dark\"] ", night)):
        for name, hexv in decls:
            if hexv.lower() not in table_hexes:
                raise Fail("%s in colors.md's %sCSS block (%s) appears in no colors.md table row" % (hexv, block, PREFIX + name))
    for name, hexv in light:
        root.append(Token("color", PREFIX + name, hexv, "color", "color", name))
    for name, hexv in night:
        dark.append(Token("color", PREFIX + name, hexv, "color", "color", name))

    # 2. fonts
    fonts = []
    for block in fenced_blocks(typog_md):
        for role, stack in re.findall(r"^\s*--font-([A-Za-z0-9_-]+)\s*:\s*([^;]+);", strip_comments(block), flags=re.M):
            fonts.append(Token("type", PREFIX + "font-" + role, " ".join(stack.split()), "fontFamily", "fontFamily", role))
    root.extend(fonts)

    # 3. space, radius, stroke, shadow, motion
    for tok, val in token_value_rows(layout_md, "Spacing Scale"):
        v = as_length(val)
        if v:
            tid = tok if tok.startswith("space-") else "space-" + tok
            root.append(Token("space", PREFIX + tid, v, "dimension", "space", tid[len("space-"):]))

    # 4. layout, only as layout.md states it
    for header, rows in tables_under(layout_md, "Containers"):
        names = [h.lower() for h in header]
        if "token" not in names or "value" not in names:
            continue
        tcol, vcol = names.index("token"), names.index("value")
        for r in rows:
            if len(r) <= max(tcol, vcol):
                continue
            tok, val = r[tcol], r[vcol]
            if tok == "container":
                m = re.fullmatch(r"(" + LENGTH + r")\s+max-width\b.*", val)
                if m:
                    root.append(Token("layout", PREFIX + "container-max", m.group(1), "dimension", "layout", "container-max"))
            elif tok == "gutter":
                m = re.fullmatch(r"(" + LENGTH + r")(?:\s*\(.*\))?", val)
                if m:
                    root.append(Token("layout", PREFIX + "gutter", m.group(1), "dimension", "layout", "gutter"))
    for header, rows in tables_under(layout_md, "Breakpoints"):
        # a Token column names each breakpoint; without one, the width does
        hdr = [h.lower() for h in header]
        tcol = hdr.index("token") if "token" in hdr else None
        for r in rows:
            m = re.fullmatch(r"(\d+\.?\d*)(px|em|rem)", r[0]) if r else None
            if m:
                named = r[tcol].strip() if tcol is not None and len(r) > tcol else ""
                if named:
                    bid = named if named.startswith("breakpoint-") else "breakpoint-" + named
                else:
                    bid = "breakpoint-" + (m.group(1) if m.group(2) == "px" else m.group(1) + m.group(2))
                root.append(Token("layout", PREFIX + bid, r[0], "dimension", "layout", bid))

    for heading, group, conv, dtype in (
        ("Radius Scale", "radius", as_radius, "dimension"),
        ("Border Strokes", "stroke", as_stroke, "dimension"),
        ("Shadow Recipes", "shadow", as_shadow, "shadow"),
    ):
        for tok, val in token_value_rows(comps_md, heading):
            v = conv(val)
            if v:
                key = tok[len(group) + 1:] if tok.startswith(group + "-") else tok
                root.append(Token(group, PREFIX + tok, v, dtype, group, key))
    for tok, val in token_value_rows(comps_md, "Motion Tokens"):
        m = as_motion(val)
        if m:
            root.append(Token("motion", PREFIX + tok, m[0], m[1], "motion", tok))

    names = [t.css for t in root]
    dupes = sorted({n for n in names if names.count(n) > 1})
    if dupes:
        raise Fail("these tokens come out twice: " + ", ".join(dupes))
    return brand_name(identity_md), root, dark


# ── two serialisations of the one list ─────────────────────────────────────

def render_css(brand, root, dark):
    lines = [
        "/* %s design tokens, generated from .brand/*.md (canonical)." % brand,
        "   Regenerate via: bash petals/scripts/export-tokens.sh",
        "   Import this file and reference the variables; never hard-code brand values. */",
        "",
        ":root {",
    ]
    section = None
    for t in root:
        if t.section != section:
            if section is not None:
                lines.append("")
            lines.append("  /* %s */" % t.section)
            section = t.section
        lines.append("  %s: %s;" % (t.css, t.value))
    lines.append("}")
    if dark:
        lines += ["", "[data-theme=\"dark\"] {"]
        lines += ["  %s: %s;" % (t.css, t.value) for t in dark]
        lines.append("}")
    return "\n".join(lines) + "\n"


GROUP_ORDER = ["color", "fontFamily", "space", "layout", "radius", "stroke", "shadow", "motion"]


def render_json(brand, root, dark):
    doc = {
        "$description": (
            "Machine view of the resolved %s brand, generated from .brand/*.md (canonical) "
            "by bash petals/scripts/export-tokens.sh. Shape follows the W3C design-tokens "
            "draft: groups of { $value, $type }. The dark group holds the dark-theme overrides." % brand
        )
    }
    for g in GROUP_ORDER:
        members = {t.key: {"$value": t.value, "$type": t.type} for t in root if t.group == g}
        if members:
            doc[g] = members
    if dark:
        doc["dark"] = {"color": {t.key: {"$value": t.value, "$type": t.type} for t in dark}}
    return json.dumps(doc, indent=2, ensure_ascii=False) + "\n"


# ── comparing committed files with the derivation ──────────────────────────

def num(x):
    return format(float(x), "g")


def canon(value, typ):
    """A form of a value that ignores spacing, quoting, hex case and units
    that mean the same thing (0.8s and 800ms)."""
    if isinstance(value, list):
        if typ == "cubicBezier":
            return "cubic-bezier(" + ",".join(num(x) for x in value) + ")"
        return ",".join(str(x).strip().strip("'\"") for x in value)
    s = " ".join(str(value).split())
    if typ == "fontFamily":
        return ",".join(p.strip().strip("'\"") for p in s.split(","))
    m = re.fullmatch(r"(\d*\.?\d+)(ms|s)", s)
    if m:
        return num(float(m.group(1)) * (1 if m.group(2) == "ms" else 1000)) + "ms"
    m = re.fullmatch(r"cubic-bezier\(([^)]*)\)", s)
    if m:
        return "cubic-bezier(" + ",".join(num(x) for x in m.group(1).split(",")) + ")"
    s = re.sub(r"\s*([,()/])\s*", r"\1", s)
    return s.lower()


def shown(value):
    if isinstance(value, list):
        return json.dumps(value, ensure_ascii=False)
    return " ".join(str(value).split())


def parse_css(path):
    out = {}
    if not os.path.exists(path):
        return out
    with open(path, encoding="utf-8") as f:
        text = strip_comments(f.read())
    for selector, body in re.findall(r"([^{}]+)\{([^}]*)\}", text):
        sel = " ".join(selector.split())
        if sel == ":root":
            prefix = ""
        elif re.fullmatch(r"\[data-theme\s*=\s*\"dark\"\]", sel):
            prefix = "[dark]"
        else:
            prefix = "[" + sel + "]"
        for name, val in re.findall(r"(--[A-Za-z0-9_-]+)\s*:\s*([^;]+);", body):
            out[prefix + name] = (shown(val), canon(val, None))
    return out


def parse_json(path):
    out = {}
    if not os.path.exists(path):
        return out
    with open(path, encoding="utf-8") as f:
        try:
            doc = json.load(f)
        except ValueError as e:
            raise Fail("%s is not valid JSON: %s" % (path, e))

    def walk(node, trail):
        if not isinstance(node, dict):
            return
        if "$value" in node:
            out[".".join(trail)] = (shown(node["$value"]), canon(node["$value"], node.get("$type")))
            return
        for k, v in node.items():
            if not k.startswith("$"):
                walk(v, trail + [k])

    walk(doc, [])
    return out


def diff_maps(kind, committed, derived):
    lines = []
    for name in list(committed) + [n for n in derived if n not in committed]:
        c, d = committed.get(name), derived.get(name)
        if c is not None and d is not None and c[1] == d[1]:
            continue
        lines.append("DIFF %s %s committed: %s derived: %s" % (kind, name, c[0] if c else "(absent)", d[0] if d else "(absent)"))
    return lines


def main(argv):
    mode = argv[0]
    if mode == "derive":
        brand_dir, out_dir = argv[1], argv[2]
        brand, root, dark = derive_tokens(brand_dir)
        css, js = render_css(brand, root, dark), render_json(brand, root, dark)
        if EM_DASH in css or EM_DASH in js:
            raise Fail("the derived files would carry an em dash; the brand bans them")
        with open(os.path.join(out_dir, "tokens.css"), "w", encoding="utf-8") as f:
            f.write(css)
        with open(os.path.join(out_dir, "tokens.json"), "w", encoding="utf-8") as f:
            f.write(js)
        return 0
    if mode == "compare":
        brand_dir, derived_dir = argv[1], argv[2]
        lines = diff_maps("css", parse_css(os.path.join(brand_dir, "tokens.css")), parse_css(os.path.join(derived_dir, "tokens.css")))
        lines += diff_maps("json", parse_json(os.path.join(brand_dir, "tokens.json")), parse_json(os.path.join(derived_dir, "tokens.json")))
        for line in lines:
            print(line)
        if lines:
            return 1
        print("tokens match the markdown")
        return 0
    raise Fail("unknown mode " + mode)


try:
    sys.exit(main(sys.argv[1:]))
except Fail as e:
    print("FATAL: %s" % e, file=sys.stderr)
    sys.exit(2)
PYEOF
)

python3 -I -c "$PY" derive "$BRAND_DIR" "$SCRATCH"

if [[ "$CHECK" -eq 1 ]]; then
  # set -e makes the comparison's status the script's: 1 when anything differs.
  python3 -I -c "$PY" compare "$BRAND_DIR" "$SCRATCH"
  exit 0
fi

mv "$SCRATCH/tokens.css" "$BRAND_DIR/tokens.css"
echo "  -> $BRAND_DIR/tokens.css"
mv "$SCRATCH/tokens.json" "$BRAND_DIR/tokens.json"
echo "  -> $BRAND_DIR/tokens.json"
echo "tokens.css + tokens.json regenerated from canonical sources"
