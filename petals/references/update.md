# /petals update Workflow (Sync)

When `/petals update` is invoked, the agent syncs the local `.brand/` with the latest version from the central repository while preserving project-level overrides.

### Step-by-step process

1. **Verify the project is connected** — Read `.petalsrc`. If it does not exist or has no `brand.source` field:
   ```
   This project is not connected to a brand source.
   Run /petals init --from <repo-url> first.
   ```
   Stop.

2. **Back up current .brand/** — Before making any changes, back up the current `.brand/` directory:
   ```bash
   cp -r .brand .brand.backup
   ```
   This ensures the current state is preserved if anything fails.

3. **Read current overrides** — Parse `.petalsrc` to extract the `overrides` map. Each override is a flattened key with a dot-separated path:
   ```yaml
   overrides:
     colors.primary: "#custom-hex"    # non-null: preserve this value
     typography.body: null             # null: inherit from central
   ```
   Store these in memory for the merge step.

4. **Run fetch-brand.sh** — Fetch the latest `.brand/` from the central source:
   ```bash
   bash scripts/fetch-brand.sh <source-from-petalrc> --target .brand.incoming
   ```
   Fetch into a temporary directory `.brand.incoming`, NOT directly into `.brand/`. If the script exits non-zero, restore from backup and report the error.

5. **Detect version from incoming** — Read the version from the fetched `.brand.incoming/identity.md` or from the fetch script output. Compare with `.petalsrc` version. If they are the same:
   ```
   Brand is already at the latest version (<version>). Nothing to update.
   ```
   Clean up `.brand.incoming` and `.brand.backup`. Stop.

6. **Merge overrides** — For each override in `.petalsrc`:
   - **Non-null value** (e.g., `colors.primary: "#OVERRIDE"`): Find and replace the corresponding value in `.brand.incoming/` files. The exact replacement strategy depends on the file format:
     - For `.brand/colors.md`: Replace the hex value in the table row and CSS custom property.
     - For `.brand/typography.md`: Replace the font family, weight, or size in the table row.
     - For `.brand/voice.md`: Replace the specific term or rule.
   - **Null value**: Do nothing — the central's current value will be used (it is already in `.brand.incoming/`).

7. **Apply the update** — Replace the live `.brand/` with the merged incoming. When this project owns a product layer (`product:` in `.petalsrc`), carry the committed layer across first, so the swap never deletes it:
   ```bash
   # only when .petalsrc names a product that owns its layer
   rsync -a .brand/products/<product>/ .brand.incoming/products/<product>/
   trash .brand
   mv .brand.incoming .brand
   ```

8. **Update .petalsrc version** — Update the `version` field in `.petalsrc` to the new version tag.

9. **Regenerate the machine views** — `.brand/tokens.css` and `.brand/tokens.json` are derived from the markdown files (see `references/extraction-guide.md` §5a); regenerate both with `bash scripts/export-tokens.sh`. If `.brand/book.html` exists, regenerate it too.

10. **Report the blast radius** — an update is not done until the project knows what it broke. Run `/petals check` against the project's primary UI entry points (or the files checked most recently). Report anything that the NEW brand version flags which the old one did not:
    ```
    Blast radius: 3 files have violations introduced by this update.
      src/components/Hero.tsx — 2 color errors (old primary #4C5E3C)
      …
    ```
    Zero findings reports `Blast radius: clean.`

11. **Clean up** — Remove the backup:
    ```bash
    trash .brand.backup
    ```

12. **Report summary** — Tell the user:
    ```
    Updated brand from <old-version> to <new-version>.
    Preserved <N> override(s): <list of preserved keys>
    Updated <M> inherited values. Blast radius: <clean | N files to fix>.
    ```

## /petals check (Drift Detection)

When `/petals check` is invoked without a file argument, it checks for brand drift — whether the central repository has a newer version than what is cached locally.

### Step-by-step process

1. **Read .petalsrc** — Get the `source` and `version` fields. If `.petalsrc` does not exist:
   ```
   No brand source configured. Run /petals init --from <repo-url> first.
   ```
   Stop.

2. **Fetch central version** — Run `bash scripts/check-brand-drift.sh`. It reads `source` and `version` from `.petalsrc`, finds the central version (a local path's `.brand/identity.md`, else the highest semver tag from `git ls-remote --tags`, else `identity.md` from raw.githubusercontent.com), and prints one JSON line. Exit codes: 0 current, 1 drift, 2 unreachable, 3 no source configured, 4 local ahead of central.

   If the central source is unreachable:
   ```
   Warning: Cannot reach brand source (<source-url>).
   Local .brand/ at version <local-version> is preserved unchanged.
   Error: <specific error>
   ```
   Do not modify any local files. Exit gracefully.

3. **Compare versions** — Compare the local version (from `.petalsrc`) with the central version:
   - If **local == central** (exit 0): Report nothing to the user. (No noise when current.)
   - If **local < central**: Report the drift:
     ```
     Brand drift detected.
     Local:  <local-version>
     Central: <central-version>
     Run /petals update to sync.
     ```
   - If **local > central**: This is unusual (local ahead of central). Report:
     ```
     Local brand (<local-version>) is ahead of central (<central-version>).
     This may indicate the central repo was rolled back. Review before updating.
     ```
