#!/usr/bin/env bash
# review.sh: show the owner pages in their own browser.
#
# Usage:
#   review.sh <page> [--note <what to look for>] [<page> [--note ...]]... [--no-open]
#   review.sh --stop
#
# Starting writes an index page in the brand into <project>/.petals-review/
# (kept out of commits through git's info/exclude), serves the project on a
# free 127.0.0.1 port from a detached server, so a harness time limit cannot
# end it mid-review, opens the index, and prints the stop command. Running it
# again adds pages to the same index on the same server. --stop ends the
# server and hands .petals-review/ to trash; without trash it names the folder
# and leaves it. Nothing is rm'd.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SCRIPT_DIR/lib/common.sh" ]]; then
  echo "petals: lib/common.sh is missing beside $(basename "$0"); install the whole petals/ directory" >&2
  exit 127
fi
source "$SCRIPT_DIR/lib/common.sh"

STOP=0; OPEN=1
PAGES=(); NOTES=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --stop)    STOP=1; shift ;;
    --no-open) OPEN=0; shift ;;
    --note)    [[ ${#PAGES[@]} -gt 0 ]] || { echo "review: --note follows the page it describes" >&2; exit 2; }
               NOTES[${#PAGES[@]}-1]="${2:-}"; shift 2 ;;
    -*)        echo "review: unknown option $1" >&2; exit 2 ;;
    *)         PAGES+=("$(from_caller "$1")"); NOTES+=(""); shift ;;
  esac
done

ROOT=$(find_project_root "$CALLER_PWD") || { echo "review: no project here (no .petalsrc or .brand/ above $CALLER_PWD)" >&2; exit 2; }
cd "$ROOT" || exit 2
DIR="$ROOT/.petals-review"

server_alive() {
  [[ -f "$DIR/server.pid" ]] || return 1
  local pid; pid=$(cat "$DIR/server.pid")
  ps -p "$pid" -o command= 2>/dev/null | grep -q 'http.server' || return 1
}

if [[ "$STOP" -eq 1 ]]; then
  if server_alive; then kill "$(cat "$DIR/server.pid")" 2>/dev/null; fi
  if [[ -e "$DIR" ]]; then
    if command -v trash >/dev/null 2>&1; then trash "$DIR"
    else echo "review: the server is stopped; remove $DIR with trash when convenient" >&2; fi
  fi
  echo "review: stopped"
  exit 0
fi

[[ ${#PAGES[@]} -gt 0 ]] || { echo "usage: review.sh <page> [--note <text>]... [--no-open] | review.sh --stop" >&2; exit 2; }
# the index is styled with the brand's tokens, under the brand's own prefix
prefix=$(brand_prefix "$ROOT/.brand/colors.md") || { echo "review: $prefix" >&2; exit 2; }
mkdir -p "$DIR"
# ask git where the exclude file is: in a worktree .git is a file, not a folder
if exclude=$(git -C "$ROOT" rev-parse --path-format=absolute --git-path info/exclude 2>/dev/null); then
  mkdir -p "$(dirname "$exclude")"
  grep -qx '.petals-review/' "$exclude" 2>/dev/null || echo '.petals-review/' >> "$exclude"
fi

# entries: one "path<TAB>note" per page, the first note kept, in the order added
touch "$DIR/entries.tsv"
for i in "${!PAGES[@]}"; do
  p=${PAGES[$i]}
  [[ -f "$p" ]] || { echo "review: no such page: $p" >&2; exit 2; }
  case "$p" in "$ROOT"/*) rel=${p#"$ROOT"/} ;; *) echo "review: $p is outside the project $ROOT" >&2; exit 2 ;; esac
  cut -f1 "$DIR/entries.tsv" | grep -qxF -- "$rel" || printf '%s\t%s\n' "$rel" "${NOTES[$i]:-}" >> "$DIR/entries.tsv"
done

# the brand's own font loading: the <link> lines typography.md gives for HTML pages
fontlinks=$(grep -hE '^[[:space:]]*<link [^>]*(fonts\.googleapis\.com|fonts\.gstatic\.com|rel="stylesheet")' "$ROOT/.brand/typography.md" 2>/dev/null | sed 's/^[[:space:]]*/  /')

python3 -I - "$DIR/entries.tsv" "$DIR/index.html" "$(basename "$ROOT")" "$fontlinks" "$prefix" <<'PY'
import html, sys
entries = [l.rstrip("\n").split("\t", 1) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
items = []
for path, *rest in entries:
    note = rest[0] if rest else ""
    items.append('      <li><a href="../%s">%s</a>%s</li>' % (
        html.escape(path, quote=True), html.escape(path),
        '\n        <p class="note">%s</p>' % html.escape(note) if note else ""))
page = """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Review: %(project)s</title>
%(fonts)s
  <link rel="stylesheet" href="../.brand/tokens.css">
  <style>
    html { font-synthesis: none; }
    body { margin: 0; background: var(%(p)sbg); color: var(%(p)stext); font-family: var(%(p)sfont-body); line-height: 1.6; }
    main { max-width: var(%(p)scontainer-max); margin: 0 auto; padding: 48px 24px; }
    h1 { font-family: var(%(p)sfont-heading); font-weight: 400; font-size: 2.4rem; line-height: 1.12; margin: 0 0 8px; }
    .lede { color: var(%(p)stext-soft); margin: 0 0 32px; }
    ol { list-style: none; padding: 0; margin: 0; }
    li { border-top: 1px solid var(%(p)sborder); padding: 16px 0; }
    a { color: var(%(p)sprimary); font-weight: 600; }
    .note { color: var(%(p)stext-soft); margin: 4px 0 0; }
    footer { margin-top: 32px; color: var(%(p)stext-soft); }
    code { font-family: var(%(p)sfont-code); }
  </style>
</head>
<body>
  <main>
    <h1>Pages to look at</h1>
    <p class="lede">%(project)s. Open each page here; what to look for is under it.</p>
    <ol>
%(items)s
    </ol>
    <footer>When you are done, run <code>petals review --stop</code>.</footer>
  </main>
</body>
</html>
""" % {"project": html.escape(sys.argv[3]), "items": "\n".join(items), "fonts": sys.argv[4], "p": sys.argv[5]}
open(sys.argv[2], "w", encoding="utf-8").write(page)
PY

if ! server_alive; then
  port=$(python3 -I -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
  nohup python3 -I -m http.server "$port" --bind 127.0.0.1 --directory "$ROOT" > "$DIR/server.log" 2>&1 &
  echo $! > "$DIR/server.pid"
  echo "$port" > "$DIR/port"
  disown 2>/dev/null || true
  # wait for the server to answer, up to a limit that only catches a hang
  for _ in $(seq 1 100); do
    python3 -I -c 'import socket,sys; s=socket.socket(); s.settimeout(0.2); sys.exit(s.connect_ex(("127.0.0.1", int(sys.argv[1]))))' "$port" && break
    sleep 0.1
  done
fi
url="http://127.0.0.1:$(cat "$DIR/port")/.petals-review/index.html"
if [[ "$OPEN" -eq 1 ]]; then
  if command -v open >/dev/null 2>&1; then open "$url"; elif command -v xdg-open >/dev/null 2>&1; then xdg-open "$url" >/dev/null 2>&1; fi
fi
echo "review: $url"
echo "stop:   petals review --stop"
