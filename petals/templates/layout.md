# Layout: {{BRAND_NAME}}

<!--
  Template: layout.md
  How elements relate: spacing scale, containers, breakpoints, density.
  Every value must be stated in the source materials (design system docs,
  CSS, grid specs). Do not invent a scale the brand never defined: a missing
  value is flagged, not guessed.
  export-tokens.sh reads the Token and Value columns below; a value that is
  prose rather than a CSS length is skipped.
-->

## Spacing Scale

<!-- One row per step. Token is a short name (sm, md, 2xl), which becomes
     --PREFIX-space-<token> (PREFIX as in colors.md). Value is a length such as 16px. -->

| Token | Value | Usage |
|-------|-------|-------|
| {{SPACING_TOKEN}} | {{SPACING_VALUE}} | {{SPACING_USAGE}} |

State the rule if the scale is generative (e.g. "multiples of 4px").

## Containers & Measure

<!-- The container Value is its width followed by "max-width", such as
     "1200px max-width, centered". The gutter Value is a length. -->

| Token | Value | Usage |
|-------|-------|-------|
| container | {{CONTAINER_MAX_WIDTH}} max-width | The content container |
| gutter | {{CONTAINER_GUTTER}} | Container horizontal padding |
| measure | {{READING_MEASURE}} | Reading-column width bounds |

## Breakpoints

<!-- One row per breakpoint. Breakpoint is the width in px; Token names it,
     which gives --PREFIX-breakpoint-<token>. check.sh flags a @media width
     not listed here. -->

| Breakpoint | Token | What changes |
|------------|-------|--------------|
| {{BREAKPOINT_WIDTH}} | {{BREAKPOINT_TOKEN}} | {{BREAKPOINT_BEHAVIOR}} |

## Density & Rhythm

- {{DENSITY_PRINCIPLES}}
