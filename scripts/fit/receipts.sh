#!/usr/bin/env bash
# scripts/fit/receipts.sh: the receipts loop's fit evidence, one check per subcommand.
#
#   scripts/fit/receipts.sh seal     `plotplot receipt seal` attaches a note on
#                                    refs/notes/plotplot/receipts whose subject digest equals
#                                    the commit's gitCommit digest recomputed independently,
#                                    whose trailing sha256 is the digest of the statement
#                                    above it, and a second seal changes nothing
#   scripts/fit/receipts.sh verify   `plotplot receipt verify --range` fails on a range with
#                                    one receipt missing and passes when every commit carries
#                                    one; a fresh clone fetches the receipts ref because
#                                    `plotplot init` set the refspec
#
# Exit 0 on pass, non-zero with a reason on failure. Every check builds the crate in release
# mode once and then works in a fresh temporary directory with HOME pointed at a temporary
# home, so nothing on this machine's user scope is read or written. Scratch goes away with
# `trash`, never `rm` (scripts/fit/lib.sh, fit_discard_scratch).
#
# Everything asserted here is recomputed by this script with git and shasum rather than read
# back from the stem, so a bug the stem has in both writing and reading is still caught.

set -uo pipefail

RECEIPTS_SH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/fit/lib.sh
. "$RECEIPTS_SH_DIR/lib.sh"

ROOT="$(fit_repo_root)"
STEM="$ROOT/target/release/plotplot"

# The ref a receipt lives on, spelled the way `src/plant/gitconfig.rs` spells it.
RECEIPTS_REF="refs/notes/plotplot/receipts"

# The bed `plotplot init --profile minimal` plants, and the version the template pins. The
# same two facts scripts/fit/stem.sh's init check stands on.
JUDGE="weeder"
JUDGE_VERSION="0.1.0"

# The session whose draft the fixture leaves waiting to be sealed.
SESSION="6f3c1b2a-8d47-4f0e-9b31-2a5c7e91d044"

# Every temporary directory this run minted, cleaned up by the EXIT trap.
SCRATCH_DIRS=()

# ---------------------------------------------------------------------------------------
# saying what happened (the shape scripts/fit/stem.sh uses, copied rather than sourced:
# stem.sh dispatches on $1 the moment it is read, so it cannot be sourced by another script)
# ---------------------------------------------------------------------------------------

fail() {
  echo "receipts.sh: $*" >&2
  exit 1
}

note() {
  echo "  $*"
}

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

require() {
  local tool
  for tool in "$@"; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool is not on PATH, and this check needs it"
  done
}

build_stem() {
  note "building the stem in release mode"
  ( cd "$ROOT" && cargo build --release ) >/dev/null 2>&1 \
    || fail "cargo build --release failed; run it yourself to see why"
  [ -x "$STEM" ] || fail "no release binary at $STEM after a successful build"
  note "stem: $("$STEM" version | head -1)"
}

# A home no vendor has ever written to, so `init` never touches this machine's user scope.
make_home() {
  local tmp="$1"
  mkdir -p "$tmp/home" "$tmp/home/.codex" || fail "could not make the temporary home"
  echo "$tmp/home"
}

# ---------------------------------------------------------------------------------------
# the fixture: a repository with three commits, an origin, and one draft waiting
# ---------------------------------------------------------------------------------------

plant_repo() {
  local repo="$1"
  mkdir -p "$repo" || fail "could not make the fixture repository at $repo"
  git init -q "$repo" || fail "git init failed in $repo"
  git -C "$repo" config user.email "fit@plotplot.invalid" || fail "could not configure git"
  git -C "$repo" config user.name "plotplot fit" || fail "could not configure git"
  git -C "$repo" remote add origin "https://github.com/jahala/fixture.git" \
    || fail "could not give the fixture an origin remote"

  printf '.plotplot/\n' > "$repo/.gitignore"
  git -C "$repo" add -A && git -C "$repo" commit -q -m "the first commit" \
    || fail "the first commit failed"

  printf '# fixture\n' > "$repo/README.md"
  git -C "$repo" add -A && git -C "$repo" commit -q -m "the second commit" \
    || fail "the second commit failed"

  mkdir -p "$repo/src"
  printf 'pub fn one() -> u8 {\n    1\n}\n' > "$repo/src/lib.rs"
  git -C "$repo" add -A && git -C "$repo" commit -q -m "the third commit" \
    || fail "the third commit failed"
}

# One session's draft, in the shape `plotplot receipt draft` writes.
plant_draft() {
  local repo="$1"
  mkdir -p "$repo/.plotplot/receipts/drafts"
  cat > "$repo/.plotplot/receipts/drafts/$SESSION.json" <<DRAFT
{
  "harness": {
    "name": "claude-code",
    "version": "2.1.265"
  },
  "models": [
    "claude-opus-5"
  ],
  "principal": "cli",
  "sessions": [
    "$SESSION"
  ],
  "cost": {
    "inputTokens": 41000,
    "outputTokens": 2100,
    "wallSeconds": null,
    "toolCalls": {
      "Bash": 41,
      "Edit": 12
    }
  },
  "friction": {
    "summary": {
      "sha256": "9c1d0f0e6a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f6071829304a5"
    }
  }
}
DRAFT
}

# The state a resolved lock leaves behind, which is what `plotplot init` needs to plant
# without a network: the manifest read out of the artifact, the bed's skill, the executable,
# and the digest of the archive it came out of. weeder has no release and the lock schema
# takes only https urls, so the judge is placed by hand exactly as scripts/fit/stem.sh's init
# check places it; that is the one thing about this fixture that is not real.
place_judge() {
  local repo="$1" digest
  mkdir -p "$repo/.plotplot/beds/$JUDGE" "$repo/.plotplot/bin"
  cp "$ROOT/contracts/fixtures/manifest/$JUDGE.garden.json" \
     "$repo/.plotplot/beds/$JUDGE/garden.json" \
    || fail "could not copy the $JUDGE manifest"
  mkdir -p "$repo/.plotplot/beds/$JUDGE/artifact"
  cat > "$repo/.plotplot/beds/$JUDGE/artifact/SKILL.md" <<'SKILL'
---
name: weeder
description: The judge of the diff. Reads what an agent produced and refuses deleted tests, stubs and secrets before they land, as SARIF.
---

# weeder

Run `weeder check` before saying done; a block-level result is a refusal, not advice.
SKILL

  cat > "$repo/.plotplot/bin/$JUDGE" <<'JUDGE'
#!/bin/sh
# The fixture judge. The bed contract, and nothing more: read the payload, allow.
cat >/dev/null
exit 0
JUDGE
  chmod +x "$repo/.plotplot/bin/$JUDGE"
  digest="$(fit_sha256 "$repo/.plotplot/bin/$JUDGE")" || fail "could not hash the judge"
  printf '%s\n' "$digest" > "$repo/.plotplot/bin/$JUDGE.sha256"
  echo "$digest"
}

write_template() {
  local template="$1" digest="$2" platform
  platform="$(fit_platform)"
  cat > "$template" <<LOCK
season = "2026.09"

[judges.$JUDGE]
version = "$JUDGE_VERSION"

[judges.$JUDGE.platforms."$platform"]
url = "https://github.com/jahala/$JUDGE/releases/download/v$JUDGE_VERSION/$JUDGE-$platform.tar.gz"
sha256 = "$digest"
LOCK
}

# ---------------------------------------------------------------------------------------
# reading a note back, with git and shasum and never with the stem
# ---------------------------------------------------------------------------------------

read_note() {
  local repo="$1" rev="$2" out="$3"
  git -C "$repo" notes --ref "$RECEIPTS_REF" show "$rev" > "$out" \
    || fail "$rev carries no note on $RECEIPTS_REF"
}

# The statement's bytes are everything above the note's last line, which is what the stem
# hashed. `sed '$d'` is the whole definition.
statement_of() {
  local note="$1" out="$2"
  sed '$d' "$note" > "$out"
}

claimed_digest_of() {
  local note="$1"
  tail -1 "$note" | sed -n 's/^sha256: \([0-9a-f]*\)$/\1/p'
}

# ---------------------------------------------------------------------------------------
# check: seal
# ---------------------------------------------------------------------------------------

check_seal() {
  require git jq shasum
  build_stem

  local tmp repo status
  tmp="$(scratch)"
  repo="$tmp/repo"

  note "planting the fixture repository at $repo"
  plant_repo "$repo"
  plant_draft "$repo"

  # 1. The seal itself.
  ( cd "$repo" && "$STEM" receipt seal ) >"$tmp/first.out" 2>"$tmp/first.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/first.out" "$tmp/first.err" >&2; fail "plotplot receipt seal exited $status, not 0"; }
  note "seal said: $(cat "$tmp/first.out")"

  read_note "$repo" HEAD "$tmp/note.txt"
  statement_of "$tmp/note.txt" "$tmp/statement.json"
  jq empty "$tmp/statement.json" \
    || { cat "$tmp/note.txt" >&2; fail "the note's statement is not JSON"; }

  # 2. The subject's digests, recomputed here with git and compared to what the note says.
  local commit tree subject_commit subject_tree subject_name
  commit="$(git -C "$repo" rev-parse HEAD)" || fail "git could not resolve HEAD"
  tree="$(git -C "$repo" rev-parse 'HEAD^{tree}')" || fail "git could not resolve HEAD^{tree}"
  subject_commit="$(jq -r '.subject[0].digest.gitCommit' "$tmp/statement.json")"
  subject_tree="$(jq -r '.subject[0].digest.gitTree' "$tmp/statement.json")"
  subject_name="$(jq -r '.subject[0].name' "$tmp/statement.json")"
  [ "$subject_commit" = "$commit" ] \
    || fail "the subject's gitCommit is $subject_commit, git says $commit"
  [ "$subject_tree" = "$tree" ] \
    || fail "the subject's gitTree is $subject_tree, git says $tree"
  [ "$subject_name" = "github.com/jahala/fixture" ] \
    || fail "the subject is named \"$subject_name\", not the origin's canonical url"
  note "subject: $subject_name, gitCommit and gitTree as git recomputes them"

  # 3. The statement is in-toto's, and the predicate is the garden's.
  [ "$(jq -r '._type' "$tmp/statement.json")" = "https://in-toto.io/Statement/v1" ] \
    || fail "the statement is not an in-toto Statement v1"
  [ "$(jq -r '.predicateType' "$tmp/statement.json")" = "https://plotplot.ai/receipt/v1" ] \
    || fail "the predicate type is not the receipt predicate"

  # 4. The trailing digest, recomputed here with shasum.
  local claimed actual
  claimed="$(claimed_digest_of "$tmp/note.txt")"
  [ -n "$claimed" ] \
    || { cat "$tmp/note.txt" >&2; fail "the note's last line is not the statement's sha256"; }
  actual="$(fit_sha256 "$tmp/statement.json")" || fail "could not hash the statement"
  [ "$claimed" = "$actual" ] \
    || fail "the note claims sha256 $claimed, shasum makes the statement $actual"
  note "sha256: $claimed, over the $(wc -c <"$tmp/statement.json" | tr -d ' ') bytes above the last line"

  # 5. The draft was folded in, and put away so it is never folded twice.
  [ "$(jq -r '.predicate.sessions[0]' "$tmp/statement.json")" = "$SESSION" ] \
    || { cat "$tmp/statement.json" >&2; fail "the predicate does not carry the draft's session"; }
  [ "$(jq -r '.predicate.cost.toolCalls.Bash' "$tmp/statement.json")" = "41" ] \
    || fail "the predicate does not carry the draft's tool counts"
  [ ! -f "$repo/.plotplot/receipts/drafts/$SESSION.json" ] \
    || fail "the folded draft is still waiting under drafts/"
  [ -f "$repo/.plotplot/receipts/sealed/$commit/$SESSION.json" ] \
    || fail "the folded draft was not put away under sealed/$commit/"
  note "the draft was folded in and moved to .plotplot/receipts/sealed/$commit/"

  # 6. And a second seal changes nothing, and says so.
  ( cd "$repo" && "$STEM" receipt seal ) >"$tmp/second.out" 2>"$tmp/second.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/second.out" "$tmp/second.err" >&2; fail "the second seal exited $status, not 0"; }
  grep -q 'unchanged' "$tmp/second.out" \
    || { cat "$tmp/second.out" >&2; fail "the second seal did not say the receipt was unchanged"; }
  read_note "$repo" HEAD "$tmp/note-again.txt"
  cmp -s "$tmp/note.txt" "$tmp/note-again.txt" \
    || { diff "$tmp/note.txt" "$tmp/note-again.txt" >&2; fail "the second seal rewrote the note"; }
  note "second seal: $(cat "$tmp/second.out"), and the note byte for byte as it was"

  echo "receipts.sh seal: pass"
}

# ---------------------------------------------------------------------------------------
# check: verify
# ---------------------------------------------------------------------------------------

check_verify() {
  require git jq
  build_stem

  local tmp repo clone home template digest status short
  tmp="$(scratch)"
  repo="$tmp/repo"
  clone="$tmp/clone"
  home="$(make_home "$tmp")"
  template="$tmp/template.lock"

  note "planting the fixture repository at $repo"
  plant_repo "$repo"
  plant_draft "$repo"

  # 1. One of the two commits in the range carries a receipt.
  ( cd "$repo" && "$STEM" receipt seal ) >"$tmp/seal-head.out" 2>&1 \
    || { cat "$tmp/seal-head.out" >&2; fail "sealing HEAD failed"; }
  short="$(git -C "$repo" rev-parse --short 'HEAD~1')" || fail "git could not resolve HEAD~1"

  ( cd "$repo" && "$STEM" receipt verify --range 'HEAD~2..HEAD' ) >"$tmp/gap.out" 2>"$tmp/gap.err"
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/gap.out" "$tmp/gap.err" >&2; fail "verify over a range with a gap exited $status, not 3"; }
  grep -q "^$short no receipt$" "$tmp/gap.out" \
    || { cat "$tmp/gap.out" >&2; fail "verify did not name $short as the commit with no receipt"; }
  note "a range with a gap: exit 3, naming $short"

  # 2. Seal the other, and the same range passes.
  ( cd "$repo" && "$STEM" receipt seal --commit 'HEAD~1' ) >"$tmp/seal-prev.out" 2>&1 \
    || { cat "$tmp/seal-prev.out" >&2; fail "sealing HEAD~1 failed"; }
  ( cd "$repo" && "$STEM" receipt verify --range 'HEAD~2..HEAD' ) >"$tmp/full.out" 2>"$tmp/full.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/full.out" "$tmp/full.err" >&2; fail "verify over a fully sealed range exited $status, not 0"; }
  [ "$(grep -c ' receipt ok$' "$tmp/full.out")" = "2" ] \
    || { cat "$tmp/full.out" >&2; fail "verify did not pass both commits of the range"; }
  note "every commit sealed: exit 0, two receipts ok"

  # 3. A signature is not this version's claim, and verify says so rather than passing.
  ( cd "$repo" && "$STEM" receipt verify HEAD --require-signed ) >"$tmp/signed.out" 2>&1
  status=$?
  [ "$status" -eq 3 ] \
    || { cat "$tmp/signed.out" >&2; fail "verify --require-signed exited $status, not 3"; }
  grep -q 'unsigned (v0)' "$tmp/signed.out" \
    || { cat "$tmp/signed.out" >&2; fail "verify --require-signed did not say the receipt is unsigned"; }
  note "--require-signed: exit 3, unsigned (v0)"

  # 4. Plant the fixture, so it carries the refspecs a receipt travels on.
  digest="$(place_judge "$repo")" || fail "could not place the judge in the fixture"
  write_template "$template" "$digest"
  note "the judge was pre-placed because $JUDGE has no release and the lock schema takes only https urls: sha256 $digest"
  ( cd "$repo" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness gemini --lock "$template" ) >"$tmp/init.out" 2>"$tmp/init.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/init.out" "$tmp/init.err" >&2; fail "plotplot init exited $status, not 0"; }
  git -C "$repo" config --local --get-all remote.origin.fetch \
    | grep -q "^+$RECEIPTS_REF:$RECEIPTS_REF$" \
    || fail "init did not set the receipts fetch refspec on the fixture"
  note "init planted the fixture and set the receipts refspecs"

  # 5. A fresh clone, which starts with git's own refspec and so without the receipts.
  git clone -q "$repo" "$clone" || fail "could not clone the fixture"
  git -C "$clone" config user.email "fit@plotplot.invalid" || fail "could not configure the clone"
  git -C "$clone" config user.name "plotplot fit" || fail "could not configure the clone"
  if git -C "$clone" show-ref --verify --quiet "$RECEIPTS_REF"; then
    # Nothing in the garden depends on clone's own behaviour here; the ref is removed so the
    # fetch below is the only thing that could have brought it back.
    git -C "$clone" update-ref -d "$RECEIPTS_REF" || fail "could not clear the clone's receipts ref"
    note "the clone brought $RECEIPTS_REF on its own; cleared it so the fetch is what is being measured"
  fi
  git -C "$clone" fetch -q origin || fail "the clone could not fetch its origin"
  if git -C "$clone" show-ref --verify --quiet "$RECEIPTS_REF"; then
    fail "the clone fetched $RECEIPTS_REF before anything asked it to, so this check would prove nothing"
  fi
  note "before init: a fetch brings the branch and not the receipts"

  # 6. Planted, the clone's fetch brings the receipts, and they verify there.
  digest="$(place_judge "$clone")" || fail "could not place the judge in the clone"
  ( cd "$clone" && HOME="$home" CODEX_HOME="$home/.codex" \
      "$STEM" init --harness gemini --lock "$template" ) >"$tmp/clone-init.out" 2>"$tmp/clone-init.err"
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/clone-init.out" "$tmp/clone-init.err" >&2; fail "plotplot init in the clone exited $status, not 0"; }
  git -C "$clone" fetch -q origin || fail "the planted clone could not fetch its origin"
  git -C "$clone" show-ref --verify --quiet "$RECEIPTS_REF" \
    || fail "the planted clone's fetch did not bring $RECEIPTS_REF"

  read_note "$repo" HEAD "$tmp/origin-note.txt"
  read_note "$clone" HEAD "$tmp/clone-note.txt"
  cmp -s "$tmp/origin-note.txt" "$tmp/clone-note.txt" \
    || { diff "$tmp/origin-note.txt" "$tmp/clone-note.txt" >&2; fail "the receipt that travelled is not the one that was sealed"; }

  ( cd "$clone" && "$STEM" receipt verify --range 'HEAD~2..HEAD' ) >"$tmp/clone-verify.out" 2>&1
  status=$?
  [ "$status" -eq 0 ] \
    || { cat "$tmp/clone-verify.out" >&2; fail "verify in the clone exited $status, not 0"; }
  note "after init: the fetch brought $RECEIPTS_REF, byte for byte, and it verifies in the clone"

  echo "receipts.sh verify: pass"
}

# ---------------------------------------------------------------------------------------

case "${1:-}" in
  seal) check_seal ;;
  verify) check_verify ;;
  *)
    echo "usage: receipts.sh {seal | verify}" >&2
    exit 2
    ;;
esac
