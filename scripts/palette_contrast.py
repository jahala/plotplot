#!/usr/bin/env python3
"""Measure WCAG 2.x contrast for the plotplot playful-garden palette and emit
the Contrast Pairings rows in the exact shape .brand/colors.md (and petals'
check.sh) expect:  | <Name> #FG on <Name> #BG | <ratio> | <class> |

Classes (petals): reading >=4.5 · labels/display >=3.0 · decorative exempt.
Run: python3 scripts/palette_contrast.py
"""

def _lin(c):
    c /= 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

def _lum(h):
    h = h.lstrip("#")
    r, g, b = (int(h[i:i+2], 16) for i in (0, 2, 4))
    return 0.2126*_lin(r) + 0.7152*_lin(g) + 0.0722*_lin(b)

def ratio(fg, bg):
    a, b = _lum(fg), _lum(bg)
    return (max(a, b) + 0.05) / (min(a, b) + 0.05)

C = {
    # paper tiers
    "Paper": "#FAF5E9", "Band": "#F2EAD6", "Surface": "#F5EEDD", "Border": "#E2D8C0",
    # ink
    "Ink": "#3A2718", "Text-soft": "#786148",
    # greens
    "Growth": "#357E2C", "Growth-deep": "#2C6B2A", "Leaf": "#4A9E3F", "Forest": "#214A2C",
    # sun
    "Sunlight": "#E89227", "Amber-ink": "#A86518",
    # semantics
    "Healthy": "#46913C", "Caution": "#B0741C", "Error": "#BC4126",
    "Info": "#3F7186", "Muted": "#9A8C72",
    # blooms / product accents
    "Poppy": "#D6502F", "Sky": "#4E88A6", "Plum": "#97539B", "Petal": "#E588A0",
    "Juniper": "#1F8A7B", "Pollen": "#C8B330", "Pollen-ink": "#7E6A08",
    "Bramble": "#8E3B5E", "Bramble-night": "#B85C82",
    # soil-night (dark / terminal)
    "Night": "#1C1610", "Night-card": "#262019",
    "Night-text": "#F3ECD9", "Night-soft": "#C9BBA0",
    "Night-green": "#84C56A", "Night-sun": "#F2A93B", "Night-leaf": "#9FD08A",
    "Night-pollen": "#D9C44A",
}

# (fg, bg, class, target)
PAIRS = [
    ("Ink", "Paper", "reading", 4.5), ("Ink", "Surface", "reading", 4.5),
    ("Ink", "Band", "reading", 4.5), ("Text-soft", "Paper", "reading", 4.5),
    ("Growth", "Paper", "reading", 4.5), ("Paper", "Growth", "reading", 4.5),
    ("Growth-deep", "Paper", "reading", 4.5), ("Forest", "Paper", "reading", 4.5),
    ("Paper", "Forest", "reading", 4.5),
    ("Ink", "Sunlight", "reading", 4.5), ("Sunlight", "Paper", "decorative", 0.0),
    ("Amber-ink", "Paper", "labels", 3.0), ("Leaf", "Paper", "decorative", 0.0),
    ("Healthy", "Surface", "labels", 3.0), ("Caution", "Paper", "labels", 3.0),
    ("Error", "Paper", "reading", 4.5), ("Info", "Paper", "labels", 3.0),
    ("Muted", "Paper", "decorative", 0.0),
    ("Poppy", "Paper", "labels", 3.0), ("Sky", "Paper", "labels", 3.0),
    ("Plum", "Paper", "labels", 3.0), ("Petal", "Paper", "decorative", 0.0),
    ("Juniper", "Paper", "labels", 3.0),
    ("Pollen", "Paper", "decorative", 0.0), ("Ink", "Pollen", "reading", 4.5),
    ("Pollen-ink", "Paper", "reading", 4.5),
    ("Bramble", "Paper", "reading", 4.5), ("Bramble", "Night", "decorative", 0.0),
    ("Bramble-night", "Night", "labels", 3.0),
    # soil-night
    ("Night-text", "Night", "reading", 4.5), ("Night-soft", "Night", "reading", 4.5),
    ("Night-green", "Night", "reading", 4.5), ("Night-sun", "Night", "reading", 4.5),
    ("Night-leaf", "Night", "reading", 4.5),
    ("Night-pollen", "Night", "reading", 4.5),
]

print(f"{'pair':<34}{'ratio':>7}  {'class':<11} verdict")
print("-" * 72)
fails = 0
rows = []
for fg, bg, cls, target in PAIRS:
    r = ratio(C[fg], C[bg])
    ok = "—" if cls == "decorative" else ("PASS" if r >= target else "FAIL")
    if ok == "FAIL":
        fails += 1
    print(f"{fg+' on '+bg:<34}{r:>6.2f}  {cls:<11} {ok}")
    rows.append(f"| {fg} {C[fg]} on {bg} {C[bg]} | {r:.1f} | {cls} |")

print(f"\n{fails} failing pair(s).\n")
print("=== paste into colors.md ## Contrast Pairings ===")
print("| Foreground on background | Ratio | Class |")
print("|---|---|---|")
for row in rows:
    print(row)
