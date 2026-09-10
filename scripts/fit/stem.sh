#!/usr/bin/env bash
# scripts/fit/stem.sh: the stem's fit evidence, one check per subcommand.
#
#   scripts/fit/stem.sh bundles      the three generated bundles install through each vendor's
#                                    own mechanism and carry the same hooks, skills and MCP
#                                    entries as the ones the stem generated
#   scripts/fit/stem.sh hook-faces   every bundle registers the dispatcher on every event a
#                                    friction kind derives from and on the session-end event,
#                                    and `plotplot hook` answers each in under 50 ms
#   scripts/fit/stem.sh lock         `plotplot lock verify` fetches a real pinned release into
#                                    an ignored directory, verifies its checksum, refuses a
#                                    mismatch, and no judge is committed anywhere
#   scripts/fit/stem.sh check        `plotplot check` runs the gate the manifests declare and
#                                    emits one valid SARIF 2.1.0 log; exit 2 on a block-level
#                                    result, 3 when the judge could not run
#   scripts/fit/stem.sh init         `plotplot init` plants a fixture repository: the garden
#                                    block beside tend2's, which it leaves byte for byte, the
#                                    harness project configs, the git law and the pull
#                                    request gate, and a second run changes nothing
#   scripts/fit/stem.sh doctor-live  `plotplot doctor --live` drives one real session per
#                                    harness through umbel and proves from the friction
#                                    journal and the receipt drafts that the before-tool,
#                                    session-end and draft faces fired, reporting a harness
#                                    that would not run with what it said
#   scripts/fit/stem.sh platform     `plotplot init --github` writes the stem's CODEOWNERS
#                                    region and applies the default branch's ruleset once,
#                                    through a recording gh, and `plotplot doctor --platform`
#                                    reads the rules back, reporting a missing rule and a
#                                    missing gh; then this repository's own table, unasserted
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

# The last directory `scratch` minted. See `scratch` for why it is a variable and not stdout.
SCRATCH_DIR=""

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

# Mint a fresh temporary directory and leave its path in SCRATCH_DIR.
#
# It answers in a variable rather than on stdout on purpose: read through a command
# substitution the function runs in a subshell, the entry it adds to SCRATCH_DIRS dies with
# that subshell, and the EXIT trap has nothing to put away. Every check's scratch survived
# its run that way, temporary home and all. Found 2026-09-09 by the doctor-live check, whose
# temporary home holds a copy of a credential and is the one scratch that must never be left
# lying in the temp root.
scratch() {
  SCRATCH_DIR="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit.XXXXXX")" \
    || fail "could not make a temp directory"
  SCRATCH_DIRS+=("$SCRATCH_DIR")
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

  # Each bed's SKILL.md where its own manifest says it is, inside the unpacked artifact:
  # weeder at the artifact root, tilth under skills/.
  mkdir -p "$repo/.plotplot/beds/weeder/artifact" "$repo/.plotplot/beds/tilth/artifact/skills"
  cat > "$repo/.plotplot/beds/weeder/artifact/SKILL.md" <<'SKILL'
# weeder

The judge of the diff: reads what an agent produced and refuses dishonest growth, as SARIF.
SKILL
  cat > "$repo/.plotplot/beds/tilth/artifact/skills/SKILL.md" <<'SKILL'
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
  scratch
  tmp="$SCRATCH_DIR"
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
  scratch
  tmp="$SCRATCH_DIR"
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
# the lockfile wrapper, against a real release
# ---------------------------------------------------------------------------------------

# The judge this check pins. tilth is the one bed of the garden with a published release, so
# it is the only real artifact a lock can name today; its per-platform digests live in the
# contracts' own `contracts/fixtures/garden.lock`, read from the platform on 2026-09-08.
LOCK_JUDGE="tilth"
LOCK_VERSION="0.10.1"

# A lock naming one judge for one platform, written into a fixture repository, never here.
write_lock() {
  local repo="$1" platform="$2" url="$3" sha="$4"
  mkdir -p "$repo" || fail "could not make the fixture repository at $repo"
  cat > "$repo/garden.lock" <<LOCK
season = "2026.09"

[judges.$LOCK_JUDGE]
version = "$LOCK_VERSION"

[judges.$LOCK_JUDGE.platforms."$platform"]
url = "$url"
sha256 = "$sha"
LOCK
}

# One digit of a digest changed, and still 64 lowercase hex, so what the lock refuses is the
# bytes and never its own shape.
bend_digest() {
  local sha="$1" head rest
  head="${sha:0:1}"
  rest="${sha:1}"
  case "$head" in
    0) echo "1$rest" ;;
    *) echo "0$rest" ;;
  esac
}

check_lock() {
  require git jq node
  build_stem

  # fit_lock_lookup is the fit runner's own lock reader, and it reads TOML through
  # smol-toml. An absent node_modules is a machine that has not run `npm ci`, which is a
  # different thing from a platform with no asset, and the two must not read the same.
  [ -d "$ROOT/node_modules/smol-toml" ] \
    || fail "the fit runner's lock reader needs smol-toml; run 'npm ci' in $ROOT first"

  local platform lookup url sha
  platform="$(fit_platform)"
  note "platform: $platform"

  # The artifact comes from the contracts' own lock fixture, so this check pins whatever the
  # contracts pin and never a URL invented here. A platform the fixture has no asset for is
  # said out loud and fails, rather than passing on a judge nobody fetched.
  lookup="$(fit_lock_lookup "$ROOT/contracts/fixtures/garden.lock" "$LOCK_JUDGE" "$platform" 2>&1)" \
    || fail "contracts/fixtures/garden.lock names no $LOCK_JUDGE artifact for $platform, so this check cannot run on this machine: $lookup"
  url="$(echo "$lookup" | jq -r '.url')"
  sha="$(echo "$lookup" | jq -r '.sha256')"
  { [ -n "$url" ] && [ "$url" != "null" ]; } || fail "the fixture lock gave no url for $LOCK_JUDGE on $platform"
  { [ -n "$sha" ] && [ "$sha" != "null" ]; } || fail "the fixture lock gave no sha256 for $LOCK_JUDGE on $platform"
  note "pinned: $url"

  local tmp good bad status
  scratch
  tmp="$SCRATCH_DIR"
  good="$tmp/planted"
  bad="$tmp/bent"

  # 1. The pinned bytes are fetched, verified and placed.
  write_lock "$good" "$platform" "$url" "$sha"
  ( cd "$good" && "$STEM" lock verify ) >"$tmp/good.out" 2>"$tmp/good.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { echo "--- plotplot lock verify ---" >&2; cat "$tmp/good.out" "$tmp/good.err" >&2; \
         fail "plotplot lock verify exited $status against the real $LOCK_JUDGE $LOCK_VERSION release"; }
  grep -q "^$LOCK_JUDGE $LOCK_VERSION fetched" "$tmp/good.out" \
    || { cat "$tmp/good.out" >&2; fail "lock verify did not report $LOCK_JUDGE as fetched"; }

  local placed="$good/.plotplot/bin/$LOCK_JUDGE"
  [ -x "$placed" ] || fail "$placed is not there or not executable after a verify that exited 0"

  local reported
  reported="$("$placed" --version 2>&1)" || fail "the fetched $LOCK_JUDGE could not be run: $reported"
  case "$reported" in
    *"$LOCK_VERSION"*) : ;;
    *) fail "the fetched $LOCK_JUDGE reports \"$reported\", not $LOCK_VERSION" ;;
  esac

  local recorded
  recorded="$(tr -d '[:space:]' < "$placed.sha256")" || fail "no digest companion beside $placed"
  [ "$recorded" = "$sha" ] \
    || fail "the companion at $placed.sha256 records $recorded, not the $sha the lock pins"
  note "fetched, verified and runnable: $(head -1 "$tmp/good.out")"

  # 2. A lock whose digest is off by one digit is refused, and nothing is placed.
  local bent
  bent="$(bend_digest "$sha")"
  [ "$bent" != "$sha" ] || fail "the bent digest came out the same as the pinned one"
  write_lock "$bad" "$platform" "$url" "$bent"
  ( cd "$bad" && "$STEM" lock verify ) >"$tmp/bad.out" 2>"$tmp/bad.err"
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/bad.out" "$tmp/bad.err" >&2; fail "a bent digest made lock verify exit $status, not 3"; }
  grep -q "^$LOCK_JUDGE $LOCK_VERSION mismatch: expected $bent, got $sha$" "$tmp/bad.out" \
    || { cat "$tmp/bad.out" >&2; fail "lock verify did not name the mismatch and both digests"; }
  if [ -d "$bad/.plotplot/bin" ] && [ -n "$(ls -A "$bad/.plotplot/bin" 2>/dev/null)" ]; then
    fail "a refused judge left files under $bad/.plotplot/bin: $(ls -A "$bad/.plotplot/bin")"
  fi
  note "refused: $(head -1 "$tmp/bad.out")"

  # 3. What was fetched is ignored by git, under this repository's own .gitignore.
  cp "$ROOT/.gitignore" "$good/.gitignore" || fail "could not copy this repository's .gitignore"
  git init -q "$good" || fail "git could not make a repository at $good"
  ( cd "$good" && git check-ignore -q ".plotplot/bin/$LOCK_JUDGE" ) \
    || fail "this repository's .gitignore does not ignore .plotplot/bin/$LOCK_JUDGE"
  note "ignored by this repository's .gitignore: .plotplot/bin/$LOCK_JUDGE"

  # 4. And nothing under .plotplot is committed here either.
  local tracked
  tracked="$(git -C "$ROOT" ls-files -- .plotplot)"
  [ -z "$tracked" ] || fail "this repository tracks files under .plotplot: $tracked"
  note "no file under .plotplot is tracked in this repository"

  echo "stem.sh lock: pass"
}

# ---------------------------------------------------------------------------------------
# `plotplot check`, against the real judge
# ---------------------------------------------------------------------------------------

# The judge this check runs. weeder is the garden's gate, and it is the one bed whose check
# face emits SARIF today; it has no release yet (the comment at the head of
# `contracts/fixtures/garden.lock` says so), so there is no artifact for `lock verify` to
# fetch and this check takes the binary the build machine already carries.
CHECK_JUDGE="weeder"

# A fixture repository with one feature and its test, committed, and weeder planted as its
# only gate. The manifest is `contracts/fixtures/manifest/weeder.garden.json`, unedited, so
# the gate command the stem runs is the one the contracts declare.
plant_check_fixture() {
  local repo="$1" judge="$2" digest
  mkdir -p "$repo" || fail "could not make the fixture repository at $repo"
  git init -q "$repo" || fail "git init failed in $repo"
  git -C "$repo" config user.email "fit@plotplot.invalid" || fail "could not configure git"
  git -C "$repo" config user.name "plotplot fit" || fail "could not configure git"

  mkdir -p "$repo/src" "$repo/tests"
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
  git -C "$repo" add -A || fail "could not stage the fixture's first commit"
  git -C "$repo" commit -qm "the feature and its test" || fail "could not commit the fixture"

  mkdir -p "$repo/.plotplot/beds/$CHECK_JUDGE" "$repo/.plotplot/bin"
  cp "$ROOT/contracts/fixtures/manifest/$CHECK_JUDGE.garden.json" \
     "$repo/.plotplot/beds/$CHECK_JUDGE/garden.json" \
    || fail "could not copy the $CHECK_JUDGE manifest"

  # weeder has no release, so nothing can be fetched and verified against a pinned digest.
  # The binary on this machine is placed by hand, its own sha256 recorded in the companion
  # `lock verify` would have written, and `garden.lock` pinned to the same digest, so the
  # fixture is coherent with what a planted repository looks like and the one thing that is
  # not real about it is said out loud, here and on stdout.
  cp "$judge" "$repo/.plotplot/bin/$CHECK_JUDGE" || fail "could not place $judge"
  chmod +x "$repo/.plotplot/bin/$CHECK_JUDGE"
  digest="$(fit_sha256 "$repo/.plotplot/bin/$CHECK_JUDGE")" \
    || fail "could not hash the placed judge"
  printf '%s\n' "$digest" > "$repo/.plotplot/bin/$CHECK_JUDGE.sha256"
  cat > "$repo/garden.lock" <<LOCK
season = "2026.09"

[judges.$CHECK_JUDGE]
version = "$("$judge" --version | awk '{print $2}')"

[judges.$CHECK_JUDGE.platforms."$(fit_platform)"]
url = "file://$repo/.plotplot/bin/$CHECK_JUDGE"
sha256 = "$digest"
LOCK

  note "the judge was pre-placed from the build machine because $CHECK_JUDGE has no release: $judge, sha256 $digest, pinned in garden.lock and recorded beside it"
}

# The names of the tools a merged log's runs carry, one to a line.
log_tools() {
  jq -r '.runs[] | .tool.driver.name' "$1"
}

# How many block-level results a merged log carries.
log_blocks() {
  jq '[.runs[] | .results // [] | .[] | select(.level == "error")] | length' "$1"
}

check_check() {
  require git jq node
  build_stem

  # The vendored SARIF schema is validated through the contracts' own validator, which reads
  # the schema's checksum out of contracts/pins.json first. An absent node_modules is a
  # machine that has not run `npm ci`, not a log that failed to validate, and the two must
  # not read the same.
  [ -d "$ROOT/node_modules/ajv-draft-04" ] \
    || fail "the contracts' SARIF validator needs ajv-draft-04; run 'npm ci' in $ROOT first"

  local judge
  judge="$(command -v "$CHECK_JUDGE")" \
    || fail "$CHECK_JUDGE is not on this machine's PATH, and it has no release for the lock to fetch, so this check has no judge to run; install $CHECK_JUDGE and run this again"
  note "judge on the build machine: $judge ($("$judge" --version))"

  local tmp repo status
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"

  note "planting the fixture repository at $repo"
  plant_check_fixture "$repo" "$judge"

  # 1. A clean tree: every gate ran, nothing blocks, and the log is SARIF 2.1.0.
  ( cd "$repo" && "$STEM" check ) >"$tmp/clean.json" 2>"$tmp/clean.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/clean.err" >&2; fail "plotplot check exited $status on a clean tree, not 0"; }
  node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$tmp/clean.json" >/dev/null \
    || { node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$tmp/clean.json" >&2; \
         fail "the log from a clean tree is not SARIF 2.1.0 by the contracts' vendored schema"; }
  [ "$(log_tools "$tmp/clean.json")" = "$CHECK_JUDGE" ] \
    || { cat "$tmp/clean.json" >&2; fail "the clean tree's log does not carry exactly $CHECK_JUDGE's run"; }
  [ "$(log_blocks "$tmp/clean.json")" -eq 0 ] \
    || { cat "$tmp/clean.json" >&2; fail "the clean tree's log carries a block-level result"; }
  note "clean tree: exit 0, one run by $CHECK_JUDGE, valid SARIF 2.1.0, no block"

  # 2. A change the judge blocks: a deleted test file.
  git -C "$repo" rm -q tests/parser.test.ts || fail "could not delete the fixture's test file"
  ( cd "$repo" && "$STEM" check ) >"$tmp/blocked.json" 2>"$tmp/blocked.err"
  status=$?
  [ "$status" -eq 2 ] \
    || { cat "$tmp/blocked.json" "$tmp/blocked.err" >&2; \
         fail "plotplot check exited $status on a deleted test file, not 2"; }
  node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$tmp/blocked.json" >/dev/null \
    || { node "$ROOT/contracts/test/lib/validate-sarif.mjs" "$tmp/blocked.json" >&2; \
         fail "the log from a blocked change is not SARIF 2.1.0"; }
  [ "$(log_tools "$tmp/blocked.json")" = "$CHECK_JUDGE" ] \
    || { cat "$tmp/blocked.json" >&2; fail "the blocked change's log does not carry exactly $CHECK_JUDGE's run"; }
  [ "$(log_blocks "$tmp/blocked.json")" -ge 1 ] \
    || { cat "$tmp/blocked.json" >&2; fail "the blocked change's log carries no block-level result"; }
  note "deleted test file: exit 2, one run by $CHECK_JUDGE, $(log_blocks "$tmp/blocked.json") block-level result(s)"

  # 3. And with the judge gone, the answer is missing rather than permissive.
  mv "$repo/.plotplot/bin/$CHECK_JUDGE" "$tmp/$CHECK_JUDGE.withdrawn" \
    || fail "could not take the judge out of the fixture"
  ( cd "$repo" && "$STEM" check ) >"$tmp/gone.json" 2>"$tmp/gone.err"
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/gone.json" "$tmp/gone.err" >&2; \
         fail "plotplot check exited $status with the judge removed, not 3 (fail closed)"; }
  grep -q "^$CHECK_JUDGE: " "$tmp/gone.err" \
    || { cat "$tmp/gone.err" >&2; fail "plotplot check did not name $CHECK_JUDGE as the gate that could not run"; }
  [ "$(jq '.runs | length' "$tmp/gone.json")" -eq 0 ] \
    || { cat "$tmp/gone.json" >&2; fail "a gate that could not run still contributed a run to the log"; }
  note "judge removed: exit 3, no run in the log, $(head -1 "$tmp/gone.err")"

  echo "stem.sh check: pass"
}

# ---------------------------------------------------------------------------------------
# check: init
# ---------------------------------------------------------------------------------------

# The bed `plotplot init --profile minimal` plants (jahala/plotplot issue 17).
INIT_JUDGE="weeder"
INIT_VERSION="0.1.0"

# tend2's block as tend2's init writes it into AGENTS.md, copied byte for byte from this
# repository's own AGENTS.md and held to that file by a test in tests/plant.rs. The stem
# never renders it; each init leaves the other's block alone (jahala/plotplot issue 28).
TEND2_BLOCK="$ROOT/tests/fixtures/AGENTS.tend2.md"
AGENTS_PROSE="Notes this repository keeps for itself."

# The status check the default branch's ruleset requires and the pull request gate's job
# reports (platform::REQUIRED_CHECK; the contract ruled 2026-09-10).
REQUIRED_CHECK="garden"

# A fixture repository and the lock template init copies from.
#
# weeder has no release (the comment at the head of contracts/fixtures/garden.lock says so)
# and the lock schema takes only https urls, so there is no artifact any lock could name that
# `lock verify` could fetch here. The judge is placed by hand instead, with its own digest in
# the companion `lock verify` writes and the same digest pinned in the template, which is
# exactly the state a resolved lock leaves behind: verify answers "verified" and fetches
# nothing. That is the one thing about this fixture that is not real, and it is said here and
# on stdout. Everything init does with it afterwards is the real code path.
plant_init_fixture() {
  local repo="$1" template="$2" platform digest
  mkdir -p "$repo" || fail "could not make the fixture repository at $repo"
  git init -q "$repo" || fail "git init failed in $repo"
  git -C "$repo" config user.email "fit@plotplot.invalid" || fail "could not configure git"
  git -C "$repo" config user.name "plotplot fit" || fail "could not configure git"
  git -C "$repo" remote add origin "https://example.invalid/jahala/fixture.git" \
    || fail "could not give the fixture an origin remote"

  # AGENTS.md as tend2's init leaves it, with a line of the repository's own under it.
  { cat "$TEND2_BLOCK" && printf '\n%s\n' "$AGENTS_PROSE"; } >"$repo/AGENTS.md" \
    || fail "could not write the fixture's AGENTS.md from $TEND2_BLOCK"

  mkdir -p "$repo/.plotplot/beds/$INIT_JUDGE" "$repo/.plotplot/bin"
  cp "$ROOT/contracts/fixtures/manifest/$INIT_JUDGE.garden.json" \
     "$repo/.plotplot/beds/$INIT_JUDGE/garden.json" \
    || fail "could not copy the $INIT_JUDGE manifest"
  # The skill carries the front matter every harness reads a skill by: the description is
  # the one line a deferring harness loads at session start, and what the context check
  # measures.
  mkdir -p "$repo/.plotplot/beds/$INIT_JUDGE/artifact"
  cat > "$repo/.plotplot/beds/$INIT_JUDGE/artifact/SKILL.md" <<'SKILL'
---
name: weeder
description: The judge of the diff. Reads what an agent produced and refuses deleted tests, stubs and secrets before they land, as SARIF.
---

# weeder

Run `weeder check` before saying done; a block-level result is a refusal, not advice.
SKILL

  cat > "$repo/.plotplot/bin/$INIT_JUDGE" <<'JUDGE'
#!/bin/sh
# The fixture judge. The bed contract, and nothing more: read the payload, allow.
cat >/dev/null
exit 0
JUDGE
  chmod +x "$repo/.plotplot/bin/$INIT_JUDGE"
  digest="$(fit_sha256 "$repo/.plotplot/bin/$INIT_JUDGE")" || fail "could not hash the judge"
  printf '%s\n' "$digest" > "$repo/.plotplot/bin/$INIT_JUDGE.sha256"

  platform="$(fit_platform)"
  cat > "$template" <<LOCK
season = "2026.09"

[judges.$INIT_JUDGE]
version = "$INIT_VERSION"

[judges.$INIT_JUDGE.platforms."$platform"]
url = "https://github.com/jahala/$INIT_JUDGE/releases/download/v$INIT_VERSION/$INIT_JUDGE-$platform.tar.gz"
sha256 = "$digest"

[judges.tilth]
version = "0.10.1"
npm = "tilth"
LOCK

  note "the judge was pre-placed because $INIT_JUDGE has no release and the lock schema takes only https urls: sha256 $digest, pinned in the template and recorded beside the judge, so verify resolves it without a fetch"
  note "the template also pins tilth, which the minimal profile drops; that is what the filter has to prove"
}

# Every file under a tree, its sha256 and its path, one to a line, git's own bookkeeping left
# out. Two of these, before and after a second run, are what idempotence means here.
tree_hash() {
  local dir="$1" f
  ( cd "$dir" && find . -type f -not -path './.git/*' | LC_ALL=C sort | while read -r f; do
      printf '%s  %s\n' "$(fit_sha256 "$f")" "$f"
    done )
}

# The dispatcher call a project-scope entry has to carry for one harness and one event.
assert_dispatcher_entry() {
  local file="$1" filter="$2" harness="$3" event="$4" command
  command="$(jq -r "$filter" "$file")" \
    || { cat "$file" >&2; fail "$file carries no entry for $harness $event"; }
  { [ -n "$command" ] && [ "$command" != "null" ]; } \
    || { cat "$file" >&2; fail "$file registers nothing on $harness $event"; }
  case "$command" in
    *"hook $harness $event") : ;;
    *) fail "$file's $event entry is \"$command\", not one dispatcher call" ;;
  esac
  case "$command" in
    *'${CLAUDE_PLUGIN_ROOT}'*|*'${extensionPath}'*)
      fail "$file's $event entry still names a bundle root variable nothing expands at project scope: $command" ;;
  esac
}

# tend2's block in an AGENTS.md is the fixture, byte for byte.
assert_tend2_block() {
  local agents="$1" when="$2"
  sed -n '/<!-- tend2:begin -->/,/<!-- tend2:end -->/p' "$agents" | cmp -s - "$TEND2_BLOCK" \
    || { cat "$agents" >&2; fail "$when: tend2's block in AGENTS.md is not byte for byte $TEND2_BLOCK"; }
}

# And it still opens the file: the file's first lines are the fixture's lines.
assert_tend2_block_first() {
  local agents="$1" when="$2"
  head -n "$(( $(wc -l <"$TEND2_BLOCK") ))" "$agents" | cmp -s - "$TEND2_BLOCK" \
    || { cat "$agents" >&2; fail "$when: tend2's block no longer starts AGENTS.md at line 1"; }
}

check_init() {
  require git jq claude
  build_stem

  local tmp repo home template status
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"

  note "planting the fixture repository at $repo"
  plant_init_fixture "$repo" "$template"

  # 1. The first run plants, and names every file and setting it changed.
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness gemini,codex --lock "$template" ) >"$tmp/first.out" 2>"$tmp/first.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/first.out" "$tmp/first.err" >&2; fail "plotplot init exited $status, not 0"; }
  note "init reported $(grep -c . "$tmp/first.out") changes"

  # 2. The lock it wrote is the template filtered to the minimal profile's one bed.
  grep -q "^\[judges\.$INIT_JUDGE\]" "$repo/garden.lock" \
    || { cat "$repo/garden.lock" >&2; fail "the written lock does not pin $INIT_JUDGE"; }
  if grep -q "^\[judges\.tilth\]" "$repo/garden.lock"; then
    cat "$repo/garden.lock" >&2
    fail "the written lock still pins tilth, which the minimal profile drops"
  fi
  note "garden.lock: $INIT_JUDGE $INIT_VERSION, and nothing else"

  # 3. The garden block is in AGENTS.md, and it names what is planted.
  grep -q '<!-- plotplot:begin -->' "$repo/AGENTS.md" \
    || { cat "$repo/AGENTS.md" >&2; fail "AGENTS.md carries no garden block"; }
  grep -q '<!-- plotplot:end -->' "$repo/AGENTS.md" \
    || fail "AGENTS.md's garden block has no end marker"
  grep -q "$INIT_JUDGE $INIT_VERSION" "$repo/AGENTS.md" \
    || { cat "$repo/AGENTS.md" >&2; fail "the garden block does not name $INIT_JUDGE $INIT_VERSION"; }
  note "AGENTS.md: the garden block names $INIT_JUDGE $INIT_VERSION"
  assert_tend2_block "$repo/AGENTS.md" "first run"
  assert_tend2_block_first "$repo/AGENTS.md" "first run"
  grep -qxF "$AGENTS_PROSE" "$repo/AGENTS.md" \
    || { cat "$repo/AGENTS.md" >&2; fail "first run: the repository's own line in AGENTS.md is gone"; }
  note "AGENTS.md after the first run: tend2's block byte for byte the fixture, still from line 1, the repository's own line kept"

  # 4. The repository's own manifest, which is never a bed.
  [ -f "$repo/garden.json" ] || fail "init wrote no garden.json beside the lock"
  [ "$(jq -r '.kind | join(",")' "$repo/garden.json")" = "repository" ] \
    || { cat "$repo/garden.json" >&2; fail "the repository's own manifest is not kind [\"repository\"]"; }
  note "garden.json: $(jq -r '.name' "$repo/garden.json"), kind repository"

  # 5. Gemini's project settings, which is where its hooks live because it has no project
  #    scope of its own.
  [ -f "$repo/.gemini/settings.json" ] || fail "init wrote no $repo/.gemini/settings.json"
  assert_dispatcher_entry "$repo/.gemini/settings.json" \
    '.hooks.BeforeTool[0].hooks[0].command' gemini BeforeTool
  assert_dispatcher_entry "$repo/.gemini/settings.json" \
    '.hooks.SessionEnd[0].hooks[0].command' gemini SessionEnd
  note "gemini: $(jq -r '.hooks | keys | join(", ")' "$repo/.gemini/settings.json")"

  # 6. Codex's project hook file.
  [ -f "$repo/.codex/hooks.json" ] || fail "init wrote no $repo/.codex/hooks.json"
  assert_dispatcher_entry "$repo/.codex/hooks.json" \
    '.hooks.PreToolUse[0].hooks[0].command' codex PreToolUse
  assert_dispatcher_entry "$repo/.codex/hooks.json" \
    '.hooks.Stop[0].hooks[0].command' codex Stop
  note "codex: $(jq -r '.hooks | keys | join(", ")' "$repo/.codex/hooks.json")"
  grep -q "trust" "$tmp/first.err" \
    || { cat "$tmp/first.err" >&2; fail "init did not say that codex waits on project trust"; }

  # 7. The git law: the five hooks, runnable, and the configuration that runs them.
  local hook
  for hook in pre-commit commit-msg pre-push pre-rebase post-commit; do
    [ -f "$repo/.githooks/$hook" ] || fail "init wrote no .githooks/$hook"
    [ -x "$repo/.githooks/$hook" ] || fail ".githooks/$hook is not executable"
  done
  [ "$(git -C "$repo" config --local core.hooksPath)" = ".githooks" ] \
    || fail "core.hooksPath is \"$(git -C "$repo" config --local core.hooksPath)\", not .githooks"
  if { git -C "$repo" config --local --get-all remote.origin.fetch; git -C "$repo" config --local --get-all remote.origin.push; } 2>/dev/null \
      | grep -q 'refs/notes/plotplot/receipts'; then
    fail "init set a refspec for the receipts ref; a fetch refspec fails every fetch until the ref exists and a push refspec sends the ref alone"
  fi
  note "git: core.hooksPath .githooks, five runnable hooks, no refspec for the receipts ref"

  # 8. The gate every pull request passes, on a hosted runner and no other.
  local workflow="$repo/.github/workflows/plotplot-check.yml"
  [ -f "$workflow" ] || fail "init wrote no pull request workflow"
  grep -qx '        run: plotplot check --strict' "$workflow" \
    || { cat "$workflow" >&2; fail "the workflow has no step running plotplot check --strict"; }
  # The job's name is the status check the ruleset requires (ruled 2026-09-10): a job
  # renamed away from it silently unprotects the default branch.
  grep -qx "    name: $REQUIRED_CHECK" "$workflow" \
    || { cat "$workflow" >&2; fail "the workflow's job is not named $REQUIRED_CHECK, the check the ruleset requires"; }
  grep -q 'pull_request' "$workflow" || fail "the workflow does not run on pull_request"
  grep -q 'runs-on: ubuntu-latest' "$workflow" || fail "the workflow names no hosted runner"
  if grep -q 'self-hosted' "$workflow"; then
    fail "the workflow names a runner of the repository's own, which a public bed never registers (jahala/plotplot issue 7)"
  fi
  [ ! -e "$repo/.github/CODEOWNERS" ] \
    || fail "plain init wrote .github/CODEOWNERS, which only --github writes"
  note "the pull request gate: job $REQUIRED_CHECK, a step running plotplot check --strict, on a hosted runner"

  # 9. And a second run changes nothing, and says so.
  tree_hash "$repo" > "$tmp/before.hashes"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness gemini,codex --lock "$template" ) >"$tmp/second.out" 2>"$tmp/second.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/second.out" "$tmp/second.err" >&2; fail "the second init exited $status, not 0"; }
  [ "$(cat "$tmp/second.out")" = "nothing to do" ] \
    || { cat "$tmp/second.out" >&2; fail "the second init did not print exactly 'nothing to do'"; }
  tree_hash "$repo" > "$tmp/after.hashes"
  diff "$tmp/before.hashes" "$tmp/after.hashes" >/dev/null \
    || { diff "$tmp/before.hashes" "$tmp/after.hashes" >&2; fail "the second init changed files"; }
  note "second run: nothing to do, and every file byte for byte as it was"
  assert_tend2_block "$repo/AGENTS.md" "second run"
  assert_tend2_block_first "$repo/AGENTS.md" "second run"
  note "AGENTS.md after the second run: tend2's block byte for byte the fixture, still from line 1"

  # 10. The other order the two blocks can stand in: the garden block above tend2's.
  check_init_above_tend2 "$repo/AGENTS.md"

  # 11. Claude's own install, through claude's own mechanism, against the temporary home.
  check_init_claude "$tmp" "$repo" "$home" "$template"

  echo "stem.sh init: pass"
}

# A second fixture repository whose AGENTS.md has the garden block at the top, one season
# old, and tend2's block under it. One init replaces the marked region and leaves tend2's
# block byte for byte, one blank line below the garden block.
check_init_above_tend2() {
  local planted="$1" tmp repo home template status
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"

  note "planting a second fixture repository, the garden block above tend2's, at $repo"
  plant_init_fixture "$repo" "$template"

  # The garden block the stem rendered for the first fixture, set back one season so init
  # has a region to replace: the stem's own output, never a hand-written copy of it.
  sed -n '/<!-- plotplot:begin -->/,/<!-- plotplot:end -->/p' "$planted" \
    | sed 's/^Season: 2026\.09\.$/Season: 2026.08./' >"$tmp/older.md"
  grep -qx 'Season: 2026.08.' "$tmp/older.md" \
    || { cat "$tmp/older.md" >&2; fail "could not set the first fixture's garden block back a season"; }
  { cat "$tmp/older.md" && echo && cat "$TEND2_BLOCK"; } >"$repo/AGENTS.md" \
    || fail "could not write the second fixture's AGENTS.md"

  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness gemini,codex --lock "$template" ) >"$tmp/above.out" 2>"$tmp/above.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/above.out" "$tmp/above.err" >&2; fail "plotplot init exited $status on the second fixture, not 0"; }
  grep -qx 'AGENTS.md' "$tmp/above.out" \
    || { cat "$tmp/above.out" >&2; fail "init did not rewrite the second fixture's AGENTS.md"; }
  [ "$(head -1 "$repo/AGENTS.md")" = '<!-- plotplot:begin -->' ] \
    || { cat "$repo/AGENTS.md" >&2; fail "the garden block no longer opens the second fixture's AGENTS.md"; }
  grep -qx 'Season: 2026.09.' "$repo/AGENTS.md" \
    || { cat "$repo/AGENTS.md" >&2; fail "the older season's garden block was not replaced"; }
  note "second layout: init replaced the older season's garden block at the top of AGENTS.md"

  assert_tend2_block "$repo/AGENTS.md" "second layout"
  sed '1,/<!-- plotplot:end -->/d' "$repo/AGENTS.md" | cmp -s - <(echo && cat "$TEND2_BLOCK") \
    || { cat "$repo/AGENTS.md" >&2; fail "second layout: tend2's block is not where it was, one blank line below the garden block"; }
  note "second layout: tend2's block byte for byte the fixture, still one blank line after the garden block"
}

# Claude Code's half of the check: the same repository, planted again with --harness claude,
# asserted through what claude itself wrote at project scope. HOME is the temporary one, so
# the marketplace claude records for itself is recorded there and nowhere else.
check_init_claude() {
  local tmp="$1" repo="$2" home="$3" template="$4" status
  note "planting claude through claude's own marketplace mechanism"

  ( cd "$repo" && HOME="$home" \
      "$STEM" init --harness claude --lock "$template" ) >"$tmp/claude.out" 2>"$tmp/claude.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/claude.out" "$tmp/claude.err" >&2; fail "plotplot init --harness claude exited $status, not 0"; }

  local market="$repo/.plotplot/marketplace/.claude-plugin/marketplace.json"
  [ -f "$market" ] || fail "init wrote no local marketplace for claude"
  [ "$(jq -r '.plugins[0].name' "$market")" = "$PLUGIN" ] \
    || { cat "$market" >&2; fail "the marketplace does not list $PLUGIN"; }

  # What project scope means for claude: the plugin enabled in the project's own settings.
  [ -f "$repo/.claude/settings.json" ] || fail "claude wrote no project settings"
  [ "$(jq -r --arg id "$PLUGIN@plotplot-local" '.enabledPlugins[$id] // false' "$repo/.claude/settings.json")" = "true" ] \
    || { cat "$repo/.claude/settings.json" >&2; fail "claude did not enable $PLUGIN@plotplot-local at project scope"; }

  # And nothing of the stem's went outside the repository: the only user-scope file is the
  # marketplace list claude keeps for itself, which init names on stderr.
  [ -f "$home/.claude/settings.json" ] \
    || fail "claude recorded no marketplace in the temporary home, so this check proved nothing about scope"
  grep -q "user scope" "$tmp/claude.err" \
    || { cat "$tmp/claude.err" >&2; fail "init did not say what claude records outside the repository"; }

  tree_hash "$repo" > "$tmp/claude-before.hashes"
  ( cd "$repo" && HOME="$home" \
      "$STEM" init --harness claude --lock "$template" ) >"$tmp/claude2.out" 2>"$tmp/claude2.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/claude2.out" "$tmp/claude2.err" >&2; fail "the second claude init exited $status, not 0"; }
  [ "$(cat "$tmp/claude2.out")" = "nothing to do" ] \
    || { cat "$tmp/claude2.out" >&2; fail "the second claude init did not print exactly 'nothing to do'"; }
  tree_hash "$repo" > "$tmp/claude-after.hashes"
  diff "$tmp/claude-before.hashes" "$tmp/claude-after.hashes" >/dev/null \
    || { diff "$tmp/claude-before.hashes" "$tmp/claude-after.hashes" >&2; fail "the second claude init changed files"; }
  note "claude: installed at project scope from $repo/.plotplot/marketplace, and a second run changed nothing"
}

# ---------------------------------------------------------------------------------------
# check: platform
# ---------------------------------------------------------------------------------------

# The repository the fixture's origin names. A url only: nothing here fetches it, and every
# call about it is answered by the recording gh below.
PLATFORM_OWNER="example-owner"
PLATFORM_REPO="example-repo"
PLATFORM_ORIGIN="https://github.com/$PLATFORM_OWNER/$PLATFORM_REPO.git"
RULESET_NAME="plotplot: the default branch"

# The twelve lines the stem's CODEOWNERS region carries for the fixture's owner, in order.
platform_region() {
  local path
  echo '# plotplot:begin'
  for path in /.githooks/ /garden.lock /garden.json /AGENTS.md /CLAUDE.md /weeder.toml \
      /.claude/settings.json /.gemini/settings.json /.codex/hooks.json \
      /.github/workflows/plotplot-check.yml /.github/CODEOWNERS /docs/tend2/; do
    echo "$path @$PLATFORM_OWNER"
  done
  echo '# plotplot:end'
}

# A gh that answers the calls the stem makes about the fixture's repository, and nothing
# else, and writes every call to a journal: argv on one line, then stdin when there was any.
# It keeps the one ruleset it was given in a state file beside the journal, so a list after a
# POST names it and a GET of it hands back the body it was given, the way GitHub would. A file
# called `rules-mode` holding `no-force` makes it report the branch without its
# non_fast_forward rule. Everything it answers about GitHub's shapes is copied from GitHub's
# own answers, read 2026-09-11 with read-only calls: the list's fields, the ruleset's and the
# ruleset_* fields a rule in force carries (cli/cli), and do_not_enforce_on_create beside a
# ruleset's status checks (astral-sh/uv, microsoft/vscode, vercel/next.js).
write_recording_gh() {
  local bin="$1"
  mkdir -p "$bin" || fail "could not make the stub's bin directory"
  cat > "$bin/gh" <<'STUB'
#!/usr/bin/env bash
# The recording gh of scripts/fit/stem.sh platform. Never the real one.
set -u
here="$(cd "$(dirname "$0")/.." && pwd)"
journal="$here/gh.journal"
ruleset="$here/gh.ruleset"
repo="repos/example-owner/example-repo"

[ "${1:-}" = "api" ] || { echo "the recording gh answers api calls only: $*" >&2; exit 1; }
printf '%s\n' "gh $*" >> "$journal"
shift
method="GET"; path=""; input="no"
while [ $# -gt 0 ]; do
  case "$1" in
    -X) method="$2"; shift 2 ;;
    --input) input="yes"; shift 2 ;;
    *) path="$1"; shift ;;
  esac
done
body=""
if [ "$input" = "yes" ]; then
  body="$(cat)"
  printf '%s\n' "$body" >> "$journal"
fi

not_found() {
  echo "gh: Not Found (HTTP 404)" >&2
  printf '{"message":"Not Found","documentation_url":"https://docs.github.com/rest","status":"404"}'
  exit 1
}

case "$method $path" in
  "GET $repo")
    printf '{"id":1,"name":"example-repo","full_name":"example-owner/example-repo","default_branch":"main"}\n' ;;
  "GET $repo/rulesets")
    if [ -f "$ruleset" ]; then
      printf '[{"id":41,"name":"plotplot: the default branch","target":"branch","source_type":"Repository","source":"example-owner/example-repo","enforcement":"active"}]\n'
    else
      printf '[]\n'
    fi ;;
  "POST $repo/rulesets")
    printf '%s\n' "$body" > "$ruleset"
    jq -c '. + {id: 41, source_type: "Repository", source: "example-owner/example-repo"}' "$ruleset" ;;
  "GET $repo/rulesets/41")
    # The body it was given, with what GitHub adds to a ruleset it hands back: its id and
    # source, and do_not_enforce_on_create beside the status checks.
    [ -f "$ruleset" ] || not_found
    jq -c '. + {id: 41, source_type: "Repository", source: "example-owner/example-repo"}
           | .rules |= map(if .type == "required_status_checks"
                           then .parameters += {do_not_enforce_on_create: false} else . end)' "$ruleset" ;;
  "PUT $repo/rulesets/41")
    [ -f "$ruleset" ] || not_found
    printf '%s\n' "$body" > "$ruleset"
    jq -c '. + {id: 41}' "$ruleset" ;;
  "GET $repo/rules/branches/main")
    source='"ruleset_source_type":"Repository","ruleset_source":"example-owner/example-repo","ruleset_id":41'
    deletion="{\"type\":\"deletion\",$source}"
    force="{\"type\":\"non_fast_forward\",$source}"
    checks="{\"type\":\"required_status_checks\",\"parameters\":{\"required_status_checks\":[{\"context\":\"garden\"}],\"strict_required_status_checks_policy\":false},$source}"
    if [ "$(cat "$here/rules-mode" 2>/dev/null)" = "no-force" ]; then
      printf '[%s,%s]\n' "$deletion" "$checks"
    else
      printf '[%s,%s,%s]\n' "$deletion" "$force" "$checks"
    fi ;;
  *) not_found ;;
esac
STUB
  chmod +x "$bin/gh" || fail "could not make the recording gh executable"
}

# How many calls in the journal used this method.
journal_count() {
  local journal="$1" method="$2"
  [ -f "$journal" ] || { echo 0; return; }
  grep -c "^gh api -X $method " "$journal"
}

# The platform table: the lines after the static table's blank line.
platform_table() {
  sed '1,/^$/d' "$1"
}

# One platform line's verdict, read from the table's second column.
platform_verdict() {
  local table="$1" check="$2"
  grep "^$check  " "$table" | sed -E "s/^$check +([a-z]+) .*/\1/"
}

# A bin directory holding git and jq and nothing else, and the PATH that is it plus the
# system directories: no gh anywhere on it.
no_gh_path() {
  local bin="$1" tool
  mkdir -p "$bin" || fail "could not make $bin"
  for tool in git jq; do
    ln -sf "$(command -v "$tool")" "$bin/$tool" || fail "could not link $tool into $bin"
  done
  echo "$bin:/usr/bin:/bin"
}

check_platform() {
  require git jq
  build_stem

  local tmp repo home template stub journal status path_with_stub path_without_gh
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"
  stub="$tmp/stub"
  journal="$stub/gh.journal"

  note "planting the fixture repository at $repo, origin $PLATFORM_ORIGIN (a url only, never fetched)"
  plant_init_fixture "$repo" "$template"
  git -C "$repo" remote set-url origin "$PLATFORM_ORIGIN" || fail "could not point origin at $PLATFORM_ORIGIN"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --lock "$template" --harness gemini,codex ) >"$tmp/plain.out" 2>"$tmp/plain.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/plain.out" "$tmp/plain.err" >&2; fail "the plain init exited $status, not 0"; }
  # doctor's static table asks after all three bundles; claude's is generated here without
  # being installed, because installing it runs claude itself, which the init check proves.
  ( cd "$repo" && HOME="$home" "$STEM" bundle build claude ) >/dev/null 2>"$tmp/bundle.err" \
    || { cat "$tmp/bundle.err" >&2; fail "plotplot bundle build claude failed on the fixture"; }

  # A CODEOWNERS of the repository's own, which --github must leave byte for byte.
  mkdir -p "$repo/.github"
  printf '# Owners this repository keeps for itself.\n*.md @someone\n' > "$repo/.github/CODEOWNERS"
  cp "$repo/.github/CODEOWNERS" "$tmp/codeowners.before"

  write_recording_gh "$stub/bin"
  path_with_stub="$stub/bin:$PATH"
  [ "$(PATH="$path_with_stub" command -v gh)" = "$stub/bin/gh" ] \
    || fail "the recording gh is not the first gh on PATH, and this check must never reach the real one"
  note "a recording gh at $stub/bin/gh, first on PATH; every call goes to $journal"

  # 1. init --github writes the region, applies the ruleset, and names both.
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_with_stub" \
      "$STEM" init --github --harness gemini,codex ) >"$tmp/github.out" 2>"$tmp/github.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/github.out" "$tmp/github.err" >&2; fail "plotplot init --github exited $status, not 0"; }
  grep -qx '.github/CODEOWNERS' "$tmp/github.out" \
    || { cat "$tmp/github.out" >&2; fail "init --github did not name .github/CODEOWNERS"; }
  grep -qxF "ruleset \"$RULESET_NAME\" applied on $PLATFORM_OWNER/$PLATFORM_REPO" "$tmp/github.out" \
    || { cat "$tmp/github.out" >&2; fail "init --github did not name the ruleset it applied"; }
  note "init --github: exit 0, changed: $(paste -sd ';' "$tmp/github.out")"

  # 2. Exactly one POST, carrying the desired body.
  [ "$(journal_count "$journal" POST)" -eq 1 ] \
    || { cat "$journal" >&2; fail "the journal holds $(journal_count "$journal" POST) POSTs, not one"; }
  [ "$(journal_count "$journal" PUT)" -eq 0 ] \
    || { cat "$journal" >&2; fail "the first run PUT a ruleset nobody had created"; }
  awk '/^gh api -X POST /{getline; print}' "$journal" > "$tmp/posted.json"
  [ "$(jq -c '[.rules[].type]' "$tmp/posted.json")" = '["deletion","non_fast_forward","required_status_checks"]' ] \
    || { cat "$tmp/posted.json" >&2; fail "the POSTed rules are not deletion, non_fast_forward, required_status_checks in that order"; }
  if grep -q 'required_linear_history' "$tmp/posted.json"; then
    fail "the POSTed ruleset carries required_linear_history, which refuses the law's merge commits"
  fi
  [ "$(jq -r '.rules[2].parameters.required_status_checks[0].context' "$tmp/posted.json")" = "$REQUIRED_CHECK" ] \
    || { cat "$tmp/posted.json" >&2; fail "the POSTed ruleset does not require the context $REQUIRED_CHECK"; }
  [ "$(jq -c '.conditions.ref_name.include' "$tmp/posted.json")" = '["~DEFAULT_BRANCH"]' ] \
    || { cat "$tmp/posted.json" >&2; fail "the POSTed ruleset does not target ~DEFAULT_BRANCH"; }
  [ "$(jq -r '.name' "$tmp/posted.json")" = "$RULESET_NAME" ] \
    || fail "the POSTed ruleset is not called $RULESET_NAME"
  note "one POST: rules $(jq -c '[.rules[].type]' "$tmp/posted.json"), context $REQUIRED_CHECK, ~DEFAULT_BRANCH, no linear history"

  # 3. The region names the owner for the twelve paths, and the repository's own lines stay.
  platform_region > "$tmp/region.expected"
  sed -n '/^# plotplot:begin$/,/^# plotplot:end$/p' "$repo/.github/CODEOWNERS" \
    | cmp -s - "$tmp/region.expected" \
    || { diff "$tmp/region.expected" <(sed -n '/^# plotplot:begin$/,/^# plotplot:end$/p' "$repo/.github/CODEOWNERS") >&2; fail "the CODEOWNERS region is not the twelve guarded paths naming @$PLATFORM_OWNER"; }
  head -c "$(wc -c < "$tmp/codeowners.before")" "$repo/.github/CODEOWNERS" | cmp -s - "$tmp/codeowners.before" \
    || { cat "$repo/.github/CODEOWNERS" >&2; fail "the repository's own CODEOWNERS lines did not survive byte for byte"; }
  note "CODEOWNERS: twelve lines naming @$PLATFORM_OWNER inside the markers, the repository's own two lines above them untouched"

  # 4. A second run changes nothing and writes nothing to the platform.
  tree_hash "$repo" > "$tmp/before.hashes"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_with_stub" \
      "$STEM" init --github --harness gemini,codex ) >"$tmp/again.out" 2>"$tmp/again.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/again.out" "$tmp/again.err" >&2; fail "the second init --github exited $status, not 0"; }
  [ "$(cat "$tmp/again.out")" = "nothing to do" ] \
    || { cat "$tmp/again.out" >&2; fail "the second init --github did not print exactly 'nothing to do'"; }
  [ "$(journal_count "$journal" POST)" -eq 1 ] && [ "$(journal_count "$journal" PUT)" -eq 0 ] \
    || { cat "$journal" >&2; fail "the second init --github wrote to the platform"; }
  tree_hash "$repo" > "$tmp/after.hashes"
  diff "$tmp/before.hashes" "$tmp/after.hashes" >/dev/null \
    || { diff "$tmp/before.hashes" "$tmp/after.hashes" >&2; fail "the second init --github changed files"; }
  note "second run: nothing to do, no POST or PUT, every file as it was"

  # 5. doctor --platform reads the three rules back.
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_with_stub" \
      "$STEM" doctor --platform ) >"$tmp/doctor.out" 2>"$tmp/doctor.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/doctor.out" "$tmp/doctor.err" >&2; fail "plotplot doctor --platform exited $status, not 0"; }
  platform_table "$tmp/doctor.out" > "$tmp/platform.table"
  [ "$(wc -l < "$tmp/platform.table" | tr -d ' ')" -eq 3 ] \
    || { cat "$tmp/doctor.out" >&2; fail "the platform table is not three lines"; }
  local check
  for check in "required check" "force push" "deletion"; do
    [ "$(platform_verdict "$tmp/platform.table" "$check")" = "ok" ] \
      || { cat "$tmp/doctor.out" >&2; fail "doctor --platform's $check line is not ok"; }
    grep "^$check  " "$tmp/platform.table" | grep -q 'main' \
      || { cat "$tmp/platform.table" >&2; fail "the $check line does not name the branch main"; }
    grep "^$check  " "$tmp/platform.table" | grep -q 'ruleset 41' \
      || { cat "$tmp/platform.table" >&2; fail "the $check line does not name ruleset 41"; }
  done
  [ "$(journal_count "$journal" POST)" -eq 1 ] && [ "$(journal_count "$journal" PUT)" -eq 0 ] \
    || { cat "$journal" >&2; fail "doctor --platform made a writing call"; }
  note "doctor --platform: exit 0, read-only calls only"
  sed 's/^/    /' "$tmp/platform.table"

  # 6. With the force-push rule gone from the branch, doctor says so and exits 3.
  echo "no-force" > "$stub/rules-mode"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_with_stub" \
      "$STEM" doctor --platform ) >"$tmp/noforce.out" 2>"$tmp/noforce.err"
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/noforce.out" "$tmp/noforce.err" >&2; fail "doctor --platform without a force-push rule exited $status, not 3"; }
  platform_table "$tmp/noforce.out" > "$tmp/noforce.table"
  [ "$(platform_verdict "$tmp/noforce.table" "force push")" = "fail" ] \
    || { cat "$tmp/noforce.out" >&2; fail "the force push line does not read fail"; }
  note "without non_fast_forward: exit 3, $(grep '^force push  ' "$tmp/noforce.table" | tr -s ' ')"
  echo "all" > "$stub/rules-mode"

  # 7. With no gh on PATH: one unavailable line and exit 3; init --github refuses with 2.
  path_without_gh="$(no_gh_path "$tmp/nogh")"
  [ -z "$(PATH="$path_without_gh" command -v gh)" ] \
    || fail "the PATH meant to hold no gh still finds one: $(PATH="$path_without_gh" command -v gh)"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_without_gh" \
      "$STEM" doctor --platform ) >"$tmp/nogh.out" 2>"$tmp/nogh.err"
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/nogh.out" "$tmp/nogh.err" >&2; fail "doctor --platform with no gh exited $status, not 3"; }
  platform_table "$tmp/nogh.out" > "$tmp/nogh.table"
  [ "$(wc -l < "$tmp/nogh.table" | tr -d ' ')" -eq 1 ] && [ "$(platform_verdict "$tmp/nogh.table" platform)" = "unavailable" ] \
    || { cat "$tmp/nogh.out" >&2; fail "with no gh the platform table is not one unavailable line"; }
  note "no gh, doctor: exit 3, $(tr -s ' ' < "$tmp/nogh.table")"

  local writes_before
  writes_before="$(grep -c . "$journal")"
  tree_hash "$repo" > "$tmp/nogh-before.hashes"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" PATH="$path_without_gh" \
      "$STEM" init --github --harness gemini,codex ) >"$tmp/nogh-init.out" 2>"$tmp/nogh-init.err"
  status=$?
  [ "$status" -eq 2 ] \
    || { cat "$tmp/nogh-init.out" "$tmp/nogh-init.err" >&2; fail "init --github with no gh exited $status, not 2"; }
  grep -q 'gh is not on PATH' "$tmp/nogh-init.err" \
    || { cat "$tmp/nogh-init.err" >&2; fail "init --github did not say that gh is not on PATH"; }
  tree_hash "$repo" > "$tmp/nogh-after.hashes"
  diff "$tmp/nogh-before.hashes" "$tmp/nogh-after.hashes" >/dev/null \
    || { diff "$tmp/nogh-before.hashes" "$tmp/nogh-after.hashes" >&2; fail "init --github with no gh planted something"; }
  [ "$(grep -c . "$journal")" -eq "$writes_before" ] \
    || fail "something reached the recording gh while no gh was on PATH"
  note "no gh, init --github: exit 2, $(grep 'gh is not on PATH' "$tmp/nogh-init.err"), nothing new planted"

  # 8. This repository itself: read-only calls through the real gh, with the planter's own
  #    login. Not asserted beyond its shape: whether the ruleset is applied yet is the
  #    conductor's act after this lands.
  local gh_config="${GH_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/gh}"
  ( cd "$ROOT" && HOME="$home" CODEX_HOME="$home/.codex" GH_CONFIG_DIR="$gh_config" \
      "$STEM" doctor --platform ) >"$tmp/self.out" 2>"$tmp/self.err"
  status=$?
  { [ "$status" -eq 0 ] || [ "$status" -eq 3 ]; } \
    || { cat "$tmp/self.out" "$tmp/self.err" >&2; fail "doctor --platform on this repository exited $status, not 0 or 3"; }
  platform_table "$tmp/self.out" > "$tmp/self.table"
  local lines
  lines="$(wc -l < "$tmp/self.table" | tr -d ' ')"
  if [ "$lines" -eq 3 ]; then
    for check in "required check" "force push" "deletion"; do
      grep -q "^$check  " "$tmp/self.table" \
        || { cat "$tmp/self.out" >&2; fail "this repository's platform table has no $check line"; }
    done
  elif [ "$lines" -eq 1 ]; then
    [ "$(platform_verdict "$tmp/self.table" platform)" = "unavailable" ] \
      || { cat "$tmp/self.out" >&2; fail "this repository's one platform line is not unavailable"; }
  else
    cat "$tmp/self.out" >&2
    fail "this repository's platform table is $lines lines, neither three nor one unavailable"
  fi
  note "this repository (origin $(git -C "$ROOT" remote get-url origin)), doctor --platform exited $status; its platform table, not asserted:"
  sed 's/^/    /' "$tmp/self.table"

  echo "stem.sh platform: pass"
}

# ---------------------------------------------------------------------------------------
# check: context
# ---------------------------------------------------------------------------------------

# The bar the stem loop names for what a planted repository costs an agent at session start
# on a deferring harness: the garden block in AGENTS.md plus one description line per skill.
CONTEXT_BAR=400

# The probe. Two bases, both named on stdout: the skills through Claude Code's own
# projection of a plugin's always-on cost (`claude plugin details`), which is what a
# deferring harness actually pays for the description lines; the garden block at four
# characters per token, the basis docs/garden-architecture-2026-09.md §2 used for the
# 8,300-token baseline this check is measured against. Neither is an exact tokenizer, and
# the numbers are printed so a reader can judge the margin, not just the verdict.
chars_per_token=4

check_context() {
  require git jq claude
  build_stem

  local tmp repo home template status always_on block_chars block_tokens total
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"

  note "planting the fixture repository at $repo"
  plant_init_fixture "$repo" "$template"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness claude --lock "$template" ) >"$tmp/init.out" 2>"$tmp/init.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/init.out" "$tmp/init.err" >&2; fail "plotplot init exited $status, not 0"; }

  # The skills, as the harness projects them: every SKILL.md in the Claude bundle is a
  # description line the deferring harness loads at session start and nothing more.
  ( cd "$repo" && HOME="$home" claude --plugin-dir "$repo/.plotplot/bundles/claude" plugin details plotplot ) \
    >"$tmp/details.out" 2>"$tmp/details.err" \
    || { cat "$tmp/details.out" "$tmp/details.err" >&2; fail "claude plugin details could not read the bundle"; }
  always_on="$(sed -n 's/^ *Always-on: *~\{0,1\}\([0-9][0-9]*\) tok.*/\1/p' "$tmp/details.out" | head -1)"
  [ -n "$always_on" ] \
    || { cat "$tmp/details.out" >&2; fail "claude plugin details printed no always-on projection"; }
  local skills
  skills="$(sed -n 's/^ *Skills (\([0-9][0-9]*\)).*/\1/p' "$tmp/details.out" | head -1)"
  [ -n "$skills" ] && [ "$skills" -ge 1 ] \
    || { cat "$tmp/details.out" >&2; fail "the harness projected no skill at all, so the probe measured nothing"; }
  note "skills: $skills description line(s), ~$always_on tokens always-on by Claude Code's own projection"

  # The garden block, between its markers, at the architecture document's basis.
  sed -n '/<!-- plotplot:begin -->/,/<!-- plotplot:end -->/p' "$repo/AGENTS.md" >"$tmp/block.md"
  [ -s "$tmp/block.md" ] || fail "AGENTS.md carries no garden block to measure"
  block_chars="$(wc -c <"$tmp/block.md" | tr -d ' ')"
  block_tokens=$(( (block_chars + chars_per_token - 1) / chars_per_token ))
  note "garden block: $(grep -c . "$tmp/block.md") lines, $block_chars characters, ~$block_tokens tokens at $chars_per_token characters per token"

  total=$(( always_on + block_tokens ))
  [ "$total" -lt "$CONTEXT_BAR" ] \
    || fail "the planted repository costs ~$total tokens at session start, not under the $CONTEXT_BAR bar"
  note "total: ~$total tokens at session start, under the $CONTEXT_BAR bar"
  echo "stem.sh context: pass"
}

# ---------------------------------------------------------------------------------------
# check: doctor-live
# ---------------------------------------------------------------------------------------

# Every harness `doctor --live` is asked to drive.
LIVE_HARNESSES="claude,gemini,codex"

# How long one worker has to answer one tool call. A worker whose pane stops moving is hung
# up on well before this by the doctor's own idle net, so this is only the outer deadline;
# it is kept under two minutes because `tend2 verify` gives an evidence script 120 seconds
# and a check that cannot be stamped is not evidence.
LIVE_TIMEOUT=90

# The one harness this check requires proof from. Claude Code is the harness this machine can
# authenticate into a temporary home; the other two are driven all the same and reported with
# whatever they said, which is the honest half of the claim.
LIVE_PROVED=claude

# Give the temporary home what Claude Code needs to authenticate, and say exactly what was
# copied, because a check that borrows a credential has to be legible about it.
#
# On macOS the account token is not in a file at all: it is a generic password in the login
# keychain, and the keychain search list is resolved from HOME, so a temporary home with no
# `Library/Keychains` reports "Not logged in" however many files are copied beside it. The
# link is to the directory the real home already has; nothing is written into it.
lend_claude_credentials() {
  local home="$1" copied=0
  mkdir -p "$home/.claude" || fail "could not make the temporary claude directory"
  if [ -f "$HOME/.claude/.credentials.json" ]; then
    cp "$HOME/.claude/.credentials.json" "$home/.claude/.credentials.json" \
      || fail "could not copy ~/.claude/.credentials.json into the temporary home"
    note "copied ~/.claude/.credentials.json into the temporary home"
    copied=$((copied + 1))
  fi
  if [ -f "$HOME/.claude.json" ]; then
    cp "$HOME/.claude.json" "$home/.claude.json" \
      || fail "could not copy ~/.claude.json into the temporary home"
    note "copied ~/.claude.json into the temporary home"
    copied=$((copied + 1))
  fi
  if [ -d "$HOME/Library/Keychains" ]; then
    mkdir -p "$home/Library" || fail "could not make the temporary Library directory"
    ln -sfn "$HOME/Library/Keychains" "$home/Library/Keychains" \
      || fail "could not link the login keychain into the temporary home"
    note "linked ~/Library/Keychains into the temporary home: on macOS the account token is a keychain item and the keychain search list is resolved from HOME"
    copied=$((copied + 1))
  fi
  [ "$copied" -gt 0 ] \
    || fail "this machine has neither ~/.claude/.credentials.json, ~/.claude.json nor a login keychain, so no temporary home can authenticate Claude Code"
}

# One field of one row of the live table, found by the row's check name.
#
# The name itself carries a space ("claude session"), so the match is anchored at the start
# of the line and needs the space the table pads with after it; that is also what keeps
# "claude session" from matching "claude session-end".
live_field() {
  local file="$1" check="$2" field="$3"
  awk -v want="$check" -v field="$field" '
    index($0, want " ") == 1 {
      rest = substr($0, length(want) + 1)
      sub(/^ +/, "", rest)
      if (field == "verdict") { sub(/ .*$/, "", rest) } else { sub(/^[^ ]+ +/, "", rest) }
      print rest
      exit
    }
  ' "$file"
}

live_verdict() { live_field "$1" "$2" verdict; }
live_detail() { live_field "$1" "$2" detail; }

# What a harness that is not the proved one is allowed to say: proved like the others, or
# unavailable with a reason it observed. Never `fail`, never absent.
assert_reported_honestly() {
  local out="$1" harness="$2" verdict detail
  verdict="$(live_verdict "$out" "$harness session")"
  detail="$(live_detail "$out" "$harness session")"
  [ -n "$verdict" ] \
    || { cat "$out" >&2; fail "the live table carries no line for $harness at all"; }
  case "$verdict" in
    unavailable)
      [ -n "$detail" ] \
        || { cat "$out" >&2; fail "$harness is unavailable with no reason, which is a silent skip"; }
      note "$harness: unavailable — $detail"
      ;;
    ok)
      assert_proved "$out" "$harness"
      note "$harness: proved as well as $LIVE_PROVED, which the plan did not expect on this machine"
      ;;
    *)
      cat "$out" >&2
      fail "$harness reported \"$verdict\", which is neither a proof nor an observed reason"
      ;;
  esac
}

# The three event classes one harness must prove: the before-tool hook refused, the
# session-end hook fired, the receipt draft was written.
assert_proved() {
  local out="$1" harness="$2" class verdict detail
  for class in "before-tool" "session-end" "receipt draft"; do
    verdict="$(live_verdict "$out" "$harness $class")"
    detail="$(live_detail "$out" "$harness $class")"
    [ -n "$verdict" ] \
      || { cat "$out" >&2; fail "the live table carries no $harness $class line"; }
    [ "$verdict" = "ok" ] \
      || { cat "$out" >&2; fail "$harness $class is \"$verdict\": $detail"; }
    note "$harness $class: $detail"
  done
}

# The session id the doctor said the harness reported, out of its session line.
live_session_id() {
  local out="$1" harness="$2"
  live_detail "$out" "$harness session" | sed -n 's/.*which the harness called \([^ ]*\).*/\1/p'
}

check_doctor_live() {
  require git jq claude umbel
  build_stem

  local tmp repo home template status out err session
  scratch
  tmp="$SCRATCH_DIR"
  repo="$tmp/repo"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"
  out="$tmp/live.out"
  err="$tmp/live.err"

  note "planting the fixture repository at $repo"
  plant_init_fixture "$repo" "$template"
  lend_claude_credentials "$home"

  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness "$LIVE_HARNESSES" --lock "$template" ) >"$tmp/init.out" 2>"$tmp/init.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/init.out" "$tmp/init.err" >&2; fail "plotplot init exited $status, not 0"; }
  note "planted $LIVE_HARNESSES; $(grep -c . "$tmp/init.out") files and settings changed"

  note "driving one real session per harness through umbel; each worker has ${LIVE_TIMEOUT}s"
  ( cd "$repo" && HOME="$home" \
      "$STEM" doctor --live --harness "$LIVE_HARNESSES" --timeout "$LIVE_TIMEOUT" ) >"$out" 2>"$err"
  status=$?
  echo "--- doctor --live said ---"
  cat "$out"
  echo "--- and on stderr ---"
  cat "$err"
  echo "--------------------------"

  # 1. Every static finding still passes, so a live failure is never a stale bundle.
  local statics
  statics="$(sed -n '1,/^$/p' "$out" | grep -c '  ok  ')"
  [ "$statics" -eq 9 ] \
    || { cat "$out" >&2; fail "only $statics of the nine static findings are ok"; }
  note "the nine static findings are ok"

  # 2. The harness this machine can authenticate proves all three event classes.
  assert_proved "$out" "$LIVE_PROVED"

  # 3. The other two are reported with what they said, never skipped and never failed.
  local harness
  for harness in gemini codex; do
    assert_reported_honestly "$out" "$harness"
  done

  # 4. A vendor that will not run is not a broken stem.
  [ "$status" -eq 0 ] \
    || { cat "$out" "$err" >&2; fail "doctor --live exited $status, not 0"; }
  note "doctor --live exited 0"

  # 5. And the proof itself, read from the journal rather than from the doctor's own table.
  session="$(live_session_id "$out" "$LIVE_PROVED")"
  [ -n "$session" ] \
    || { cat "$out" >&2; fail "the doctor named no session id for $LIVE_PROVED"; }
  note "$LIVE_PROVED reported session $session"

  local journal kind line
  journal="$tmp/journal.jsonl"
  cat "$repo"/.plotplot/friction/*.jsonl > "$journal" 2>/dev/null \
    || fail "the friction journal under $repo/.plotplot/friction/ is not there at all"
  for kind in tool.denied session.ended; do
    line="$(jq -c --arg s "$session" --arg k "$kind" \
      'select(.["gen_ai.conversation.id"] == $s and .["plotplot.kind"] == $k)' "$journal")"
    [ -n "$line" ] \
      || { cat "$journal" >&2; fail "the journal holds no $kind record for $session"; }
    note "journal $kind: $line"
  done
  [ "$(jq -r --arg s "$session" \
      'select(.["gen_ai.conversation.id"] == $s and .["plotplot.kind"] == "tool.denied") | .["plotplot.rule"]' \
      "$journal")" = "deny.no-verify" ] \
    || { cat "$journal" >&2; fail "the refusal for $session names another rule than deny.no-verify"; }
  note "the refusal names the rule deny.no-verify"

  local draft="$repo/.plotplot/receipts/drafts/$session.json"
  [ -f "$draft" ] || fail "no receipt draft at $draft"
  note "receipt draft: $(jq -c '{harness: .harness.name, sessions: .sessions}' "$draft")"

  echo "stem.sh doctor-live: pass"
}

# ---------------------------------------------------------------------------------------

case "${1:-}" in
  bundles) check_bundles ;;
  hook-faces) check_hook_faces ;;
  lock) check_lock ;;
  check) check_check ;;
  init) check_init ;;
  context) check_context ;;
  doctor-live) check_doctor_live ;;
  platform) check_platform ;;
  *)
    echo "usage: stem.sh {bundles | hook-faces | lock | check | init | context | doctor-live | platform}" >&2
    exit 2
    ;;
esac
