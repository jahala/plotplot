# /petals init Workflow

When `/petals init <folder>` is invoked, follow the extraction process documented in `references/extraction-guide.md`. Core principles:

1. **Explicit only** — only write values explicitly stated in source. Never infer hex from color names, weights from font names, or examples from tone descriptions.
2. **Flag ambiguity** — when a value is missing or unclear, flag it in the output with a clear question.
3. **Copy assets verbatim** — use `cp` or equivalent to copy asset files (SVGs, PNGs, JPGs, fonts) to `.brand/assets/`.
4. **Use templates** — populate `templates/*.md` with extracted values to produce `.brand/*.md`.
5. **Use the tools the agent has** — for file-based sources, use the agent's native capabilities: file reading, vision, parsing, Bash. For website URLs, use Playwright MCP (`browser_navigate`, `browser_evaluate`, `browser_snapshot`) and WebFetch — tools the agent already carries. Never install new dependencies. Follow `references/extraction-guide.md` for the full process.

## /petals init --from Workflow (Distribution)

When `/petals init --from <repo-url>` is invoked, the agent connects the project to a central brand repository and fetches `.brand/` into the local project.

### Distribution model

The central brand repository is a git repository with `.brand/` at its root. Projects pull `.brand/` from it — they do not commit `.brand/` themselves. `.brand/` is a cache. `.petalsrc` at the project root records where the brand came from and which version is in use. One exception: a product in a family brand owns its product layer, so it commits `.brand/products/<product>/` and ignores the rest of `.brand/`.

```
Central brand repo (github.com/company/brand, tagged v1.2.0)
        |
        |  fetch-brand.sh
        v
Project .brand/  (gitignored — cache, not source)
Project .petalsrc  (committed — source, version, overrides)
```

### Step-by-step process

1. **Check for existing .brand/** — If `.brand/` already exists, warn the user:
   ```
   .brand/ already exists (version v1.1.0 from github.com/company/brand).
   Running init --from will overwrite it. Continue? (y/N)
   ```
   Wait for confirmation. If denied, exit without changes. If `.brand/` does not exist, proceed.

2. **Check for existing .petalsrc** — If `.petalsrc` already exists, do NOT modify it without confirmation. Warn:
   ```
   .petalsrc already exists with source: github.com/other/repo.
   Overwriting will change the brand source. Continue? (y/N)
   ```
   Wait for confirmation.

3. **Run fetch-brand.sh** — Execute the fetch script with the provided repo URL:
   ```bash
   bash scripts/fetch-brand.sh <repo-url> --target .brand
   ```
   The script auto-selects the best strategy (local path, git clone, GitHub API, raw HTTP). If the script exits non-zero, report the error to the user and STOP. Do not create or modify any files. A file the project commits under the target (a product layer it owns, for example) is never overwritten: the script keeps it and names it on stderr.

4. **Record source and version in .petalsrc** — Parse the JSON output from fetch-brand.sh to get the version. Create or update `.petalsrc`:
   ```yaml
   brand:
     source: <repo-url>
     version: <detected-version>   # e.g. v1.1.0
     product: <product-name>       # optional: this project's product context
                                   # in a family brand (resolves products/<name>/)
     overrides: {}
   ```
   When the fetched brand carries `products/`, ask which product this project is (or none, to use the umbrella alone) and record it — the check, book, and tokens resolve against it.
   The `overrides` map starts empty. Users add entries to override specific brand values for this project (see `references/petalsrc.md`).

5. **Add .brand/ to .gitignore** — Check if `.gitignore` exists. If not, create it. Append `.brand/` to `.gitignore` if not already present. Use:
   ```bash
   grep -qxF '.brand/' .gitignore 2>/dev/null || echo '.brand/' >> .gitignore
   ```
   When this project is a product with its own layer (`product:` is set in `.petalsrc`), ignore the umbrella's files and keep the layer instead:
   ```
   /.brand/*
   !/.brand/products/
   /.brand/products/*
   !/.brand/products/<product>/
   ```

6. **Verify output** — Confirm the output is correct:
   - `.brand/` directory exists and contains files
   - `.petalsrc` exists with `source`, `version`, and `overrides` fields
   - `.gitignore` contains `.brand/`
   - `git status` does not show `.brand/` as untracked (if in a git repo)

7. **Report summary** — Tell the user:
   ```
   Connected to brand: <repo-url>
   Version: <version>
   Files in .brand/: <count>
   .brand/ is gitignored (cache, not source code)
   ```
