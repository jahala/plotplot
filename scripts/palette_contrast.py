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
    # blooms, one family per row, light to dark in order of use
    "Mull": "#76D4FF",
    "Glean": "#0096B3",
    "Cultivate": "#09636B", "Cultivate-night": "#1E6F77",
    "Graft": "#FEAA68",
    "Petals": "#CC6A37",
    "Trellis": "#A33C26", "Trellis-night": "#AA422C",
    "Tend": "#98D98D",
    "Pleach": "#779C46",
    "Umbel": "#545E05", "Umbel-night": "#5F6A18",
    "Transplant": "#FD81ED",
    "Pollen": "#BD59D2",
    "Harvest": "#7D2EB0", "Harvest-night": "#8A3EBF",
    "Tilth": "#F7D4B7",
    "Mycelium": "#C9A18D",
    "Weeder": "#A37A6E",
    "Stake": "#774F4C", "Stake-night": "#835A57",
    "Hedge": "#C9C7E7",
    "Gate": "#8F94B2",
    "Well": "#4D5972", "Well-night": "#58647E",
    "Gardener": "#C7C784",
    "Almanac": "#A19251",
    "Upkeep": "#6E520D", "Upkeep-night": "#7D6021",
    "Juniper": "#1F8A7B", "Pollen-ink": "#A844BC", "Pollen-ink-night": "#BD59D2", "Weeder-word": "#8F675B", "Weeder-word-night": "#A37A6E",
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
    # each bloom: on paper (a label at 3:1, or decorative when it is a plate behind text), at night (3:1),
    # and the plate text it carries in both themes (4.5:1)
    ("Mull", "Paper", "decorative", 0.0), ("Mull", "Night", "labels", 3.0), ("Night", "Mull", "reading", 4.5), ("Night", "Mull", "reading", 4.5),
    ("Glean", "Paper", "labels", 3.0), ("Glean", "Night", "labels", 3.0), ("Night", "Glean", "reading", 4.5), ("Night", "Glean", "reading", 4.5),
    ("Cultivate", "Paper", "labels", 3.0), ("Cultivate-night", "Night", "labels", 3.0), ("Night-text", "Cultivate", "reading", 4.5), ("Night-text", "Cultivate-night", "reading", 4.5),
    ("Graft", "Paper", "decorative", 0.0), ("Graft", "Night", "labels", 3.0), ("Night", "Graft", "reading", 4.5), ("Night", "Graft", "reading", 4.5),
    ("Petals", "Paper", "labels", 3.0), ("Petals", "Night", "labels", 3.0), ("Night", "Petals", "reading", 4.5), ("Night", "Petals", "reading", 4.5),
    ("Trellis", "Paper", "labels", 3.0), ("Trellis-night", "Night", "labels", 3.0), ("Night-text", "Trellis", "reading", 4.5), ("Night-text", "Trellis-night", "reading", 4.5),
    ("Tend", "Paper", "decorative", 0.0), ("Tend", "Night", "labels", 3.0), ("Night", "Tend", "reading", 4.5), ("Night", "Tend", "reading", 4.5),
    ("Pleach", "Paper", "decorative", 0.0), ("Pleach", "Night", "labels", 3.0), ("Night", "Pleach", "reading", 4.5), ("Night", "Pleach", "reading", 4.5),
    ("Umbel", "Paper", "labels", 3.0), ("Umbel-night", "Night", "labels", 3.0), ("Night-text", "Umbel", "reading", 4.5), ("Night-text", "Umbel-night", "reading", 4.5),
    ("Transplant", "Paper", "decorative", 0.0), ("Transplant", "Night", "labels", 3.0), ("Night", "Transplant", "reading", 4.5), ("Night", "Transplant", "reading", 4.5),
    ("Pollen", "Paper", "labels", 3.0), ("Pollen", "Night", "labels", 3.0), ("Night", "Pollen", "reading", 4.5), ("Night", "Pollen", "reading", 4.5),
    ("Harvest", "Paper", "labels", 3.0), ("Harvest-night", "Night", "labels", 3.0), ("Night-text", "Harvest", "reading", 4.5), ("Night-text", "Harvest-night", "reading", 4.5),
    ("Tilth", "Paper", "decorative", 0.0), ("Tilth", "Night", "labels", 3.0), ("Night", "Tilth", "reading", 4.5), ("Night", "Tilth", "reading", 4.5),
    ("Mycelium", "Paper", "decorative", 0.0), ("Mycelium", "Night", "labels", 3.0), ("Night", "Mycelium", "reading", 4.5), ("Night", "Mycelium", "reading", 4.5),
    ("Weeder", "Paper", "labels", 3.0), ("Weeder", "Night", "labels", 3.0), ("Night", "Weeder", "reading", 4.5), ("Night", "Weeder", "reading", 4.5),
    ("Stake", "Paper", "labels", 3.0), ("Stake-night", "Night", "labels", 3.0), ("Night-text", "Stake", "reading", 4.5), ("Night-text", "Stake-night", "reading", 4.5),
    ("Hedge", "Paper", "decorative", 0.0), ("Hedge", "Night", "labels", 3.0), ("Night", "Hedge", "reading", 4.5), ("Night", "Hedge", "reading", 4.5),
    ("Gate", "Paper", "decorative", 0.0), ("Gate", "Night", "labels", 3.0), ("Night", "Gate", "reading", 4.5), ("Night", "Gate", "reading", 4.5),
    ("Well", "Paper", "labels", 3.0), ("Well-night", "Night", "labels", 3.0), ("Night-text", "Well", "reading", 4.5), ("Night-text", "Well-night", "reading", 4.5),
    ("Gardener", "Paper", "decorative", 0.0), ("Gardener", "Night", "labels", 3.0), ("Night", "Gardener", "reading", 4.5), ("Night", "Gardener", "reading", 4.5),
    ("Almanac", "Paper", "decorative", 0.0), ("Almanac", "Night", "labels", 3.0), ("Night", "Almanac", "reading", 4.5), ("Night", "Almanac", "reading", 4.5),
    ("Upkeep", "Paper", "labels", 3.0), ("Upkeep-night", "Night", "labels", 3.0), ("Night-text", "Upkeep", "reading", 4.5), ("Night-text", "Upkeep-night", "reading", 4.5),
    ("Juniper", "Paper", "labels", 3.0), ("Pollen-ink", "Paper", "reading", 4.5), ("Pollen-ink-night", "Night", "reading", 4.5),
    ("Weeder-word", "Paper", "reading", 4.5), ("Weeder-word-night", "Night", "reading", 4.5),
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
