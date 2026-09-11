#!/usr/bin/env bash
# scripts/fit/weeder.sh: weeder's fit evidence, the contract F1 to F6 of
# docs/building-the-garden.md §3, against the release contracts/fixtures/garden.lock pins.
#
#   scripts/fit/weeder.sh        every claim below, in order
#   scripts/fit/weeder.sh f1     manifest: the tarball for this platform matches the lock's digest
#                                and carries garden.json at its top level; it validates against
#                                contracts/manifest.schema.json, names weeder, and its version is
#                                the lock's
#   scripts/fit/weeder.sh f2     SARIF: the manifest's check command, run from the artifact on a
#                                fixture repository with one staged change, emits a log that
#                                validates against the vendored SARIF 2.1.0 schema, whose tool is
#                                weeder at the lock's version
#   scripts/fit/weeder.sh f3     standalone install: the extracted binary reports the lock's
#                                version with no other garden tool on PATH, outside any repository
#   scripts/fit/weeder.sh f4     declared context cost: upfront_tokens is declared and zero, no MCP
#                                face is registered, and the SKILL.md description costs no more
#                                than the manifest declares, at four characters per token
#   scripts/fit/weeder.sh f5     metric committed: the manifest's metric command runs in the tree
#                                of the pinned tag, what it writes is committed at that tag byte
#                                for byte, and it states a blocked count, a false-positive count
#                                and a percentage
#   scripts/fit/weeder.sh f6     brand layer and footer: the pinned tag carries weeder's product
#                                layer and both marks, and its page's footer carries the plotplot
#                                band and the garden row of .brand/components.md with weeder's pill
#                                current
#
# One line per assertion: `ok`, `not ok` or `unevaluable`, then the claim and what was seen.
# Exit 0 when every assertion passed, 1 on any failure, 3 when none failed but a claim could not
# be evaluated on this machine (the reason goes to stderr), 2 on a usage error. A claim that
# cannot be proven standalone is unevaluable, never passed.
#
# Everything is read from the pinned release and the umbrella's own contracts, never from a
# checkout on this machine. The version, the artifact, the repository and the tag all come from
# the lock, so the next release is checked by moving the lock and nothing here. The artifact is
# verified against the lock's digest; the files read at the tag (the source archive the metric
# runs in, and the raw reads F5 and F6 make) are the tag's as GitHub serves them, since the lock
# pins no digest for them. Scratch goes away with `trash`, never `rm` (scripts/fit/lib.sh,
# fit_discard_scratch).

set -uo pipefail

WEEDER_SH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/fit/lib.sh
. "$WEEDER_SH_DIR/lib.sh"

ROOT="$(fit_repo_root)"
LOCK="$ROOT/contracts/fixtures/garden.lock"
BED="weeder"

# The run's verdict so far: 0 while every assertion passed, 3 once one was unevaluable, 1 once
# one failed. A failure outranks an unevaluable claim.
STATUS=0

# The garden's cost basis for a description line, the one scripts/fit/stem.sh's context check
# and docs/garden-architecture-2026-09.md §2 use.
CHARS_PER_TOKEN=4

# ---------------------------------------------------------------------------------------
# saying what happened
# ---------------------------------------------------------------------------------------

ok() {
  echo "ok $*"
}

not_ok() {
  echo "not ok $*"
  STATUS=1
}

# The claim on stdout and the reason it could not be evaluated on stderr.
unevaluable() {
  local claim="$1" why="$2"
  echo "unevaluable $claim"
  echo "weeder.sh: $claim: $why" >&2
  [ "$STATUS" -eq 1 ] || STATUS=3
}

# ---------------------------------------------------------------------------------------
# scratch
# ---------------------------------------------------------------------------------------

# The one temporary directory this run works in, minted after the arguments are read and put
# away by the EXIT trap.
RUN_DIR=""

discard_run_dir() {
  [ -n "$RUN_DIR" ] && [ -d "$RUN_DIR" ] && fit_discard_scratch "$RUN_DIR"
}
trap discard_run_dir EXIT

# ---------------------------------------------------------------------------------------
# fetching
# ---------------------------------------------------------------------------------------

# Why a fetch failed: "absent" when the server says the thing is not there, "unreachable"
# otherwise (no network, a timeout, a server error), so a missing file fails a claim and a
# missing network never does.
why_unfetched() {
  local url="$1" code
  case "$url" in
    file://*) echo "absent"; return ;;
  esac
  code="$(curl -sSL -o /dev/null -w '%{http_code}' "$url" 2>/dev/null)"
  case "$code" in
    404|410) echo "absent" ;;
    *) echo "unreachable" ;;
  esac
}

# Fetch url to dest through fit_fetch. 0 when fetched; 1 when the server says it is not there;
# 3 when it could not be asked.
fetch() {
  local url="$1" dest="$2"
  fit_fetch "$url" "$dest" 2>/dev/null && return 0
  [ "$(why_unfetched "$url")" = "absent" ] && return 1
  return 3
}

# ---------------------------------------------------------------------------------------
# what the lock pins
# ---------------------------------------------------------------------------------------

# Set once by resolve_pin: the lock's version, url and digest for this platform, and the
# repository and tag its release url names. PIN_STATE is ok, not_ok or unevaluable, with the
# reason in PIN_WHY.
PIN_STATE=""
PIN_WHY=""
PLATFORM=""
LOCK_VERSION=""
LOCK_URL=""
LOCK_SHA=""
REPO_SLUG=""
TAG=""

resolve_pin() {
  [ -n "$PIN_STATE" ] && return
  PLATFORM="$(fit_platform)"
  if ! command -v node >/dev/null 2>&1; then
    PIN_STATE="unevaluable" PIN_WHY="node is not on PATH, and the lock is read through scripts/fit/lock-lookup.mjs"
    return
  fi
  if [ ! -d "$ROOT/node_modules/smol-toml" ]; then
    PIN_STATE="unevaluable" PIN_WHY="the lock reader needs smol-toml; run 'npm ci' in $ROOT first"
    return
  fi
  local lookup
  if ! lookup="$(fit_lock_lookup "$LOCK" "$BED" "$PLATFORM" 2>&1)"; then
    PIN_STATE="unevaluable" PIN_WHY="the lock pins no $BED artifact for $PLATFORM, so this machine cannot fetch one: $lookup"
    return
  fi
  LOCK_VERSION="$(echo "$lookup" | jq -r '.version // empty')"
  LOCK_URL="$(echo "$lookup" | jq -r '.url // empty')"
  LOCK_SHA="$(echo "$lookup" | jq -r '.sha256 // empty')"
  if [ -z "$LOCK_VERSION" ] || [ -z "$LOCK_URL" ] || [ -z "$LOCK_SHA" ]; then
    PIN_STATE="not_ok" PIN_WHY="the lock's $BED entry for $PLATFORM lacks a version, url or sha256: $lookup"
    return
  fi
  # A GitHub release asset url names the repository and the tag the release was cut from.
  case "$LOCK_URL" in
    https://github.com/*/*/releases/download/*/*)
      local rest="${LOCK_URL#https://github.com/}"
      REPO_SLUG="$(echo "$rest" | cut -d/ -f1-2)"
      TAG="$(echo "$rest" | cut -d/ -f5)"
      ;;
  esac
  PIN_STATE="ok"
}

# ---------------------------------------------------------------------------------------
# the artifact, fetched and verified once per run
# ---------------------------------------------------------------------------------------

# Set once by resolve_artifact: the directory the verified tarball was extracted into.
ARTIFACT_STATE=""
ARTIFACT_WHY=""
ARTIFACT_DIR=""

resolve_artifact() {
  [ -n "$ARTIFACT_STATE" ] && return
  resolve_pin
  if [ "$PIN_STATE" != "ok" ]; then
    ARTIFACT_STATE="$PIN_STATE" ARTIFACT_WHY="$PIN_WHY"
    return
  fi
  local archive="$RUN_DIR/${LOCK_URL##*/}" status
  fetch "$LOCK_URL" "$archive"
  status=$?
  case "$status" in
    1) ARTIFACT_STATE="not_ok" ARTIFACT_WHY="the release carries no asset at $LOCK_URL"; return ;;
    3) ARTIFACT_STATE="unevaluable" ARTIFACT_WHY="could not fetch $LOCK_URL from this machine"; return ;;
  esac
  local actual
  actual="$(fit_sha256 "$archive")"
  if [ "$actual" != "$LOCK_SHA" ]; then
    ARTIFACT_STATE="not_ok" ARTIFACT_WHY="the tarball at $LOCK_URL hashes to $actual, not the lock's $LOCK_SHA; nothing from it is run"
    return
  fi
  ARTIFACT_DIR="$RUN_DIR/artifact"
  if ! fit_extract "$archive" "$ARTIFACT_DIR" 2>"$RUN_DIR/extract.err"; then
    ARTIFACT_STATE="not_ok" ARTIFACT_WHY="the verified tarball would not extract: $(head -1 "$RUN_DIR/extract.err")"
    return
  fi
  ARTIFACT_STATE="ok"
}

# Resolve the artifact for a claim, and say so on that claim's line when it cannot be had.
need_artifact() {
  local claim="$1"
  resolve_artifact
  case "$ARTIFACT_STATE" in
    ok) return 0 ;;
    not_ok) not_ok "$claim: $ARTIFACT_WHY" ;;
    *) unevaluable "$claim" "$ARTIFACT_WHY" ;;
  esac
  return 1
}

# The artifact's garden.json, for a claim that reads it; F1 is the claim that proves it.
need_manifest() {
  local claim="$1"
  need_artifact "$claim" || return 1
  [ -f "$ARTIFACT_DIR/garden.json" ] && jq -e . "$ARTIFACT_DIR/garden.json" >/dev/null 2>&1 && return 0
  not_ok "$claim: the artifact carries no readable garden.json at its top level"
  return 1
}

# The PATH every run of the artifact gets: its own directory and the system's, no garden tool.
artifact_path() {
  fit_clean_path "$ARTIFACT_DIR"
}

# ---------------------------------------------------------------------------------------
# the bed's repository at the pinned tag
# ---------------------------------------------------------------------------------------

# Read one file of the bed's repository at the pinned tag by an anonymous raw read. 0 when
# read, 1 when the tag does not carry it, 3 when it could not be asked.
raw_read() {
  local path="$1" dest="$2"
  mkdir -p "$(dirname "$dest")"
  fetch "https://raw.githubusercontent.com/$REPO_SLUG/$TAG/$path" "$dest"
}

# Resolve the repository and tag for a claim that reads the tag, and say so when the lock's
# url does not name them.
need_tag() {
  local claim="$1"
  resolve_pin
  case "$PIN_STATE" in
    ok) ;;
    not_ok) not_ok "$claim: $PIN_WHY"; return 1 ;;
    *) unevaluable "$claim" "$PIN_WHY"; return 1 ;;
  esac
  [ -n "$REPO_SLUG" ] && [ -n "$TAG" ] && return 0
  unevaluable "$claim" "the lock's url $LOCK_URL is not a GitHub release asset, so it names no repository and tag to read"
  return 1
}

# ---------------------------------------------------------------------------------------
# the garden footer
# ---------------------------------------------------------------------------------------

# The first element of an HTML file that opens with the given tag and class, through its
# closing tag, on the lines it spans.
html_block() {
  local file="$1" opening="$2" closing="$3"
  awk -v opening="$opening" -v closing="$closing" \
    'index($0, opening) { on = 1 } on { print } on && index($0, closing) { exit }' "$file"
}

# The pills of a garden row read on stdin, one to a line: the bed's name, then " current" on
# the pill the page marks as its own.
garden_pills() {
  local anchor name
  tr '\n' ' ' \
    | grep -oE '<a [^>]*>([^<]|<span[^>]*>[^<]*</span>)*</a>' \
    | while IFS= read -r anchor; do
        name="$(echo "$anchor" | sed -e 's/<[^>]*>//g' -e 's/^ *//' -e 's/ *$//')"
        case "$anchor" in
          *'class="is-current"'*|*'class="'*' is-current"'*|*'class="is-current '*) echo "$name current" ;;
          *) echo "$name" ;;
        esac
      done
}

# ---------------------------------------------------------------------------------------
# F1 manifest
# ---------------------------------------------------------------------------------------

check_f1() {
  local claim="F1 manifest"
  need_artifact "$claim" || return
  ok "$claim: the $PLATFORM tarball at $LOCK_URL matches the lock's sha256 $LOCK_SHA"

  local manifest="$ARTIFACT_DIR/garden.json"
  if [ ! -f "$manifest" ]; then
    not_ok "$claim: the tarball carries no garden.json at its top level ($(ls "$ARTIFACT_DIR" | tr '\n' ' '))"
    return
  fi
  ok "$claim: the tarball carries garden.json at its top level"

  if [ ! -x "$ROOT/node_modules/.bin/ajv" ] || [ ! -d "$ROOT/node_modules/ajv-formats" ]; then
    unevaluable "$claim: garden.json validates against contracts/manifest.schema.json" \
      "the schema is validated with ajv and ajv-formats; run 'npm ci' in $ROOT first"
    return
  fi
  if ( cd "$ROOT" && npx --no-install ajv validate -s contracts/manifest.schema.json -d "$manifest" \
        --spec=draft2020 -c ajv-formats ) >"$RUN_DIR/ajv.log" 2>&1; then
    ok "$claim: garden.json validates against contracts/manifest.schema.json"
  else
    cat "$RUN_DIR/ajv.log" >&2
    not_ok "$claim: garden.json does not validate against contracts/manifest.schema.json (ajv's errors on stderr)"
    return
  fi

  local name version
  name="$(jq -r '.name // empty' "$manifest")"
  version="$(jq -r '.version // empty' "$manifest")"
  if [ "$name" = "$BED" ] && [ "$version" = "$LOCK_VERSION" ]; then
    ok "$claim: garden.json names $name at $version, the lock's version"
  else
    not_ok "$claim: garden.json names \"$name\" at \"$version\", not $BED at the lock's $LOCK_VERSION"
  fi
}

# ---------------------------------------------------------------------------------------
# F2 SARIF
# ---------------------------------------------------------------------------------------

# A repository with a feature and its test committed, and the test's deletion staged: one
# change, and one weeder refuses at block level, so the log carries a result for the schema to
# hold rather than an empty envelope.
plant_f2_fixture() {
  local repo="$1"
  mkdir -p "$repo/src" "$repo/tests" || return 1
  git init -q "$repo" || return 1
  git -C "$repo" config user.email "fit@plotplot.invalid" || return 1
  git -C "$repo" config user.name "plotplot fit" || return 1
  cat > "$repo/src/parser.ts" <<'SOURCE'
export function parse(text: string): number {
  return text.length;
}
SOURCE
  cat > "$repo/tests/parser.test.ts" <<'TEST'
import { test, expect } from "vitest";

import { parse } from "../src/parser";

test("parses an empty string", () => {
  expect(parse("")).toBe(0);
});
TEST
  git -C "$repo" add -A && git -C "$repo" commit -qm "the feature and its test" || return 1
  git -C "$repo" rm -q tests/parser.test.ts
}

check_f2() {
  local claim="F2 SARIF"
  need_manifest "$claim" || return

  local check
  check="$(jq -r '.check // empty' "$ARTIFACT_DIR/garden.json")"
  if [ -z "$check" ]; then
    not_ok "$claim: garden.json names no check command, and weeder is a gate"
    return
  fi

  # The command exactly as the manifest spells it, resolved through a PATH that holds only the
  # artifact and the system, so it can only be the artifact's own binary that answers.
  local words resolved
  read -r -a words <<< "$check"
  resolved="$(env -i PATH="$(artifact_path)" /bin/sh -c 'command -v "$1"' sh "${words[0]}")"
  if [ "$resolved" != "$ARTIFACT_DIR/${words[0]}" ]; then
    not_ok "$claim: the check command \"$check\" resolves to \"$resolved\" on a clean PATH, not to the artifact's own ${words[0]}"
    return
  fi
  ok "$claim: the manifest's check command \"$check\" resolves to the artifact's own binary on a clean PATH"

  if ! command -v git >/dev/null 2>&1; then
    unevaluable "$claim: the check command judges a fixture repository" "git is not on PATH, so no fixture repository can be made"
    return
  fi
  local repo="$RUN_DIR/f2/repo" home="$RUN_DIR/f2/home" log="$RUN_DIR/f2/check.sarif" status
  mkdir -p "$home"
  if ! plant_f2_fixture "$repo" >"$RUN_DIR/f2/plant.log" 2>&1; then
    cat "$RUN_DIR/f2/plant.log" >&2
    unevaluable "$claim: the check command judges a fixture repository" "git could not make the fixture repository at $repo"
    return
  fi
  ( cd "$repo" && env -i PATH="$(artifact_path)" HOME="$home" "${words[@]}" ) >"$log" 2>"$log.err"
  status=$?
  case "$status" in
    0|2) ok "$claim: \"$check\" judged the fixture's one staged change, the deletion of a test, and exited $status" ;;
    *)
      cat "$log.err" >&2
      not_ok "$claim: \"$check\" exited $status on the fixture repository, which is no verdict (its stderr above)"
      return
      ;;
  esac

  if [ ! -d "$ROOT/node_modules/ajv-draft-04" ]; then
    unevaluable "$claim: the log validates against contracts/vendor/sarif-schema-2.1.0.json" \
      "the contracts' SARIF validator needs ajv-draft-04; run 'npm ci' in $ROOT first"
    return
  fi
  if ! node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$log" >"$RUN_DIR/f2/validate.log" 2>&1; then
    cat "$RUN_DIR/f2/validate.log" >&2
    not_ok "$claim: the log does not validate against contracts/vendor/sarif-schema-2.1.0.json (the validator's errors on stderr)"
    return
  fi
  local results
  results="$(jq '[.runs[] | .results // [] | .[]] | length' "$log")"
  if [ "$results" -lt 1 ]; then
    not_ok "$claim: the log validates but carries no result for a deleted test, so the schema held only the empty envelope"
    return
  fi
  ok "$claim: the log validates against contracts/vendor/sarif-schema-2.1.0.json, with $results result(s)"

  local runs tools
  runs="$(jq '.runs | length' "$log")"
  tools="$(jq -r '[.runs[] | .tool.driver | "\(.name) \(.version)"] | unique | join(", ")' "$log")"
  if [ "$runs" -ge 1 ] && [ "$tools" = "$BED $LOCK_VERSION" ]; then
    ok "$claim: every run's tool component is $BED $LOCK_VERSION, the lock's version"
  else
    not_ok "$claim: the log's tool components are \"$tools\" over $runs run(s), not $BED $LOCK_VERSION"
  fi
}

# ---------------------------------------------------------------------------------------
# F3 standalone install
# ---------------------------------------------------------------------------------------

check_f3() {
  local claim="F3 standalone install"
  need_artifact "$claim" || return

  local dir="$RUN_DIR/f3" clean
  mkdir -p "$dir/home"
  clean="$(artifact_path)"

  # Outside any repository: no directory from here to the root holds a .git.
  local up="$dir"
  while [ "$up" != "/" ] && [ -n "$up" ]; do
    if [ -e "$up/.git" ]; then
      unevaluable "$claim" "the temp root sits inside the git repository at $up, so no directory outside one could be had"
      return
    fi
    up="$(dirname "$up")"
  done

  # No other garden tool on the PATH it runs with: every bed of the umbrella's garden row, and
  # the stem, looked up on that PATH, and only weeder found, in the artifact.
  local beds bed where others=""
  beds="$(html_block "$ROOT/.brand/components.md" '<nav class="gf-garden"' '</nav>' | garden_pills | awk '$1 != "'"$BED"'" {print $1}')"
  if [ -z "$beds" ]; then
    not_ok "$claim: .brand/components.md's reference garden row names no other bed, so there is nothing to keep off the PATH"
    return
  fi
  for bed in $beds plotplot; do
    where="$(env -i PATH="$clean" /bin/sh -c 'command -v "$1"' sh "$bed")"
    [ -n "$where" ] && others="$others $bed ($where)"
  done
  if [ -n "$others" ]; then
    not_ok "$claim: the PATH it runs with reaches other garden tools:$others"
    return
  fi
  ok "$claim: the PATH it runs with, the artifact's directory then /usr/bin and /bin, reaches none of $(echo "$beds" | tr '\n' ' ')plotplot"

  local out status
  out="$(cd "$dir" && env -i PATH="$clean" HOME="$dir/home" "$BED" --version 2>&1)"
  status=$?
  if [ "$status" -eq 0 ] && [ "$(echo "$out" | head -1)" = "$BED $LOCK_VERSION" ]; then
    ok "$claim: \"$BED --version\" says \"$(echo "$out" | head -1)\", the lock's version, from a directory outside any repository"
  else
    not_ok "$claim: \"$BED --version\" exited $status saying \"$out\", not \"$BED $LOCK_VERSION\""
  fi
}

# ---------------------------------------------------------------------------------------
# F4 declared context cost
# ---------------------------------------------------------------------------------------

# The description in a SKILL.md's front matter, on one line: a plain or quoted value, or the
# indented lines of a folded or literal block joined by spaces.
skill_description() {
  awk '
    NR == 1 && $0 == "---" { front = 1; next }
    front && $0 == "---" { exit }
    !front { exit }
    block && /^[ \t]/ { sub(/^[ \t]+/, ""); text = text (text == "" ? "" : " ") $0; next }
    block { exit }
    /^description:/ {
      value = $0
      sub(/^description:[ \t]*/, "", value)
      if (value ~ /^[>|][-+]?$/ || value == "") { block = 1; next }
      if (value ~ /^".*"$/ || value ~ /^\047.*\047$/) value = substr(value, 2, length(value) - 2)
      text = value
      exit
    }
    END { print text }
  ' "$1"
}

check_f4() {
  local claim="F4 declared context cost"
  need_manifest "$claim" || return
  local manifest="$ARTIFACT_DIR/garden.json"

  local declared
  declared="$(jq -r 'if (.context | type) == "object" and (.context | has("upfront_tokens")) then (.context.upfront_tokens | tostring) else "absent" end' "$manifest")"
  case "$declared" in
    absent|null)
      not_ok "$claim: garden.json declares no context.upfront_tokens ($declared)"
      return
      ;;
  esac
  ok "$claim: garden.json declares context.upfront_tokens = $declared"

  if [ "$declared" = "0" ]; then
    ok "$claim: the declared cost is zero"
  else
    not_ok "$claim: the declared cost is $declared, not zero"
  fi

  if [ "$(jq 'has("faces") and (.faces | type) == "object" and (.faces | has("mcp"))' "$manifest")" = "false" ]; then
    ok "$claim: garden.json registers no MCP face, so no tool schema is paid at session start"
  else
    not_ok "$claim: garden.json registers an MCP face ($(jq -c '.faces.mcp' "$manifest")), whose tool schemas a session pays before speaking"
  fi

  local skill
  skill="$(jq -r '.faces.skill // empty' "$manifest")"
  if [ -z "$skill" ]; then
    ok "$claim: garden.json registers no skill face, so no description line is paid at session start"
    return
  fi
  if [ ! -f "$ARTIFACT_DIR/$skill" ]; then
    not_ok "$claim: garden.json names the skill $skill, and the artifact does not carry it"
    return
  fi
  local description chars tokens
  description="$(skill_description "$ARTIFACT_DIR/$skill")"
  if [ -z "$description" ]; then
    not_ok "$claim: $skill carries no description in its front matter to measure"
    return
  fi
  chars="$(printf '%s' "$description" | wc -c | tr -d ' ')"
  tokens=$(( (chars + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN ))
  if [ "$declared" -ge "$tokens" ] 2>/dev/null; then
    ok "$claim: the $skill description is $chars characters, ~$tokens tokens at $CHARS_PER_TOKEN characters per token, within the $declared declared"
  else
    not_ok "$claim: the $skill description is $chars characters, ~$tokens tokens at $CHARS_PER_TOKEN characters per token, over the $declared the manifest declares"
  fi
}

# ---------------------------------------------------------------------------------------
# F5 metric committed
# ---------------------------------------------------------------------------------------

# The tree of the pinned tag, from the release's own source archive, left in SOURCE_DIR.
# 0 when unpacked, 1 when the tag has no archive, 3 when it could not be fetched.
SOURCE_DIR=""
fetch_source() {
  local archive="$RUN_DIR/f5/source.tar.gz" status
  mkdir -p "$RUN_DIR/f5/source"
  fetch "https://github.com/$REPO_SLUG/archive/refs/tags/$TAG.tar.gz" "$archive"
  status=$?
  [ "$status" -eq 0 ] || return "$status"
  fit_extract "$archive" "$RUN_DIR/f5/source" || return 1
  SOURCE_DIR="$(find "$RUN_DIR/f5/source" -mindepth 1 -maxdepth 1 -type d | head -1)"
  [ -n "$SOURCE_DIR" ]
}

check_f5() {
  local claim="F5 metric committed"
  need_manifest "$claim" || return

  local metric
  metric="$(jq -r '.metric // empty' "$ARTIFACT_DIR/garden.json")"
  if [ -z "$metric" ]; then
    not_ok "$claim: garden.json names no metric command"
    return
  fi
  ok "$claim: garden.json names the metric command \"$metric\""

  need_tag "$claim" || return
  local status
  fetch_source
  status=$?
  case "$status" in
    0) ;;
    1) not_ok "$claim: $REPO_SLUG has no source archive for the tag $TAG"; return ;;
    *) unevaluable "$claim: the metric command runs at $TAG" "could not fetch $REPO_SLUG's source archive for $TAG from this machine"; return ;;
  esac

  # Every file in the tree is set to one old moment, so whatever the metric writes, even the
  # same bytes it found, is newer than the mark and is known to be its result.
  find "$SOURCE_DIR" -exec touch -t 200001010000 {} + || { unevaluable "$claim: the metric command runs at $TAG" "could not reset the times in $SOURCE_DIR"; return; }
  local mark="$RUN_DIR/f5/mark"
  touch -t 200001010001 "$mark"

  mkdir -p "$RUN_DIR/f5/home"
  ( cd "$SOURCE_DIR" && env -i PATH="$(artifact_path)" HOME="$RUN_DIR/f5/home" /bin/bash -c "$metric" ) \
    >"$RUN_DIR/f5/metric.out" 2>"$RUN_DIR/f5/metric.err"
  status=$?
  case "$status" in
    0) ok "$claim: \"$metric\" runs in the tree of $TAG with no other garden tool on PATH: $(tail -1 "$RUN_DIR/f5/metric.out")" ;;
    3)
      cat "$RUN_DIR/f5/metric.err" >&2
      unevaluable "$claim: the metric command runs at $TAG" "\"$metric\" exited 3, could not run on this machine (its stderr above)"
      return
      ;;
    *)
      cat "$RUN_DIR/f5/metric.err" >&2
      not_ok "$claim: \"$metric\" exited $status in the tree of $TAG (its stderr above)"
      return
      ;;
  esac

  local written
  written="$(cd "$SOURCE_DIR" && find . -type f -newer "$mark" | sed 's#^\./##' | LC_ALL=C sort)"
  if [ -z "$written" ]; then
    not_ok "$claim: \"$metric\" wrote no file, so it has no result to be committed"
    return
  fi

  # Each file the metric wrote is committed at the tag, and the tag's copy is what the metric
  # just computed: the latest result is the committed one.
  local path committed stale=0 numbers stated=""
  while IFS= read -r path; do
    committed="$RUN_DIR/f5/committed/$path"
    raw_read "$path" "$committed"
    status=$?
    case "$status" in
      0) ;;
      1) not_ok "$claim: \"$metric\" wrote $path, and $TAG does not carry it: its result is not committed"; stale=1; continue ;;
      *) unevaluable "$claim: $path is committed at $TAG" "could not read $path at $TAG from raw.githubusercontent.com"; stale=1; continue ;;
    esac
    if cmp -s "$committed" "$SOURCE_DIR/$path"; then
      ok "$claim: $path, which the metric wrote, is committed at $TAG byte for byte"
    else
      diff "$committed" "$SOURCE_DIR/$path" >&2
      not_ok "$claim: $path at $TAG is not what the metric computes from the same tree (the difference on stderr): the committed result is stale"
      stale=1
      continue
    fi
    # The calibration numbers: a blocked count, a false-positive count and a percentage, as
    # numbers at the top level of a JSON result.
    if jq -e 'type == "object"' "$committed" >/dev/null 2>&1; then
      numbers="$(jq -r '
        [to_entries[] | select(.value | type == "number")] as $n
        | [($n[] | select(.key | test("^blocked"))), ($n[] | select(.key | test("false_positive"))), ($n[] | select(.key | test("percent$")))]
        | if (map(.key) | map(test("^blocked")) | any) and (map(.key) | map(test("false_positive")) | any) and (map(.key) | map(test("percent$")) | any)
          then map("\(.key) \(.value)") | unique | join(", ") else empty end
      ' "$committed")"
      if [ -n "$numbers" ]; then
        ok "$claim: $path states the calibration numbers: $numbers"
        stated="$path"
      fi
    fi
  done <<< "$written"
  [ "$stale" -eq 0 ] || return
  [ -n "$stated" ] \
    || not_ok "$claim: no file the metric wrote states a blocked count, a false-positive count and a percentage as JSON numbers ($(echo "$written" | tr '\n' ' '))"
}

# ---------------------------------------------------------------------------------------
# F6 brand layer and footer
# ---------------------------------------------------------------------------------------

check_f6() {
  local claim="F6 brand layer and footer"
  need_tag "$claim" || return

  # The product layer and its two marks, named the way every product layer in the umbrella's
  # own .brand/products/ names them.
  local layer=".brand/products/$BED" file status missing="" unread=""
  for file in identity.md colors.md voice.md "assets/$BED-mark.svg" "assets/$BED-mark-night.svg"; do
    raw_read "$layer/$file" "$RUN_DIR/f6/$layer/$file"
    status=$?
    case "$status" in
      0)
        if [ ! -s "$RUN_DIR/f6/$layer/$file" ]; then
          missing="$missing $file (empty)"
        elif [ "${file%.svg}" != "$file" ] && ! grep -q '<svg' "$RUN_DIR/f6/$layer/$file"; then
          missing="$missing $file (no svg)"
        fi
        ;;
      1) missing="$missing $file" ;;
      *) unread="$unread $file" ;;
    esac
  done
  if [ -n "$missing" ]; then
    not_ok "$claim: $layer/ at $TAG lacks:$missing"
  elif [ -n "$unread" ]; then
    unevaluable "$claim: $layer/ at $TAG" "could not read$unread from raw.githubusercontent.com"
  else
    ok "$claim: $layer/ at $TAG carries identity.md, colors.md, voice.md, assets/$BED-mark.svg and assets/$BED-mark-night.svg"
  fi

  local page="$RUN_DIR/f6/index.html"
  raw_read "index.html" "$page"
  status=$?
  case "$status" in
    0) ;;
    1) not_ok "$claim: $TAG carries no index.html"; return ;;
    *) unevaluable "$claim: the page's garden footer" "could not read index.html at $TAG from raw.githubusercontent.com"; return ;;
  esac

  local reference="$ROOT/.brand/components.md" footer="$RUN_DIR/f6/footer.html"
  html_block "$page" '<footer class="gf"' '</footer>' > "$footer"
  if [ ! -s "$footer" ]; then
    not_ok "$claim: index.html at $TAG carries no garden footer (<footer class=\"gf\">)"
    return
  fi

  # The plotplot band: the reference's band, and its link home.
  local home
  home="$(html_block "$reference" '<div class="gf-plot"' '</div>' | grep -o 'class="gf-plotbrand" href="[^"]*"' | head -1)"
  if [ -z "$home" ]; then
    not_ok "$claim: .brand/components.md's reference footer carries no plotplot band to check against"
  elif grep -q '<div class="gf-plot"' "$footer" && grep -qF "$home" "$footer"; then
    ok "$claim: the footer carries the plotplot band, $home"
  else
    not_ok "$claim: the footer carries no plotplot band with $home"
  fi

  # The garden row: the same beds as the reference's, and weeder's pill the one current.
  local want got current
  want="$(html_block "$reference" '<nav class="gf-garden"' '</nav>' | garden_pills | awk '{print $1}' | LC_ALL=C sort)"
  got="$(html_block "$footer" '<nav class="gf-garden"' '</nav>' | garden_pills)"
  current="$(echo "$got" | awk '$2 == "current" {print $1}' | tr '\n' ' ' | sed 's/ $//')"
  got="$(echo "$got" | awk 'NF {print $1}' | LC_ALL=C sort)"
  if [ -z "$want" ]; then
    not_ok "$claim: .brand/components.md's reference footer carries no garden row to check against"
  elif [ "$got" = "$want" ]; then
    ok "$claim: the footer's garden row lists the $(echo "$want" | wc -l | tr -d ' ') beds of the reference: $(echo "$want" | tr '\n' ' ' | sed 's/ $//')"
  else
    diff <(echo "$want") <(echo "$got") >&2
    not_ok "$claim: the footer's garden row is not the reference's (reference < > page on stderr)"
  fi
  if [ "$current" = "$BED" ]; then
    ok "$claim: $BED's pill is the one current"
  else
    not_ok "$claim: the current pill is \"$current\", not $BED alone"
  fi
}

# ---------------------------------------------------------------------------------------

case "${1:-}" in
  ""|f1|f2|f3|f4|f5|f6) ;;
  *)
    echo "unevaluable $1: this script proves the fit contract (f1 to f6); the claim '$1' has no evidence in it yet"
    echo "weeder.sh: '$1' is not implemented; the loop's check stays open until it is" >&2
    exit 3
    ;;
esac

for tool in curl jq; do
  command -v "$tool" >/dev/null 2>&1 && continue
  echo "unevaluable $BED fit contract"
  echo "weeder.sh: $tool is not on PATH, and every claim reads the release through it" >&2
  exit 3
done

RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit.XXXXXX")" || {
  echo "unevaluable $BED fit contract"
  echo "weeder.sh: could not make a temp directory" >&2
  exit 3
}

case "${1:-}" in
  "") check_f1; check_f2; check_f3; check_f4; check_f5; check_f6 ;;
  f1) check_f1 ;;
  f2) check_f2 ;;
  f3) check_f3 ;;
  f4) check_f4 ;;
  f5) check_f5 ;;
  f6) check_f6 ;;
esac

exit "$STATUS"
