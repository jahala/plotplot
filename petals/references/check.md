# /petals check `<file>`

Run the deterministic check first:

```
bash scripts/check.sh <file>                  # text
bash scripts/check.sh <file> --format sarif   # one SARIF 2.1.0 log
```

It judges colour, typography families and weights, layout, surface, voice terms, content, reuse (a control rebuilt where a component contract covers it) and the accessibility a page's source shows (alt text, labels, button names, focus, zoom, clickable divs), in milliseconds, with no model. Its rules live in `scripts/check.sh` and its tests, and its messages say what to change. Exit 0 is clean or warnings only, exit 1 is at least one error, exit 3 means it could not judge (no brand, for example) and says why. Exit 3 is never a pass.

The script cannot judge four things. They are your part, after it runs.

## Typography roles

The script checks which families a page loads and declares, and the weights and styles each face ships. Whether a heading or body element got the right face needs context, so it is yours:

1. Extract font families from `.brand/typography.md`: heading font and body font (from the "Font Families" table).
2. Read the target file and scan for `font-family` declarations. In CSS, JSX/TSX, and other code files, look for patterns like:
   - `font-family: 'Raleway'` / `fontFamily: 'Raleway'`
   - `font-family: "Open Sans"` / `fontFamily: "Open Sans"`
3. For each `font-family` declaration, determine if the element is a heading or body:
   - Headings: `h1`, `h2`, `h3`, `h4`, `h5`, `h6` elements, or class names containing `heading`, `title`, `headline`
   - Body: `p`, `span`, `div`, `body`, `li`, `a`, `button`, `label`, `input` elements, or class names containing `body`, `text`, `content`, `paragraph`, `copy`
   - If the role is ambiguous (generic `div` or `span` without clear context), default to body-level checking.
4. Check each declaration:
   - **Heading element uses heading font**: No violation.
   - **Heading element uses body font**: Warning (suggest heading font).
   - **Body element uses body font**: No violation.
   - **Body element uses heading font**: **Error** — body text must use the body font.
5. Report typography errors as:
   ```
   ERROR [typography] line <N>: '<found-font>' used for body text. Body must use '<body-font>'.
   ```

If zero typography violations:
```
PASS [typography] Font families match the brand hierarchy.
```

## Tone

Read the copy against the tone rules in `.brand/voice.md`, following steps 4 to 7 of `references/voice-guide.md`. Report tone findings as warnings. Also dismiss a forbidden-term hit the script made inside quoted output it did not recognise as quoted, and say why.

## Chart series order

A categorical series that does not follow the order in colors.md's "Data Visualization" section is a warning. The script checks each chart colour against the palette, not the order.

## The brand book (only for .brand/book.html)

When the target file is `.brand/book.html`, run a staleness check before the script runs. The book is a generated view of `.brand/` — it has no authority of its own. Every CSS value in the book must match the canonical source files.

**Process**:

1. **Read canonical sources**: Load `.brand/colors.md` (CSS custom properties block in the `:root`) and `.brand/typography.md` (font families, font imports). Read the book's `<meta name="source-fingerprint">` tag.

2. **Recompute the fingerprint**: Concatenate all `.brand/*.md` files (plus product layers in `.brand/products/*/*.md` if present) and compute `sha256`. Compare against the book's `source-fingerprint` meta tag.
   - **Match**: Book is current. Run the script.
   - **Mismatch or missing**: Report drift:
     ```
   ERROR [book] book.html is stale — fingerprint does not match canonical .brand/ sources.
   Regenerate with /petals book.
     ```
     Exit non-zero. Do not run the script — a stale book needs regeneration, not dimension-by-dimension repair.

3. **Verify CSS :root values** (if fingerprint matches): For each color custom property under the brand's token prefix (the one colors.md's CSS block declares, `--pp-*` for plotplot), confirm the hex matches the value in `.brand/colors.md`. For `--font-*` properties, confirm the font family matches `.brand/typography.md`. Mismatches are errors: the book is a view, and every value must come from the canonical files.

## Letting a finding through

A finding can go through on the record with a `Petals-allow: <rule> <reason>` trailer on the commit. It stays in the log as a note with its reason. A trailer without a reason allows nothing. See `references/guardrails.md` for the gate and the hooks.
