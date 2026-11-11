#!/usr/bin/env bash
# petals/scripts/check.sh — the deterministic core of /petals check.
#
# Covers the pure-math dimensions without an agent in the loop:
#   color    — colour literals vs the palette: hex, oklch(), oklab(), rgb()
#              and hsl(), a colour outside sRGB reported as such (data-URI/
#              base64/url() excluded), nearest palette match by RGB distance;
#              contrast ratios for
#              same-rule color/background pairs (one-level var() resolution):
#              < 3.0 error, 3.0–4.5 warning (verify the pair class), >= 4.5 pass
#   layout   — padding/margin/gap px vs the spacing scale (multiples rule),
#              @media widths vs the documented breakpoints
#   surface  — border-radius vs the radius scale, overshoot cubic-beziers,
#              un-tinted (pure black) shadows, box-shadows vs the Shadow
#              Recipes (in a dark theme, the recipes' Night values)
#   voice    — forbidden terms (with replacements), unambiguous banned
#              terminology phrases, capitalized product names
#
# Stays with the agent (run via the skill): typography role classification,
# tone, and dismissing quoted-context voice hits the heuristic misses.
# Quote-context heuristic: lines containing never / use instead / forbidden /
# class="no" are skipped by the voice pass (they quote the rule, not break it).
#
# Usage: check.sh <file> [--brand <dir>]   (default brand dir: .brand)
# Exit: 0 when zero errors, 1 when any error present (binary pass/fail),
# 2 for a wrong option.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SCRIPT_DIR/lib/common.sh" ]]; then
  echo "petals: lib/common.sh is missing beside $(basename "$0"); install the whole petals/ directory" >&2
  exit 127
fi
source "$SCRIPT_DIR/lib/common.sh"
ROOT=$(find_project_root) && cd "$ROOT"
TARGET="${1:?usage: check.sh <file> [--brand <dir>] [--product <name>]}"
shift || true
BRAND=""; PRODUCT=""; FORMAT="text"; STAGE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --brand)   BRAND="${2:?--brand needs a dir}"; shift 2 ;;
    --stage)   STAGE="${2:?--stage needs mockup or production}"; shift 2 ;;
    --format)  FORMAT="${2:?--format needs text or sarif}"; shift 2 ;;
    --product) PRODUCT="${2:?--product needs a name}"; shift 2 ;;
    *) shift ;;
  esac
done
case "$FORMAT" in text|sarif) ;; *) echo "ERROR --format must be text or sarif"; exit 2 ;; esac
# Paths the caller typed are relative to where they called from.
TARGET=$(from_caller "$TARGET")
[ -n "$BRAND" ] && BRAND=$(from_caller "$BRAND")
BRAND="${BRAND:-.brand}"
# product context: flag > .petalsrc > the single products/ dir > flat brand
# "--product -" says there is no product context; .petalsrc is not consulted
if [ "$PRODUCT" = "-" ]; then PRODUCT=""; NO_PRODUCT=1; else NO_PRODUCT=0; fi
if [ -z "$PRODUCT" ] && [ "$NO_PRODUCT" = 0 ] && [ -f .petalsrc ]; then
  PRODUCT=$(awk -F': *' '/^[ ]*product:/{print $2; exit}' .petalsrc 2>/dev/null)
fi
[ -f "$TARGET" ] || { echo "ERROR target file not found: $TARGET"; exit 1; }
[ -d "$BRAND" ] || { echo "No brand configured. Run /petals init first."; exit 1; }

if [ -z "$PRODUCT" ] && [ "$NO_PRODUCT" = 0 ] && [ -d "$BRAND/products" ]; then
  PCOUNT=$(ls -1 "$BRAND/products" 2>/dev/null | wc -l | tr -d " ")
  [ "$PCOUNT" = "1" ] && PRODUCT=$(ls -1 "$BRAND/products")
fi
PRODUCT_VOICE=""
[ -n "$PRODUCT" ] && [ -f "$BRAND/products/$PRODUCT/voice.md" ] && PRODUCT_VOICE="$BRAND/products/$PRODUCT/voice.md"

# ── tokens drift detection ──
# tokens.css against its canonical sources, under the brand's own token
# prefix (brand_prefix in lib/common.sh): four colours against colors.md's CSS
# block, the heading and body faces against typography.md's Font Families
# table. Without one prefix in colors.md this comparison cannot be made, so it
# is skipped with a warning that says why; no prefix is guessed, and the rest
# of the brand is still judged.
TOKENS_CSS="$BRAND/tokens.css"
if [ -f "$TOKENS_CSS" ]; then
  if ! TP=$(brand_prefix "$BRAND/colors.md"); then
    echo "WARNING [tokens] token drift not checked: $TP" >&2
  else
    # The value of the first declaration of a custom property in a file.
    declared() { awk -v name="$1:" '{ i=index($0, name); if(i){ v=substr($0, i+length(name)); sub(/;.*/, "", v); gsub(/^[ \t]+|[ \t]+$/, "", v); print v; exit } }' "$2" 2>/dev/null; }
    # The first family of a font stack, unquoted.
    first_face() { printf '%s\n' "$1" | awk '{ sub(/,.*/, ""); gsub(/^[ \t\047\042]+|[ \t\047\042]+$/, ""); print }'; }
    # The family typography.md's Font Families table gives a role.
    role_face() { awk -F'|' -v role="$1" '/^## /{ f=($0 ~ /^## Font Families/); next } f && NF>=4 && $2 !~ /Role|---/ { r=$2; v=$3; gsub(/^ +| +$/, "", v); if(tolower(r) ~ role && v!=""){ print v; exit } }' "$BRAND/typography.md" 2>/dev/null; }

    for role in primary accent text bg; do
      CANON=$(declared "$TP$role" "$BRAND/colors.md"); TOKEN=$(declared "$TP$role" "$TOKENS_CSS")
      if [ -n "$CANON" ] && [ -n "$TOKEN" ] && [ "$CANON" != "$TOKEN" ]; then
        echo "WARNING [tokens] tokens.css $TP$role: $TOKEN does not match colors.md: $CANON" >&2
      fi
    done
    for role in heading body; do
      CANON=$(role_face "$role"); TOKEN=$(first_face "$(declared "${TP}font-$role" "$TOKENS_CSS")")
      if [ -n "$CANON" ] && [ -n "$TOKEN" ] && [ "$CANON" != "$TOKEN" ]; then
        echo "WARNING [tokens] tokens.css ${TP}font-$role: $TOKEN does not match typography.md: $CANON" >&2
      fi
    done
  fi
fi

# Product accents from the umbrella Product Accents table ("HEX|name" rows).
# In a product context, other products accents are markers, not UI colors.
# Product accents: "HEX|product" rows from colors.md's Product Accents table.
# The product column is found by its header (Product, Tool or Name) and the
# accent is the row's first hex, so the table may grow columns (v1.9.0 has
# Row | Material | Tool | Bloom | ...) without turning these checks off.
ACCENT_ROWS=$(awk -F'|' '
  /^## Product Accents/ { f=1; next }
  /^## / { f=0 }
  f && /^\|/ && !hdr { for(c=2;c<NF;c++){ h=tolower($c); gsub(/^ +| +$/, "", h); if(h ~ /^(product|tool|name)$/) pc=c }; hdr=1; next }
  f && /^\|/ && pc && $0 !~ /^\|[ -]+\|/ {
    hex=""; for(c=2;c<NF;c++) if(match($c, /#[0-9a-fA-F]{6}/)){ hex=toupper(substr($c, RSTART, RLENGTH)); break }
    nm=$pc; gsub(/^ +| +$/, "", nm)
    if(hex!="" && nm!="") print hex "|" nm
  }' "$BRAND/colors.md" 2>/dev/null)
ACCENTS=$(printf '%s\n' "$ACCENT_ROWS" | grep -v '^$' | tr '\n' '\036')
PRODUCT_NAMES=$(printf '%s\n' "$ACCENT_ROWS" | awk -F'|' 'NF>=2 {print tolower($2)}' | tr '\n' '\034')

PALETTE=$(grep -ohE '#[0-9a-fA-F]{6}' "$BRAND/colors.md" 2>/dev/null | tr 'a-f' 'A-F' | sort -u | tr '\n' ' ' | sed 's/ $//')
RADII=$(grep -ohE '^\| radius-[a-z]+ \| [0-9]+px' "$BRAND/components.md" 2>/dev/null | grep -oE '[0-9]+px' | tr '\n' ' ' | sed 's/ $//')
BREAKPOINTS=$(grep -ohE '^\| [0-9]+px' "$BRAND/layout.md" 2>/dev/null | grep -oE '[0-9]+' | tr '\n' ' ' | sed 's/ $//')
SPACE_UNIT=4

# Documented contrast pairs from colors.md's Contrast Pairings table:
# "HEX1|HEX2|class" rows, \036-joined. A documented class overrides the
# generic thresholds (the table is the brand's own classification).
DOCPAIRS=$(awk '/^## Contrast Pairings/{f=1;next} /^## /{f=0} f && /^\|.* on .*\|/ {
  n=0; delete hx
  line=$0
  while(match(line, /#[0-9a-fA-F]{6}/)){ hx[++n]=toupper(substr(line,RSTART,RLENGTH)); line=substr(line,RSTART+RLENGTH) }
  if(n>=2){
    nf=split($0, cols, "|")
    cls=cols[nf-1]; gsub(/^ +| +$/, "", cls)
    print hx[1] "|" hx[2] "|" cls
  }
}' "$BRAND/colors.md" 2>/dev/null | tr '\n' '\036')

# Forbidden terms: "term<TAB>alternatives" rows from voice.md's Forbidden Terms
# table, newline -> \036 (awk -v cannot carry newlines portably)
FORBIDDEN=$(awk -F'|' '/^## Forbidden Terms/{f=1} /^## /{if(f&&$0!~/Forbidden/)f=0} f && NF>=4 && $2!~/Term|---/ {gsub(/^ +| +$/,"",$2); gsub(/^ +| +$/,"",$4); if($2!="") print $2"\t"$4}' "$BRAND/voice.md" $PRODUCT_VOICE 2>/dev/null | tr '\n' '\036')
# Terminology: "canonical<TAB>banned_synonym" from voice.md Terminology table
TERMS=$(awk -F'|' '/^## Terminology/{f=1} /^## /{if(f&&$0!~/Terminology/)f=0} f && NF>=4 && $2!~/Use|---/ {gsub(/^ +| +$/,"",$2); gsub(/^ +| +$/,"",$3); if($3!=""){ n=split($3, syns, ","); for(i=1;i<=n;i++){ gsub(/^ +| +$/,"",syns[i]); gsub(/"/,"",syns[i]); if(syns[i]!="") print $2"\t"syns[i] }}}' "$BRAND/voice.md" $PRODUCT_VOICE 2>/dev/null | tr '\n' '\036')

# Stroke widths: whole-pixel values from components.md's Border Strokes table.
# A spacing value equal to one is a hairline seam, not off-scale.
STROKES=$(awk -F'|' '/^## Border Strokes/{f=1;next} /^## /{f=0} f && NF>=4 && $2!~/Token|---/ { if(match($3, /[0-9.]+px/)){ v=substr($3, RSTART, RLENGTH-2); if(v ~ /^[0-9]+$/) printf "%s ", v } }' "$BRAND/components.md" 2>/dev/null)

# Every stroke width (decimals too), for the surface audit of borders and
# outlines.
STROKEW=$(awk -F'|' '/^## Border Strokes/{f=1;next} /^## /{f=0} f && NF>=4 && $2!~/Token|---/ { if(match($3, /[0-9.]+px/)) printf "%s ", substr($3, RSTART, RLENGTH-2) }' "$BRAND/components.md" 2>/dev/null)
# Shadow recipes: "token|value|night" rows of components.md's Shadow Recipes
# tables, newline -> \036, the columns found by their headers (Token or
# Name, Value, Night) as export-tokens.sh finds them. The night value, from
# the optional Night column, is read only for a brand with a dark theme
# (brand_has_dark in lib/common.sh); otherwise it stays empty, and no night
# rule applies. A night cell in prose is skipped, as export-tokens.sh skips
# it (as_shadow there; the night-shadows spec compares the two).
SHADOW_NIGHT=0
if brand_has_dark "$BRAND/colors.md"; then SHADOW_NIGHT=1; fi
SHADOWS=$(awk -F'|' -v night="$SHADOW_NIGHT" '
  function cell(c,   v){ v=(c ? $c : ""); gsub(/`/, "", v); gsub(/^[ \t]+|[ \t]+$/, "", v); return v }
  function isshadow(v,   r){ r=v; gsub(/rgba?\([ 0-9.,%\/]+\)/, "", r); gsub(/#[0-9A-Fa-f]+/, "", r); gsub(/inset|px|rem|em/, "", r); return r ~ /^[0-9., -]+$/ && r ~ /[0-9]/ }
  /^## Shadow Recipes/{ f=1; hdr=0; next }
  /^## /{ f=0 }
  f && !/^[ \t]*\|/{ hdr=0; next }
  f && !hdr { tc=0; vc=0; nc=0; for(c=2;c<NF;c++){ h=tolower(cell(c)); if(h=="token" || h=="name") tc=c; else if(h=="value") vc=c; else if(h=="night") nc=c }; hdr=1; next }
  f && vc && $0 !~ /^[ \t]*\|[ \t:|-]+$/ { v=cell(vc); if(v!="") nv=(night ? cell(nc) : ""); print cell(tc) "|" v "|" (isshadow(nv) ? nv : "") }' "$BRAND/components.md" 2>/dev/null | tr '\n' '\036')
# Shadow tints: the colours components.md's Shadow Recipes section names, as
# hex, the Night column's among them. A shadow may use one even when
# colors.md does not hold it (a night tint, say); the section writes them as
# #RRGGBB or rgb triples with the alpha left open, rgba(R, G, B, ...).
SHADOWTINTS=$(awk '/^## Shadow Recipes/{f=1;next} /^## /{f=0} f {
  s=$0
  while(match(s, /#[0-9a-fA-F]{6}|rgba?\([ ]*[0-9]+[ ]*,[ ]*[0-9]+[ ]*,[ ]*[0-9]+/)){
    t=substr(s, RSTART, RLENGTH); s=substr(s, RSTART+RLENGTH)
    if(t ~ /^#/){ print toupper(t); continue }
    sub(/^[^(]*\(/, "", t); split(t, ch, /[ ]*,[ ]*/)
    if(ch[1]<=255 && ch[2]<=255 && ch[3]<=255) printf "#%02X%02X%02X\n", ch[1], ch[2], ch[3]
  }
}' "$BRAND/components.md" 2>/dev/null | sort -u | tr '\n' ' ')

# Em dashes: judged only when voice.md bans them
EMBAN=0
grep -qiE 'no em dash|em dashes? (are|is) (banned|forbidden)|never (use )?(an )?em dash' "$BRAND/voice.md" $PRODUCT_VOICE 2>/dev/null && EMBAN=1

# Markup files hold colours only in style/script blocks and attribute values;
# everywhere else a "#abc" is an anchor or a word
case "$TARGET" in *.html|*.htm|*.svg|*.xhtml) MARKUP=1 ;; *) MARKUP=0 ;; esac

# Font families: the "Font Family" column of typography.md's Font Families
# table, deduplicated, newline -> \036
PRODUCT_TYPO=""
[ -n "$PRODUCT" ] && [ -f "$BRAND/products/$PRODUCT/typography.md" ] && PRODUCT_TYPO="$BRAND/products/$PRODUCT/typography.md"
FONTS=$(awk -F'|' '/^## Font Families/{f=1;next} /^## /{f=0} f && NF>=4 && $2!~/Role|---/ {gsub(/^ +| +$/,"",$3); if($3!="" && !seen[$3]++) print $3}' "$BRAND/typography.md" $PRODUCT_TYPO 2>/dev/null | tr '\n' '\036')

# Faces: "role|family|weights|style" rows from typography.md's Font Families
# table, newline -> \036. Used to catch faux bold and italic.
FACES=$(awk -F'|' '/^## Font Families/{f=1;next} /^## /{f=0} f && NF>=6 && $2!~/Role|---/ {for(i=2;i<=5;i++) gsub(/^ +| +$/,"",$i); if($3!="") print $2"|"$3"|"$4"|"$5}' "$BRAND/typography.md" $PRODUCT_TYPO 2>/dev/null | tr '\n' '\036')
case "$TARGET" in *.css|*.scss|*.less) CSSFILE=1 ;; *) CSSFILE=0 ;; esac
case "$TARGET" in *.md|*.markdown) MDFILE=1 ;; *) MDFILE=0 ;; esac

# Garden row: the beds, in order, are the reference row in
# components.md. A page's .gf-garden must list exactly those, each pill with
# its own bloom, and a product page (one with the plotplot band, .gf-plot)
# marks exactly one pill current. Findings are "line|message", \036-joined.
GARDEN_BEDS=$(awk '/class="gf-garden"/{f=1} f && match($0, /gf-dot"><\/span>[a-z0-9-]+/){ print substr($0, RSTART+15, RLENGTH-15) } f && /<\/nav>/{exit}' "$BRAND/components.md" 2>/dev/null | tr '\n' ' ' | sed 's/ $//')
GARDEN=""
if [ -n "$GARDEN_BEDS" ] && grep -q 'class="gf-garden"' "$TARGET" 2>/dev/null; then
  GARDEN=$(awk -v beds="$GARDEN_BEDS" '
    /class="gf-plot"/ { product=1 }
    /class="gf-garden"/ { inrow=1; rowline=NR }
    inrow && match($0, /gf-dot"><\/span>[a-z0-9-]+/) {
      nm=substr($0, RSTART+15, RLENGTH-15); n++; name[n]=nm; at[n]=NR; seen[nm]=1
      if(match($0, /--bloom:[ ]*var\(--pp-[a-z0-9-]+\)/)){ b=substr($0, RSTART, RLENGTH); sub(/.*--pp-/, "", b); sub(/\)$/, "", b); if(b!=nm) print NR "|the garden row gives \"" nm "\" the bloom of \"" b "\". Each pill carries its own --pp-" nm "." }
      if($0 ~ /is-current/) current++
    }
    inrow && /<\/nav>/ { inrow=0 }
    END{
      if(!rowline) exit
      nb=split(beds, bed, " ")
      for(i=1;i<=nb;i++) want[bed[i]]=1
      for(i=1;i<=n;i++) if(!(name[i] in want)) print at[i] "|the garden row lists \"" name[i] "\", which is not one of the brand\047s beds (components.md: " beds ")."
      missing=0
      for(i=1;i<=nb;i++) if(!(bed[i] in seen)){ missing++; print rowline "|the garden row is missing \"" bed[i] "\" (components.md lists the full garden)." }
      if(!missing && n==nb){ for(i=1;i<=n;i++) if(name[i]!=bed[i]){ print rowline "|the garden row is out of the brand\047s order (components.md: " beds ")."; break } }
      if(product && current+0!=1) print rowline "|a product page marks exactly one current pill in the garden row (found " current+0 ")."
    }' "$TARGET" 2>/dev/null | tr '\n' '\036')
fi

# Custom properties per theme: "L|--name|#HEX" for the default
# (light) theme and "D|--name|#HEX" for values set under a dark marker
# ([data-theme="dark"], .dark, prefers-color-scheme: dark). Braces are tracked
# character by character, since minified CSS puts several blocks on a line.
THEMEVARS=$(awk '
  { text = text $0 "\n" }
  END{
    gsub(/\/\*([^*]|\*+[^*\/])*\*+\//, "", text)
    # split once on the delimiters, keeping each as its own token
    gsub(/[{};]/, "\034&\034", text)
    n=split(text, tok, "\034")
    depth=0; acc=""
    for(i=1;i<=n;i++){
      c=tok[i]
      if(c=="{"){ depth++; sel[depth]=acc; acc="" }
      else if(c=="}"){ decls(acc, depth); acc=""; if(depth>0) depth-- }
      else if(c==";"){ decls(acc, depth); acc="" }
      else acc=acc c
    }
  }
  function decls(a, d,   j, dark, nm, v){
    if(!match(a, /--[A-Za-z0-9-]+[ \t\n]*:[ \t\n]*(#[0-9a-fA-F]+|[A-Za-z]+\([^()]*\))/)) return
    a=substr(a, RSTART, RLENGTH); nm=a; sub(/[ \t\n]*:.*/, "", nm); v=a; sub(/^[^:]*:[ \t\n]*/, "", v)
    if(v !~ /^#/ && tolower(v) !~ /^(rgba?|hsla?|oklch|oklab)\(/) return
    gsub(/[ \t\n]+/, " ", v)
    dark=0
    for(j=1;j<=d;j++) if(sel[j] ~ /data-theme[ ]*=[ ]*["\047]?(dark|night)|\.dark([^a-z-]|$)|prefers-color-scheme[ ]*:[ ]*dark/) dark=1
    print (dark ? "D" : "L") "|" nm "|" v
  }' "$TARGET" 2>/dev/null | tr '\n' '\036')

# The file as SARIF names it: relative to the project root when inside it.
case "$TARGET" in "$PWD"/*) URI="${TARGET#"$PWD"/}" ;; *) URI="$TARGET" ;; esac

# Component contracts (reuse): "contract|home|class|covers" per Home table row
# of .brand/components/**/*.md. A home path is relative to the project root,
# the folder that holds the brand folder; rows whose home is this file are
# dropped, since a raw element in its own home is the component itself.
PROJ_ROOT=$(cd "$BRAND/.." && pwd -P)
SELF_ABS="$(cd "$(dirname "$TARGET")" && pwd -P)/$(basename "$TARGET")"
SELF_REL="${SELF_ABS#"$PROJ_ROOT"/}"
CONTRACTS=""
HAS_CONTRACTS=0
if [ -d "$BRAND/components" ]; then
  while IFS= read -r cf; do
    rel="${cf#"$BRAND"/}"
    rows=$(awk -F'|' -v c="$rel" -v self="$SELF_REL" '
      /^## /{h=($0 ~ /^## Home/); next}
      h && NF>=5 && $2 !~ /File|---/ {
        for(i=2;i<=4;i++){ gsub(/^[ \t`]+|[ \t`]+$/, "", $i) }
        if($2!="" && $4!="" && $2!=self) print c "|" $2 "|" $3 "|" $4
        if($2!="" && $4!="") any=1
      }
      END{ if(any) print "#has" }' "$cf")
    case "$rows" in *"#has"*) HAS_CONTRACTS=1 ;; esac
    rows=$(printf '%s\n' "$rows" | grep -v '^#has$' | tr '\n' '\036')
    CONTRACTS="$CONTRACTS$rows"
  done < <(find "$BRAND/components" -name '*.md' -type f | sort)
fi

# Stage: --stage wins; else a mockup glob in .petalsrc's
# stages: mockup: list that matches the file; else production.
case "$STAGE" in ""|mockup|production) ;; *) echo "ERROR --stage must be mockup or production"; exit 2 ;; esac
if [ -z "$STAGE" ]; then
  STAGE=production
  if [ -f .petalsrc ]; then
    while IFS= read -r glob; do
      # shellcheck disable=SC2053  # the glob is meant to match as a pattern
      [ -n "$glob" ] && [[ "$URI" == $glob ]] && { STAGE=mockup; break; }
    done < <(awk '/^stages:/{s=1;next} s && /^[^ ]/{s=0} s && /^[ ]+mockup:/{m=1;next} s && m && /^[ ]+[a-z]+:/{m=0} s && m && /^[ ]+- /{sub(/^[ ]+- /,""); gsub(/["\047]/,""); print}' .petalsrc)
  fi
fi

awk -v palette="$PALETTE" -v radii="$RADII" -v bps="$BREAKPOINTS" \
    -v unit="$SPACE_UNIT" -v forbidden="$FORBIDDEN" -v docpairs="$DOCPAIRS" -v accents="$ACCENTS" -v product="$PRODUCT" -v terms="$TERMS" -v pnames="$PRODUCT_NAMES" -v sarif="$([ "$FORMAT" = sarif ] && echo 1 || echo 0)" -v uri="$URI" -v fonts="$FONTS" -v markup="$MARKUP" -v md="$MDFILE" -v strokes="$STROKES" -v emban="$EMBAN" -v faces="$FACES" -v cssfile="$CSSFILE" -v garden="$GARDEN" -v stage="$STAGE" -v contracts="$CONTRACTS" -v hascontracts="$HAS_CONTRACTS" -v themevars="$THEMEVARS" -v strokew="$STROKEW" -v shadows="$SHADOWS" -v shadowtints="$SHADOWTINTS" '
function hexval(c,   i){ return index("0123456789ABCDEF", toupper(c)) - 1 }
function h2(s){ return hexval(substr(s,1,1))*16 + hexval(substr(s,2,1)) }
function expand(h){ if(length(h)==4) return "#" substr(h,2,1) substr(h,2,1) substr(h,3,1) substr(h,3,1) substr(h,4,1) substr(h,4,1); return h }
function rgb(h, a){ h=expand(toupper(h)); a[1]=h2(substr(h,2,2)); a[2]=h2(substr(h,4,2)); a[3]=h2(substr(h,6,2)) }
function chan(c){ c=c/255; return c<=0.03928 ? c/12.92 : ((c+0.055)/1.055)^2.4 }
function lum(h,   a){ rgb(h,a); return 0.2126*chan(a[1]) + 0.7152*chan(a[2]) + 0.0722*chan(a[3]) }
function ratio(f,b,   lf,lb,hi,lo){ lf=lum(f); lb=lum(b); hi=(lf>lb?lf:lb); lo=(lf>lb?lb:lf); return (hi+0.05)/(lo+0.05) }
function nearest(h,   a,b,i,d,best,bd){ rgb(h,a); bd=1e9; for(i=1;i<=np;i++){ rgb(pal[i],b); d=(a[1]-b[1])^2+(a[2]-b[2])^2+(a[3]-b[3])^2; if(d<bd){bd=d;best=pal[i]} } return best }
function nearname(h,   n){ n=nearest(h); return (n in accent_owner) ? n " (" accent_owner[n] "\047s product accent)" : n }

# ── colour notations: oklch(), oklab(), rgb()/rgba(), hsl()/hsla() ──
# cfn reads one colour function, written whole as "name(...)", and returns
# "#RRGGBB"; "!" when the colour lies outside sRGB, never moved into it; or ""
# when it is not a literal (var(), calc(), relative syntax, a bad argument).
# calpha keeps the alpha it read, 1 when none is written. One converter per
# notation. The OKLab and gamma arithmetic is new-brand.py\047s, and
# tests/colour-notations.spec.sh compares the two on a sweep.
function cfn(s,   nm, body){
  nm=tolower(s); sub(/\(.*/, "", nm)
  body=s; sub(/^[^(]*\(/, "", body); sub(/\)$/, "", body)
  cbad=0; calpha=1
  if(nm=="rgb" || nm=="rgba") return fromrgb(body)
  if(nm=="hsl" || nm=="hsla") return fromhsl(body)
  if(nm=="oklab") return fromoklab(body)
  if(nm=="oklch") return fromoklch(body)
  return ""
}
# Splits a function body into its n channels, cp[1..n], and reads calpha.
# legacy allows the comma syntax rgb() and hsl() still have. 0 when malformed.
function cparts(body, n, legacy,   m, k, al){
  al=""; gsub(/^[ \t\n]+|[ \t\n]+$/, "", body)
  if(index(body, ",")){
    if(!legacy || index(body, "/")) return 0
    m=split(body, cp, /[ \t\n]*,[ \t\n]*/)
    if(m==n+1){ al=cp[m]; m=n }
  } else {
    k=index(body, "/")
    if(k){ al=substr(body, k+1); body=substr(body, 1, k-1); gsub(/^[ \t\n]+|[ \t\n]+$/, "", al); sub(/[ \t\n]+$/, "", body) }
    m=split(body, cp, /[ \t\n]+/)
  }
  if(m!=n) return 0
  if(al!=""){ calpha=cnum(al, 1); if(calpha<0) calpha=0; if(calpha>1) calpha=1 }
  return !cbad
}
# A number or a percentage; full is what 100% stands for. none is 0.
function cnum(tok, full){
  if(tok=="none") return 0
  if(tok ~ /^[+-]?([0-9]+\.?[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?%$/) return substr(tok, 1, length(tok)-1) / 100 * full
  if(tok ~ /^[+-]?([0-9]+\.?[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?$/) return tok + 0
  cbad=1; return 0
}
# A hue in degrees, from a bare number or deg, rad, grad or turn.
function cangle(tok,   u, v){
  if(tok=="none") return 0
  u="deg"; v=tok
  if(match(tok, /(deg|rad|grad|turn)$/)){ u=substr(tok, RSTART); v=substr(tok, 1, RSTART-1) }
  if(v !~ /^[+-]?([0-9]+\.?[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?$/){ cbad=1; return 0 }
  v+=0
  if(u=="rad") return v * (180 / atan2(0, -1))
  if(u=="grad") return v * 0.9
  if(u=="turn") return v * 360
  return v
}
# A gamma-encoded channel, 0 to 1, as a byte (new-brand.py rounds the same way).
function cbyte(c){ c=int(c * 255 + 0.5); return (c > 255) ? 255 : (c < 0 ? 0 : c) }
function fromrgb(body,   i, v, out){
  if(!cparts(body, 3, 1)) return ""
  out=0
  for(i=1;i<=3;i++){ v=cnum(cp[i], 255); if(v<0 || v>255) out=1; cc[i]=int(v + 0.5) }
  if(cbad) return ""
  if(out) return "!"
  return sprintf("#%02X%02X%02X", cc[1], cc[2], cc[3])
}
# CSS Color 4\047s hsl-to-rgb. It works in percent, where whole-number inputs
# stay exact, so a channel that lands on a half rounds up as browsers do
# (hsl(0 80% 50%) is rgb(230, 26, 26)); the spec checks it against exact
# rational arithmetic.
function fromhsl(body,   h, s, l, a, i, k, v){
  if(!cparts(body, 3, 1)) return ""
  h=cangle(cp[1]); s=cnum(cp[2], 100); l=cnum(cp[3], 100)
  if(cbad) return ""
  if(s<0 || s>100 || l<0 || l>100) return "!"
  h=h % 360; if(h<0) h+=360
  a=s * (l < 100-l ? l : 100-l) / 100
  split("0 8 4", hn, " ")
  for(i=1;i<=3;i++){
    k=(hn[i] + h/30) % 12
    v=k-3; if(9-k < v) v=9-k; if(v>1) v=1; if(v<-1) v=-1
    cc[i]=int((l - a*v) * 255 / 100 + 0.5)
  }
  return sprintf("#%02X%02X%02X", cc[1], cc[2], cc[3])
}
# OKLab to "#RRGGBB" through linear sRGB: new-brand.py\047s oklab_to_linear,
# in_gamut and to_gamma. A colour outside sRGB is "!".
function oklabhex(L, a, b,   l_, m_, s_, l, m, s, i){
  l_ = L + 0.3963377774 * a + 0.2158037573 * b
  m_ = L - 0.1055613458 * a - 0.0638541728 * b
  s_ = L - 0.0894841775 * a - 1.2914855480 * b
  l = l_*l_*l_; m = m_*m_*m_; s = s_*s_*s_
  cc[1] = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s
  cc[2] = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s
  cc[3] = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
  for(i=1;i<=3;i++){
    if(cc[i] < -1e-7 || cc[i] > 1 + 1e-7) return "!"
    cc[i] = (cc[i] < 0) ? 0 : (cc[i] > 1 ? 1 : cc[i])
    cc[i] = (cc[i] <= 0.0031308) ? 12.92 * cc[i] : 1.055 * cc[i] ^ (1 / 2.4) - 0.055
  }
  return sprintf("#%02X%02X%02X", cbyte(cc[1]), cbyte(cc[2]), cbyte(cc[3]))
}
function fromoklab(body,   L, a, b){
  if(!cparts(body, 3, 0)) return ""
  L=cnum(cp[1], 1); a=cnum(cp[2], 0.4); b=cnum(cp[3], 0.4)
  if(cbad) return ""
  L=(L < 0) ? 0 : (L > 1 ? 1 : L)
  return oklabhex(L, a, b)
}
function fromoklch(body,   L, C, h, rad){
  if(!cparts(body, 3, 0)) return ""
  L=cnum(cp[1], 1); C=cnum(cp[2], 0.4); h=cangle(cp[3])
  if(cbad) return ""
  L=(L < 0) ? 0 : (L > 1 ? 1 : L); if(C < 0) C=0
  rad=h * (atan2(0, -1) / 180)
  return oklabhex(L, C * cos(rad), C * sin(rad))
}
# A colour value as the opaque hex contrast needs, else "": a translucent
# colour takes its look from whatever lies behind it, which the check cannot see.
function opaque(v,   hx){
  if(!match(v, /^[A-Za-z]+\([^()]*\)/)) return ""
  hx=cfn(substr(v, 1, RLENGTH))
  return (hx ~ /^#/ && calpha >= 1) ? hx : ""
}
# One colour literal against the palette. shown is the text as written; hx
# is its hex, "!" outside sRGB, "" when it is not a literal. pre is the line
# up to the literal: in a shadow declaration a Shadow Recipes tint is on-brand.
function judgecolour(shown, hx, pre,   found, i){
  if(hx=="") return
  if(hx=="!"){ err("color", "line " NR ": " shown " is outside sRGB, so it is not in the palette (every palette colour is an sRGB hex). Use a palette colour."); return }
  found=0
  for(i=1;i<=np;i++) if(pal[i]==hx){ found=1; break }
  if(!found && (hx in tint_ok)){ sub(/.*[;{\042]/, "", pre); if(tolower(pre) ~ /shadow[a-z0-9-]*[ \t]*:/) found=1 }
  if(!found) err("color", "line " NR ": " shown (shown ~ /^#/ ? "" : " (" hx ")") " is not in the palette. Did you mean " nearname(hx) "?")
  else if(product!="" && (hx in accent_owner) && accent_owner[hx]!=product)
    warn("color", "line " NR ": " hx " is " accent_owner[hx] "\047s product accent. Product accents mark products, not " product " UI (products/" product "/colors.md).")
}
function err(dim,msg){ if(sarif) keep("error", dim, msg); else printf "ERROR [%s] %s\n", dim, msg; e[dim]++ }
function warn(dim,msg){ if(sarif) keep("warning", dim, msg); else printf "WARNING [%s] %s\n", dim, msg; w[dim]++ }
function note(dim,msg){ if(sarif) keep("note", dim, msg); else printf "NOTE [%s] %s\n", dim, msg; notecount[dim]++ }
function content(msg){ if(stage=="mockup") note("content", msg); else err("content", msg) }
function keep(level, dim, msg){ nres++; rlevel[nres]=level; rdim[nres]=dim; rmsg[nres]=msg }

# ── SARIF: one rule id per kind of finding, named <dimension>/<rule>.
# The table below is the one place a finding gets its id and description.
function ruleid(dim, msg){
  if(dim=="color"){ if(msg ~ /not in the palette/) return "color/off-palette"; if(msg ~ /product accent/) return "color/product-accent"; return "color/contrast" }
  if(dim=="layout"){ if(msg ~ /breakpoint/) return "layout/breakpoint"; return "layout/spacing" }
  if(dim=="surface"){ if(msg ~ /radius/) return "surface/radius"; if(msg ~ /stroke tokens/) return "surface/stroke"; if(msg ~ /shadow/) return "surface/shadow"; if(msg ~ /garden row|current pill/) return "surface/garden-row"; return "surface/motion" }
  if(dim=="voice"){ if(msg ~ /forbidden term/) return "voice/forbidden-term"; if(msg ~ /canonical term/) return "voice/terminology"; if(msg ~ /em dash/) return "voice/em-dash"; return "voice/casing" }
  if(dim=="content"){ if(msg ~ /placeholder/) return "content/placeholder"; if(msg ~ /href=/) return "content/dead-link"; return "content/dummy-contact" }
  if(dim=="reuse") return "reuse/rebuilt-component"
  if(dim=="access"){ if(msg ~ /image without alt/) return "access/img-alt"; if(msg ~ /form field/) return "access/unlabelled-field"; if(msg ~ /button with no name/) return "access/unnamed-button"; if(msg ~ /focus outline/) return "access/focus-removed"; if(msg ~ /zooming/) return "access/zoom-blocked"; return "access/fake-button" }
  if(dim=="typography"){ if(msg ~ /has no (weight|italic)/) return "typography/faux-style"; if(msg ~ /never loads/) return "typography/unloaded-family"; return "typography/family" }
  return dim "/" dim
}
function rules_init(){
  rdesc["color/off-palette"]="A colour that is not in the palette"
  rdesc["color/contrast"]="A text and background pair below its contrast minimum"
  rdesc["color/product-accent"]="Another product\047s accent in this product\047s UI"
  rdesc["layout/spacing"]="Spacing off the scale"
  rdesc["layout/breakpoint"]="A breakpoint the brand does not document"
  rdesc["surface/radius"]="A radius off the radius scale"
  rdesc["surface/stroke"]="A border, outline or ring width off the stroke tokens"
  rdesc["surface/shadow"]="A shadow that is not a documented recipe"
  rdesc["surface/motion"]="An easing that bounces or overshoots"
  rdesc["surface/garden-row"]="A footer garden row that differs from the brand\047s"
  rdesc["voice/forbidden-term"]="A term the brand forbids"
  rdesc["voice/terminology"]="A synonym where the brand has a canonical term"
  rdesc["voice/casing"]="A product name not in lowercase"
  rdesc["voice/em-dash"]="An em dash where the brand bans them"
  rdesc["typography/family"]="A font family outside typography.md"
  rdesc["typography/unloaded-family"]="A brand family the page never loads"
  rdesc["typography/faux-style"]="A weight or italic the face does not have"
  rdesc["content/placeholder"]="Placeholder copy in a production page"
  rdesc["content/dummy-contact"]="A dummy phone number, address or name in a production page"
  rdesc["content/dead-link"]="A link to # in a production page"
  rdesc["access/img-alt"]="An image without alt text"
  rdesc["access/unlabelled-field"]="A form field without a label"
  rdesc["access/unnamed-button"]="A button with no accessible name"
  rdesc["access/focus-removed"]="A focus outline removed with no visible replacement"
  rdesc["access/zoom-blocked"]="A viewport that blocks zooming"
  rdesc["access/fake-button"]="A clickable div or span keyboard users cannot reach"
  rdesc["reuse/rebuilt-component"]="A control rebuilt by hand where a component contract covers it"
}
function jesc(s){ gsub(/\\/, "\\\\", s); gsub(/"/, "\\\"", s); gsub(/\t/, "\\t", s); gsub(/\r/, "", s); gsub(/\n/, "\\n", s); return s }
function emit_sarif(   i, n, k, ln, text, sug, first){
  rules_init()
  printf "{\"$schema\":\"https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json\",\"version\":\"2.1.0\",\"runs\":[{\"tool\":{\"driver\":{\"name\":\"petals\",\"informationUri\":\"https://github.com/jahala/petals\",\"rules\":["
  n=0; for(k in rdesc) ord[++n]=k
  for(i=1;i<=n;i++) for(k=i+1;k<=n;k++) if(ord[k]<ord[i]){ t=ord[i]; ord[i]=ord[k]; ord[k]=t }
  for(i=1;i<=n;i++) printf "%s{\"id\":\"%s\",\"shortDescription\":{\"text\":\"%s\"}}", (i>1?",":""), ord[i], jesc(rdesc[ord[i]])
  printf "]}},\"results\":["
  for(i=1;i<=nres;i++){
    text=rmsg[i]; ln=0
    if(match(text, /^line [0-9]+: /)){ ln=substr(text, 6, RLENGTH-7)+0; text=substr(text, RLENGTH+1) }
    printf "%s{\"ruleId\":\"%s\",\"level\":\"%s\",\"message\":{\"text\":\"%s\"}", (i>1?",":""), ruleid(rdim[i], rmsg[i]), rlevel[i], jesc(text)
    printf ",\"locations\":[{\"physicalLocation\":{\"artifactLocation\":{\"uri\":\"%s\"}%s}}]", jesc(uri), (ln>0 ? ",\"region\":{\"startLine\":" ln "}" : "")
    sug=""
    if(match(text, /Did you mean [^?]*\?/)){ sug=substr(text, RSTART+13, RLENGTH-14) }
    if(sug!="") printf ",\"properties\":{\"suggestion\":\"%s\"}", jesc(sug)
    printf "}"
  }
  printf "]}]}\n"
}
function quotectx(s){ return (s ~ /never|use instead|forbidden|class="no"|class="ba bad"/) }

# ── typography: weights and styles a face does not have ──
# Rules are read from stylesheets and <style> blocks with comments stripped.
# A rule is judged when it names its face (a family, or a var() whose name
# says its role); findings wait for END, since font-synthesis: none anywhere
# on the page stops the browser faking them.
function hasweight(f, w,   ws, n, i, r){
  n=split(famweights[f], ws, ",")
  for(i=1;i<=n;i++){
    gsub(/^ +| +$/, "", ws[i])
    if(ws[i]==w) return 1
    if(match(ws[i], /^[0-9]+ *(\.\.|-) *[0-9]+$/)){ split(ws[i], r, / *(\.\.|-) */); if(w+0>=r[1]+0 && w+0<=r[2]+0) return 1 }
  }
  return 0
}
function faceof(body,   v, parts, first, lf){
  if(!match(body, /font-family[ \t]*:[^;]*/)) return ""
  v=substr(body, RSTART, RLENGTH); sub(/^[^:]*:[ \t]*/, "", v)
  split(v, parts, ","); first=parts[1]; gsub(/^[ \t"\047]+|[ \t"\047]+$/, "", first); lf=tolower(first)
  if(lf ~ /^var\(/){
    if(lf ~ /heading|display/) return roleface["heading"]
    if(lf ~ /code|mono/) return roleface["code"]
    if(lf ~ /interface|-ui/) return roleface["interface"]
    if(lf ~ /body|text|sans/) return roleface["body"]
    return ""
  }
  return lf
}
function judgerule(sel, body, sline,   f, w, st){
  if(body ~ /font-synthesis[ \t]*:[ \t]*none/) synthoff=1
  f=faceof(body)
  if(f=="" || !(f in famweights)) return
  w=""
  if(match(body, /font-weight[ \t]*:[ \t]*[0-9a-z]+/)){ w=substr(body, RSTART, RLENGTH); sub(/^[^:]*:[ \t]*/, "", w); if(w=="bold") w="700"; else if(w=="normal") w="400"; else if(w !~ /^[0-9]+$/) w="" }
  else if(sel ~ /(^|[ ,>+~])(h[1-6]|b|strong|th)([ ,.:#\[>+~]|$)/) w="700"
  if(w!="" && !hasweight(f, w)) faux[++nfaux]="line " sline ": \"" famname[f] "\" has no weight " w " (typography.md: " famweights[f] "), so the browser fakes it. Ask for a weight it has, or set font-synthesis: none."
  st=""
  if(match(body, /font-style[ \t]*:[ \t]*(italic|oblique)/)) st="italic"
  if(st!="" && tolower(famstyles[f]) !~ /italic/) faux[++nfaux]="line " sline ": \"" famname[f] "\" has no italic (typography.md: " famstyles[f] "), so the browser fakes it. Drop the italic, or set font-synthesis: none."
}
# ── markdown: fenced blocks and inline code are quoted output, as <pre> and
# <code> are in a page. Blank them for every rule; line numbers stay. ──
md {
  if($0 ~ /^[ \t]*(```|~~~)/){ infence=!infence; $0="" }
  else if(infence) $0=""
  else gsub(/`[^`]*`/, "", $0)
}
{
  if(nfaces>0 && (cssfile || markup)){
    cl=$0
    if(markup){ if(cl ~ /<style[ >]/){ instyle=1; sub(/.*<style[^>]*>/, "", cl) } else if(!instyle) cl="" }
    if(markup && cl ~ /<\/style>/){ sub(/<\/style>.*/, "", cl); closestyle=1 } else closestyle=0
    # strip comments, carrying an open comment across lines
    out=""
    while(cl!=""){
      if(incss){ k=index(cl, "*/"); if(k==0){ cl="" } else { cl=substr(cl, k+2); incss=0 } }
      else { k=index(cl, "/*"); if(k==0){ out=out cl; cl="" } else { out=out substr(cl, 1, k-1); cl=substr(cl, k+2); incss=1 } }
    }
    for(ci=1; ci<=length(out); ci++){
      ch=substr(out, ci, 1)
      if(ch=="{"){ cursel=racc; gsub(/^[ \t]+|[ \t]+$/, "", cursel); racc=""; inrule=1; rstart=NR }
      else if(ch=="}"){ if(inrule) judgerule(cursel, racc, rstart); inrule=0; racc="" }
      else if(ch==";" && !inrule){ racc="" }
      else racc=racc ch
    }
    racc=racc " "
    if(closestyle) instyle=0
  }
}

# ── voice: em dashes, when voice.md bans them ──
# Copy is an error. A comment is a warning, since copied comments are how
# dashes spread. Quoted output in <pre> and inline <code> is skipped.
# ── copy and comments: one split per line, shared by the voice and content
# rules. Quoted output (<pre>, inline <code>) is neither, and a line in a
# quoted-example context ("never ...", "use instead") is skipped.
{
  el=$0; copytext=""; commenttext=""; iscopy=0
  if(markup && el ~ /<(style|script)[ >]/) incodeblock=1
  if(el ~ /<pre[ >]/) inpre=1
  if(!inpre && !quotectx(el)){
    iscopy=1
    gsub(/<code[^>]*>[^<]*<\/code>/, "", el)
    rest=el
    while(rest!=""){
      if(incom){
        endpos=(comtype=="css") ? index(rest, "*/") : index(rest, "-->")
        if(endpos==0){ commenttext=commenttext rest; rest="" }
        else { commenttext=commenttext substr(rest, 1, endpos-1); rest=substr(rest, endpos+(comtype=="css"?2:3)); incom=0 }
      } else {
        a=index(rest, "/*"); b=index(rest, "<!--")
        if(a==0 && b==0){ seg=rest; rest="" }
        else if(a>0 && (b==0 || a<b)){ seg=substr(rest, 1, a-1); rest=substr(rest, a+2); incom=1; comtype="css" }
        else { seg=substr(rest, 1, b-1); rest=substr(rest, b+4); incom=1; comtype="html" }
        if(!incodeblock) copytext=copytext seg
      }
    }
  }
  if($0 ~ /<\/pre>/) inpre=0
  if(markup && $0 ~ /<\/(style|script)>/) incodeblock=0
  # the words the voice rules read: copy and comments (comments get copied
  # into pages), never code; in markup, tags and their attributes are not words
  voicetext=copytext " " commenttext
  if(markup) gsub(/<[^>]*>/, " ", voicetext)
}

# ── voice: em dashes, when voice.md bans them ──
emban && iscopy {
  if(index(commenttext, "—")>0) warn("voice", "line " NR ": em dash in a comment. voice.md bans em dashes in copy, and copied comments are how they spread.")
  if(index(copytext, "—")>0 || copytext ~ /&mdash;/) err("voice", "line " NR ": em dash in copy. voice.md bans em dashes: use a comma, a full stop, or a colon.")
}

# ── content: what is not true yet. Errors in production, notes in
# a mockup, where believable filler is how a design gets judged.
iscopy {
  lc=tolower(copytext)
  if(lc ~ /lorem ipsum/ || lc ~ /your (text|name|company|title|email|copy|headline) here/ || lc ~ /\[(company|product|customer|client|first|last|your)[^]]*\]/)
    content("line " NR ": placeholder copy. Write the real words, or keep this page under a mockup path.")
  if(match(copytext, /(\(555\) ?|555[-. ])[0-9]{3,4}([-. ][0-9]{4})?/))
    content("line " NR ": " substr(copytext, RSTART, RLENGTH) " is a dummy phone number (555). Use a real one, or keep this page under a mockup path.")
  if(match(lc, /[a-z0-9._%+-]+@example\.(com|org|net)/))
    content("line " NR ": " substr(copytext, RSTART, RLENGTH) " is a dummy address (example.com). Use a real one, or keep this page under a mockup path.")
  if(lc ~ /(^|[^a-z])(john|jane) doe([^a-z]|$)/)
    content("line " NR ": John or Jane Doe is a dummy name. Use a real one, or keep this page under a mockup path.")
  if(copytext ~ /href[ ]*=[ ]*("#"|\047#\047)/)
    content("line " NR ": href=\"#\" goes nowhere. Link the real page, or keep this page under a mockup path.")
}

# ── reuse: a control rebuilt where a component contract covers it ──
# Each Covers selector is a tag, optionally with [attr=value]. A selector with a
# class (a.btn) names a use of the component, never a rebuild, so it is skipped.
function hasattr(tg, an, av,   re){
  re = "[ \t]" an "[ \t]*=[ \t]*[\042\047]?" av
  if(av=="") return (tg ~ ("[ \t]" an "([ \t=/>]|$)"))
  return (tg ~ (re "([\042\047 \t/>]|$)"))
}
function hasclass(tg, cl,   v){
  if(!match(tg, /[ \t](class|className)[ \t]*=[ \t]*\{?["\047][^"\047]*["\047]/)) return 0
  v=substr(tg, RSTART, RLENGTH); sub(/^[^"\047]*["\047]/, "", v); sub(/["\047]$/, "", v)
  return ((" " v " ") ~ ("[ \t]" cl "[ \t]"))
}
BEGIN{
  nsel=0
  nc=split(contracts, crow, "\036")
  for(i=1;i<=nc;i++){
    if(crow[i]=="") continue
    split(crow[i], cf, "|")
    ns=split(cf[4], sels, ",")
    for(j=1;j<=ns;j++){
      sl=sels[j]; gsub(/^[ \t]+|[ \t]+$/, "", sl)
      if(sl=="" || sl ~ /\./) continue
      tag=sl; an=""; av=""
      if(match(sl, /\[[^]]*\]/)){
        at=substr(sl, RSTART+1, RLENGTH-2); tag=substr(sl, 1, RSTART-1)
        k=index(at, "=")
        if(k){ an=substr(at, 1, k-1); av=substr(at, k+1); gsub(/["\047]/, "", av) } else an=at
      }
      nsel++; stag[nsel]=tag; sattr[nsel]=an; sval[nsel]=av; scls[nsel]=cf[3]; shome[nsel]=cf[2]; scon[nsel]=cf[1]
    }
  }
}
iscopy && nsel>0 {
  rest=copytext
  while(match(rest, /<[a-z][a-z0-9-]*([ \t\/][^>]*)?>/)){
    tg=substr(rest, RSTART, RLENGTH); rest=substr(rest, RSTART+RLENGTH)
    tname=tg; sub(/^</, "", tname); sub(/[ \t\/>].*$/, "", tname)
    for(k=1;k<=nsel;k++){
      if(stag[k]!=tname) continue
      if(sattr[k]!="" && !hasattr(tg, sattr[k], sval[k])) continue
      cl=scls[k]; sub(/^\./, "", cl)
      if(cl!="" && hasclass(tg, cl)) continue
      shown=tname; if(sattr[k]!="") shown=shown " " sattr[k] (sval[k]!="" ? "=" sval[k] : "")
      err("reuse", "line " NR ": a raw <" shown "> rebuilds a component. Use " scls[k] " from " shome[k] " (" scon[k] ").")
      break
    }
  }
}

# ── access: what the source of a page shows about who can use it ──
# Each rule locks a real person out when broken, so each is an error in every
# stage. Quoted markup (<pre>, <code>) is not judged; only copy is.
function attrval(tg, an,   v){
  if(!match(tg, "[ \t]" an "[ \t]*=[ \t]*(\042[^\042]*\042|\047[^\047]*\047|\\{[^}]*\\}|[^ \t>]+)")) return "\001"
  v=substr(tg, RSTART, RLENGTH); sub(/^[^=]*=[ \t]*/, "", v); gsub(/^["\047{]|["\047}]$/, "", v)
  return v
}
function named(tg){ return (attrval(tg, "aria-label")!="\001" && attrval(tg, "aria-label")!="") || attrval(tg, "aria-labelledby")!="\001" || attrval(tg, "title")!="\001" }
iscopy {
  rest=copytext
  while(1){
    if(match(rest, /<\/?[a-zA-Z][^>]*>/)){ txt=substr(rest, 1, RSTART-1); tg=substr(rest, RSTART, RLENGTH); rest=substr(rest, RSTART+RLENGTH) }
    else { txt=rest; tg="" }
    if(inbtn) btntext=btntext txt
    if(tg=="") break
    tn=tg; sub(/^<\/?/, "", tn); sub(/[ \t\/>].*$/, "", tn)
    closing=(tg ~ /^<\//)
    if(closing){
      if(tn=="label") inlabel=0
      if(tn=="button" && inbtn){
        t=btntext; gsub(/&nbsp;|&#160;|[ \t]/, "", t)
        if(!btnnamed && t=="") err("access", "line " btnline ": a button with no name. Screen readers say only \"button\"; add text, or aria-label.")
        inbtn=0
      }
      continue
    }
    if(tn=="img"){
      if(inbtn){ a=attrval(tg, "alt"); if(a!="\001" && a!="") btnnamed=1 }
      if(attrval(tg, "alt")=="\001") err("access", "line " NR ": an image without alt text. Screen readers read out the file name; describe what it shows, or set alt=\"\" for decoration.")
    }
    else if(tn=="label"){
      f=attrval(tg, "for"); if(f=="\001") f=attrval(tg, "htmlFor")
      if(f!="\001") labelfor[f]=1; else inlabel=1
    }
    else if(tn=="input" || tn=="select" || tn=="textarea"){
      ty=tolower(attrval(tg, "type"))
      if(tn=="input" && ty ~ /^(hidden|submit|button|reset|image)$/) continue
      if(inlabel || named(tg)) continue
      fid=attrval(tg, "id")
      if(fid!="\001"){ nfield++; fieldid[nfield]=fid; fieldline[nfield]=NR }
      else err("access", "line " NR ": a form field without a label. A placeholder vanishes as people type; add a <label for>, or aria-label.")
    }
    else if(tn=="button"){
      inbtn=1; btnline=NR; btntext=""; btnnamed=named(tg)
      if(tg ~ /\/>$/) inbtn=0
    }
    else if(tn=="div" || tn=="span"){
      if((attrval(tg, "onclick")!="\001" || attrval(tg, "onClick")!="\001") && !(attrval(tg, "role")!="\001" && (attrval(tg, "tabindex")!="\001" || attrval(tg, "tabIndex")!="\001")))
        err("access", "line " NR ": a clickable <" tn ">. Keyboard users cannot reach or press it; use a <button>.")
    }
    else if(tn=="meta" && tolower(attrval(tg, "name"))=="viewport"){
      vc=tolower(attrval(tg, "content")); why=""
      if(match(vc, /user-scalable[ \t]*=[ \t]*(no|0)/)) why=substr(vc, RSTART, RLENGTH)
      else if(match(vc, /maximum-scale[ \t]*=[ \t]*[0-9.]+/)){ ms=substr(vc, RSTART, RLENGTH); v=ms; sub(/^[^=]*=[ \t]*/, "", v); if(v+0 < 2) why=ms }
      if(why!="") err("access", "line " NR ": the viewport blocks zooming (" why "). People who need larger text cannot zoom in; remove it.")
    }
  }
}

# ── typography: font families a line loads or declares ──
{
  fl=$0
  if(fl ~ /@font-face/) inface=1
  s=fl
  while(match(s, /family=[A-Za-z0-9+]+/)){
    nm=substr(s, RSTART+7, RLENGTH-7); gsub(/\+/, " ", nm); s=substr(s, RSTART+RLENGTH)
    loadedany=1; loaded[tolower(nm)]=1
    if(nfam>0 && !(tolower(nm) in brandfam) && !((NR ":" tolower(nm)) in seenf)){ seenf[NR ":" tolower(nm)]=1; err("typography", "line " NR ": loads \"" nm "\", which is not a typography.md family (" famlist ").") }
  }
  s=fl
  while(match(s, /(font-family|fontFamily|--[A-Za-z0-9-]*font[A-Za-z0-9-]*)[ \t]*:[ \t]*[^;}]+/)){
    decl=substr(s, RSTART, RLENGTH); s=substr(s, RSTART+RLENGTH)
    v=decl; sub(/^[^:]*:[ \t]*/, "", v)
    # a custom property is a font stack only when it holds a list or a quoted name
    if(decl ~ /^--/ && v !~ /,/ && v !~ /["\047]/) continue
    split(v, parts, ","); first=parts[1]; gsub(/^[ \t"\047`]+|[ \t"\047`]+$/, "", first)
    lf=tolower(first)
    if(lf=="" || lf ~ /^var\(/ || lf ~ /^[0-9.]/ || lf ~ /^(inherit|initial|unset|revert|serif|sans-serif|monospace|system-ui|cursive|fantasy|emoji|math|ui-[a-z-]+|-apple-system)$/) continue
    if(inface){ loadedany=1; loaded[lf]=1 }
    if(nfam>0 && !(lf in brandfam) && !((NR ":" lf) in seenf)){ seenf[NR ":" lf]=1; err("typography", "line " NR ": \"" first "\" is not a typography.md family (" famlist ").") }
  }
  if(inface && fl ~ /}/) inface=0
}

BEGIN{
  nstw=split(strokew, sw, " "); for(i=1;i<=nstw;i++) strokew_ok[sw[i]+0]=1
  ntint=split(shadowtints, tints, " "); for(i=1;i<=ntint;i++) tint_ok[tints[i]]=1
  # every recipe, light or night, is a documented shadow; a light recipe
  # whose row has a night value is the light half of that row (nightof)
  nshadow=split(shadows, shr, "\036")
  for(i=1;i<=nshadow;i++){
    if(split(shr[i], sr, "|")<3) continue
    v=tolower(sr[2]); gsub(/[ \t]/, "", v); nv=tolower(sr[3]); gsub(/[ \t]/, "", nv)
    shadow_ok[v]=1
    if(nv!=""){ shadow_ok[nv]=1; night_ok[nv]=1; nightof[v]=sr[3]; nightname[v]=sr[1] }
  }
  ntv=split(themevars, tvrows, "\036")
  for(i=1;i<=ntv;i++){
    if(split(tvrows[i], tv, "|")<3) continue
    if(tv[1]=="D"){ dvars[tv[2]]=resolve(tv[3]); hasdark=1 } else lvars[tv[2]]=resolve(tv[3])
  }
  nfaces=split(faces, facerows, "\036")
  for(i=1;i<=nfaces;i++){
    if(split(facerows[i], fr, "|")<4) continue
    # a family listed under several roles has every weight any row lists
    lfam=tolower(fr[2]); famname[lfam]=fr[2]
    famweights[lfam]=(lfam in famweights) ? famweights[lfam] ", " fr[3] : fr[3]
    famstyles[lfam]=(lfam in famstyles) ? famstyles[lfam] ", " fr[4] : fr[4]
    rl=tolower(fr[1])
    if(rl ~ /head/) roleface["heading"]=lfam
    else if(rl ~ /code|mono/) roleface["code"]=lfam
    else if(rl ~ /interface|ui/) roleface["interface"]=lfam
    else if(rl ~ /body|text/) roleface["body"]=lfam
  }
  nst=split(strokes, strk, " "); for(i=1;i<=nst;i++) stroke_px[strk[i]]=1
  nfam=split(fonts, famrows, "\036"); famlist=""
  for(i=1;i<=nfam;i++){ if(famrows[i]=="") continue; brandfam[tolower(famrows[i])]=famrows[i]; famlist=famlist (famlist==""?"":", ") famrows[i] }
  if(famlist=="") nfam=0
  np=split(palette, pal, " ")
  nr=split(radii, rad, " ")
  nb=split(bps, bp, " ")
  na=split(accents, arows, "\036")
  for(i=1;i<=na;i++){ m=split(arows[i], akv, "|"); if(m>=2) accent_owner[akv[1]]=akv[2] }
  ndp=split(docpairs, dprows, "\036")
  for(i=1;i<=ndp;i++){ m=split(dprows[i], dpkv, "|"); if(m>=3) docclass[dpkv[1] "|" dpkv[2]] = tolower(dpkv[3]) }
  nf=split(forbidden, frows, "\036")
  for(i=1;i<=nf;i++){ split(frows[i], kv, "\t"); fterm[i]=kv[1]; falt[i]=kv[2] }
  nt=split(terms, trows, "\036")
  for(i=1;i<=nt;i++){ split(trows[i], tkv, "\t"); tcanon[i]=tkv[1]; tsyn[i]=tkv[2] }
  nprod=split(pnames, prodrows, "\034")
  npn=0; for(i=1;i<=nprod;i++) if(prodrows[i]!="") prodnames[++npn]=prodrows[i]; nprod=npn
  blockstart=0; block=""
}

{
  line=$0
  # In a markup page a <script> block is program data, not styling: its
  # colours (a canvas gradient stop, a fillStyle string) and object literals
  # are not judged by the CSS rules below (issue #74). A file that is all
  # script (.tsx, .js) is not markup and stays judged.
  if(markup){
    sl=""
    while(line!=""){
      if(inscript){
        k=index(line, "</script>")
        if(k==0){ line=""; break }
        line=substr(line, k+9); inscript=0
      } else if(match(line, /<script[ >][^>]*>|<script>/)){
        sl=sl substr(line, 1, RSTART-1); line=substr(line, RSTART+RLENGTH); inscript=1
      } else { sl=sl line; line="" }
    }
    line=sl
  }
  # the theme the line opens in, then the line read into it
  ldepth=thdepth; ldark=thdark; lacc=thacc
  darkafter(line, 1)
  # ── collect custom properties (one-level var resolution) ──
  tmp=line
  while(match(tmp, /--[A-Za-z0-9-]+:[ ]*#[0-9a-fA-F]{3,6}/)){
    decl=substr(tmp, RSTART, RLENGTH)
    split(decl, dkv, ":"); gsub(/ /,"",dkv[2])
    vars[dkv[1]] = toupper(expand(dkv[2]))
    tmp=substr(tmp, RSTART+RLENGTH)
  }
  tmp=line
  while(match(tmp, /--[A-Za-z0-9-]+:[ ]*[A-Za-z]+\([^()]*\)/)){
    cvn=substr(tmp, RSTART, RLENGTH); tmp=substr(tmp, RSTART+RLENGTH)
    cvv=cvn; sub(/^[^:]*:[ ]*/, "", cvv); sub(/:.*/, "", cvn)
    if(tolower(cvv) ~ /^(rgba?|hsla?|oklch|oklab)\(/) vars[cvn]=resolve(cvv)
  }

  # ── color: hex and colour-function literals vs palette (with exclusions) ──
  clean=line
  # attribute values that name things, never colours
  gsub(/(href|xlink:href|id|class|src|srcset|name|for|action|alt|title|content|value|placeholder|aria-[a-z-]+|data-[a-z-]+)[ ]*=[ ]*("[^"]*"|\047[^\047]*\047)/, "", clean)
  if(markup){
    if(clean ~ /<style[ >]/ || clean ~ /<script[ >]/) incode=1
    if(!incode){
      # outside style and script blocks only tags carry colours: drop text
      gsub(/>[^<]*</, "><", clean); sub(/^[^<]*</, "<", clean); sub(/>[^>]*$/, ">", clean)
      if(clean !~ /</) clean=""
    }
    if(clean ~ /<\/style>/ || clean ~ /<\/script>/) incode=0
  }
  gsub(/url\([^)]*\)/, "", clean)
  gsub(/data:[^"'\''() ]*/, "", clean)
  gsub(/--[A-Za-z0-9-]+:[ ]*#[0-9a-fA-F]+/, "", clean)  # var defs already palette-checked as literals below would dup
  tmp=clean; seen=0
  while(match(tmp, /#[0-9a-fA-F]{6}|#[0-9a-fA-F]{3}[^0-9a-fA-F]/)){
    hx=substr(tmp, RSTART, RLENGTH); sub(/[^0-9a-fA-F#]$/, "", hx); hpre=substr(clean, 1, seen+RSTART-1)
    seen+=RSTART+RLENGTH-1; tmp=substr(tmp, RSTART+RLENGTH)
    judgecolour(hx, toupper(expand(hx)), hpre)
  }
  # colour functions, custom property values among them; one with var() or
  # calc() inside holds a parenthesis, so it never matches and is not judged
  tmp=clean; seen=0
  while(match(tolower(tmp), /(^|[^a-z0-9_-])(rgba?|hsla?|oklch|oklab)\([^()]*\)/)){
    fx=substr(tmp, RSTART, RLENGTH); fpre=substr(clean, 1, seen+RSTART-1)
    seen+=RSTART+RLENGTH-1; tmp=substr(tmp, RSTART+RLENGTH)
    sub(/^[^A-Za-z]/, "", fx)
    judgecolour(fx, cfn(fx), fpre)
  }
  # var definitions are also brand values — check them too
  tmp=line
  while(match(tmp, /--[A-Za-z0-9-]+:[ ]*#[0-9a-fA-F]{3,6}/)){
    decl=substr(tmp, RSTART, RLENGTH); split(decl, dkv, ":"); gsub(/ /,"",dkv[2])
    uhx=toupper(expand(dkv[2])); found=0
    for(i=1;i<=np;i++) if(pal[i]==uhx){found=1;break}
    if(!found) err("color", "line " NR ": " dkv[2] " is not in the palette. Did you mean " nearest(uhx) "?")
    tmp=substr(tmp, RSTART+RLENGTH)
  }

  # ── JSX inline style contrast ──
  if(line ~ /style[ ]*=[ ]*\{\{/ && line !~ /\/\*.*\*\// && line !~ /<!-- /){
    jsxfg=""; jsxbg=""
    if(match(line, /color:[ ]*"(#[0-9a-fA-F]{3,6}|[A-Za-z]+\([^()\042]*\))"/)){
      c=substr(line, RSTART, RLENGTH)
      gsub(/color:[ ]*"/, "", c); gsub(/"/, "", c)
      if(c ~ /^#/ || tolower(c) ~ /^(rgba?|hsla?|oklch|oklab)\(/) jsxfg=resolve(c)
    }
    if(match(line, /background(Color)?:[ ]*"(#[0-9a-fA-F]{3,6}|[A-Za-z]+\([^()\042]*\))"/)){
      c=substr(line, RSTART, RLENGTH)
      sub(/background(Color)?:[ ]*"/, "", c); gsub(/"/, "", c)
      if(c ~ /^#/ || tolower(c) ~ /^(rgba?|hsla?|oklch|oklab)\(/) jsxbg=resolve(c)
    }
    judgepair(jsxfg, jsxbg, NR, "")
  }
  # ── rule-block accumulation for contrast ──
  if(line ~ /{/ && blockstart==0){ blockstart=NR; block="" }
  if(blockstart){ block=block " " line }
  if(line ~ /}/ && blockstart){
    bsel2=block; sub(/\{.*/, "", bsel2)
    vfg=""; vbg=""
    if(match(block, /[^-a-z]color:[ ]*[^;}]*/)){ vfg=substr(block, RSTART, RLENGTH); sub(/^[^:]*:[ ]*/, "", vfg) }
    if(match(block, /background(-color)?:[ ]*[^;}]*/)){ vbg=substr(block, RSTART, RLENGTH); sub(/^[^:]*:[ ]*/, "", vbg) }
    if(vfg!="" && vbg!=""){
      bsel=block; sub(/\{.*/, "", bsel)
      if(bsel ~ /data-theme[ ]*=[ ]*["\047]?(dark|night)|\.dark([^a-z-]|$)/) themes="dark"
      else if(hasdark) themes="light dark"
      else themes=""
      if(themes==""){ judgepair(resolve(vfg), resolve(vbg), blockstart, "") }
      else {
        nth=split(themes, th, " ")
        lf=resolvet(vfg, "light"); lb=resolvet(vbg, "light"); df=resolvet(vfg, "dark"); db=resolvet(vbg, "dark")
        if(nth==2 && lf==df && lb==db) judgepair(lf, lb, blockstart, "")
        else for(ti=1;ti<=nth;ti++) judgepair(resolvet(vfg, th[ti]), resolvet(vbg, th[ti]), blockstart, " (" th[ti] " theme)")
      }
    }
    # access: an outline removed in a rule with no replacement in that rule
    if(block ~ /outline[ 	]*:[ 	]*(none|0)([ 	;}]|$)/ && bsel2 !~ /:focus:not\(:focus-visible\)/ && block !~ /box-shadow|outline-(color|style|width)/){ noutline++; outlineline[noutline]=blockstart }
    if(bsel2 ~ /:focus-visible/ && block ~ /outline|box-shadow/) focusvisible=1
    blockstart=0; block=""
  }

  # ── layout: spacing scale + breakpoints ──
  if(line ~ /(padding|margin|gap)[a-z-]*[ ]*:/ && line !~ /--/){
    # scan only the spacing declaration values, not the whole line
    tmp=""
    rest=line
    while(match(rest, /(padding|margin|gap)[a-z-]*[ ]*:[ ]*[^;}]*/)){
      tmp=tmp " " substr(rest, RSTART, RLENGTH)
      rest=substr(rest, RSTART+RLENGTH)
    }
    while(match(tmp, /[0-9]+px/)){
      v=substr(tmp, RSTART, RLENGTH)+0
      if(v % unit != 0 && !(v in stroke_px)){
        lo=int(v/unit)*unit; hi=lo+unit
        warn("layout", "line " NR ": " v "px is outside the spacing scale. Did you mean " lo "px or " hi "px?")
      }
      tmp=substr(tmp, RSTART+RLENGTH)
    }
  }
  if(line ~ /@media/ && match(line, /[0-9]+px/)){
    v=substr(line, RSTART, RLENGTH)+0; found=0
    for(i=1;i<=nb;i++) if(bp[i]==v){found=1;break}
    if(!found && nb>0) warn("layout", "line " NR ": " v "px is not a documented breakpoint (layout.md: adapt within the brand breakpoints).")
  }

  # ── surface: radius scale, overshoot easings, un-tinted shadows ──
  if(line ~ /border[a-z-]*radius[ ]*:/ && line !~ /--radius/){
    # scan only the radius declaration values, not the whole line
    tmp=""
    rest=line
    while(match(rest, /border[a-z-]*radius[ ]*:[ ]*[^;}]*/)){
      tmp=tmp " " substr(rest, RSTART, RLENGTH)
      rest=substr(rest, RSTART+RLENGTH)
    }
    while(match(tmp, /[0-9]+px|50%/)){
      v=substr(tmp, RSTART, RLENGTH)
      if(v!="50%"){
        vn=v+0; found=0
        for(i=1;i<=nr;i++) if(rad[i]==vn"px"){found=1;break}
        if(!found && vn!=999){
          lo=0; hi=99999
          for(i=1;i<=nr;i++){ rv=rad[i]+0; if(rv<vn && rv>lo) lo=rv; if(rv>vn && rv<hi) hi=rv }
          warn("surface", "line " NR ": " vn "px is off the radius scale. Did you mean " lo "px or " hi "px?")
        }
      }
      tmp=substr(tmp, RSTART+RLENGTH)
    }
  }
  if(match(line, /cubic-bezier\([^)]*\)/)){
    cb=substr(line, RSTART, RLENGTH)
    gsub(/cubic-bezier\(|\)/, "", cb); n=split(cb, prm, ",")
    over=0
    for(i=1;i<=n;i++){ gsub(/ /,"",prm[i]); if(prm[i]+0 < -0.0001 || prm[i]+0 > 1.0001) over=1 }
    if(over) warn("surface", "line " NR ": cubic-bezier with overshoot parameters. Brand motion settles and never bounces (components.md).")
  }
  if(line ~ /(bounce|elastic|ease-.*-back)/ && line ~ /(transition|animation)/)
    warn("surface", "line " NR ": bounce/elastic easing. Brand motion settles and never bounces (components.md).")
  if(line ~ /box-shadow[ ]*:/ && (line ~ /#000([^0-9a-fA-F]|$)/ || line ~ /rgba\(0,[ ]*0,[ ]*0/))
    warn("surface", "line " NR ": shadow uses pure black. Brand shadows are ink-tinted (components.md Shadow Recipes).")

  # border and outline widths against the stroke tokens
  if(nstw>0 && line ~ /(border|outline)/){
    rest=line
    while(match(rest, /(border(-top|-right|-bottom|-left)?(-width)?|outline(-width)?)[ ]*:[ ]*[^;}]*/)){
      d=substr(rest, RSTART, RLENGTH); rest=substr(rest, RSTART+RLENGTH)
      prop=d; sub(/[ ]*:.*/, "", prop); v=d; sub(/^[^:]*:[ ]*/, "", v)
      if(match(v, /[0-9]*\.?[0-9]+px/)){
        bw=substr(v, RSTART, RLENGTH-2)+0
        if(bw!=0 && !(bw in strokew_ok)) warn("surface", "line " NR ": " bw "px " (prop ~ /outline/ ? "outline" : "border") " is off the stroke tokens. Did you mean " neareststroke(bw) "? (components.md Border Strokes)")
      }
    }
  }
  # a box-shadow is a token, none, or a documented recipe
  if(nshadow>0 && match(line, /box-shadow[ ]*:[ ]*[^;}]*/)){
    sv=substr(line, RSTART, RLENGTH); sdark=darkafter(substr(line, 1, RSTART-1), 0)
    sub(/^[^:]*:[ ]*/, "", sv); gsub(/!important/, "", sv)
    nsv=tolower(sv); gsub(/[ \t]/, "", nsv)
    # an inset ring with no offset and no blur is a stroke drawn without
    # moving layout: judge its width as a stroke, not as a shadow
    if(match(tolower(sv), /^[ \t]*inset[ \t]+0[ \t]+0[ \t]+0[ \t]+[0-9]*\.?[0-9]+px/)){
      rw=tolower(sv); sub(/^[ \t]*inset[ \t]+0[ \t]+0[ \t]+0[ \t]+/, "", rw); sub(/px.*/, "", rw); rw=rw+0
      if(nstw>0 && rw!=0 && !(rw in strokew_ok)) warn("surface", "line " NR ": " rw "px ring is off the stroke tokens. Did you mean " neareststroke(rw) "? (components.md Border Strokes)")
      nsv=""
    }
    if(nsv!="" && nsv!="none" && nsv !~ /^var\(/ && nsv !~ /^(inherit|initial|unset)$/ && nsv !~ /rgba\(0,0,0|#000([^0-9a-f]|$)/ && !(nsv in shadow_ok))
      warn("surface", "line " NR ": box-shadow matches no shadow recipe. Use a shadow token or a recipe from components.md Shadow Recipes.")
    # in a dark theme, a recipe with a night value is held to that value
    else if(sdark && (nsv in nightof) && !(nsv in night_ok))
      warn("surface", "line " NR ": box-shadow in the dark theme is " (nightname[nsv]!="" ? nightname[nsv] "\047s" : "a") " light recipe. Its night recipe is " nightof[nsv] " (components.md Shadow Recipes, Night).")
  }
  # ── voice: forbidden terms, terminology, product casing ──
  if(iscopy){
    vl=tolower(voicetext)
    for(i=1;i<=nf;i++){
      if(fterm[i]!="" && vl ~ ("(^|[^a-z])" tolower(fterm[i]))){
        err("voice", "line " NR ": \"" fterm[i] "\" is a forbidden term. Use \"" falt[i] "\" instead.")
      }
    }
    for(i=1;i<=nt;i++){
      # a synonym is a whole word, and not the tail of a domain (plotplot.ai)
      if(tsyn[i]!="" && vl ~ ("(^|[^a-z.])" tolower(tsyn[i]) "($|[^a-z])")){
        warn("voice", "line " NR ": \"" tolower(tsyn[i]) "\": use the canonical term \"" tcanon[i] "\" (voice.md Terminology).")
      }
    }
    if(nprod>0){
      tmp=vl
      lref=voicetext
      prodpat="("
      for(i=1;i<=nprod;i++){ prodpat = prodpat (i>1?"|":"") prodnames[i] }
      prodpat = prodpat ")"
      while(match(tmp, "(^|[^a-z])" prodpat "($|[^a-z])")){
        seg=substr(lref, RSTART, RLENGTH)
        gsub(/[^A-Za-z]/, "", seg)
        if(seg!="" && seg!=tolower(seg)) warn("voice", "line " NR ": \"" seg "\": product names are lowercase, always (voice.md).")
        tmp=substr(tmp, RSTART+RLENGTH)
        lref=substr(lref, RSTART+RLENGTH)
      }
    }
  }
}

# Judge one foreground/background pair; tag names the theme when it matters.
function judgepair(fg, bg, ln, tag,   r, key, cls, pre){
  if(fg=="" || bg=="") return
  r=ratio(fg,bg)
  key=fg "|" bg; cls=(key in docclass) ? docclass[key] : ((bg "|" fg) in docclass ? docclass[bg "|" fg] : "")
  pre="line " ln ": " fg " on " bg " is " sprintf("%.2f",r) ":1"
  if(cls ~ /decorative/) err("color", pre ", a documented decorative-only pair (colors.md Contrast Pairings): never words." tag)
  else if(cls ~ /label|display/){ if(r<3.0) err("color", pre ", below the 3.0:1 labels minimum (colors.md Contrast Pairings)." tag) }
  else if(cls ~ /reading/){ if(r<4.5) err("color", pre ", below the 4.5:1 reading minimum (colors.md Contrast Pairings)." tag) }
  else if(r<3.0) err("color", pre ", below the 3.0:1 labels minimum (colors.md Contrast Pairings)." tag)
  else if(r<4.5) warn("color", pre ", labels & display only; verify the pair class for reading copy (4.5:1)." tag)
}

# The theme at the end of text s, read on from the start of the current line
# (ldepth, ldark, lacc): 1 inside a block whose selector, or an enclosing
# one, marks a dark theme, the markers THEMEVARS reads. thdark is the depth
# that block opened at, 0 outside any. With keep set, the state after s is
# carried to the next line.
function darkafter(s, keep,   t, n, i, tk, d, dd, acc){
  gsub(/\/\*([^*]|\*+[^*\/])*\*+\//, "", s)
  t=s; gsub(/[{};]/, "\034&\034", t); n=split(t, tk, "\034")
  d=ldepth; dd=ldark; acc=lacc
  for(i=1;i<=n;i++){
    if(tk[i]=="{"){ d++; if(!dd && acc ~ /data-theme[ ]*=[ ]*["\047]?(dark|night)|\.dark([^a-z-]|$)|prefers-color-scheme[ ]*:[ ]*dark/) dd=d; acc="" }
    else if(tk[i]=="}"){ if(dd==d) dd=0; if(d>0) d--; acc="" }
    else if(tk[i]==";") acc=""
    else acc=acc tk[i]
  }
  if(keep){ thdepth=d; thdark=dd; thacc=acc }
  return dd>0
}

# Resolve a value in one theme: dark overrides sit on top of the light values;
# a value the pre-pass did not see falls back to the streaming resolver.
function resolvet(v, t,   nm){
  sub(/^[ ]+/, "", v)
  if(v ~ /^var\(/){
    nm=v; gsub(/^var\(|\).*$/, "", nm); gsub(/[, ].*$/, "", nm)
    if(t=="dark" && (nm in dvars)) return dvars[nm]
    if(nm in lvars) return lvars[nm]
    return resolve(v)
  }
  return resolve(v)
}

function neareststroke(w,   i, best, bd, d){ bd=1e9; for(i=1;i<=nstw;i++){ d=w-sw[i]; if(d<0) d=-d; if(d<bd){ bd=d; best=sw[i] } } return best "px" }

function resolve(v,   m){
  sub(/^[ \t\n]+/, "", v)
  if(tolower(v) ~ /^(rgba?|hsla?|oklch|oklab)\(/) return opaque(v)
  gsub(/^[ ]+|[ ;}!].*$/, "", v)
  if(v ~ /^var\(/){ gsub(/^var\(|\).*$/, "", v); gsub(/[, ].*$/, "", v); return (v in vars) ? vars[v] : "" }
  if(v ~ /^#[0-9a-fA-F]+$/) return toupper(expand(v))
  return ""
}

END{
  ng=split(garden, grows, "\036")
  for(i=1;i<=ng;i++){ if(grows[i]=="") continue; k=index(grows[i], "|"); err("surface", "line " substr(grows[i], 1, k-1) ": " substr(grows[i], k+1)) }
  if(!synthoff) for(i=1;i<=nfaux;i++) err("typography", faux[i])
  for(i=1;i<=nfield;i++) if(!(fieldid[i] in labelfor)) err("access", "line " fieldline[i] ": a form field without a label. A placeholder vanishes as people type; add a <label for>, or aria-label.")
  if(!focusvisible) for(i=1;i<=noutline;i++) err("access", "line " outlineline[i] ": focus outline removed with no :focus-visible style. Keyboard users lose their place; restore it with :focus-visible.")
  if(loadedany) for(k in brandfam) if(!(k in loaded)) warn("typography", "typography.md names \"" brandfam[k] "\" but the page never loads it.")
  if(sarif){
    emit_sarif()
  } else {
    printf "\n--- petals check (deterministic) ---\n"
    printf "Colors:   %d error(s), %d warning(s)\n", e["color"]+0,  w["color"]+0
    printf "Layout:   %d error(s), %d warning(s)\n", e["layout"]+0, w["layout"]+0
    printf "Surface:  %d error(s), %d warning(s)\n", e["surface"]+0, w["surface"]+0
    printf "Voice:    %d error(s), %d warning(s)\n", e["voice"]+0,  w["voice"]+0
    printf "Typography: %d error(s), %d warning(s) (families; heading and body roles stay with the agent)\n", e["typography"]+0, w["typography"]+0
    printf "Content:  %d error(s), %d warning(s), %d note(s) (%s stage)\n", e["content"]+0, w["content"]+0, notecount["content"]+0, stage
    printf "Access:   %d error(s), %d warning(s) (what the source shows; rendered checks stay with a browser)\n", e["access"]+0, w["access"]+0
    if(hascontracts) printf "Reuse:    %d error(s), %d warning(s)\n", e["reuse"]+0, w["reuse"]+0
    else             printf "Reuse:    no contracts (.brand/components/ holds none)\n"
    te = e["color"]+e["layout"]+e["surface"]+e["voice"]+e["typography"]+e["content"]+e["reuse"]+e["access"]
    tw = w["color"]+w["layout"]+w["surface"]+w["voice"]+w["typography"]+w["content"]+w["reuse"]+w["access"]
    if(te==0) printf "Result: PASS (%d warning(s) to review)\n", tw
    else      printf "Result: FAIL (%d error(s) to fix, %d warning(s) to review).\n", te, tw
  }
  exit (e["color"]+e["layout"]+e["surface"]+e["voice"]+e["typography"]+e["content"]+e["reuse"]+e["access"] > 0 ? 1 : 0)
}
' "$TARGET"
AWK_EXIT=$?

exit $AWK_EXIT
