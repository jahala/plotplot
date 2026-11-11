#!/usr/bin/env bash
# petals/scripts/lib/common.sh — shared helpers for all petals scripts.

# The directory the caller ran the script from. Scripts move to the project
# root after sourcing this file, so a path the caller typed is resolved
# against this directory first.
CALLER_PWD="$PWD"

# Resolve a path the caller typed, relative to where they called from.
from_caller() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *)  printf '%s/%s\n' "$CALLER_PWD" "$1" ;;
  esac
}

# Same, for a source that may be a URL or a remote shorthand: resolve it only
# when it names something that exists from the caller's directory.
from_caller_if_exists() {
  case "$1" in
    /*|*://*|git@*) printf '%s\n' "$1" ;;
    *) if [[ -e "$CALLER_PWD/$1" ]]; then printf '%s/%s\n' "$CALLER_PWD" "$1"; else printf '%s\n' "$1"; fi ;;
  esac
}

# Walk up from a directory (default: $PWD) looking for the project root.
# A project root is a directory containing .petalsrc or .brand/.
# Prints the absolute path and returns 0, or returns 1 if not found.
find_project_root() {
  local dir="${1:-$PWD}"
  while [[ "$dir" != "/" ]]; do
    if [[ -f "$dir/.petalsrc" ]] || [[ -d "$dir/.brand" ]]; then
      echo "$dir"
      return 0
    fi
    dir=$(dirname "$dir")
  done
  return 1
}

# The custom-property prefix a brand declares, such as --pp- or --acme-: the
# one "--<word>-" every property in colors.md's own CSS blocks (:root and
# [data-theme="dark"], inside code fences) starts with. Every script reads the
# prefix here and nowhere else. Prints the prefix and returns 0; when colors.md
# is missing, declares no properties, or mixes prefixes, prints why and
# returns 1. It never guesses one.
brand_prefix() {
  local colors="$1"
  [[ -f "$colors" ]] || { echo "$colors is missing, so the brand's token prefix is unknown"; return 1; }
  awk -v file="$colors" '
    /^[ \t]*```/ { infence = !infence; next }
    infence { text = text $0 "\n" }
    END {
      gsub(/\/\*([^*]|\*+[^*\/])*\*+\//, "", text)
      while (match(text, /[^{}]*\{[^{}]*\}/)) {
        rule = substr(text, RSTART, RLENGTH); text = substr(text, RSTART + RLENGTH)
        sel = rule; sub(/\{.*/, "", sel); sub(/^.*;/, "", sel); gsub(/^[ \t\n]+|[ \t\n]+$/, "", sel)
        if (sel != ":root" && sel !~ /^\[data-theme[ ]*=[ ]*\042dark\042\]$/) continue
        body = rule; sub(/^[^{]*\{/, "", body)
        while (match(body, /--[A-Za-z0-9_-]+[ \t\n]*:/)) {
          name = substr(body, RSTART, RLENGTH); body = substr(body, RSTART + RLENGTH)
          sub(/[ \t\n]*:$/, "", name)
          p = match(name, /^--[A-Za-z0-9]+-/) ? substr(name, 1, RLENGTH) : name
          if (!(p in seen)) { seen[p] = 1; n++; list = list (n > 1 ? ", " : "") p }
        }
      }
      if (n == 0) { print file " declares no custom properties in a CSS :root block, so the brand\047s token prefix is unknown"; exit 1 }
      if (n > 1) { print file " uses more than one token prefix in its CSS blocks (" list "); a brand declares one, such as --acme-"; exit 1 }
      print list
    }' "$colors"
}

# Whether a brand has a dark theme: a [data-theme="dark"] CSS block inside
# one of colors.md's code fences, the block export-tokens.sh derives the dark
# tokens from. Returns 0 when it has one, 1 when it has none or colors.md is
# missing. A brand's night values (the Shadow Recipes Night column) count
# only when it has one; every script asks here.
brand_has_dark() {
  [[ -f "$1" ]] || return 1
  awk '
    /^[ \t]*```/ { infence = !infence; next }
    infence { text = text $0 "\n" }
    END {
      gsub(/\/\*([^*]|\*+[^*\/])*\*+\//, "", text)
      exit (text ~ /(^|[;{}\n])[ \t\n]*\[data-theme[ ]*=[ ]*\042dark\042\][ \t\n]*\{/) ? 0 : 1
    }' "$1"
}
