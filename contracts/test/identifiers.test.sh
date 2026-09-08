#!/usr/bin/env bash
# Parses each fixture in contracts/fixtures/identifiers/ against the grammar pinned in
# contracts/identifiers.md, with the parser written directly in this script. Run from the
# repository root, no arguments. Prints one line per assertion. Exits 0 if every assertion
# passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixdir="$repo_root/contracts/fixtures/identifiers"

fail=0
assert() {
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then
    echo "ok - $*"
  else
    echo "not ok - $*"
    fail=1
  fi
}

read_one_line() {
  # read_one_line <file>: prints the file's first line with a trailing newline stripped
  local f="$1"
  [ -f "$f" ] || { echo ""; return 1; }
  head -n1 "$f"
}

# --- session id ---------------------------------------------------------------------
f="$fixdir/session-id.txt"
if [ -f "$f" ]; then
  val="$(read_one_line "$f")"
  len=${#val}
  if [ -n "$val" ] && [[ "$val" != *"/"* ]] && [[ "$val" != *[[:space:]]* ]] && [ "$len" -lt 128 ]; then
    assert 0 "session-id.txt: '$val' is a non-empty opaque token under 128 bytes with no path separator or whitespace"
  else
    assert 1 "session-id.txt: '$val' matches the session id grammar"
  fi
else
  assert 1 "session-id.txt exists"
fi

# --- commit sha ----------------------------------------------------------------------
f="$fixdir/commit-sha.txt"
if [ -f "$f" ]; then
  val="$(read_one_line "$f")"
  if [[ "$val" =~ ^[0-9a-f]{40}$ ]] || [[ "$val" =~ ^[0-9a-f]{64}$ ]]; then
    assert 0 "commit-sha.txt: '$val' is 40 or 64 lowercase hex characters"
  else
    assert 1 "commit-sha.txt: '$val' matches the commit sha grammar"
  fi
else
  assert 1 "commit-sha.txt exists"
fi

# --- loop id and check ordinal ---------------------------------------------------------
f="$fixdir/loop-check.txt"
if [ -f "$f" ]; then
  val="$(read_one_line "$f")"
  if [[ "$val" =~ ^([a-z][a-z0-9-]*):c([1-9][0-9]*)$ ]]; then
    loop_id="${BASH_REMATCH[1]}"
    ordinal="${BASH_REMATCH[2]}"
    assert 0 "loop-check.txt: '$val' parses as loop id '$loop_id', check ordinal c$ordinal"
  else
    assert 1 "loop-check.txt: '$val' matches the <loop-id>:c<ordinal> grammar"
  fi
else
  assert 1 "loop-check.txt exists"
fi

# --- season --------------------------------------------------------------------------
f="$fixdir/season.txt"
if [ -f "$f" ]; then
  val="$(read_one_line "$f")"
  if [[ "$val" =~ ^[0-9]{4}\.(0[1-9]|1[0-2])$ ]]; then
    assert 0 "season.txt: '$val' matches YYYY.MM"
  else
    assert 1 "season.txt: '$val' matches the season grammar"
  fi
else
  assert 1 "season.txt exists"
fi
season_val="$val"

# --- proposal id and its three markers ------------------------------------------------
proposal_id_re='[a-z][a-z0-9-]*:[0-9]{4}\.(0[1-9]|1[0-2]):[a-z][a-z0-9-]*'
f="$fixdir/proposal-marker.txt"
if [ -f "$f" ]; then
  line_total=$(grep -c '' "$f")
  if [ "$line_total" -ge 3 ]; then
    assert 0 "proposal-marker.txt has at least three lines"
  else
    assert 1 "proposal-marker.txt has at least three lines (got $line_total)"
  fi

  practices_line="$(sed -n '1p' "$f")"
  shape_line="$(sed -n '2p' "$f")"
  skill_line="$(sed -n '3p' "$f")"

  if [[ "$practices_line" =~ ·\ from\ ($proposal_id_re)[[:space:]]*$ ]]; then
    assert 0 "proposal-marker.txt line 1 is a practices-line marker for '${BASH_REMATCH[1]}'"
  else
    assert 1 "proposal-marker.txt line 1 matches '· from <proposal-id>'"
  fi

  if [[ "$shape_line" =~ ^#\ from\ ($proposal_id_re)[[:space:]]*$ ]]; then
    assert 0 "proposal-marker.txt line 2 is a shape-comment marker for '${BASH_REMATCH[1]}'"
  else
    assert 1 "proposal-marker.txt line 2 matches '# from <proposal-id>'"
  fi

  if [[ "$skill_line" =~ ^\<!--\ from\ ($proposal_id_re)\ --\>[[:space:]]*$ ]]; then
    assert 0 "proposal-marker.txt line 3 is a skill/loop-comment marker for '${BASH_REMATCH[1]}'"
  else
    assert 1 "proposal-marker.txt line 3 matches '<!-- from <proposal-id> -->'"
  fi
else
  assert 1 "proposal-marker.txt exists"
fi

if [ "$fail" -eq 0 ]; then
  echo "identifiers.test.sh: all assertions passed"
  exit 0
else
  echo "identifiers.test.sh: failures present"
  exit 1
fi
