#!/usr/bin/env python3
"""Invent a brand: turn one direction request into a full design system.

When a project has no brand, the agent drafts three short directions from the owner's brief
(docs/plans/2026-11-11-invent-a-brand.md). This script does the arithmetic for one of them, once
and exactly: every colour role in a light and a dark theme, the contrast pairs the brand will
document, and the type, spacing, radius and motion scales. There are no preset palettes or
themes: every value comes from the request and the rules below.

  python3 petals/scripts/new-brand.py system <request.json>   print the system as JSON on stdout

The request is one JSON object, every field required:
  name       lowercase letters and digits, starting with a letter; the token prefix is --<name>-
  key        {"hex": "#RRGGBB", "hold": "exact" | "bounded" | "flexible", "within": <distance>}
             the owner's colour; "within" (a CIEDE2000 distance) is given for a bounded hold only.
             Or {"hue": <0 to 360>} when the owner gave no colour: the hue of the familiar colour
             wheel (HSL), 0 red, 120 green, 240 blue
  loudness   quiet | balanced | loud     how saturated the palette is, and how far the type steps
  neutrals   warm | neutral | cool       the tint of the greys
  contrast   standard | high             the floors below
  shape      sharp | soft | round        the radius scale
  density    compact | roomy             the spacing unit, line heights and motion durations
A bad request exits 2 with one line naming the field and why. A request whose rules cannot all
hold (roles that cannot be kept apart) exits 1 naming the roles.

The rules the arithmetic holds:
  contrast   computed exactly as check.sh computes it (WCAG 2.x relative luminance, its chan,
             lum and ratio functions), so the generator and the check give the same answer;
             tests/directions.spec.sh compares the two on every pair. Floors by class:
               standard  reading 4.5:1, labels 3:1
               high      reading 7:1, labels 4.5:1
             Text, soft text, words in the primary colour and the status colours are reading
             pairs on bg and surface; the accent is a labels pair; text on a primary fill
             (on-primary) is reading: bg or text when one reaches the floor, else a shade pushed
             toward white or black. On a seed that keeps its value no colour may reach 7:1; then
             the pair is labels, which white or black always reaches. A primary that cannot
             carry words is decorative on bg, or labels when it reaches that floor.
  the seed   exact: the seed is the primary in both themes, never changed. When it cannot be
             words on bg or surface, it keeps its role for fills and marks, and words in the
             primary colour take a derived shade, primary-text, in both themes (where the seed
             passes, primary-text is the seed). bounded: the primary moves along its lightness
             no further than "within" from the seed; when that cannot reach the floor it stays
             the seed and takes primary-text as an exact seed does. flexible: the seed is where
             the lightness walk starts.
  distance   roles a person must tell apart are at least DISTANCE apart in CIEDE2000, in both
             themes: primary and accent, the four status colours with each other, and each
             status colour and the primary (and primary-text when there is one) and the
             accent, so a highlight is never read as an error or a notice. DISTANCE is a
             dial, set to 12: issue #14's family keeps its blooms 12.9 apart and 12 from its
             other inks. Measured on 384 requests (every 15 degrees of hue at each loudness and
             contrast, and 60 random seeds held exact and flexible): 12 holds for all; 14
             refuses 5, 16 refuses 28, 20 refuses 163. A status colour starts at the hue its
             meaning carries (error red, warning amber, success green, info blue) and turns up
             to 30 degrees from it, or moves up to 0.12 further from the background in
             lightness, the smallest move first, to clear the distance; the receipt says how
             far. The accent starts opposite the primary and turns up to 60 degrees.
  determinism  the same request prints the same bytes: no clock, no randomness, a fixed walk.

How a colour is found: OKLCH (a perceptual lightness, chroma, hue space) gives each role a start;
the walk changes only the lightness, in steps of 0.002, toward the dark end in the light theme and
the light end in the dark theme, keeping the hue and as much chroma as sRGB allows, and takes the
first hex that meets every floor. Every check runs on the final six-digit hex.

The scales: type steps from a 16px body by a ratio (quiet 1.2, balanced 1.25, loud 1.333), with
line heights by density; spacing is multiples of 4px (compact on a 4px unit, roomy on 8px), as
check.sh's spacing rule wants; the radius scale by shape; motion durations by density, with one
easing that settles and never overshoots. Fonts are not this script's: the type scale has sizes
and no faces.

The output: name, prefix, the request, floors, distance, light and dark palettes (the roles
colors.md names, plus on-primary and, when needed, primary-text), pairs as
{fg, bg, theme, class, ratio} with role names, the type, space, radius and motion scales, and a
receipt of every seed, hold and value a rule moved, with why.
"""
import json
import math
import re
import sys

USAGE = "usage: new-brand.py system <request.json>"

FLOORS = {"standard": {"reading": 4.5, "labels": 3.0}, "high": {"reading": 7.0, "labels": 4.5}}
DISTANCE = 12.0
STEP = 0.002  # one step of the lightness walk, in OKLCH lightness

FIELDS = ["name", "key", "loudness", "neutrals", "contrast", "shape", "density"]
CHOICES = {
    "loudness": ["quiet", "balanced", "loud"],
    "neutrals": ["warm", "neutral", "cool"],
    "contrast": ["standard", "high"],
    "shape": ["sharp", "soft", "round"],
    "density": ["compact", "roomy"],
}
THEMES = ("light", "dark")
# the walk's direction: away from the background
AWAY = {"light": -1, "dark": 1}

# Where each role's lightness walk starts, in OKLCH lightness, and the chroma of the neutral tint.
NEUTRAL_START = {
    "light": {"bg": 0.985, "surface": 0.955, "border": 0.87, "text": 0.25, "text-soft": 0.50},
    "dark": {"bg": 0.18, "surface": 0.225, "border": 0.35, "text": 0.94, "text-soft": 0.76},
}
NEUTRAL_CHROMA = {"bg": 0.008, "surface": 0.010, "border": 0.014, "text": 0.014, "text-soft": 0.018}
# The temperature of the greys, as a wheel hue: warm leans orange, cool leans blue.
NEUTRAL_HUE = {"warm": 35, "cool": 220}
COLOUR_START = {"light": 0.55, "dark": 0.74}
# OKLCH chroma by loudness, for a primary from a hue, the accent and the status colours.
CHROMA = {"quiet": 0.07, "balanced": 0.12, "loud": 0.17}
STATUS_CHROMA = {"quiet": 0.09, "balanced": 0.13, "loud": 0.16}
# The hue each status meaning carries, on the familiar wheel.
STATUS_HUE = {"error": 0, "warning": 40, "success": 130, "info": 210}
STATUS = ["error", "warning", "success", "info"]
STATUS_TURN = 30  # how far a status hue may turn from its meaning, in OKLCH degrees
ACCENT_TURN = 180  # the accent sits opposite the primary on the wheel

TYPE_RATIO = {"quiet": 1.2, "balanced": 1.25, "loud": 1.333}
RADIUS = {"sharp": [0, 2, 4, 6], "soft": [2, 4, 8, 12], "round": [4, 8, 16, 24]}


class RequestError(Exception):
    """The request breaks the interface; the message names the field."""

    def __init__(self, field, why):
        super().__init__("%s: %s" % (field, why))


class RuleError(Exception):
    """The rules cannot all hold for this request."""


# ---- contrast, as check.sh computes it (its rgb, chan, lum and ratio functions) -----------------

def rgb(h):
    return int(h[1:3], 16), int(h[3:5], 16), int(h[5:7], 16)


def chan(c):
    c = c / 255
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(h):
    r, g, b = rgb(h)
    return 0.2126 * chan(r) + 0.7152 * chan(g) + 0.0722 * chan(b)


def ratio(f, b):
    lf, lb = lum(f), lum(b)
    hi, lo = (lf, lb) if lf > lb else (lb, lf)
    return (hi + 0.05) / (lo + 0.05)


# ---- colour spaces --------------------------------------------------------------------------------

def to_linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def to_gamma(c):
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def cbrt(x):
    return math.copysign(abs(x) ** (1 / 3), x)


def linear_to_oklab(r, g, b):
    l_ = cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
    m_ = cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
    s_ = cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
    return (0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_)


def oklab_to_linear(L, a, b):
    l_ = L + 0.3963377774 * a + 0.2158037573 * b
    m_ = L - 0.1055613458 * a - 0.0638541728 * b
    s_ = L - 0.0894841775 * a - 1.2914855480 * b
    l, m, s = l_ ** 3, m_ ** 3, s_ ** 3
    return (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)


def hex_to_oklch(h):
    L, a, b = linear_to_oklab(*(to_linear(c / 255) for c in rgb(h)))
    return L, math.hypot(a, b), math.degrees(math.atan2(b, a)) % 360


def wheel_to_oklch_hue(hue):
    """The OKLCH hue of the fully saturated colour at this hue of the familiar (HSL) wheel."""
    h = (hue % 360) / 60
    x = 1 - abs(h % 2 - 1)
    r, g, b = [(1, x, 0), (x, 1, 0), (0, 1, x), (0, x, 1), (x, 0, 1), (1, 0, x)][int(h) % 6]
    _, a, bb = linear_to_oklab(to_linear(r), to_linear(g), to_linear(b))
    return math.degrees(math.atan2(bb, a)) % 360


def in_gamut(rgb_lin):
    return all(-1e-7 <= c <= 1 + 1e-7 for c in rgb_lin)


def oklch_to_hex(L, C, h):
    """The six-digit hex of an OKLCH colour, keeping its lightness and hue and as much of its
    chroma as sRGB holds (the largest in-gamut chroma, found by bisection)."""
    L = min(max(L, 0.0), 1.0)
    a, b = math.cos(math.radians(h)), math.sin(math.radians(h))
    lin = oklab_to_linear(L, C * a, C * b)
    if not in_gamut(lin):
        lo, hi = 0.0, C
        for _ in range(40):
            mid = (lo + hi) / 2
            if in_gamut(oklab_to_linear(L, mid * a, mid * b)):
                lo = mid
            else:
                hi = mid
        lin = oklab_to_linear(L, lo * a, lo * b)
    return "#" + "".join("%02X" % min(255, max(0, math.floor(to_gamma(min(max(c, 0.0), 1.0)) * 255 + 0.5)))
                         for c in lin)


# ---- CIEDE2000 ------------------------------------------------------------------------------------

def hex_to_lab(h):
    r, g, b = (to_linear(c / 255) for c in rgb(h))
    x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047
    y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b
    z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883

    def f(t):
        return t ** (1 / 3) if t > (6 / 29) ** 3 else t / (3 * (6 / 29) ** 2) + 4 / 29
    return 116 * f(y) - 16, 500 * (f(x) - f(y)), 200 * (f(y) - f(z))


def ciede2000(lab1, lab2):
    """CIEDE2000 colour difference of two CIELAB colours (Sharma, Wu and Dalal, 2005)."""
    L1, a1, b1 = lab1
    L2, a2, b2 = lab2
    cbar = (math.hypot(a1, b1) + math.hypot(a2, b2)) / 2
    g = 0.5 * (1 - math.sqrt(cbar ** 7 / (cbar ** 7 + 25 ** 7)))
    a1p, a2p = (1 + g) * a1, (1 + g) * a2
    c1p, c2p = math.hypot(a1p, b1), math.hypot(a2p, b2)
    h1p = 0.0 if a1p == 0 and b1 == 0 else math.degrees(math.atan2(b1, a1p)) % 360
    h2p = 0.0 if a2p == 0 and b2 == 0 else math.degrees(math.atan2(b2, a2p)) % 360
    dlp, dcp = L2 - L1, c2p - c1p
    if c1p * c2p == 0:
        dhp = 0.0
    else:
        dhp = h2p - h1p
        if dhp > 180:
            dhp -= 360
        elif dhp < -180:
            dhp += 360
    dHp = 2 * math.sqrt(c1p * c2p) * math.sin(math.radians(dhp / 2))
    lbarp, cbarp = (L1 + L2) / 2, (c1p + c2p) / 2
    if c1p * c2p == 0:
        hbarp = h1p + h2p
    elif abs(h1p - h2p) <= 180:
        hbarp = (h1p + h2p) / 2
    elif h1p + h2p < 360:
        hbarp = (h1p + h2p + 360) / 2
    else:
        hbarp = (h1p + h2p - 360) / 2
    t = (1 - 0.17 * math.cos(math.radians(hbarp - 30)) + 0.24 * math.cos(math.radians(2 * hbarp))
         + 0.32 * math.cos(math.radians(3 * hbarp + 6)) - 0.20 * math.cos(math.radians(4 * hbarp - 63)))
    dtheta = 30 * math.exp(-(((hbarp - 275) / 25) ** 2))
    rc = 2 * math.sqrt(cbarp ** 7 / (cbarp ** 7 + 25 ** 7))
    sl = 1 + 0.015 * (lbarp - 50) ** 2 / math.sqrt(20 + (lbarp - 50) ** 2)
    sc = 1 + 0.045 * cbarp
    sh = 1 + 0.015 * cbarp * t
    rt = -math.sin(math.radians(2 * dtheta)) * rc
    return math.sqrt((dlp / sl) ** 2 + (dcp / sc) ** 2 + (dHp / sh) ** 2 + rt * (dcp / sc) * (dHp / sh))


def delta_e(h1, h2):
    return ciede2000(hex_to_lab(h1), hex_to_lab(h2))


# ---- the walk -------------------------------------------------------------------------------------

def walk(start, theme, ok, limit=None, extra=0):
    """Walk an OKLCH colour's lightness away from the theme's background, from its own value, and
    return the first hex for which ok(hex) holds, then `extra` steps further while ok still holds.
    limit(hex) false stops the walk. None when no lightness passes."""
    L0, C, h = start
    d = AWAY[theme]
    steps = int(math.ceil((L0 if d < 0 else 1 - L0) / STEP))
    for i in range(steps + 1):
        hx = oklch_to_hex(L0 + d * i * STEP, C, h)
        if limit is not None and not limit(hx):
            return None
        if ok(hx):
            if extra:
                j = i + extra
                further = oklch_to_hex(L0 + d * j * STEP, C, h)
                while not ok(further) and j < steps:
                    j += 1
                    further = oklch_to_hex(L0 + d * j * STEP, C, h)
                return further if ok(further) else hx
            return hx
    return None


def floor_ok(floor, *bgs):
    return lambda hx: all(ratio(hx, b) >= floor for b in bgs)


def fmt(r):
    return "%.2f:1" % r


# ---- the request ----------------------------------------------------------------------------------

def is_number(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v)


def parse_request(text):
    try:
        req = json.loads(text)
    except ValueError as exc:
        raise RequestError("request", "not valid JSON (%s)" % exc)
    if not isinstance(req, dict):
        raise RequestError("request", "must be a JSON object")
    for k in req:
        if k not in FIELDS:
            raise RequestError(k, "not a field of a direction request (fields: %s)" % ", ".join(FIELDS))
    for k in FIELDS:
        if k not in req:
            raise RequestError(k, "missing")
    name = req["name"]
    if not isinstance(name, str) or not re.fullmatch(r"[a-z][a-z0-9]*", name):
        raise RequestError("name", "must be lowercase letters and digits, starting with a letter")
    for k, allowed in CHOICES.items():
        if req[k] not in allowed:
            raise RequestError(k, "must be %s or %s (got %s)" % (", ".join(allowed[:-1]), allowed[-1], json.dumps(req[k])))
    key = req["key"]
    if not isinstance(key, dict):
        raise RequestError("key", "must be an object: {\"hex\", \"hold\"} or {\"hue\"}")
    if "hue" in key and "hex" in key:
        raise RequestError("key", "gives both a hue and a hex; give one")
    if "hue" in key:
        extra = sorted(set(key) - {"hue"})
        if extra:
            raise RequestError("key." + extra[0], "a hue key carries only the hue")
        if not is_number(key["hue"]) or not 0 <= key["hue"] <= 360:
            raise RequestError("key.hue", "must be a number from 0 to 360")
        out_key = {"hue": key["hue"]}
    elif "hex" in key:
        extra = sorted(set(key) - {"hex", "hold", "within"})
        if extra:
            raise RequestError("key." + extra[0], "not a field of a hex key (hex, hold, within)")
        if not isinstance(key["hex"], str) or not re.fullmatch(r"#[0-9A-Fa-f]{6}", key["hex"]):
            raise RequestError("key.hex", "must be a six-digit hex such as #2F6B4F")
        hold = key.get("hold")
        if hold not in ("exact", "bounded", "flexible"):
            raise RequestError("key.hold", "must be exact, bounded or flexible (got %s)" % json.dumps(hold))
        if hold == "bounded":
            if "within" not in key:
                raise RequestError("key.within", "a bounded hold needs the CIEDE2000 distance the seed may move")
            if not is_number(key["within"]) or key["within"] <= 0:
                raise RequestError("key.within", "must be a positive number (a CIEDE2000 distance)")
        elif "within" in key:
            raise RequestError("key.within", "applies to a bounded hold only")
        out_key = {"hex": key["hex"].upper(), "hold": hold}
        if hold == "bounded":
            out_key["within"] = key["within"]
    else:
        raise RequestError("key", "needs a hex and a hold, or a hue")
    out = {"name": name, "key": out_key}
    for k in FIELDS[2:]:
        out[k] = req[k]
    return out


# ---- the system -----------------------------------------------------------------------------------

def build(req):
    floors = FLOORS[req["contrast"]]
    reading, labels = floors["reading"], floors["labels"]
    receipt = []
    pal = {t: {} for t in THEMES}

    def note(role, theme, frm, to, why):
        receipt.append({"role": role, "theme": theme, "from": frm, "to": to, "why": why})

    # neutrals: bg, surface and border sit where they start; text and soft text walk to the floor
    tint = req["neutrals"]
    nhue = wheel_to_oklch_hue(NEUTRAL_HUE[tint]) if tint in NEUTRAL_HUE else 0.0
    for t in THEMES:
        for role in ("bg", "surface", "border"):
            pal[t][role] = oklch_to_hex(NEUTRAL_START[t][role], NEUTRAL_CHROMA[role] if tint in NEUTRAL_HUE else 0.0, nhue)
        for role in ("text", "text-soft"):
            start = (NEUTRAL_START[t][role], NEUTRAL_CHROMA[role] if tint in NEUTRAL_HUE else 0.0, nhue)
            first = oklch_to_hex(*start)
            hx = walk(start, t, floor_ok(reading, pal[t]["bg"], pal[t]["surface"]))
            if hx is None:
                raise RuleError("%s cannot reach %s:1 on the %s background" % (role, reading, t))
            pal[t][role] = hx
            if hx != first:
                note(role, t, first, hx, "%s to reach the %s:1 reading floor on bg and surface (now %s on bg)"
                     % ("darkened" if t == "light" else "lightened", reading, fmt(ratio(hx, pal[t]["bg"]))))

    # the primary
    key = req["key"]
    words = {t: floor_ok(reading, pal[t]["bg"], pal[t]["surface"]) for t in THEMES}
    needs_text = False
    if "hue" in key:
        ph = wheel_to_oklch_hue(key["hue"])
        for t in THEMES:
            start = (COLOUR_START[t], CHROMA[req["loudness"]], ph)
            first = oklch_to_hex(*start)
            hx = walk(start, t, words[t])
            if hx is None:
                raise RuleError("the primary cannot reach %s:1 on the %s background" % (reading, t))
            pal[t]["primary"] = hx
            note("primary", t, first, hx, "from wheel hue %s at %s chroma; %s to reach the %s:1 reading floor on bg and surface (now %s on bg)"
                 % (key["hue"], req["loudness"], "darkened" if t == "light" else "lightened", reading, fmt(ratio(hx, pal[t]["bg"])))
                 if hx != first else "from wheel hue %s at %s chroma; meets the %s:1 reading floor where it starts" % (key["hue"], req["loudness"], reading))
    else:
        seed, hold = key["hex"], key["hold"]
        sl = hex_to_oklch(seed)
        ph = sl[2]
        for t in THEMES:
            if words[t](seed):
                pal[t]["primary"] = seed
                note("primary", t, seed, seed, "seed held %s; it meets the %s:1 reading floor on bg and surface as it is (%s on bg)"
                     % (hold, reading, fmt(ratio(seed, pal[t]["bg"]))))
                continue
            hx = None
            if hold == "flexible":
                hx = walk(sl, t, words[t])
            elif hold == "bounded":
                hx = walk(sl, t, words[t], limit=lambda c: delta_e(c, seed) <= key["within"])
            if hx is not None:
                pal[t]["primary"] = hx
                note("primary", t, seed, hx, "seed held %s%s; %s from %s to %s to reach the %s:1 reading floor on bg and surface (now %s on bg, %.2f from the seed)"
                     % (hold, " within %s" % key["within"] if hold == "bounded" else "",
                        "darkened" if t == "light" else "lightened", seed, hx, reading, fmt(ratio(hx, pal[t]["bg"])), delta_e(hx, seed)))
            else:
                pal[t]["primary"] = seed
                needs_text = True
                note("primary", t, seed, seed, "seed held %s%s; at %s on bg it cannot carry words%s, so it keeps its role for fills and marks"
                     % (hold, " within %s" % key["within"] if hold == "bounded" else "", fmt(ratio(seed, pal[t]["bg"])),
                        " within the bound" if hold == "bounded" else ""))
        if needs_text:
            for t in THEMES:
                p = pal[t]["primary"]
                if words[t](p):
                    pal[t]["primary-text"] = p
                    note("primary-text", t, p, p, "words in the primary colour use the primary itself in this theme (%s on bg)" % fmt(ratio(p, pal[t]["bg"])))
                else:
                    hx = walk(hex_to_oklch(p), t, words[t])
                    if hx is None:
                        raise RuleError("no shade of the primary reaches %s:1 on the %s background" % (reading, t))
                    pal[t]["primary-text"] = hx
                    note("primary-text", t, p, hx, "words in the primary colour take this shade of the seed, %s to reach the %s:1 reading floor on bg and surface (now %s on bg)"
                         % ("darkened" if t == "light" else "lightened", reading, fmt(ratio(hx, pal[t]["bg"]))))

    # on-primary: the text on a primary fill
    on_class = {}
    for t in THEMES:
        p = pal[t]["primary"]
        best = max((pal[t]["bg"], pal[t]["text"]), key=lambda c: ratio(c, p))
        if ratio(best, p) >= reading:
            name = "bg" if best == pal[t]["bg"] else "text"
            pal[t]["on-primary"] = best
            on_class[t] = "reading"
            note("on-primary", t, None, best, "text on a primary fill takes %s, %s on the primary" % (name, fmt(ratio(best, p))))
            continue
        # neither neutral reaches the floor: walk each, the better first, further from the
        # primary, toward white or black ("dark" walks toward white, "light" toward black)
        found = None
        for c in sorted((pal[t]["bg"], pal[t]["text"]), key=lambda c: -ratio(c, p)):
            found = walk(hex_to_oklch(c), "dark" if lum(c) > lum(p) else "light", lambda x: ratio(x, p) >= reading)
            if found is not None:
                pal[t]["on-primary"] = found
                on_class[t] = "reading"
                note("on-primary", t, c, found, "text on a primary fill: no neutral reaches %s:1 on the primary, so this shade is pushed to %s"
                     % (reading, fmt(ratio(found, p))))
                break
        if found is not None:
            continue
        hx = max(("#FFFFFF", "#000000"), key=lambda c: ratio(c, p))
        if ratio(hx, p) < labels:
            raise RuleError("no text colour reaches %s:1 on the primary %s" % (labels, p))
        pal[t]["on-primary"] = hx
        on_class[t] = "labels"
        note("on-primary", t, None, hx, "text on a primary fill: no colour reaches the %s:1 reading floor on the primary; %s reaches %s, so a primary fill carries labels and large type only"
             % (reading, hx, fmt(ratio(hx, p))))

    family = ["primary"] + (["primary-text"] if "primary-text" in pal["light"] else [])

    # the accent: opposite the primary, walked to the labels floor, turned until it stands apart
    achroma = CHROMA[req["loudness"]]
    placed = False
    for turn in [ACCENT_TURN + d for d in (0, 15, -15, 30, -30, 45, -45, 60, -60)]:
        cand = {}
        for t in THEMES:
            cand[t] = walk((COLOUR_START[t], achroma, (ph + turn) % 360), t, floor_ok(labels, pal[t]["bg"], pal[t]["surface"]))
        if all(cand[t] and all(delta_e(cand[t], pal[t][f]) >= DISTANCE for f in family) for t in THEMES):
            for t in THEMES:
                pal[t]["accent"] = cand[t]
                note("accent", t, None, cand[t], "%s degrees round from the primary at %s chroma; meets the %s:1 labels floor on bg and surface (%s on bg, %.2f from the primary)"
                     % (turn, req["loudness"], labels, fmt(ratio(cand[t], pal[t]["bg"])), delta_e(cand[t], pal[t]["primary"])))
            placed = True
            break
    if not placed:
        raise RuleError("the accent cannot stand %s from the primary in both themes" % DISTANCE)

    # status colours: the hue of their meaning, turned or stepped only as far as the distance needs
    schroma = STATUS_CHROMA[req["loudness"]]
    turns = [0]
    for d in range(6, STATUS_TURN + 1, 6):
        turns += [d, -d]
    tries = sorted(((abs(tn) / 6 + ex / 20, i, tn, ex) for i, tn in enumerate(turns) for ex in (0, 20, 40, 60)))
    for s in STATUS:
        anchor = wheel_to_oklch_hue(STATUS_HUE[s])
        others = family + ["accent"] + [x for x in STATUS if x in pal["light"]]
        for _, _, tn, ex in tries:
            cand = {}
            for t in THEMES:
                cand[t] = walk((COLOUR_START[t], schroma, (anchor + tn) % 360), t, words[t], extra=ex)
            if all(cand[t] and all(delta_e(cand[t], pal[t][o]) >= DISTANCE for o in others) for t in THEMES):
                break
        else:
            raise RuleError("%s cannot stand %s from %s in both themes" % (s, DISTANCE, ", ".join(others)))
        for t in THEMES:
            pal[t][s] = cand[t]
            near = min(others, key=lambda o: delta_e(cand[t], pal[t][o]))
            how = "at the hue of its meaning" if tn == 0 else "turned %+d degrees from the hue of its meaning" % tn
            if ex:
                how += ", %.2f further from the background in lightness" % (ex * STEP)
            note(s, t, None, cand[t], "%s; meets the %s:1 reading floor on bg and surface (%s on bg); nearest role %s at %.2f"
                 % (how, reading, fmt(ratio(cand[t], pal[t]["bg"])), near, delta_e(cand[t], pal[t][near])))

    # the documented pairs
    pairs = []

    def pair(t, fg, bg, cls):
        pairs.append({"fg": fg, "bg": bg, "theme": t, "class": cls, "ratio": float("%.2f" % ratio(pal[t][fg], pal[t][bg]))})

    for t in THEMES:
        for fg in ("text", "text-soft"):
            for bg in ("bg", "surface"):
                pair(t, fg, bg, "reading")
        for bg in ("bg", "surface"):
            r = min(ratio(pal[t]["primary"], pal[t][x]) for x in ("bg", "surface"))
            pair(t, "primary", bg, "reading" if r >= reading else "labels" if r >= labels else "decorative")
        if "primary-text" in pal[t]:
            for bg in ("bg", "surface"):
                pair(t, "primary-text", bg, "reading")
        pair(t, "on-primary", "primary", on_class[t])
        for bg in ("bg", "surface"):
            pair(t, "accent", bg, "labels")
        for s in ("error", "success", "warning", "info"):
            for bg in ("bg", "surface"):
                pair(t, s, bg, "reading")
    seen = {}
    for p in pairs:
        k = (pal[p["theme"]][p["fg"]], pal[p["theme"]][p["bg"]])
        if seen.setdefault(k, p["class"]) != p["class"]:
            raise RuleError("%s on %s would be documented as both %s and %s" % (k[0], k[1], seen[k], p["class"]))

    order = ["primary", "primary-text", "on-primary", "accent", "text", "text-soft", "bg", "surface", "border",
             "error", "success", "warning", "info"]
    palettes = {t: {r: pal[t][r] for r in order if r in pal[t]} for t in THEMES}

    # the scales
    ratio_t = TYPE_RATIO[req["loudness"]]
    compact = req["density"] == "compact"
    body_line = 1.45 if compact else 1.6
    head_line = 1.15 if compact else 1.2
    levels = [("h1", 4, head_line - 0.05), ("h2", 3, head_line), ("h3", 2, head_line),
              ("body", 0, body_line), ("small", -0.5, body_line - 0.05), ("caption", -1, body_line - 0.1)]
    type_scale = {"base": "16px", "ratio": ratio_t,
                  "levels": [{"level": lv, "size": "%dpx" % math.floor(16 * ratio_t ** e + 0.5), "line": round(ln, 2)}
                             for lv, e, ln in levels]}
    unit = 4 if compact else 8
    steps = [1, 2, 3, 4, 6, 8, 12, 16] if compact else [0.5, 1, 2, 3, 4, 6, 8, 12]
    names = ["2xs", "xs", "sm", "md", "lg", "xl", "2xl", "3xl"]
    space = {"unit": "%dpx" % unit, "scale": {n: "%dpx" % int(unit * s) for n, s in zip(names, steps)}}
    radius = {"radius-%s" % n: "%dpx" % v for n, v in zip(("xs", "sm", "md", "lg"), RADIUS[req["shape"]])}
    radius["radius-full"] = "999px"
    motion = {"ease": "cubic-bezier(0.2, 0, 0, 1)",
              "dur-micro": "0.1s" if compact else "0.15s",
              "dur-entrance": "0.2s" if compact else "0.3s",
              "stagger-step": "0.03s" if compact else "0.05s"}

    seeds = [{"role": "seed", "theme": None, "from": key["hex"], "to": key["hex"],
              "why": "the owner's colour, hold %s%s" % (key["hold"], " within %s" % key["within"] if key["hold"] == "bounded" else "")}] \
        if "hex" in key else [{"role": "seed", "theme": None, "from": None, "to": None,
                               "why": "no colour given; the primary comes from wheel hue %s" % key["hue"]}]
    return {
        "name": req["name"],
        "prefix": "--%s-" % req["name"],
        "request": req,
        "floors": floors,
        "distance": DISTANCE,
        "light": palettes["light"],
        "dark": palettes["dark"],
        "pairs": pairs,
        "type": type_scale,
        "space": space,
        "radius": radius,
        "motion": motion,
        "receipt": seeds + receipt,
    }


def main(argv):
    if len(argv) != 3 or argv[1] != "system":
        print(USAGE, file=sys.stderr)
        return 2
    try:
        with open(argv[2], encoding="utf-8") as f:
            text = f.read()
    except OSError as exc:
        print("new-brand: request: cannot read %s (%s)" % (argv[2], exc.strerror), file=sys.stderr)
        return 2
    try:
        req = parse_request(text)
        system = build(req)
    except RequestError as exc:
        print("new-brand: %s" % exc, file=sys.stderr)
        return 2
    except RuleError as exc:
        print("new-brand: %s" % exc, file=sys.stderr)
        return 1
    sys.stdout.write(json.dumps(system, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
