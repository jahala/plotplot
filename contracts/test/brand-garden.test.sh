#!/usr/bin/env bash
# The brand's garden footer: every kind of pill the footer's markup uses is styled as a pill,
# and the current-bed rule reaches every kind. A bed with no page yet is a <span>, a bed with a
# page is an <a>; a bed page copies components.md verbatim, so an unstyled kind there ships
# unstyled on every bed. Checks components.md and the reference pages. Run from the repository
# root, no arguments. Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root" || exit 1

fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then echo "ok - $*"; else echo "not ok - $*"; fail=1; fi
}

for file in .brand/components.md .brand/reference/landing.html .brand/reference/product.html; do
  [ -f "$file" ]; assert $? "$file exists"
  # the tags directly inside the garden nav, one per line
  tags="$(awk '/<nav class="gf-garden"/{on=1; next} on && /<\/nav>/{on=0} on' "$file" \
    | sed -n 's/^[[:space:]]*<\([a-z]*\)[[:space:]].*/\1/p' | sort -u)"
  [ -n "$tags" ]; assert $? "$file: the garden nav holds pills"
  # every selector list that styles garden pills, with spaces removed
  css="$(tr -d ' \n' < "$file")"
  for tag in $tags; do
    printf '%s' "$css" | grep -q -E "(^|[,}])\.gf-garden>?${tag}(,[^{]*)?\{display:inline-flex" \
      || printf '%s' "$css" | grep -q -E "\.gf-garden>?${tag}[,{][^}]*display:inline-flex"
    assert $? "$file: a <$tag> pill in the garden is styled as a pill"
    if printf '%s' "$css" | grep -q 'is-current{'; then
      printf '%s' "$css" | grep -q -E "\.gf-garden>?(${tag})?\.is-current[,{]"
      assert $? "$file: the current-bed rule reaches a <$tag> pill"
    fi
  done
done

if [ "$fail" -eq 0 ]; then
  echo "brand-garden.test.sh: all assertions passed"
else
  echo "brand-garden.test.sh: failures present"
fi
exit "$fail"
