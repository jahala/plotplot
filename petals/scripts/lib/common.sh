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
