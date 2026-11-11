# Brand Typography: {{BRAND_NAME}}

<!--
  Template: typography.md
  Font weights and styles must be stated in the source. Do not guess weights
  from a family name.
-->

## Font Families

<!-- One row per role. Weight lists every weight the face ships as the brand
     loads it: single weights (400, 600) or a variable range (400..700).
     Style is normal, or normal, italic when the italic is loaded too.
     check.sh reads this table: a page that asks a face for a weight or style
     not listed here gets it faked by the browser, and that is an error. Keep
     the roles Heading, Body and Code; add Interface when it is a different
     face from Body. -->

| Role | Font Family | Weight | Style |
|------|------------|--------|-------|
| Heading | {{FONT_HEADING}} | {{FONT_HEADING_WEIGHTS}} | {{FONT_HEADING_STYLES}} |
| Body | {{FONT_BODY}} | {{FONT_BODY_WEIGHTS}} | {{FONT_BODY_STYLES}} |
| Code | {{FONT_CODE}} | {{FONT_CODE_WEIGHTS}} | {{FONT_CODE_STYLES}} |

## Weight Scale

| Weight Name | Numeric | Usage |
|-------------|---------|-------|
| Light | {{WEIGHT_LIGHT}} | {{WEIGHT_LIGHT_USAGE}} |
| Regular | {{WEIGHT_REGULAR}} | {{WEIGHT_REGULAR_USAGE}} |
| Medium | {{WEIGHT_MEDIUM}} | {{WEIGHT_MEDIUM_USAGE}} |
| SemiBold | {{WEIGHT_SEMIBOLD}} | {{WEIGHT_SEMIBOLD_USAGE}} |
| Bold | {{WEIGHT_BOLD}} | {{WEIGHT_BOLD_USAGE}} |

## Type Scale

| Level | Size | Line Height | Weight | Use |
|-------|------|------------|--------|-----|
| h1 | {{SIZE_H1}} | {{LINE_H1}} | {{WEIGHT_H1}} | {{USE_H1}} |
| h2 | {{SIZE_H2}} | {{LINE_H2}} | {{WEIGHT_H2}} | {{USE_H2}} |
| h3 | {{SIZE_H3}} | {{LINE_H3}} | {{WEIGHT_H3}} | {{USE_H3}} |
| body | {{SIZE_BODY}} | {{LINE_BODY}} | {{WEIGHT_BODY}} | {{USE_BODY}} |
| small | {{SIZE_SMALL}} | {{LINE_SMALL}} | {{WEIGHT_SMALL}} | {{USE_SMALL}} |
| caption | {{SIZE_CAPTION}} | {{LINE_CAPTION}} | {{WEIGHT_CAPTION}} | {{USE_CAPTION}} |

## Font Loading

<!-- The <link> lines every HTML page puts in its <head> to load the faces,
     one per line. petals review copies them into its index page. -->

```html
<link rel="stylesheet" href="{{FONT_STYLESHEET_URL}}">
```

## @font-face Declarations

```css
/* Heading font: {{FONT_HEADING}} */
@font-face {
  font-family: '{{FONT_HEADING}}';
  src: url('{{FONT_HEADING_URL_WOFF2}}') format('woff2'),
       url('{{FONT_HEADING_URL_WOFF}}') format('woff');
  font-weight: {{FONT_HEADING_WEIGHT_RANGE}};
  font-style: normal;
  font-display: swap;
}

/* Body font: {{FONT_BODY}} */
@font-face {
  font-family: '{{FONT_BODY}}';
  src: url('{{FONT_BODY_URL_WOFF2}}') format('woff2'),
       url('{{FONT_BODY_URL_WOFF}}') format('woff');
  font-weight: {{FONT_BODY_WEIGHT_RANGE}};
  font-style: normal;
  font-display: swap;
}
```

## Fallback Stacks

<!-- export-tokens.sh turns each --font-<role> line into the token
     --PREFIX-font-<role>, PREFIX as in colors.md. Keep heading, body and code. -->

```css
--font-heading: '{{FONT_HEADING}}', {{FONT_HEADING_FALLBACK}};
--font-body: '{{FONT_BODY}}', {{FONT_BODY_FALLBACK}};
--font-code: '{{FONT_CODE}}', {{FONT_CODE_FALLBACK}};
```

## Responsive Scale

| Breakpoint | Body Size | h1 Size | Scale Factor |
|------------|-----------|---------|-------------|
| < {{BP_MOBILE}} | {{SIZE_BODY_MOBILE}} | {{SIZE_H1_MOBILE}} | {{SCALE_MOBILE}} |
| {{BP_TABLET}}+ | {{SIZE_BODY_TABLET}} | {{SIZE_H1_TABLET}} | {{SCALE_TABLET}} |
| {{BP_DESKTOP}}+ | {{SIZE_BODY_DESKTOP}} | {{SIZE_H1_DESKTOP}} | {{SCALE_DESKTOP}} |

## Pairings

| Heading | Body | Use Case |
|---------|------|---------|
| {{FONT_HEADING}} | {{FONT_BODY}} | {{PAIRING_USE_1}} |
| {{FONT_HEADING_ALT}} | {{FONT_BODY_ALT}} | {{PAIRING_USE_2}} |
