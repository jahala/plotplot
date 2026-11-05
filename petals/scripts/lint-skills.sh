#!/usr/bin/env bash
# lint-skills.sh: find the skills a harness would drop without a word (issue #7).
#
# Usage: lint-skills.sh [project-dir]   (default: the current directory)
#
# Every SKILL.md under the project (petals/, .claude/skills/, .agents/skills/
# and the like, a few levels deep; copies kept under docs/ or test fixtures are
# documentation, not installed skills) is checked for three things:
#   - its frontmatter has a non-empty description;
#   - its name equals the directory it sits in;
#   - an always-on instruction file at the project root (AGENTS.md, CLAUDE.md,
#     .claude/CLAUDE.md) names it.
# Each failure is one ERROR line naming the file. Exit 0 when every skill
# passes, 1 when any fails, 2 when the directory does not exist.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SCRIPT_DIR/lib/common.sh" ]]; then
  echo "petals: lib/common.sh is missing beside $(basename "$0"); install the whole petals/ directory" >&2
  exit 127
fi
source "$SCRIPT_DIR/lib/common.sh"

DIR=$(from_caller "${1:-.}")
[[ -d "$DIR" ]] || { echo "ERROR no such directory: $DIR"; exit 2; }

ALWAYS_ON=()
for f in AGENTS.md CLAUDE.md .claude/CLAUDE.md; do
  [[ -f "$DIR/$f" ]] && ALWAYS_ON+=("$DIR/$f")
done

fails=0; count=0
while IFS= read -r skill; do
  count=$((count+1))
  rel=${skill#"$DIR"/}
  dir=$(basename "$(dirname "$skill")")
  # frontmatter: the lines between the first two '---' lines
  front=$(awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} NR>1{print}' "$skill")
  name=$(printf '%s\n' "$front" | sed -nE 's/^name:[[:space:]]*"?([^"]*)"?[[:space:]]*$/\1/p' | head -1)
  # a description is a non-empty value on its own line, or a folded block below it
  desc=$(printf '%s\n' "$front" | awk '/^description:/{d=1; v=$0; sub(/^description:[[:space:]]*/, "", v); if(v!="" && v!=">-" && v!=">" && v!="|" && v!="|-"){print v; exit} next} d && /^[[:space:]]+[^[:space:]]/{print; exit} d && /^[^[:space:]]/{exit}')
  if [[ -z "$desc" ]]; then echo "ERROR $rel: no description; the harness cannot tell when to load it."; fails=$((fails+1)); fi
  if [[ -z "$name" ]]; then echo "ERROR $rel: no name"; fails=$((fails+1))
  elif [[ "$name" != "$dir" ]]; then echo "ERROR $rel: name \"$name\" differs from its directory \"$dir\"; the harness finds it by the directory."; fails=$((fails+1)); fi
  check_name=${name:-$dir}
  named=0
  for f in ${ALWAYS_ON[@]+"${ALWAYS_ON[@]}"}; do
    grep -qiwF -- "$check_name" "$f" && { named=1; break; }
  done
  if [[ "$named" -eq 0 ]]; then echo "ERROR $rel: no always-on instruction (AGENTS.md, CLAUDE.md) names \"$check_name\"."; fails=$((fails+1)); fi
done < <(find "$DIR" -maxdepth 5 -name SKILL.md -not -path '*/node_modules/*' -not -path '*/.git/*' -not -path '*/test/fixtures/*' -not -path '*/docs/*' | sort)

if [[ "$fails" -eq 0 ]]; then echo "ok: $count skill(s) checked, none would be dropped."; exit 0; fi
echo "FAIL: $fails problem(s) in $count skill(s)."
exit 1
