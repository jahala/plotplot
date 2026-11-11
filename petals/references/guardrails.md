# Guardrails (edit-time and merge-time)

The SKILL auto-trigger covers generation time. Two more lines of defense, both optional recipes a project can adopt:

**Edit-time — a deterministic hook.** The palette check is pure text matching; a Claude Code `PostToolUse` hook can run it on every edited UI file without an agent in the loop:

```json
{
  "hooks": {
    "PostToolUse": [{
      "matcher": "Edit|Write",
      "hooks": [{ "type": "command", "command": "sh .claude/hooks/petals-hex-guard.sh" }]
    }]
  }
}
```

where `petals-hex-guard.sh` wraps the deterministic engine:

```sh
#!/bin/sh
# nudge, never block: report findings, exit 0
bash scripts/check.sh "$1" 2>/dev/null | grep -E 'ERROR|WARNING' || true
```

**Merge-time — CI.** A pull-request job runs the full check on changed UI files:

```yaml
- name: brand check (deterministic — gates)
  run: |
    for f in $(git diff --name-only origin/main -- '*.tsx' '*.css' '*.html'); do
      bash scripts/check.sh "$f" || exit 1
    done
- name: brand check (agent — typography + tone, advisory)
  run: claude -p "/petals check $(git diff --name-only origin/main -- '*.tsx' '*.css' '*.html' | tr '\n' ' ')"
```

The deterministic engine is exact and gates the merge; the agent pass covers typography and tone and advises. No tokens are spent unless the cheap gate passes.

**Merge-time, `petals gate`.** `petals/bin/petals gate [--base <ref>] [--format sarif|text]` is the single entry point for hosts that run one command and read one result. It finds the UI files a change touches (`--base <ref>` judges `<ref>...HEAD`; without it, the working tree against HEAD, untracked files included), runs `check.sh` on each, and writes one SARIF 2.1.0 log. Any error-level result makes the exit status 1. The gate reads `.petalsrc` and `.brand/` as the base carries them, never the change's copy, so a change cannot loosen its own rules. A `gate:` section with an `exclude:` list of globs in `.petalsrc` skips paths. With no brand at the base or in the working tree it exits 3 and prints nothing on stdout. A finding is allowed through on the record with a `Petals-allow: <rule> <reason>` trailer, in the commit being prepared (`--message-file`, which a commit-msg hook passes) or in any commit of the change: it stays in the log as a note whose SARIF suppression carries the reason. `--strict` reports it at its own level again. A trailer without a reason allows nothing.

**Commit-time, `petals guard <hook>`.** The stem's planted git hooks call `petals guard pre-commit`, which judges the index (what will be committed, not the working tree), and `petals guard commit-msg <message-file>`, which judges it again with that message's `Petals-allow:` trailers. Exit 0 lets the commit through; otherwise each blocking finding is printed on stderr and the commit is refused. Other hooks pass through. A planted repository gets `petals` from a release: one tarball whose only member is a self-contained `petals` that unpacks its files into `${XDG_CACHE_HOME:-~/.cache}/petals/<content sha>/` on first run (one copy per version; safe to delete). `bash scripts/build-release.sh <dir>` builds it, byte-identical for one commit, with its sha256.
