# Where petals lives

petals has two pieces, one lifecycle each:

| Piece | What it is | Where it lives | Lifecycle |
|-------|-----------|---------------|-----------|
| **Skill directory** | SKILL.md, scripts, references, templates | Your platform's skills directory | Install once. Update when petals improves. |
| **Brand directory** | `.brand/` — colors, typography, voice, identity, layout, components | Project root | Fetched from central brand repo. Updates when the brand changes. |

Never put `.brand/` inside the skill directory. Never duplicate the scripts into projects.

### Install (once, anywhere)

petals is a single directory. Install it wherever your agent looks for skills:

| Platform | Install path |
|----------|-------------|
| Claude Code | `~/.claude/skills/petals/` |
| Codex CLI | `~/.codex/skills/petals/` |
| Gemini CLI | `~/.gemini/skills/petals/` |
| Cross-platform | `~/.agents/skills/petals/` (Codex, OpenCode, Gemini) |

Use a symlink — it stays current with the repo:

```bash
ln -s /path/to/petals ~/.claude/skills/petals
```

The agent discovers `SKILL.md` on session start. No project-level setup needed. No copy-paste into every project. `bash scripts/lint-skills.sh [project-dir]` reports any skill a harness would drop without a word: no description, a name that differs from its directory, or no always-on instruction (AGENTS.md, CLAUDE.md) that names it.

### How scripts are found

Every script path in SKILL.md and in these references is relative to the skill directory, the folder that holds SKILL.md. `scripts/check.sh` means "the `scripts/` directory next to `SKILL.md`" — whether petals was installed globally or per-project. The agent resolves `{skill-root}/scripts/check.sh` from wherever it loaded this file.

The scripts find `.brand/` by walking up from `$PWD` — they don't need to be in the project. `scripts/lib/common.sh` does this automatically.

### CI (the one reason to put petals in a project)

If you want the deterministic check engine in CI, copy or symlink the skill directory into the project before the CI job runs. The scripts don't need a per-project copy otherwise. A repository that keeps a committed copy (the plotplot umbrella does) takes it with `bash scripts/vendor.sh <repo>` from a clean petals checkout, which stamps `petals/VERSION` with the commit, its tag and a checksum per file, and runs `bash petals/scripts/vendor.sh --check .` in CI so a hand-edited or stale copy fails.
