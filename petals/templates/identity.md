# Brand Identity: {{BRAND_NAME}}

<!--
  Template: identity.md
  All fields come from explicit source statements.
  "not specified" is preferred over invented values.
  Scripts read the table: export-tokens.sh names the tokens files after Brand
  Name, and check-brand-drift.sh reads Brand Version (v1.2.3).
-->

| Field | Value |
|-------|-------|
| Brand Name | {{BRAND_NAME}} |
| Tagline | {{BRAND_TAGLINE}} |
| Mission | {{BRAND_MISSION}} |
| Audience | {{BRAND_AUDIENCE}} |
| Founded | {{FOUNDED}} |
| Industry | {{INDUSTRY}} |
| Website | {{WEBSITE}} |
| Brand Version | {{BRAND_VERSION}} |

## Story

{{BRAND_STORY}}

## Positioning

### Value Proposition
{{VALUE_PROPOSITION}}

### Differentiators
- {{DIFFERENTIATOR_1}}
- {{DIFFERENTIATOR_2}}
- {{DIFFERENTIATOR_3}}

## Notes

- Fields marked "not specified" were not found in the source materials.
- To update these values, edit the source brand assets and re-run `/petals init`.
