# Surface & Motion: {{BRAND_NAME}}

<!--
  Template: components.md
  Machine-readable surface and motion tokens: radius, borders, shadows,
  easings, durations, component conventions.
  Every value must be stated in the source materials (design system docs,
  CSS, component libraries). Do not invent a scale the brand never defined:
  a missing token is flagged, not guessed.
  export-tokens.sh reads the Token and Value columns below, and check.sh
  holds pages to the radius, stroke and shadow values. A value that is prose
  rather than a CSS value is skipped. A token becomes --PREFIX-<token>
  (PREFIX as in colors.md); keep the radius-, stroke- and shadow- names.
-->

## Radius Scale

| Token | Value | Usage |
|-------|-------|-------|
| radius-xs | {{RADIUS_XS}} | {{RADIUS_XS_USAGE}} |
| radius-sm | {{RADIUS_SM}} | {{RADIUS_SM_USAGE}} |
| radius-md | {{RADIUS_MD}} | {{RADIUS_MD_USAGE}} |
| radius-lg | {{RADIUS_LG}} | {{RADIUS_LG_USAGE}} |
| radius-full | {{RADIUS_FULL}} | Pills, badges, dots |

## Border Strokes

| Token | Value | Usage |
|-------|-------|-------|
| stroke-hairline | {{STROKE_HAIRLINE}} | Structural borders, dividers |
| stroke-emphasis | {{STROKE_EMPHASIS}} | Emphasized borders, underlines |
| stroke-rule | {{STROKE_RULE}} | Accent rules, markers |

## Shadow Recipes

State the shadow colour policy (for example ink-tinted, never pure black)
and the full CSS value of each recipe.

<!-- A brand with a dark theme that states its night shadows adds a Night
     column after Value: | Token | Value | Night | Usage |. Each cell is the
     recipe's full CSS value in the dark theme, as the source states it.
     export-tokens.sh writes it into the [data-theme="dark"] block of
     tokens.css and the dark group of tokens.json, and check.sh holds a
     page's dark-theme box-shadows to it. Leave a cell empty when the source
     gives no night value for that recipe, and leave the column out when it
     gives none: no night value is guessed, and without a dark theme in
     colors.md the column is not read. -->

| Token | Value | Usage |
|-------|-------|-------|
| shadow-rest | {{SHADOW_REST}} | Resting panes |
| shadow-lift | {{SHADOW_LIFT}} | Hover lift |
| shadow-float | {{SHADOW_FLOAT}} | Floating artifacts |

## Motion Tokens

<!-- Value is one cubic-bezier() or easing keyword, or one duration such as
     0.3s. A range (0.15-0.3s) stays prose and is not a token. -->

| Token | Value | Usage |
|-------|-------|-------|
| ease | {{MOTION_EASE}} | Entrances and settles |
| dur-entrance | {{MOTION_DUR_ENTRANCE}} | Section / card entrances |
| dur-micro | {{MOTION_DUR_MICRO}} | Hovers, color shifts |
| stagger-step | {{MOTION_STAGGER}} | Sibling delay inside groups |

Rules:

- {{MOTION_PRINCIPLES}}
- `prefers-reduced-motion: reduce` MUST disable all entrances and staggers.
- Forbidden: {{MOTION_FORBIDDEN}}

## Component Conventions

| Component | Convention |
|-----------|------------|
| Button (primary) | {{COMPONENT_BUTTON_PRIMARY}} |
| Card | {{COMPONENT_CARD}} |
| Focus state | {{COMPONENT_FOCUS}} |

## Applying in frameworks

The brand wins over framework defaults. Mirror tokens as CSS custom
properties; extend a CSS framework's theme (`borderRadius`, `boxShadow`,
`transitionTimingFunction`) instead of using stock utilities; override
component-library theme variables once at the theme layer.
