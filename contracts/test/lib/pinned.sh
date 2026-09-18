# Sourced by the tests that read a file at the commit contracts/pins.json pins.
#
# read_pinned NAME prints the text of the file pinned under the key NAME (lower-cased): its
# repository, file and commit come from contracts/pins.json, and the text comes from the
# platform (gh, authenticated, since two of the beds are private today). A local checkout
# may be named instead through PLOTPLOT_PIN_<NAME>_FILE, for a machine without gh. Returns
# 3 when the file cannot be read, so the caller can say "unevaluable" and never report an
# unverifiable pin as agreement. The caller runs from the repository root and sets
# $test_name for the messages.
read_pinned() {
  local name="$1" override_var="PLOTPLOT_PIN_$1_FILE" repo file ref who="${test_name:-pinned.sh}"
  repo="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].repo)")"
  file="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].file)")"
  ref="$(node -e "console.log(require('./contracts/pins.json')['$1'.toLowerCase()].ref)")"
  if [ -n "${!override_var:-}" ]; then
    [ -f "${!override_var}" ] && cat "${!override_var}" && return 0
    echo "$who: $override_var names a file that does not exist: ${!override_var}" >&2; return 3
  fi
  if ! command -v gh >/dev/null 2>&1; then
    echo "$who: gh is not installed and $override_var is unset; $name cannot be verified" >&2; return 3
  fi
  local body
  if ! body="$(gh api "repos/$repo/contents/$file?ref=$ref" --jq .content 2>/dev/null)"; then
    echo "$who: gh could not read $repo:$file at $ref (not authenticated, or the ref is gone); $name cannot be verified" >&2; return 3
  fi
  printf '%s' "$body" | base64 -d 2>/dev/null || printf '%s' "$body" | base64 -D
}
