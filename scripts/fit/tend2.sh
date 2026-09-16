#!/usr/bin/env bash
# scripts/fit/tend2.sh: tend2's fit evidence, the contract F1 to F6 of
# docs/building-the-garden.md §3, against the revision contracts/fixtures/garden.lock pins.
#
#   scripts/fit/tend2.sh        every claim below, in order
#   scripts/fit/tend2.sh f1     manifest: the clone at the lock's rev carries garden.json at its root;
#                               it validates against contracts/manifest.schema.json, names tend2,
#                               and its version is the lock's
#   scripts/fit/tend2.sh f2     SARIF: the manifest's check command, run from the clone's build on
#                               the map it names, emits a log that validates against the vendored
#                               SARIF 2.1.0 schema, whose tool is tend2 at the manifest's version;
#                               the log judges the map's pages rather than refusing to read the
#                               path, and the same command on a fixture map with one page that has
#                               no title carries a result for that page
#   scripts/fit/tend2.sh f3     standalone install: on a PATH that reaches no other garden tool, the
#                               clone installs from its lockfile, every smoke command the manifest
#                               declares exits as declared, and tend2 --version reports the
#                               manifest's version
#   scripts/fit/tend2.sh f4     declared context cost: context.upfront_tokens is declared, and every
#                               skill description a session loads plus the MCP tools as a client
#                               lists them cost no more, at four characters per token
#   scripts/fit/tend2.sh f5     metric committed: the manifest's metric command runs in the clone,
#                               and every file it writes is committed at the pinned rev byte for
#                               byte (where the result names the commit it measured, it is
#                               reproduced at that commit)
#   scripts/fit/tend2.sh f6     brand layer and footer: the pinned rev commits tend2's product layer
#                               and both marks, its page passes petals/scripts/check.sh against this
#                               repository's .brand with zero errors, and its footer carries the
#                               plotplot band and the garden row of .brand/components.md with
#                               tend2's pill current
#
# One line per assertion: `ok`, `not ok` or `unevaluable`, then the claim and what was seen.
# Exit 0 when every assertion passed, 1 on any failure, 3 when none failed but a claim could not
# be evaluated on this machine (the reason goes to stderr). A claim that cannot be proven
# standalone is unevaluable, never passed.
#
# Everything is read from one clone of the repository the lock names, at the rev it pins, made
# by fit_clone into a scratch directory; never from a tend2 on this machine. The rev and the
# version come from the lock, and the check, metric, smoke commands, faces and declared cost
# from the clone's garden.json, so the next revision is checked by moving the lock and nothing
# here. Every command of the clone runs with an emptied environment and a PATH of a scratch bin
# directory and the system directories (fit_clean_path). node, npm and npx are the system's
# JavaScript toolchain for this purpose: the directory they live in may hold other garden tools,
# so the bin directory links those three alone, beside links to the bins the clone's
# package.json declares. Scratch goes away with `trash`, never `rm` (fit_discard_scratch).

set -uo pipefail

TEND2_SH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/fit/lib.sh
. "$TEND2_SH_DIR/lib.sh"

ROOT="$(fit_repo_root)"
LOCK="$ROOT/contracts/fixtures/garden.lock"
BED="tend2"

# The run's verdict so far: 0 while every assertion passed, 3 once one was unevaluable, 1 once
# one failed. A failure outranks an unevaluable claim.
STATUS=0

# The garden's cost basis for a description line, the one scripts/fit/weeder.sh uses.
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
  echo "tend2.sh: $claim: $why" >&2
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
# the clone at the pinned rev
# ---------------------------------------------------------------------------------------

# Set once by resolve_clone: the lock's version, url and rev, and the clone. CLONE_STATE is ok,
# not_ok or unevaluable, with the reason in CLONE_WHY.
CLONE_STATE=""
CLONE_WHY=""
LOCK_VERSION=""
GIT_URL=""
GIT_REV=""
CLONE=""

resolve_clone() {
  [ -n "$CLONE_STATE" ] && return
  if [ ! -d "$ROOT/node_modules/smol-toml" ]; then
    CLONE_STATE="unevaluable" CLONE_WHY="the lock reader needs smol-toml; run 'npm ci' in $ROOT first"
    return
  fi
  local lookup
  if ! lookup="$(fit_lock_lookup "$LOCK" "$BED" 2>&1)"; then
    CLONE_STATE="not_ok" CLONE_WHY="the lock pins no $BED: $lookup"
    return
  fi
  LOCK_VERSION="$(echo "$lookup" | jq -r '.version // empty')"
  GIT_URL="$(echo "$lookup" | jq -r '.git.url // empty')"
  GIT_REV="$(echo "$lookup" | jq -r '.git.rev // empty')"
  if [ -z "$LOCK_VERSION" ] || [ -z "$GIT_URL" ] || [ -z "$GIT_REV" ]; then
    CLONE_STATE="unevaluable" CLONE_WHY="the lock's $BED entry names no version, git url and rev, and this script reads $BED from a clone: $lookup"
    return
  fi
  CLONE="$RUN_DIR/clone"
  if fit_clone "$GIT_URL" "$GIT_REV" "$CLONE" 2>"$RUN_DIR/clone.err"; then
    CLONE_STATE="ok"
    return
  fi
  # A repository that answers but will not give the rev is a finding; one that does not answer
  # is this machine's network.
  if GIT_TERMINAL_PROMPT=0 git ls-remote -q "$GIT_URL" HEAD >/dev/null 2>&1; then
    CLONE_STATE="not_ok" CLONE_WHY="$GIT_URL answers and does not serve the lock's rev: $(tail -1 "$RUN_DIR/clone.err")"
  else
    CLONE_STATE="unevaluable" CLONE_WHY="could not reach $GIT_URL from this machine: $(tail -1 "$RUN_DIR/clone.err")"
  fi
}

# Resolve the clone for a claim, and say so on that claim's line when it cannot be had.
need_clone() {
  local claim="$1"
  resolve_clone
  case "$CLONE_STATE" in
    ok) return 0 ;;
    not_ok) not_ok "$claim: $CLONE_WHY" ;;
    *) unevaluable "$claim" "$CLONE_WHY" ;;
  esac
  return 1
}

# The clone's garden.json, for a claim that reads it; F1 is the claim that proves it.
need_manifest() {
  local claim="$1"
  need_clone "$claim" || return 1
  [ -f "$CLONE/garden.json" ] && jq -e . "$CLONE/garden.json" >/dev/null 2>&1 && return 0
  not_ok "$claim: the clone at $GIT_REV carries no readable garden.json at its root"
  return 1
}

# ---------------------------------------------------------------------------------------
# the clean toolchain and the build
# ---------------------------------------------------------------------------------------

# Set once by resolve_toolchain: the scratch bin directory and the PATH every command of the
# clone runs with.
TOOLCHAIN_STATE=""
TOOLCHAIN_WHY=""
BIN_DIR=""
CLEAN_PATH=""

resolve_toolchain() {
  [ -n "$TOOLCHAIN_STATE" ] && return
  BIN_DIR="$RUN_DIR/bin"
  mkdir -p "$BIN_DIR" "$RUN_DIR/home"
  local tool found real
  for tool in node npm npx; do
    if ! found="$(command -v "$tool")"; then
      TOOLCHAIN_STATE="unevaluable" TOOLCHAIN_WHY="$tool is not on this machine's PATH, and $BED builds, tests and runs with it"
      return
    fi
    real="$(node -e 'process.stdout.write(require("fs").realpathSync(process.argv[1]))' "$found")" || real="$found"
    ln -s "$real" "$BIN_DIR/$tool"
  done
  # The bins the clone's package.json declares, linked to their files in the clone; the links
  # dangle until the build writes their targets.
  local name target
  while IFS=$'\t' read -r name target; do
    [ -n "$name" ] && ln -s "$CLONE/$target" "$BIN_DIR/$name"
  done < <(jq -r '.bin // {} | if type == "string" then {"'"$BED"'": .} else . end | to_entries[] | "\(.key)\t\(.value)"' "$CLONE/package.json" 2>/dev/null)
  CLEAN_PATH="$(fit_clean_path "$BIN_DIR")"
  TOOLCHAIN_STATE="ok"
}

# Run a command in a directory with the emptied environment and the clean PATH.
run_clean() {
  local dir="$1"
  shift
  ( cd "$dir" && env -i PATH="$CLEAN_PATH" HOME="$RUN_DIR/home" "$@" )
}

# Where a command name resolves on the clean PATH, or nothing.
clean_which() {
  env -i PATH="$CLEAN_PATH" /bin/sh -c 'command -v "$1"' sh "$1"
}

# Set once by resolve_install and resolve_build.
INSTALL_STATE=""
INSTALL_WHY=""
BUILD_STATE=""
BUILD_WHY=""

resolve_install() {
  [ -n "$INSTALL_STATE" ] && return
  resolve_toolchain
  if [ "$TOOLCHAIN_STATE" != "ok" ]; then
    INSTALL_STATE="$TOOLCHAIN_STATE" INSTALL_WHY="$TOOLCHAIN_WHY"
    return
  fi
  if [ ! -f "$CLONE/package.json" ] || [ ! -f "$CLONE/package-lock.json" ]; then
    INSTALL_STATE="not_ok" INSTALL_WHY="the clone carries no package.json and package-lock.json to install from"
    return
  fi
  if run_clean "$CLONE" npm ci >"$RUN_DIR/install.log" 2>&1; then
    INSTALL_STATE="ok"
  else
    tail -20 "$RUN_DIR/install.log" >&2
    INSTALL_STATE="not_ok" INSTALL_WHY="'npm ci' failed in the clone with the clean PATH (its log's tail on stderr)"
  fi
}

resolve_build() {
  [ -n "$BUILD_STATE" ] && return
  resolve_install
  if [ "$INSTALL_STATE" != "ok" ]; then
    BUILD_STATE="$INSTALL_STATE" BUILD_WHY="$INSTALL_WHY"
    return
  fi
  if ! jq -e '.scripts.build' "$CLONE/package.json" >/dev/null 2>&1; then
    BUILD_STATE="ok"
    return
  fi
  if run_clean "$CLONE" npm run build >"$RUN_DIR/build.log" 2>&1; then
    BUILD_STATE="ok"
  else
    tail -20 "$RUN_DIR/build.log" >&2
    BUILD_STATE="not_ok" BUILD_WHY="'npm run build' failed in the clone with the clean PATH (its log's tail on stderr)"
  fi
}

# Resolve the built clone for a claim that runs it.
need_build() {
  local claim="$1"
  resolve_build
  case "$BUILD_STATE" in
    ok) return 0 ;;
    not_ok) not_ok "$claim: $BUILD_WHY" ;;
    *) unevaluable "$claim" "$BUILD_WHY" ;;
  esac
  return 1
}

# ---------------------------------------------------------------------------------------
# reading pages and skills
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

# The characters on stdin, counted as characters rather than bytes.
char_count() {
  node -e 'let s = ""; process.stdin.on("data", (d) => { s += d; }).on("end", () => process.stdout.write(String([...s].length)))'
}

# ---------------------------------------------------------------------------------------
# F1 manifest
# ---------------------------------------------------------------------------------------

check_f1() {
  local claim="F1 manifest"
  need_clone "$claim" || return
  ok "$claim: $GIT_URL cloned at the lock's rev $GIT_REV"

  local manifest="$CLONE/garden.json"
  if [ ! -f "$manifest" ]; then
    not_ok "$claim: the clone carries no garden.json at its root"
    return
  fi
  ok "$claim: the clone carries garden.json at its root"

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

# Validate a SARIF log and name its tool; prints why on failure. 0 valid and tend2 at the
# manifest's version, 1 otherwise.
sarif_holds() {
  local log="$1" version="$2" tools
  if ! node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$log" >"$log.validate" 2>&1; then
    cat "$log.validate" >&2
    echo "does not validate against contracts/vendor/sarif-schema-2.1.0.json (the validator's errors on stderr)"
    return 1
  fi
  tools="$(jq -r '[.runs[] | .tool.driver | "\(.name) \(.version)"] | unique | join(", ")' "$log")"
  if [ "$(jq '.runs | length' "$log")" -lt 1 ] || [ "$tools" != "$BED $version" ]; then
    echo "validates, and its tool components are \"$tools\", not $BED $version"
    return 1
  fi
  echo "validates against contracts/vendor/sarif-schema-2.1.0.json, and every run's tool component is $BED $version"
}

check_f2() {
  local claim="F2 SARIF"
  need_manifest "$claim" || return
  local manifest="$CLONE/garden.json"

  local check version
  check="$(jq -r '.check // empty' "$manifest")"
  version="$(jq -r '.version // empty' "$manifest")"
  if [ -z "$check" ]; then
    not_ok "$claim: garden.json names no check command, and $BED is a gate"
    return
  fi
  need_build "$claim" || return

  # The command exactly as the manifest spells it, resolved through the clean PATH, so it can
  # only be the clone's own build that answers.
  local words resolved
  read -r -a words <<< "$check"
  resolved="$(clean_which "${words[0]}")"
  if [ "$resolved" != "$BIN_DIR/${words[0]}" ]; then
    not_ok "$claim: the check command \"$check\" resolves to \"$resolved\" on the clean PATH, not to the clone's own ${words[0]}"
    return
  fi
  ok "$claim: the manifest's check command \"$check\" resolves to the clone's own build on the clean PATH"

  if [ ! -d "$ROOT/node_modules/ajv-draft-04" ]; then
    unevaluable "$claim: the log validates against contracts/vendor/sarif-schema-2.1.0.json" \
      "the contracts' SARIF validator needs ajv-draft-04; run 'npm ci' in $ROOT first"
    return
  fi

  # On the clone's own map. lint exits 0 clean and 1 on findings; either is a verdict.
  local dir="$RUN_DIR/f2" log status said
  mkdir -p "$dir"
  log="$dir/map.sarif"
  run_clean "$CLONE" "${words[@]}" >"$log" 2>"$log.err"
  status=$?
  case "$status" in
    0|1) ok "$claim: \"$check\" ran in the clone and exited $status" ;;
    *)
      cat "$log.err" >&2
      not_ok "$claim: \"$check\" exited $status in the clone, which is no verdict (its stderr above)"
      return
      ;;
  esac
  if said="$(sarif_holds "$log" "$version")"; then
    ok "$claim: the log on the clone's map $said"
  else
    not_ok "$claim: the log on the clone's map $said"
    return
  fi

  # A path the command could not read is a refusal, not a judgement of the pages under it.
  local refused
  refused="$(jq -r '[.runs[] | .results // [] | .[] | select(.ruleId == "unreadable") | .locations[]?.physicalLocation.artifactLocation.uri] | unique | join(", ")' "$log")"
  if [ -n "$refused" ]; then
    not_ok "$claim: the log's results say the check could not read $refused, so \"$check\" judged no page of the map ($(jq -c '[.runs[] | .results // [] | .[] | .ruleId] | group_by(.) | map({(.[0]): length}) | add // {}' "$log"))"
  else
    ok "$claim: the log judges the map's pages, with $(jq '[.runs[] | .results // [] | .[]] | length' "$log") result(s) and no unreadable path"
  fi

  # On a fixture map at the same paths, with one page whose payload has no title: the log must
  # carry a result for that page, so the schema held a finding and not only an empty envelope.
  local fixture="$dir/fixture" word page="" fixture_log="$dir/fixture.sarif"
  mkdir -p "$fixture"
  for word in "${words[@]:1}"; do
    case "$word" in -*) continue ;; esac
    if [ -d "$CLONE/$word" ]; then
      page="$word/untitled.tend2.html"
    elif [ -f "$CLONE/$word" ]; then
      page="$(dirname "$word")/untitled.tend2.html"
    else
      continue
    fi
    break
  done
  if [ -z "$page" ]; then
    not_ok "$claim: \"$check\" names no path in the clone to plant a fixture map at"
    return
  fi
  mkdir -p "$fixture/$(dirname "$page")"
  cat > "$fixture/$page" <<'PAGE'
<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>untitled · fixture</title>
<body>
<script type="text/markdown" id="loop">
**Goal.** A page with no title line, so lint has one finding to report.

## Tests
- [ ] (code) the fixture exists · scripts/fixture.sh
</script>
PAGE
  run_clean "$fixture" "${words[@]}" >"$fixture_log" 2>"$fixture_log.err"
  status=$?
  if [ "$status" -ne 0 ] && [ "$status" -ne 1 ]; then
    cat "$fixture_log.err" >&2
    not_ok "$claim: \"$check\" exited $status on the fixture map, which is no verdict (its stderr above)"
    return
  fi
  if ! said="$(sarif_holds "$fixture_log" "$version")"; then
    not_ok "$claim: the log on the fixture map $said"
    return
  fi
  local found
  found="$(jq -r --arg page "$page" '[.runs[] | .results // [] | .[] | select(.ruleId != "unreadable") | select([.locations[]?.physicalLocation.artifactLocation.uri] | index($page)) | .ruleId] | unique | join(", ")' "$fixture_log")"
  if [ -n "$found" ]; then
    ok "$claim: on a fixture map with an untitled page at $page, the log $said, and carries $found for that page"
  else
    not_ok "$claim: on a fixture map with an untitled page at $page, the log carries no result for that page (its results: $(jq -c '[.runs[] | .results // [] | .[] | {ruleId, uri: .locations[0]?.physicalLocation.artifactLocation.uri}]' "$fixture_log"))"
  fi
}

# ---------------------------------------------------------------------------------------
# F3 standalone install
# ---------------------------------------------------------------------------------------

check_f3() {
  local claim="F3 standalone install"
  need_manifest "$claim" || return
  local manifest="$CLONE/garden.json"
  resolve_toolchain
  if [ "$TOOLCHAIN_STATE" != "ok" ]; then
    unevaluable "$claim" "$TOOLCHAIN_WHY"
    return
  fi

  # No other garden tool on the PATH it runs with: every bed of the umbrella's garden row, and
  # the stem, looked up on that PATH, and only tend2 found, in the scratch bin directory.
  local beds bed where others=""
  beds="$(html_block "$ROOT/.brand/components.md" '<nav class="gf-garden"' '</nav>' | garden_pills | awk '$1 != "'"$BED"'" {print $1}')"
  if [ -z "$beds" ]; then
    not_ok "$claim: .brand/components.md's reference garden row names no other bed, so there is nothing to keep off the PATH"
    return
  fi
  for bed in $beds plotplot; do
    where="$(clean_which "$bed")"
    [ -n "$where" ] && others="$others $bed ($where)"
  done
  if [ -n "$others" ]; then
    not_ok "$claim: the clean PATH reaches other garden tools:$others"
    return
  fi
  ok "$claim: the clean PATH, links to node, npm, npx and the clone's bins then /usr/bin and /bin, reaches none of $(echo "$beds" | tr '\n' ' ')plotplot"

  resolve_install
  case "$INSTALL_STATE" in
    ok) ok "$claim: 'npm ci' installs the clone from its lockfile with the clean PATH" ;;
    not_ok) not_ok "$claim: $INSTALL_WHY"; return ;;
    *) unevaluable "$claim" "$INSTALL_WHY"; return ;;
  esac

  local count
  count="$(jq '.smoke.commands // [] | length' "$manifest")"
  if [ "$count" -eq 0 ]; then
    not_ok "$claim: garden.json declares no smoke commands, so nothing proves its own suite runs"
    return
  fi
  local i argv timeout expect cmd status
  for (( i = 0; i < count; i++ )); do
    argv=()
    while IFS= read -r cmd; do argv+=("$cmd"); done < <(jq -r ".smoke.commands[$i].argv[]" "$manifest")
    timeout="$(jq -r ".smoke.commands[$i].timeout_ms // 5000" "$manifest")"
    expect="$(jq -r ".smoke.commands[$i].expect_exit // 0" "$manifest")"
    cmd="${argv[*]}"
    if [ -z "$(clean_which "${argv[0]}")" ]; then
      unevaluable "$claim: the smoke command \"$cmd\"" "it needs ${argv[0]}, which the clean PATH cannot provide"
      continue
    fi
    # perl's alarm outlives its exec, so a command past its declared limit is stopped (exit 142).
    run_clean "$CLONE" perl -e 'alarm(shift); exec { $ARGV[0] } @ARGV or exit 127' "$(( (timeout + 999) / 1000 ))" "${argv[@]}" \
      >"$RUN_DIR/smoke-$i.log" 2>&1
    status=$?
    if [ "$status" -eq "$expect" ]; then
      ok "$claim: the smoke command \"$cmd\" exits $status, as declared: $(grep -v '^[[:space:]]*$' "$RUN_DIR/smoke-$i.log" | tail -1 | cut -c1-160)"
    elif [ "$status" -eq 142 ]; then
      not_ok "$claim: the smoke command \"$cmd\" ran past its declared ${timeout} ms and was stopped"
    else
      tail -30 "$RUN_DIR/smoke-$i.log" >&2
      not_ok "$claim: the smoke command \"$cmd\" exits $status, not the declared $expect (its log's tail on stderr)"
    fi
  done

  local cli version out
  cli="$(jq -r '.faces.cli // empty' "$manifest")"
  version="$(jq -r '.version // empty' "$manifest")"
  if [ -z "$cli" ] || [ "$(clean_which "$cli")" != "$BIN_DIR/$cli" ]; then
    not_ok "$claim: the manifest's cli face \"$cli\" is no bin the clone's package.json declares"
    return
  fi
  out="$(run_clean "$CLONE" "$cli" --version 2>&1)"
  status=$?
  case "$status:$(echo "$out" | head -1)" in
    "0:$version"|"0:$cli $version") ok "$claim: \"$cli --version\" from the clone's build says \"$(echo "$out" | head -1)\", the manifest's version" ;;
    *) not_ok "$claim: \"$cli --version\" exited $status saying \"$out\", not the manifest's $version" ;;
  esac
}

# ---------------------------------------------------------------------------------------
# F4 declared context cost
# ---------------------------------------------------------------------------------------

# Every SKILL.md a session loads, relative to the clone, one to a line: the one the manifest
# names, and every skill under the skill directories of each plugin the clone ships (the
# plugin manifest's `skills` paths, or its default skills/ directory).
skill_files() {
  local manifest="$CLONE/garden.json" named plugin_json plugin_root dirs dir
  {
    named="$(jq -r '.faces.skill // empty' "$manifest")"
    [ -n "$named" ] && echo "$named"
    while IFS= read -r plugin_json; do
      plugin_root="$(dirname "$(dirname "$plugin_json")")"
      dirs="$(jq -r 'if (.skills | type) == "string" then .skills elif (.skills | type) == "array" then .skills[] else "skills" end' "$CLONE/$plugin_json")"
      while IFS= read -r dir; do
        dir="${dir#./}"
        [ -d "$CLONE/$plugin_root/$dir" ] || continue
        ( cd "$CLONE" && find "$plugin_root/$dir" -name SKILL.md -not -path '*/node_modules/*' )
      done <<< "$dirs"
    done < <(cd "$CLONE" && find . -path ./node_modules -prune -o -path ./.git -prune -o -path '*/.claude-plugin/plugin.json' -print | sed 's#^\./##')
  } | sed 's#^\./##' | LC_ALL=C sort -u
}

# The MCP tools as a client lists them, measured in characters of their JSON, read in process:
# the module the clone's .mcp.json launches for the bed is imported for its createServer, which
# is joined to a client through the SDK's in-memory transport, so no server process starts and
# nothing listens. Prints "<tools> <characters>"; on failure prints why and returns 1.
mcp_tool_chars() {
  local module
  module="$(jq -r --arg bed "$BED" '.mcpServers[$bed] // empty | select(.command == "node") | .args[0] // empty' "$CLONE/.mcp.json" 2>/dev/null)"
  if [ -z "$module" ] || [ ! -f "$CLONE/$module" ]; then
    echo "the clone's .mcp.json names no built node module for the $BED server, so its tool definitions cannot be read without starting it"
    return 1
  fi
  # The module path travels in the environment, never in argv: a module may start its server
  # when its own name is the script argument, as tend2's does. stdin is empty, so a server that
  # starts anyway reads end of file and stops.
  run_clean "$CLONE" FIT_MCP_MODULE="$module" node --input-type=module -e '
    const path = process.env.FIT_MCP_MODULE;
    const module = await import(new URL(path, `file://${process.cwd()}/`).href);
    if (typeof module.createServer !== "function") {
      process.stdout.write(`${path} exports no createServer, so its tool definitions cannot be read without starting it`);
      process.exit(1);
    }
    const { InMemoryTransport } = await import("@modelcontextprotocol/sdk/inMemory.js");
    const { Client } = await import("@modelcontextprotocol/sdk/client/index.js");
    const [serverSide, clientSide] = InMemoryTransport.createLinkedPair();
    const server = module.createServer();
    await server.connect(serverSide);
    const client = new Client({ name: "plotplot-fit", version: "0.0.0" });
    await client.connect(clientSide);
    const { tools } = await client.listTools();
    process.stdout.write(`${tools.length} ${JSON.stringify(tools).length}`);
    await client.close();
    await server.close();
  ' </dev/null 2>"$RUN_DIR/f4-mcp.err"
}

check_f4() {
  local claim="F4 declared context cost"
  need_manifest "$claim" || return
  local manifest="$CLONE/garden.json"

  local declared
  declared="$(jq -r 'if (.context | type) == "object" and (.context | has("upfront_tokens")) then (.context.upfront_tokens | tostring) else "absent" end' "$manifest")"
  case "$declared" in
    absent|null)
      not_ok "$claim: garden.json declares no measured context.upfront_tokens ($declared)"
      return
      ;;
  esac
  ok "$claim: garden.json declares context.upfront_tokens = $declared"

  local skills skill description chars skill_chars=0 described=0 missing=""
  skills="$(skill_files)"
  while IFS= read -r skill; do
    [ -n "$skill" ] || continue
    if [ ! -f "$CLONE/$skill" ]; then
      missing="$missing $skill (absent)"
      continue
    fi
    description="$(skill_description "$CLONE/$skill")"
    if [ -z "$description" ]; then
      missing="$missing $skill (no description)"
      continue
    fi
    chars="$(printf '%s' "$description" | char_count)"
    skill_chars=$(( skill_chars + chars ))
    described=$(( described + 1 ))
  done <<< "$skills"
  if [ -n "$missing" ]; then
    not_ok "$claim: skills the manifest and the plugin declare cannot be measured:$missing"
    return
  fi
  local skill_tokens=$(( (skill_chars + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN ))
  ok "$claim: $described skill description(s) a session loads are $skill_chars characters, ~$skill_tokens tokens"

  local tool_tokens=0
  if [ "$(jq '(.faces.mcp // null) != null' "$manifest")" = "true" ]; then
    need_build "$claim" || return
    local measured tools tool_chars declared_tools
    if ! measured="$(mcp_tool_chars)"; then
      cat "$RUN_DIR/f4-mcp.err" >&2
      unevaluable "$claim: the MCP tools' names and descriptions" \
        "measured the skills alone at ~$skill_tokens tokens; $measured"
      return
    fi
    tools="${measured%% *}"
    tool_chars="${measured##* }"
    tool_tokens=$(( (tool_chars + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN ))
    declared_tools="$(jq -r '.faces.mcp.tools' "$manifest")"
    if [ "$tools" = "$declared_tools" ]; then
      ok "$claim: the MCP face lists $tools tools, as garden.json declares, whose names, descriptions and schemas are $tool_chars characters, ~$tool_tokens tokens"
    else
      not_ok "$claim: the MCP face lists $tools tools, and garden.json declares $declared_tools"
    fi
  fi

  local total=$(( skill_tokens + tool_tokens ))
  if [ "$total" -le "$declared" ]; then
    ok "$claim: the measured cost, ~$total tokens at $CHARS_PER_TOKEN characters per token, is within the $declared declared"
  else
    not_ok "$claim: the measured cost, ~$total tokens at $CHARS_PER_TOKEN characters per token ($skill_tokens for skills, $tool_tokens for tools), is over the $declared garden.json declares"
  fi
}

# ---------------------------------------------------------------------------------------
# F5 metric committed
# ---------------------------------------------------------------------------------------

# Run the metric in a tree and print, one to a line, the files it wrote there. Every file
# outside .git and node_modules older than the mark is known not to be its result.
run_metric() {
  local tree="$1" metric="$2" log="$3" mark="$3.mark"
  touch "$mark"
  sleep 1
  run_clean "$tree" /bin/bash -c "$metric" >"$log" 2>"$log.err"
  local status=$?
  ( cd "$tree" && find . \( -path ./.git -o -path ./node_modules \) -prune -o -type f -newer "$mark" -print ) \
    | sed 's#^\./##' | LC_ALL=C sort >"$log.written"
  return "$status"
}

check_f5() {
  local claim="F5 metric committed"
  need_manifest "$claim" || return

  local metric
  metric="$(jq -r '.metric // empty' "$CLONE/garden.json")"
  if [ -z "$metric" ]; then
    not_ok "$claim: garden.json names no metric command"
    return
  fi
  need_build "$claim" || return

  local dir="$RUN_DIR/f5" status
  mkdir -p "$dir"
  run_metric "$CLONE" "$metric" "$dir/at-pin.log"
  status=$?
  if [ "$status" -ne 0 ]; then
    cat "$dir/at-pin.log.err" >&2
    not_ok "$claim: \"$metric\" exited $status in the clone (its stderr above)"
    return
  fi
  ok "$claim: \"$metric\" runs in the clone at $GIT_REV: $(tail -1 "$dir/at-pin.log")"
  if [ ! -s "$dir/at-pin.log.written" ]; then
    not_ok "$claim: \"$metric\" wrote no file, so it has no result to be committed"
    return
  fi

  local path committed short named at
  while IFS= read -r path; do
    committed="$dir/committed/$path"
    mkdir -p "$(dirname "$committed")"
    if ! git -C "$CLONE" show "$GIT_REV:$path" >"$committed" 2>/dev/null; then
      not_ok "$claim: \"$metric\" wrote $path, and $GIT_REV does not commit it: its result is not committed"
      continue
    fi
    if cmp -s "$committed" "$CLONE/$path"; then
      ok "$claim: $path, which the metric wrote, is committed at $GIT_REV byte for byte"
      continue
    fi

    # A result that names the commit it was measured at cannot be committed at that commit. The
    # committed copy then names an earlier one: when the tree there differs from the pin only in
    # what the metric writes, the metric is run there and must write the committed bytes.
    short="$(grep -oE '[0-9a-f]{7,40}' "$CLONE/$path" | awk -v rev="$GIT_REV" 'index(rev, $0) == 1' | head -1)"
    if [ -z "$short" ]; then
      diff "$committed" "$CLONE/$path" >&2
      not_ok "$claim: $path at $GIT_REV is not what the metric computes from the same tree (the difference on stderr): the committed result is stale"
      continue
    fi
    if [ "$(git -C "$CLONE" rev-parse --is-shallow-repository)" = "true" ] \
      && ! GIT_TERMINAL_PROMPT=0 git -C "$CLONE" fetch -q --unshallow origin >/dev/null 2>&1; then
      unevaluable "$claim: $path is the metric's result" \
        "the metric names the commit it ran at ($short), so its result is reproduced at the commit the committed copy names, and the history to find that commit could not be fetched from $GIT_URL"
      continue
    fi
    named=""
    for at in $(grep -oE '[0-9a-f]{7,40}' "$committed"); do
      at="$(git -C "$CLONE" rev-parse -q --verify "$at^{commit}" 2>/dev/null)" || continue
      [ "$at" != "$GIT_REV" ] && git -C "$CLONE" merge-base --is-ancestor "$at" "$GIT_REV" && { named="$at"; break; }
    done
    if [ -z "$named" ]; then
      diff "$committed" "$CLONE/$path" >&2
      not_ok "$claim: the metric's $path names the commit it ran at ($short), and the committed copy names no earlier commit of $GIT_REV's history it was measured at (the difference on stderr)"
      continue
    fi
    local changed
    changed="$(git -C "$CLONE" diff --name-only "$named" "$GIT_REV" | grep -vxF -f "$dir/at-pin.log.written")"
    if [ -n "$changed" ]; then
      not_ok "$claim: the committed $path was measured at $named, and the tree has changed since in $(echo "$changed" | head -5 | tr '\n' ' '): the committed result is not the latest"
      continue
    fi
    if ! git -C "$CLONE" worktree add -q --detach "$dir/at-named" "$named" >/dev/null 2>&1; then
      unevaluable "$claim: $path is the metric's result at $named" "git could not check out $named beside the clone"
      continue
    fi
    run_metric "$dir/at-named" "$metric" "$dir/at-named.log"
    status=$?
    if [ "$status" -ne 0 ]; then
      cat "$dir/at-named.log.err" >&2
      not_ok "$claim: \"$metric\" exited $status at $named, the commit the committed $path names (its stderr above)"
    elif cmp -s "$committed" "$dir/at-named/$path"; then
      ok "$claim: $path names the commit it measured, $named, whose tree is $GIT_REV's apart from the result; the metric run there writes the committed bytes"
    else
      diff "$committed" "$dir/at-named/$path" >&2
      not_ok "$claim: $path committed at $GIT_REV was measured at $named, whose tree is the pin's apart from the result, and the metric run there writes other bytes (committed < > rerun on stderr): the committed result is not reproducible"
    fi
  done <"$dir/at-pin.log.written"
}

# ---------------------------------------------------------------------------------------
# F6 brand layer and footer
# ---------------------------------------------------------------------------------------

check_f6() {
  local claim="F6 brand layer and footer"
  need_clone "$claim" || return

  # The product layer and its two marks, named the way every product layer in the umbrella's
  # own .brand/products/ names them, committed at the pin.
  local layer=".brand/products/$BED" file missing=""
  for file in identity.md colors.md voice.md "assets/$BED-mark.svg" "assets/$BED-mark-night.svg"; do
    if ! git -C "$CLONE" cat-file -e "$GIT_REV:$layer/$file" 2>/dev/null; then
      missing="$missing $file"
    elif [ ! -s "$CLONE/$layer/$file" ]; then
      missing="$missing $file (empty)"
    elif [ "${file%.svg}" != "$file" ] && ! grep -q '<svg' "$CLONE/$layer/$file"; then
      missing="$missing $file (no svg)"
    fi
  done
  if [ -n "$missing" ]; then
    not_ok "$claim: $layer/ at $GIT_REV lacks:$missing"
  else
    ok "$claim: $layer/ is committed at $GIT_REV with identity.md, colors.md, voice.md, assets/$BED-mark.svg and assets/$BED-mark-night.svg"
  fi

  local page="$CLONE/index.html"
  if [ ! -f "$page" ]; then
    not_ok "$claim: the clone at $GIT_REV carries no index.html"
    return
  fi

  local errors
  ( cd "$ROOT" && bash petals/scripts/check.sh "$page" --brand "$ROOT/.brand" ) >"$RUN_DIR/f6-petals.log" 2>&1
  errors=$?
  if [ "$errors" -eq 0 ]; then
    ok "$claim: index.html passes petals/scripts/check.sh against this repository's .brand with zero errors"
  else
    grep -E '^(ERROR|Result)' "$RUN_DIR/f6-petals.log" >&2
    not_ok "$claim: index.html fails petals/scripts/check.sh against this repository's .brand: $(grep '^Result' "$RUN_DIR/f6-petals.log" | head -1) (the errors on stderr)"
  fi

  local reference="$ROOT/.brand/components.md" footer="$RUN_DIR/f6-footer.html"
  html_block "$page" '<footer class="gf"' '</footer>' >"$footer"
  if [ ! -s "$footer" ]; then
    not_ok "$claim: index.html carries no garden footer (<footer class=\"gf\">)"
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

  # The garden row: the same beds as the reference's, and tend2's pill the one current.
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
    echo "unevaluable $1: this script proves f1 to f6; the claim '$1' has no evidence in it yet"
    echo "tend2.sh: '$1' is not implemented; the loop's check stays open until it is" >&2
    exit 3
    ;;
esac

for tool in git jq node perl; do
  command -v "$tool" >/dev/null 2>&1 && continue
  echo "unevaluable $BED fit contract"
  echo "tend2.sh: $tool is not on PATH, and every claim reads the clone through it" >&2
  exit 3
done

RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit.XXXXXX")" || {
  echo "unevaluable $BED fit contract"
  echo "tend2.sh: could not make a temp directory" >&2
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
