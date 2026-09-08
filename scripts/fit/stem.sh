#!/usr/bin/env bash
# scripts/fit/stem.sh: the stem's fit evidence, one check per subcommand.
#
#   scripts/fit/stem.sh bundles      the three generated bundles install through each vendor's
#                                    own mechanism and carry the same hooks, skills and MCP
#                                    entries as the ones the stem generated
#   scripts/fit/stem.sh hook-faces   every bundle registers the dispatcher on every event a
#                                    friction kind derives from and on the session-end event,
#                                    and `plotplot hook` answers each in under 50 ms
#
# Exit 0 on pass, non-zero with a reason on failure. Every check builds the crate in release
# mode once, then runs in a fresh temporary directory with HOME and CODEX_HOME pointed at a
# temporary home, so nothing on this machine's user scope is read or written. Scratch goes
# away with `trash`, never `rm` (scripts/fit/lib.sh, fit_discard_scratch).
#
# Where a vendor command needs a login or the network and fails for that reason rather than
# because of the bundle, the check fails and says which vendor and why. Nothing is skipped.

set -uo pipefail

STEM_SH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/fit/lib.sh
. "$STEM_SH_DIR/lib.sh"

ROOT="$(fit_repo_root)"
STEM="$ROOT/target/release/plotplot"

# The bundle every vendor installs, and the marketplace the two marketplace vendors read it
# from. One name, so a failure message never has to say which of two spellings it meant.
PLUGIN="plotplot"
MARKETPLACE="plotplot-fit"

# Every temporary directory this run minted, cleaned up by the EXIT trap in the order they
# were made. bash 3.2 has no associative arrays and no `mapfile`; a plain array is enough.
SCRATCH_DIRS=()

# Where the installer that just ran put the bundle. Written by `install_through_*`, read by
# `check_bundles`; see the comment there for why this is not a return value.
INSTALLED=""

# ---------------------------------------------------------------------------------------
# saying what happened
# ---------------------------------------------------------------------------------------

fail() {
  echo "stem.sh: $*" >&2
  exit 1
}

note() {
  echo "  $*"
}

# A vendor command that failed for a reason of its own rather than because of the bundle:
# a login it wants, a network it cannot reach, a version that does not have the subcommand.
# Named as such and still a failure, because a check that goes quiet proves nothing.
vendor_fail() {
  local vendor="$1" what="$2" log="$3"
  echo "stem.sh: $vendor could not $what" >&2
  echo "--- $vendor said ---" >&2
  tail -20 "$log" >&2
  echo "--------------------" >&2
  exit 1
}

# ---------------------------------------------------------------------------------------
# scratch, and the temporary user scope every check runs against
# ---------------------------------------------------------------------------------------

scratch() {
  local dir
  dir="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit.XXXXXX")" || fail "could not make a temp directory"
  SCRATCH_DIRS+=("$dir")
  echo "$dir"
}

discard_scratch() {
  local dir
  for dir in "${SCRATCH_DIRS[@]+"${SCRATCH_DIRS[@]}"}"; do
    [ -d "$dir" ] || continue
    fit_discard_scratch "$dir"
  done
}
trap discard_scratch EXIT

# The path a vendor writes into its own configuration, which on macOS is the resolved one
# (/private/var/... rather than /var/...). Comparisons against a vendor's own records use
# this, so a symlinked temp root never reads as a mismatch.
real_path() {
  ( cd "$1" 2>/dev/null && pwd -P ) || echo "$1"
}

require() {
  local tool
  for tool in "$@"; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool is not on PATH, and this check needs it"
  done
}

# ---------------------------------------------------------------------------------------
# the crate, built once per invocation
# ---------------------------------------------------------------------------------------

build_stem() {
  note "building the stem in release mode"
  ( cd "$ROOT" && cargo build --release ) >/dev/null 2>&1 \
    || fail "cargo build --release failed; run it yourself to see why"
  [ -x "$STEM" ] || fail "no release binary at $STEM after a successful build"
  note "stem: $("$STEM" version | head -1)"
}

# ---------------------------------------------------------------------------------------
# the fixture repository both checks plant
# ---------------------------------------------------------------------------------------

# A planted repository holding the contracts' own two manifests: weeder, which declares hooks
# on all three harnesses and three git hooks, and tilth, which declares a skill and an MCP
# face with no launch line. Both come from `contracts/fixtures/manifest/`, unedited, so the
# bundles this check installs are generated from the same manifests the contracts' own tests
# read.
#
# `.plotplot/bin/weeder` is a fixture judge, not a stub of the stem: weeder has no release
# yet (the comment at the head of `contracts/fixtures/garden.lock` says so), so there is no
# real binary to fetch, and a bed the manifest registers but nobody can run makes the
# dispatcher answer "cannot judge" — which is right, and which would hide what these checks
# are about. It speaks the bed contract the umbrella ruled on 2026-09-08 exactly: it reads
# the payload on stdin and exits 0 to allow. Every line of the stem it exercises is real.
plant_fixture() {
  local repo="$1" bed
  mkdir -p "$repo" || fail "could not make the fixture repository at $repo"
  git init -q "$repo" || fail "git init failed in $repo"

  cp "$ROOT/contracts/fixtures/garden.lock" "$repo/garden.lock" \
    || fail "could not copy the fixture garden.lock"

  for bed in weeder tilth; do
    mkdir -p "$repo/.plotplot/beds/$bed"
    cp "$ROOT/contracts/fixtures/manifest/$bed.garden.json" "$repo/.plotplot/beds/$bed/garden.json" \
      || fail "could not copy the $bed manifest"
  done

  # Each bed's SKILL.md where its own manifest says it is: weeder at the artifact root,
  # tilth under skills/.
  mkdir -p "$repo/.plotplot/beds/tilth/skills"
  cat > "$repo/.plotplot/beds/weeder/SKILL.md" <<'SKILL'
# weeder

The judge of the diff: reads what an agent produced and refuses dishonest growth, as SARIF.
SKILL
  cat > "$repo/.plotplot/beds/tilth/skills/SKILL.md" <<'SKILL'
# tilth

Code intelligence: tree-sitter indexed lookups, smart code reading for agents.
SKILL

  mkdir -p "$repo/.plotplot/bin"
  cat > "$repo/.plotplot/bin/weeder" <<'JUDGE'
#!/bin/sh
# The fixture judge. The bed contract, and nothing more: read the payload, allow.
cat >/dev/null
exit 0
JUDGE
  chmod +x "$repo/.plotplot/bin/weeder"
}

build_bundles() {
  local repo="$1" log="$2"
  ( cd "$repo" && "$STEM" bundle build ) >"$log" 2>&1 \
    || { echo "--- plotplot bundle build ---" >&2; cat "$log" >&2; fail "plotplot bundle build failed"; }
}

# ---------------------------------------------------------------------------------------
# reading a bundle: the three sets a bundle carries
# ---------------------------------------------------------------------------------------

# Where each vendor keeps its hook file inside a bundle (the brief's §5 table).
hooks_file() {
  case "$1" in
    claude|gemini) echo "hooks/hooks.json" ;;
    codex) echo "hooks.json" ;;
    *) fail "no hook file is known for $1" ;;
  esac
}

# Where each vendor keeps its MCP servers: Claude and Codex in `.mcp.json`, Gemini inside its
# own extension manifest. A bundle with no channel bed to plant has no MCP entries at all,
# and an absent file reads as the empty set rather than as a failure.
bundle_mcp_names() {
  local bundle="$1" harness="$2"
  case "$harness" in
    claude|codex)
      [ -f "$bundle/.mcp.json" ] || return 0
      jq -r '(.mcpServers // {}) | keys[]' "$bundle/.mcp.json" | LC_ALL=C sort
      ;;
    gemini)
      [ -f "$bundle/gemini-extension.json" ] || return 0
      jq -r '(.mcpServers // {}) | keys[]' "$bundle/gemini-extension.json" | LC_ALL=C sort
      ;;
  esac
}

# The whole MCP block, with the vendor's own root variable normalised away, so the three can
# be compared to each other rather than only to themselves.
bundle_mcp_normalised() {
  local bundle="$1" harness="$2" file
  case "$harness" in
    claude|codex) file="$bundle/.mcp.json" ;;
    gemini) file="$bundle/gemini-extension.json" ;;
  esac
  if [ ! -f "$file" ]; then
    echo '{}'
    return 0
  fi
  jq -S '.mcpServers // {}' "$file" \
    | sed -e 's/\${CLAUDE_PLUGIN_ROOT}/${BUNDLE_ROOT}/g' -e 's/\${extensionPath}/${BUNDLE_ROOT}/g'
}

bundle_events() {
  local bundle="$1" harness="$2" file
  file="$bundle/$(hooks_file "$harness")"
  [ -f "$file" ] || fail "$harness bundle has no $(hooks_file "$harness")"
  jq -r '.hooks | keys[]' "$file" | LC_ALL=C sort
}

# Every hook command in a bundle, with the vendor's root variable normalised away.
bundle_commands() {
  local bundle="$1" harness="$2" file
  file="$bundle/$(hooks_file "$harness")"
  jq -r '.hooks | to_entries[] | .value[] | .hooks[] | .command' "$file" \
    | sed -e 's/\${CLAUDE_PLUGIN_ROOT}/${BUNDLE_ROOT}/g' -e 's/\${extensionPath}/${BUNDLE_ROOT}/g' \
    | LC_ALL=C sort
}

# Each skill the bundle carries, as `<name> <sha256 of its SKILL.md>`, so two bundles carrying
# the same names but different content do not read as equal.
bundle_skills() {
  local bundle="$1" dir name
  [ -d "$bundle/skills" ] || return 0
  for dir in "$bundle"/skills/*/; do
    [ -d "$dir" ] || continue
    name="$(basename "$dir")"
    [ -f "$dir/SKILL.md" ] || fail "the skill $name carries no SKILL.md"
    echo "$name $(fit_sha256 "$dir/SKILL.md")"
  done | LC_ALL=C sort
}

# One bundle against another, on the three sets the loop names. Used twice: a generated
# bundle against the copy a vendor installed, and the three generated bundles against each
# other on the sets that do not depend on a vendor's own spelling.
same_sets() {
  local left="$1" right="$2" harness="$3" what="$4" set_name
  for set_name in events commands skills mcp; do
    local a b
    case "$set_name" in
      events)
        a="$(bundle_events "$left" "$harness")"; b="$(bundle_events "$right" "$harness")"
        [ -n "$a" ] || fail "$what: the generated bundle registers no events at all"
        ;;
      commands) a="$(bundle_commands "$left" "$harness")"; b="$(bundle_commands "$right" "$harness")" ;;
      skills) a="$(bundle_skills "$left")"; b="$(bundle_skills "$right")" ;;
      mcp) a="$(bundle_mcp_normalised "$left" "$harness")"; b="$(bundle_mcp_normalised "$right" "$harness")" ;;
    esac
    if [ "$a" != "$b" ]; then
      echo "stem.sh: $what: the $set_name differ" >&2
      diff <(echo "$a") <(echo "$b") >&2
      exit 1
    fi
  done
}

# ---------------------------------------------------------------------------------------
# the events, mapped off the vendors' own names
# ---------------------------------------------------------------------------------------

# The five things every harness fires, whatever it calls them. Every other event a bundle
# registers is one the other two harnesses do not have at all, and `shared_events` checks
# that rather than letting an unmapped name pass unnoticed.
#
# Sources: `contracts/friction-profile.md`, "the pinned kinds", the derived-from column; the
# brief's §5 event lists; and the 2026-09-08 correction on the stem loop's Tried, that Codex
# has no SessionEnd and its session end rides Stop.
semantic_event() {
  local harness="$1" event="$2"
  case "$harness:$event" in
    claude:PreToolUse|gemini:BeforeTool|codex:PreToolUse) echo "before-tool" ;;
    claude:PostToolUse|gemini:AfterTool|codex:PostToolUse) echo "after-tool" ;;
    claude:Stop|gemini:AfterAgent|codex:Stop) echo "stop" ;;
    claude:SessionEnd|gemini:SessionEnd) echo "session-end" ;;
    claude:PreCompact|gemini:PreCompress|codex:PreCompact) echo "compaction" ;;
    *) echo "" ;;
  esac
}

# The events a harness has that the other two do not, which are therefore allowed to be in
# one bundle's hook file and not in another's.
harness_only_event() {
  local harness="$1" event="$2"
  case "$harness:$event" in
    claude:PostToolUseFailure|claude:PermissionDenied) return 0 ;;
    *) return 1 ;;
  esac
}

# One bundle's events, as the five shared names. Codex's Stop is both its stop and its
# session end, so it earns both.
shared_events() {
  local bundle="$1" harness="$2" event semantic
  {
    for event in $(bundle_events "$bundle" "$harness"); do
      semantic="$(semantic_event "$harness" "$event")"
      if [ -z "$semantic" ]; then
        harness_only_event "$harness" "$event" \
          || fail "the $harness bundle registers \"$event\", which is neither one of the five events every harness fires nor one only $harness has"
        continue
      fi
      echo "$semantic"
      if [ "$harness:$event" = "codex:Stop" ]; then
        echo "session-end"
      fi
    done
  } | LC_ALL=C sort -u
}

# ---------------------------------------------------------------------------------------
# the temporary user scope
# ---------------------------------------------------------------------------------------

# A home no vendor has ever written to, and the two variables the three vendors read to find
# it. Exported by the caller, never set globally here, so a helper cannot forget one.
make_home() {
  local tmp="$1"
  mkdir -p "$tmp/home" "$tmp/home/.codex" || fail "could not make the temporary home"
  echo "$tmp/home"
}

# ---------------------------------------------------------------------------------------
# check: bundles
# ---------------------------------------------------------------------------------------

install_through_claude() {
  local tmp="$1" repo="$2" home="$3" market log installed
  market="$tmp/marketplace-claude"
  log="$tmp/claude.log"
  mkdir -p "$market/.claude-plugin" "$market/plugins"
  cp -R "$repo/.plotplot/bundles/claude" "$market/plugins/$PLUGIN" \
    || fail "could not stage the claude bundle into the marketplace"

  # Claude Code 2.1.265 takes a marketplace-relative path string as a plugin's `source`; the
  # object forms its schema documents are for remote sources and it refuses them here.
  cat > "$market/.claude-plugin/marketplace.json" <<JSON
{
  "name": "$MARKETPLACE",
  "owner": { "name": "plotplot" },
  "plugins": [
    {
      "name": "$PLUGIN",
      "description": "the garden, planted",
      "source": "./plugins/$PLUGIN"
    }
  ]
}
JSON

  ( cd "$repo" && HOME="$home" claude plugin marketplace add "$market" ) >"$log" 2>&1 \
    || vendor_fail "claude" "add the local marketplace at $market" "$log"

  ( cd "$repo" && HOME="$home" claude plugin install "$PLUGIN@$MARKETPLACE" --scope project --yes ) >>"$log" 2>&1 \
    || vendor_fail "claude" "install $PLUGIN@$MARKETPLACE at project scope" "$log"

  # What Claude wrote under the temporary home: the plugin's own record, holding the path it
  # unpacked the bundle to.
  local record="$home/.claude/plugins/installed_plugins.json"
  [ -f "$record" ] || vendor_fail "claude" "record the install (no $record)" "$log"
  installed="$(jq -r --arg id "$PLUGIN@$MARKETPLACE" '.plugins[$id][0].installPath // empty' "$record")"
  [ -n "$installed" ] || vendor_fail "claude" "name an install path for $PLUGIN@$MARKETPLACE" "$log"
  [ -d "$installed" ] || fail "claude recorded an install path that is not there: $installed"

  # And what it wrote into the project, which is what project scope means.
  local settings="$repo/.claude/settings.json"
  [ -f "$settings" ] || fail "claude wrote no project settings at $settings"
  [ "$(jq -r --arg id "$PLUGIN@$MARKETPLACE" '.enabledPlugins[$id] // false' "$settings")" = "true" ] \
    || fail "claude did not enable $PLUGIN@$MARKETPLACE in $settings"

  INSTALLED="$installed"
}

install_through_gemini() {
  local tmp="$1" repo="$2" home="$3" bundle log installed
  bundle="$repo/.plotplot/bundles/gemini"
  log="$tmp/gemini.log"

  # Gemini CLI 0.46.0 asks, on a terminal it does not have here, whether the source folder is
  # trusted, and its `--consent` flag covers only the extension warning. The answer lives in
  # the temporary home's own trusted-folders file, so the question is settled the way a
  # planter settles it and nothing outside this run is trusted.
  mkdir -p "$home/.gemini"
  jq -n --arg bundle "$(real_path "$bundle")" --arg repo "$(real_path "$repo")" \
    '{($bundle): "TRUST_FOLDER", ($repo): "TRUST_FOLDER"}' > "$home/.gemini/trustedFolders.json" \
    || fail "could not write the temporary home's trusted folders"

  ( cd "$repo" && HOME="$home" gemini extensions install "$bundle" --consent --skip-settings </dev/null ) >"$log" 2>&1 \
    || vendor_fail "gemini" "install the extension at $bundle" "$log"

  ( cd "$repo" && HOME="$home" gemini extensions validate "$bundle" </dev/null ) >>"$log" 2>&1 \
    || vendor_fail "gemini" "validate the extension at $bundle" "$log"

  installed="$home/.gemini/extensions/$PLUGIN"
  [ -d "$installed" ] || vendor_fail "gemini" "unpack the extension to $installed" "$log"
  INSTALLED="$installed"
}

install_through_codex() {
  local tmp="$1" repo="$2" home="$3" market log installed codex_home
  market="$tmp/marketplace-codex"
  log="$tmp/codex.log"
  codex_home="$home/.codex"
  mkdir -p "$market/.agents/plugins" "$market/plugins" "$codex_home"
  cp -R "$repo/.plotplot/bundles/codex" "$market/plugins/$PLUGIN" \
    || fail "could not stage the codex bundle into the marketplace"

  cat > "$market/.agents/plugins/marketplace.json" <<JSON
{
  "name": "$MARKETPLACE",
  "plugins": [
    {
      "name": "$PLUGIN",
      "source": { "source": "local", "path": "./plugins/$PLUGIN" }
    }
  ]
}
JSON

  # Codex loads a plugin's hooks only for a project it trusts (the brief's §5), so the
  # fixture repository is trusted in the temporary CODEX_HOME and nowhere else.
  printf '[projects."%s"]\ntrust_level = "trusted"\n' "$(real_path "$repo")" > "$codex_home/config.toml"

  ( cd "$repo" && HOME="$home" CODEX_HOME="$codex_home" codex plugin marketplace add "$market" </dev/null ) >"$log" 2>&1 \
    || vendor_fail "codex" "add the local marketplace at $market" "$log"

  ( cd "$repo" && HOME="$home" CODEX_HOME="$codex_home" codex plugin add "$PLUGIN@$MARKETPLACE" </dev/null ) >>"$log" 2>&1 \
    || vendor_fail "codex" "install $PLUGIN@$MARKETPLACE" "$log"

  [ "$(grep -c "^\[plugins\.\"$PLUGIN@$MARKETPLACE\"\]" "$codex_home/config.toml")" -ge 1 ] \
    || fail "codex did not record $PLUGIN@$MARKETPLACE in $codex_home/config.toml"

  installed="$(grep -o '^Installed plugin root: .*' "$log" | tail -1 | sed 's/^Installed plugin root: //')"
  [ -n "$installed" ] || vendor_fail "codex" "name the plugin root it installed to" "$log"
  [ -d "$installed" ] || fail "codex named an install root that is not there: $installed"
  INSTALLED="$installed"
}

check_bundles() {
  require jq git claude gemini codex
  build_stem

  local tmp repo home
  tmp="$(scratch)"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"

  note "planting the fixture repository at $repo"
  plant_fixture "$repo"
  build_bundles "$repo" "$tmp/build.log"

  # Each installer is called directly and answers in INSTALLED, never through `$( )`: a
  # command substitution runs in a subshell, and a vendor's own failure has to end the check
  # rather than exit a subshell and leave the reason behind.
  local vendor
  for vendor in claude gemini codex; do
    note "installing the $vendor bundle through $vendor's own mechanism"
    INSTALLED=""
    case "$vendor" in
      claude) install_through_claude "$tmp" "$repo" "$home" ;;
      gemini) install_through_gemini "$tmp" "$repo" "$home" ;;
      codex) install_through_codex "$tmp" "$repo" "$home" ;;
    esac
    [ -n "$INSTALLED" ] || fail "$vendor reported no install path"
    note "$vendor installed to $INSTALLED"
    same_sets "$repo/.plotplot/bundles/$vendor" "$INSTALLED" "$vendor" \
      "the $vendor bundle $vendor installed is not the one the stem generated"
    note "$vendor: the installed bundle carries the same hooks, skills and MCP entries"
  done

  # And the three generated bundles against each other, on the sets that do not depend on a
  # vendor's own spelling.
  local claude_events gemini_events codex_events
  claude_events="$(shared_events "$repo/.plotplot/bundles/claude" claude)"
  gemini_events="$(shared_events "$repo/.plotplot/bundles/gemini" gemini)"
  codex_events="$(shared_events "$repo/.plotplot/bundles/codex" codex)"
  [ "$claude_events" = "$gemini_events" ] \
    || { echo "stem.sh: claude and gemini register different events" >&2; diff <(echo "$claude_events") <(echo "$gemini_events") >&2; exit 1; }
  [ "$claude_events" = "$codex_events" ] \
    || { echo "stem.sh: claude and codex register different events" >&2; diff <(echo "$claude_events") <(echo "$codex_events") >&2; exit 1; }
  note "the three bundles register the same events: $(echo "$claude_events" | tr '\n' ' ')"

  local claude_skills gemini_skills codex_skills
  claude_skills="$(bundle_skills "$repo/.plotplot/bundles/claude")"
  gemini_skills="$(bundle_skills "$repo/.plotplot/bundles/gemini")"
  codex_skills="$(bundle_skills "$repo/.plotplot/bundles/codex")"
  [ -n "$claude_skills" ] || fail "the bundles carry no skills, so comparing them proves nothing"
  [ "$claude_skills" = "$gemini_skills" ] && [ "$claude_skills" = "$codex_skills" ] \
    || { echo "stem.sh: the three bundles carry different skills" >&2; diff <(echo "$claude_skills") <(echo "$gemini_skills") >&2; diff <(echo "$claude_skills") <(echo "$codex_skills") >&2; exit 1; }
  note "the three bundles carry the same skills: $(echo "$claude_skills" | awk '{print $1}' | tr '\n' ' ')"

  local claude_mcp gemini_mcp codex_mcp
  claude_mcp="$(bundle_mcp_normalised "$repo/.plotplot/bundles/claude" claude)"
  gemini_mcp="$(bundle_mcp_normalised "$repo/.plotplot/bundles/gemini" gemini)"
  codex_mcp="$(bundle_mcp_normalised "$repo/.plotplot/bundles/codex" codex)"
  [ "$claude_mcp" = "$gemini_mcp" ] && [ "$claude_mcp" = "$codex_mcp" ] \
    || { echo "stem.sh: the three bundles carry different MCP entries" >&2; diff <(echo "$claude_mcp") <(echo "$gemini_mcp") >&2; diff <(echo "$claude_mcp") <(echo "$codex_mcp") >&2; exit 1; }
  note "the three bundles carry the same MCP entries: $(bundle_mcp_names "$repo/.plotplot/bundles/claude" claude | tr '\n' ' ')(none from these two manifests; tilth declares a tool count and no launch line, and the stem says so on stderr)"

  echo "stem.sh bundles: pass"
}

# ---------------------------------------------------------------------------------------
# check: hook-faces
# ---------------------------------------------------------------------------------------

# Every event a friction kind derives from that this harness has, plus the harness's
# session-end event. Read off `contracts/friction-profile.md`'s pinned-kinds table, mapped
# onto each vendor's own names from the brief's §5, and spelled here rather than asked of the
# binary, so this check can disagree with the binary.
friction_events() {
  case "$1" in
    claude)
      # tool.denied PermissionDenied · tool.failed PostToolUseFailure · tool.retry, test.loop
      # PreToolUse · file.reread, search.fanout, edit.churn PostToolUse · stop.refused Stop ·
      # context.compacted PreCompact · session.ended SessionEnd
      echo "PreToolUse PostToolUse PostToolUseFailure PermissionDenied Stop PreCompact SessionEnd"
      ;;
    gemini)
      # tool.failed rides AfterTool's error · stop.refused is AfterAgent · context.compacted
      # is PreCompress. Gemini fires no PermissionDenied and no PostToolUseFailure.
      echo "BeforeTool AfterTool AfterAgent PreCompress SessionEnd"
      ;;
    codex)
      # Codex 0.133.0 has no SessionEnd, so session.ended rides Stop (stem loop, Tried,
      # 2026-09-08), and it fires neither PermissionDenied nor PostToolUseFailure.
      echo "PreToolUse PostToolUse Stop PreCompact"
      ;;
  esac
}

session_end_event() {
  case "$1" in
    claude|gemini) echo "SessionEnd" ;;
    codex) echo "Stop" ;;
  esac
}

before_tool_event() {
  case "$1" in
    claude|codex) echo "PreToolUse" ;;
    gemini) echo "BeforeTool" ;;
  esac
}

# The payload the vendor would hand the hook, from the corpus under tests/fixtures.
payload_file() {
  echo "$ROOT/tests/fixtures/payloads/$1/$2.json"
}

# The conversation that payload belongs to, read from the payload rather than listed here:
# each fixture names its own session, and two events of one harness need not share one.
session_of() {
  local file="$1" id
  [ -f "$file" ] || fail "no fixture payload at $file"
  id="$(jq -r '.session_id // empty' "$file")"
  [ -n "$id" ] || fail "the fixture payload at $file names no session_id"
  echo "$id"
}

# Which millisecond clock this machine has. BSD `date` has no %N, so python3 leads and
# `date +%s%N` is the fallback where it is real; a machine with neither stops the check
# rather than letting it report a number it did not measure.
pick_clock() {
  if command -v python3 >/dev/null 2>&1; then
    STEM_CLOCK="python3"
    return 0
  fi
  local sample
  sample="$(date +%s%N 2>/dev/null)"
  case "$sample" in
    *[!0-9]*|"") fail "this machine has neither python3 nor a date that understands %N, so a millisecond clock cannot be read" ;;
  esac
  STEM_CLOCK="date"
}

# Milliseconds since the epoch, on whichever clock `pick_clock` found.
now_ms() {
  case "${STEM_CLOCK:-}" in
    python3) python3 -c 'import time; print(int(time.time() * 1000))' ;;
    date) echo $(( $(date +%s%N) / 1000000 )) ;;
    *) fail "no millisecond clock was picked" ;;
  esac
}

# The median of the numbers on stdin, for an odd or even count alike.
median() {
  LC_ALL=C sort -n | awk '{ v[NR] = $1 } END { if (NR % 2) print v[(NR + 1) / 2]; else print int((v[NR / 2] + v[NR / 2 + 1]) / 2) }'
}

dispatch_once() {
  local repo="$1" harness="$2" event="$3" out="$4"
  ( cd "$repo" && "$STEM" hook "$harness" "$event" < "$(payload_file "$harness" "$event")" ) >"$out" 2>"$out.err"
}

check_hook_faces() {
  require jq git
  build_stem
  pick_clock
  note "clock: $STEM_CLOCK"

  local tmp repo
  tmp="$(scratch)"
  repo="$tmp/repo"

  note "planting the fixture repository at $repo"
  plant_fixture "$repo"
  build_bundles "$repo" "$tmp/build.log"

  # 1. Every event a friction kind derives from, and the session-end event, has an entry.
  local harness event registered wanted
  for harness in claude gemini codex; do
    registered="$(bundle_events "$repo/.plotplot/bundles/$harness" "$harness")"
    [ -n "$registered" ] || fail "the $harness bundle's hook file registers no events at all"
    wanted="$(friction_events "$harness") $(session_end_event "$harness")"
    for event in $wanted; do
      echo "$registered" | grep -qx "$event" \
        || fail "the $harness bundle registers no entry for $event, which the friction profile derives a kind from"
      local command
      command="$(jq -r --arg e "$event" '.hooks[$e][0].hooks[0].command' \
        "$repo/.plotplot/bundles/$harness/$(hooks_file "$harness")")"
      case "$command" in
        *"/bin/plotplot\" hook $harness $event") : ;;
        *) fail "the $harness entry for $event is \"$command\", not a dispatcher call for $harness $event" ;;
      esac
    done
    note "$harness: every friction event and the session end dispatch to the stem ($(echo "$wanted" | wc -w | tr -d ' ') entries)"
  done

  # 2. The dispatcher answers, on the before-tool event and on the session-end event.
  local before session out
  for harness in claude gemini codex; do
    before="$(before_tool_event "$harness")"
    session="$(session_end_event "$harness")"

    out="$tmp/$harness.before"
    dispatch_once "$repo" "$harness" "$before" "$out" \
      || { echo "--- stderr ---" >&2; cat "$out.err" >&2; fail "plotplot hook $harness $before did not exit 0"; }

    out="$tmp/$harness.session"
    dispatch_once "$repo" "$harness" "$session" "$out" \
      || { echo "--- stderr ---" >&2; cat "$out.err" >&2; fail "plotplot hook $harness $session did not exit 0"; }

    # The friction emitter wrote the session's end, and the receipt writer drafted it.
    local id journal ended draft
    id="$(session_of "$(payload_file "$harness" "$session")")"
    journal="$(ls "$repo"/.plotplot/friction/*.jsonl 2>/dev/null | head -1)"
    [ -n "$journal" ] || fail "$harness $session wrote no friction journal under $repo/.plotplot/friction"
    ended="$(jq -c --arg id "$id" 'select(.["plotplot.kind"] == "session.ended" and .["gen_ai.conversation.id"] == $id)' "$journal" | wc -l | tr -d ' ')"
    [ "$ended" -ge 1 ] || fail "$harness $session appended no session.ended line for $id to $journal"

    draft="$repo/.plotplot/receipts/drafts/$id.json"
    [ -f "$draft" ] || fail "$harness $session wrote no receipt draft at $draft"
    [ "$(jq -r '.sessions[0]' "$draft")" = "$id" ] \
      || fail "the receipt draft at $draft is not this session's"
    note "$harness: $before and $session answered, session.ended journalled, draft written"
  done

  # 3. And it answers fast enough for the budget the vendors give a hook.
  local budget=50 start end run runs got_median
  for harness in claude gemini codex; do
    before="$(before_tool_event "$harness")"
    runs="$tmp/$harness.runs"
    : > "$runs"
    for run in 1 2 3 4 5 6 7 8 9 10; do
      start="$(now_ms)"
      dispatch_once "$repo" "$harness" "$before" "$tmp/$harness.timing" \
        || fail "plotplot hook $harness $before failed while being timed"
      end="$(now_ms)"
      echo $(( end - start )) >> "$runs"
    done
    got_median="$(median < "$runs")"
    note "$harness $before: median $got_median ms over ten runs ($(tr '\n' ' ' < "$runs"))"
    [ "$got_median" -lt "$budget" ] \
      || fail "$harness $before took a median of $got_median ms, over the ${budget}ms budget"
  done

  echo "stem.sh hook-faces: pass"
}

# ---------------------------------------------------------------------------------------

case "${1:-}" in
  bundles) check_bundles ;;
  hook-faces) check_hook_faces ;;
  *)
    echo "usage: stem.sh {bundles | hook-faces}" >&2
    exit 2
    ;;
esac
