# /petals logo `[variant]`

When `/petals logo` is invoked, the agent hands out the logo *with its rules* — the asset alone invites redrawing.

1. **Guard**: if `.brand/` does not exist, report: `No brand configured. Run /petals init first.` and stop.
2. **Resolve the variant**:
   - default → the product's mark when a product context is set (`.brand/products/<product>/assets/<product>-mark.svg`), else the umbrella's mark in `.brand/assets/` (the `*-mark.svg` file `identity.md` names)
   - `night` → the same file with `-night` before `.svg`, for dark surfaces
   - `mono` → the all-ink variant if present; if absent, instruct: copy the mark and set every path fill to the brand ink colour — the ONLY permitted recolour.
3. **Return** the asset path AND print the "Logo Usage" rules from `.brand/identity.md` (minimum size, clear space, exact fills, the never-list).
4. **When placing the logo in generated UI**: copy the asset file or inline its exact markup — never redraw, approximate, or restyle it. The color audit treats a logo with altered fills as an **error** (it is not a palette question; it is the mark).
