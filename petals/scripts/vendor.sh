#!/usr/bin/env bash
# vendor.sh: keep a copy of petals inside another repository, stamped (issue #23).
#
# Usage:
#   vendor.sh [--from <petals-checkout>] <dest-repo>   copy petals/ into <dest-repo>/petals
#   vendor.sh --check <dest-repo>                      verify that copy against its stamp
#
# The copy is exactly the files git tracks under petals/ at the source's HEAD,
# so the source must have no uncommitted changes there. <dest-repo>/petals/VERSION
# records the petals commit, its tag when HEAD has one, and a sha256 per file.
# --check needs no network: it fails when a file was edited, added or removed
# since the stamp, naming each. Taking a new petals release is one command:
# run vendor.sh again from the newer checkout.
#
# Exit codes: 0 done (or, with --check, the copy matches its stamp);
# 1 with --check, the copy differs; 2 wrong usage, a dirty source, or no stamp.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SCRIPT_DIR/lib/common.sh" ]]; then
  echo "petals: lib/common.sh is missing beside $(basename "$0"); install the whole petals/ directory" >&2
  exit 127
fi
source "$SCRIPT_DIR/lib/common.sh"

usage() { echo "usage: vendor.sh [--from <petals-checkout>] <dest-repo> | vendor.sh --check <dest-repo>" >&2; exit 2; }

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }

FROM=""; DEST=""; CHECK=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --from)  [[ $# -ge 2 ]] || usage; FROM=$(from_caller "$2"); shift 2 ;;
    --check) CHECK=1; shift ;;
    -*)      usage ;;
    *)       [[ -z "$DEST" ]] || usage; DEST=$(from_caller "$1"); shift ;;
  esac
done
[[ -n "$DEST" ]] || usage
COPY="$DEST/petals"

if [[ "$CHECK" -eq 1 ]]; then
  [[ -f "$COPY/VERSION" ]] || { echo "vendor: $COPY/VERSION is missing; this copy carries no stamp"; exit 2; }
  stamp=$(head -1 "$COPY/VERSION")
  WORK=$(mktemp -d)
  trap 'rm -rf "$WORK"' EXIT
  awk 'NR>3 && NF==2 {print $2 "\t" $1}' "$COPY/VERSION" | LC_ALL=C sort > "$WORK/want"
  (cd "$COPY" && find . -type f ! -name VERSION | sed 's#^\./##' | LC_ALL=C sort | while IFS= read -r f; do printf '%s\t%s\n' "$f" "$(sha256 "$f")"; done) > "$WORK/have"
  problems=$(LC_ALL=C join -t "$(printf '\t')" -a1 -a2 -e MISSING -o 0,1.2,2.2 "$WORK/want" "$WORK/have" |
    awk -F'\t' '$2=="MISSING"{print "extra: " $1; next} $3=="MISSING"{print "missing: " $1; next} $2!=$3{print "changed: " $1}')
  if [[ -z "$problems" ]]; then
    echo "vendor: $COPY matches ${stamp}"
    exit 0
  fi
  echo "vendor: $COPY differs from ${stamp}:"
  printf '%s\n' "$problems" | sed 's/^/  /'
  echo "  Run vendor.sh again from that petals checkout to restore it, or from a newer one to update it."
  exit 1
fi

[[ -n "$FROM" ]] || FROM=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null) || { echo "vendor: $SCRIPT_DIR is not inside a git checkout; pass --from"; exit 2; }
git -C "$FROM" rev-parse --git-dir >/dev/null 2>&1 || { echo "vendor: $FROM is not a git checkout"; exit 2; }
[[ -d "$DEST" ]] || { echo "vendor: no such directory: $DEST"; exit 2; }
if [[ -n "$(git -C "$FROM" status --porcelain -- petals/)" ]]; then
  echo "vendor: $FROM has uncommitted changes under petals/, so the copy would match no commit. Commit them first."
  exit 2
fi
commit=$(git -C "$FROM" rev-parse HEAD)
tag=$(git -C "$FROM" describe --tags --exact-match HEAD 2>/dev/null || echo untagged)

# Build the copy in a staging folder, then sync it over the old one, so files
# petals no longer ships leave the copy too.
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/petals"
git -C "$FROM" ls-files -z -- petals/ | while IFS= read -r -d '' f; do
  mkdir -p "$STAGE/$(dirname "$f")"
  cp -p "$FROM/$f" "$STAGE/$f"
done
{
  echo "petals $tag $commit"
  echo "source https://github.com/jahala/petals"
  echo "files $(cd "$STAGE/petals" && find . -type f | wc -l | tr -d ' ')"
  (cd "$STAGE/petals" && find . -type f | sed 's#^\./##' | LC_ALL=C sort | while IFS= read -r f; do printf '%s  %s\n' "$(sha256 "$f")" "$f"; done)
} > "$STAGE/VERSION.tmp"
mv "$STAGE/VERSION.tmp" "$STAGE/petals/VERSION"
mkdir -p "$COPY"
rsync -a --delete "$STAGE/petals/" "$COPY/"
echo "vendor: $COPY is petals $tag $commit"
