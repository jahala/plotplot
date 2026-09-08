#!/usr/bin/env bash
# Validates contracts/fixtures/friction.jsonl against the pinned envelope and kind
# tables in contracts/friction-profile.md. Run from the repository root, no arguments.
# Prints one line per assertion. Exits 0 if every assertion passed, 1 otherwise.
set -u

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture="$repo_root/contracts/fixtures/friction.jsonl"
profile="$repo_root/contracts/friction-profile.md"

fail=0

assert() {
  # assert <0-or-nonzero> <description...>
  local status="$1"; shift
  if [ "$status" -eq 0 ]; then
    echo "ok - $*"
  else
    echo "not ok - $*"
    fail=1
  fi
}

if ! command -v jq >/dev/null 2>&1; then
  echo "not ok - jq is available to parse the fixture"
  exit 1
fi

[ -f "$fixture" ]; assert $? "fixture exists at contracts/fixtures/friction.jsonl"
[ -f "$profile" ]; assert $? "profile exists at contracts/friction-profile.md"

if [ "$fail" -ne 0 ]; then
  echo "friction-profile.test.sh: cannot continue without the fixture and the profile"
  exit 1
fi

# The pinned kind list and per-kind attribute rules, transcribed from
# contracts/friction-profile.md's "the pinned kinds" and "per-kind attributes" tables.
# This file is the small parser and rule set the profile names; there is no separate
# machine-readable copy of the profile to load, so keeping this list in step with the
# profile's tables is a manual discipline, checked by review, not by this script.
pinned_kinds="tool.denied tool.failed tool.retry file.reread search.fanout edit.churn test.loop stop.refused context.compacted session.ended gate.retry worker.wedged model.call"
path_required_kinds="tool.denied tool.failed file.reread edit.churn test.loop"

is_in_list() {
  local needle="$1" list="$2" item
  for item in $list; do
    [ "$item" = "$needle" ] && return 0
  done
  return 1
}

extra_required_for() {
  case "$1" in
    tool.denied) echo 'gen_ai.operation.name gen_ai.tool.name' ;;
    tool.failed) echo 'gen_ai.operation.name gen_ai.tool.name error.type' ;;
    tool.retry) echo 'gen_ai.tool.name gen_ai.tool.call.id' ;;
    file.reread) echo 'gen_ai.tool.name' ;;
    search.fanout) echo 'gen_ai.tool.name' ;;
    edit.churn) echo 'gen_ai.tool.name' ;;
    test.loop) echo 'gen_ai.tool.name' ;;
    stop.refused) echo 'plotplot.rule' ;;
    context.compacted) echo 'gen_ai.conversation.compacted' ;;
    session.ended) echo 'gen_ai.usage.input_tokens gen_ai.usage.output_tokens' ;;
    gate.retry) echo 'plotplot.node plotplot.gate' ;;
    worker.wedged) echo 'plotplot.worker' ;;
    model.call) echo 'gen_ai.provider.name gen_ai.request.model gen_ai.usage.input_tokens gen_ai.usage.output_tokens' ;;
    *) echo '' ;;
  esac
}

has_key() {
  # has_key <json-line> <key>: true if the flat key is present, value may be null
  jq -e --arg k "$2" 'has($k)' >/dev/null 2>&1 <<<"$1"
}

has_nonnull_key() {
  jq -e --arg k "$2" 'has($k) and (.[$k] != null)' >/dev/null 2>&1 <<<"$1"
}

line_no=0
line_count=0
seen_kinds=""
while IFS= read -r line || [ -n "$line" ]; do
  line_no=$((line_no + 1))
  [ -z "$line" ] && continue
  line_count=$((line_count + 1))

  # exactly one JSON object on this physical line: slurping the line must yield
  # an array of length 1, and that one value must be an object.
  count_and_type=$(jq -cs '[length, (.[0] | type)]' <<<"$line" 2>/dev/null)
  if [ "$count_and_type" = "[1,\"object\"]" ]; then
    assert 0 "line $line_no is exactly one JSON object"
  else
    assert 1 "line $line_no is exactly one JSON object (got: ${count_and_type:-unparseable})"
    continue
  fi

  kind=$(jq -r '.["plotplot.kind"] // empty' <<<"$line")
  if [ -n "$kind" ]; then
    assert 0 "line $line_no has plotplot.kind ($kind)"
  else
    assert 1 "line $line_no has plotplot.kind"
    continue
  fi
  seen_kinds="$seen_kinds $kind"

  if is_in_list "$kind" "$pinned_kinds"; then
    assert 0 "line $line_no plotplot.kind '$kind' is on the pinned list"
  else
    assert 1 "line $line_no plotplot.kind '$kind' is on the pinned list"
  fi

  for key in time event.name plotplot.count; do
    if has_nonnull_key "$line" "$key"; then
      assert 0 "line $line_no ($kind) has non-null $key"
    else
      assert 1 "line $line_no ($kind) has non-null $key"
    fi
  done

  for key in plotplot.harness gen_ai.conversation.id; do
    if has_key "$line" "$key"; then
      assert 0 "line $line_no ($kind) carries the envelope key $key (null allowed)"
    else
      assert 1 "line $line_no ($kind) carries the envelope key $key (null allowed)"
    fi
  done

  time_val=$(jq -r '.time // empty' <<<"$line")
  case "$time_val" in
    *Z)
      assert 0 "line $line_no ($kind) time '$time_val' ends in Z"
      stripped="${time_val%Z}"
      stripped="${stripped%.*}"
      if date -u -j -f "%Y-%m-%dT%H:%M:%S" "$stripped" "+%s" >/dev/null 2>&1; then
        assert 0 "line $line_no ($kind) time '$time_val' parses as a UTC timestamp"
      else
        assert 1 "line $line_no ($kind) time '$time_val' parses as a UTC timestamp"
      fi
      ;;
    *)
      assert 1 "line $line_no ($kind) time '$time_val' ends in Z"
      ;;
  esac

  if is_in_list "$kind" "$path_required_kinds"; then
    path_kind=$(jq -r '.["plotplot.path.kind"] // empty' <<<"$line")
    if [ "$path_kind" = "outside" ]; then
      if jq -e 'has("plotplot.path") | not' >/dev/null 2>&1 <<<"$line"; then
        assert 0 "line $line_no ($kind) marks path.kind outside and omits plotplot.path"
      else
        assert 1 "line $line_no ($kind) marks path.kind outside and omits plotplot.path"
      fi
    else
      p=$(jq -r '.["plotplot.path"] // empty' <<<"$line")
      if [ -n "$p" ] && [[ "$p" != /* ]]; then
        assert 0 "line $line_no ($kind) has repo-relative plotplot.path ($p)"
      else
        assert 1 "line $line_no ($kind) has a non-empty, repo-relative plotplot.path"
      fi
    fi
  fi

  missing=""
  for key in $(extra_required_for "$kind"); do
    has_key "$line" "$key" || missing="$missing $key"
  done
  if [ -z "$missing" ]; then
    assert 0 "line $line_no ($kind) carries its per-kind required attributes"
  else
    assert 1 "line $line_no ($kind) carries its per-kind required attributes (missing:$missing)"
  fi
done <"$fixture"

[ "$line_count" -eq 3 ]; assert $? "the fixture has exactly three lines (got $line_count)"

for demo_kind in tool.failed gate.retry model.call; do
  if is_in_list "$demo_kind" "$seen_kinds"; then
    assert 0 "the fixture includes a $demo_kind line"
  else
    assert 1 "the fixture includes a $demo_kind line"
  fi
done

if [ "$fail" -eq 0 ]; then
  echo "friction-profile.test.sh: all assertions passed"
  exit 0
else
  echo "friction-profile.test.sh: failures present"
  exit 1
fi
