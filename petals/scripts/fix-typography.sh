#!/usr/bin/env bash
# fix-typography.sh: stop the browser faking bold and italic on a page.
#
# When a rule asks a face for a weight or style it does not have (check.sh
# reports "has no weight" or "has no italic"), the browser synthesises one,
# and the glyphs look smeared. `html{font-synthesis:none}` stops that for the
# whole page, so nobody has to hand-collect selectors.
#
# Usage: fix-typography.sh <file>
#   An .html/.htm page gets the rule as the first line of its first <style>
#   block; a .css/.scss/.less stylesheet gets it as its first line. A file that
#   already sets font-synthesis: none is left as it is. Exit 0 on success,
#   1 when the file is missing or has no <style> block to write into.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SCRIPT_DIR/lib/common.sh" ]]; then
  echo "petals: lib/common.sh is missing beside $(basename "$0"); install the whole petals/ directory" >&2
  exit 127
fi
source "$SCRIPT_DIR/lib/common.sh"

TARGET="${1:?usage: fix-typography.sh <file>}"
TARGET=$(from_caller "$TARGET")
[[ -f "$TARGET" ]] || { echo "fix-typography: no such file: $TARGET" >&2; exit 1; }

RULE='html{font-synthesis:none}'

if grep -qE 'font-synthesis[[:space:]]*:[[:space:]]*none' "$TARGET"; then
  echo "fix-typography: $TARGET already sets font-synthesis: none; nothing to do."
  exit 0
fi

TMP=$(mktemp)
case "$TARGET" in
  *.html|*.htm)
    if ! grep -q '<style' "$TARGET"; then
      rm -f "$TMP"
      echo "fix-typography: $TARGET has no <style> block; add $RULE to its stylesheet." >&2
      exit 1
    fi
    awk -v rule="$RULE" '!done && /<style[^>]*>/ { print; print "  " rule; done=1; next } { print }' "$TARGET" > "$TMP"
    ;;
  *)
    { echo "$RULE"; cat "$TARGET"; } > "$TMP"
    ;;
esac
mv "$TMP" "$TARGET"
echo "fix-typography: wrote $RULE into $TARGET."
