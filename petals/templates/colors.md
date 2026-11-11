# Brand Colors: {{BRAND_NAME}}

<!--
  Template: colors.md
  Every colour value must be stated in the source materials as a hex, RGB or
  CMYK value. Do not infer a hex from a colour name or a visual sample.

  petals' scripts read this file. check.sh judges pages against every hex in
  it and against the Contrast Pairings table. export-tokens.sh derives the
  colour tokens from the CSS block at the end. Keep the headings and the table
  columns as they are, and write each hex with six digits.
-->

## Primary Palette

| Role | Hex | RGB | CMYK | Usage |
|------|-----|-----|------|-------|
| Primary | {{COLOR_PRIMARY_HEX}} | {{COLOR_PRIMARY_RGB}} | {{COLOR_PRIMARY_CMYK}} | {{COLOR_PRIMARY_USAGE}} |
| Accent | {{COLOR_ACCENT_HEX}} | {{COLOR_ACCENT_RGB}} | {{COLOR_ACCENT_CMYK}} | {{COLOR_ACCENT_USAGE}} |
| Text | {{COLOR_TEXT_HEX}} | {{COLOR_TEXT_RGB}} | {{COLOR_TEXT_CMYK}} | {{COLOR_TEXT_USAGE}} |
| Text (soft) | {{COLOR_TEXT_SOFT_HEX}} | {{COLOR_TEXT_SOFT_RGB}} | {{COLOR_TEXT_SOFT_CMYK}} | {{COLOR_TEXT_SOFT_USAGE}} |
| Background | {{COLOR_BG_HEX}} | {{COLOR_BG_RGB}} | {{COLOR_BG_CMYK}} | {{COLOR_BG_USAGE}} |
| Surface | {{COLOR_SURFACE_HEX}} | {{COLOR_SURFACE_RGB}} | {{COLOR_SURFACE_CMYK}} | {{COLOR_SURFACE_USAGE}} |
| Border | {{COLOR_BORDER_HEX}} | {{COLOR_BORDER_RGB}} | {{COLOR_BORDER_CMYK}} | {{COLOR_BORDER_USAGE}} |

<!-- Add a row for every other colour the source names (a hover shade, a
     decorative fill). A colour in no table is off-palette to check.sh. -->

## Semantic Roles

| Role | Hex | Usage |
|------|-----|-------|
| Error | {{COLOR_ERROR_HEX}} | {{COLOR_ERROR_USAGE}} |
| Success | {{COLOR_SUCCESS_HEX}} | {{COLOR_SUCCESS_USAGE}} |
| Warning | {{COLOR_WARNING_HEX}} | {{COLOR_WARNING_USAGE}} |
| Info | {{COLOR_INFO_HEX}} | {{COLOR_INFO_USAGE}} |

## Light / Dark Mode Variants

<!-- One row per colour the dark block below changes, with both values. A
     brand with no dark theme deletes this table and the dark block. -->

| Role | Light | Dark |
|------|-------|------|
| Primary | {{COLOR_PRIMARY_HEX}} | {{PRIMARY_DARK}} |
| Accent | {{COLOR_ACCENT_HEX}} | {{ACCENT_DARK}} |
| Text | {{COLOR_TEXT_HEX}} | {{TEXT_DARK}} |
| Text (soft) | {{COLOR_TEXT_SOFT_HEX}} | {{TEXT_SOFT_DARK}} |
| Background | {{COLOR_BG_HEX}} | {{BG_DARK}} |
| Surface | {{COLOR_SURFACE_HEX}} | {{SURFACE_DARK}} |
| Border | {{COLOR_BORDER_HEX}} | {{BORDER_DARK}} |

## Usage Rules

- Primary colour occupies about {{PRIMARY_RATIO}} of the visible UI.
- Accent colour is for {{ACCENT_USAGE}}.
- Never use the error colour for a state that is not an error.
- Body text meets 4.5:1 against its background; labels and large display type meet 3:1.

## Contrast Pairings

<!-- One row per text colour on a background it is used on, in both themes.
     The first cell names both colours with their hex, foreground first, as
     "Name #HEX on Name #HEX". Ratio is the measured WCAG 2.x contrast. Class
     is the brand's rule for the pair: reading, labels or decorative.
     check.sh holds a page to the class: a reading pair under 4.5:1 or a
     labels pair under 3.0:1 is an error, and a decorative pair set as words
     is always an error. -->

Classes: **reading** at least 4.5:1 (body copy, captions), **labels** at least 3.0:1 (labels, tags, type of 24px and up), **decorative** never words (marks, fills, washes).

| Foreground on background | Ratio | Class |
|---|---|---|
| {{FG_NAME_1}} {{FG_HEX_1}} on {{BG_NAME_1}} {{BG_HEX_1}} | {{RATIO_1}} | {{CLASS_1}} |
| {{FG_NAME_2}} {{FG_HEX_2}} on {{BG_NAME_2}} {{BG_HEX_2}} | {{RATIO_2}} | {{CLASS_2}} |

## Forbidden Combinations

- {{FORBIDDEN_COMBO_1}}
- {{FORBIDDEN_COMBO_2}}

## CSS Custom Properties

<!-- export-tokens.sh derives tokens.css from these two blocks.
     PREFIX is the brand's token namespace: one lowercase word of letters
     and digits, such as acme, which gives --acme-primary. Use the prefix the
     source's own custom properties share; when the source has none, use the
     brand name. Every property in both blocks carries it, and the scripts
     read it from here, so a block that mixes prefixes is refused.
     Keep the role names below: petals' pages and checks use them. Add a
     property for every other colour in the tables above. Each hex must also
     appear in a table row. -->

```css
:root {
  /* primary palette */
  --{{PREFIX}}-primary: {{COLOR_PRIMARY_HEX}};
  --{{PREFIX}}-accent: {{COLOR_ACCENT_HEX}};
  --{{PREFIX}}-text: {{COLOR_TEXT_HEX}};
  --{{PREFIX}}-text-soft: {{COLOR_TEXT_SOFT_HEX}};
  --{{PREFIX}}-bg: {{COLOR_BG_HEX}};
  --{{PREFIX}}-surface: {{COLOR_SURFACE_HEX}};
  --{{PREFIX}}-border: {{COLOR_BORDER_HEX}};

  /* semantic */
  --{{PREFIX}}-error: {{COLOR_ERROR_HEX}};
  --{{PREFIX}}-success: {{COLOR_SUCCESS_HEX}};
  --{{PREFIX}}-warning: {{COLOR_WARNING_HEX}};
  --{{PREFIX}}-info: {{COLOR_INFO_HEX}};
}

[data-theme="dark"] {
  --{{PREFIX}}-primary: {{PRIMARY_DARK}};
  --{{PREFIX}}-accent: {{ACCENT_DARK}};
  --{{PREFIX}}-text: {{TEXT_DARK}};
  --{{PREFIX}}-text-soft: {{TEXT_SOFT_DARK}};
  --{{PREFIX}}-bg: {{BG_DARK}};
  --{{PREFIX}}-surface: {{SURFACE_DARK}};
  --{{PREFIX}}-border: {{BORDER_DARK}};
}
```
