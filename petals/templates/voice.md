# Brand Voice & Tone: {{BRAND_NAME}}

<!--
  Template: voice.md
  Forbidden terms and tone rules must be stated in the source.
  Do not invent forbidden terms or tone descriptors.
  check.sh reads the Forbidden Terms and Terminology tables. It flags em
  dashes in copy only when the Em dashes line below bans them; fill that line
  only from the source.
-->

## Personality Traits

- {{PERSONALITY_TRAIT_1}}
- {{PERSONALITY_TRAIT_2}}
- {{PERSONALITY_TRAIT_3}}
- {{PERSONALITY_TRAIT_4}}

## Tone Spectrum

| Context | Formality | Warmth | Directness |
|---------|-----------|--------|-----------|
| Marketing | {{TONE_MKT_FORMAL}} | {{TONE_MKT_WARMTH}} | {{TONE_MKT_DIRECT}} |
| Documentation | {{TONE_DOCS_FORMAL}} | {{TONE_DOCS_WARMTH}} | {{TONE_DOCS_DIRECT}} |
| UI Microcopy | {{TONE_UI_FORMAL}} | {{TONE_UI_WARMTH}} | {{TONE_UI_DIRECT}} |
| Error Messages | {{TONE_ERR_FORMAL}} | {{TONE_ERR_WARMTH}} | {{TONE_ERR_DIRECT}} |
| Social Media | {{TONE_SOCIAL_FORMAL}} | {{TONE_SOCIAL_WARMTH}} | {{TONE_SOCIAL_DIRECT}} |

## Forbidden Terms

<!-- One row per term. A page that uses one is an error, and the finding
     offers Use Instead. -->

| Term | Reason | Use Instead |
|------|--------|------------|
| {{FORBIDDEN_TERM_1}} | {{FORBIDDEN_REASON_1}} | {{FORBIDDEN_ALT_1}} |
| {{FORBIDDEN_TERM_2}} | {{FORBIDDEN_REASON_2}} | {{FORBIDDEN_ALT_2}} |
| {{FORBIDDEN_TERM_3}} | {{FORBIDDEN_REASON_3}} | {{FORBIDDEN_ALT_3}} |

## Terminology

<!-- The brand's canonical terms. Not lists the synonyms to avoid, comma
     separated; a page that uses one gets a warning naming the term in Use. -->

| Use | Not | Why |
|-----|-----|-----|
| {{CANONICAL_TERM_1}} | {{AVOIDED_SYNONYMS_1}} | {{TERM_REASON_1}} |

## Grammar & Formatting

- **Case:** {{GRAMMAR_CASE}}
- **Voice:** {{GRAMMAR_VOICE}}
- **Contractions:** {{GRAMMAR_CONTRACTIONS}}
- **Sentence length:** {{GRAMMAR_SENTENCE_LENGTH}}
- **Paragraph structure:** {{GRAMMAR_PARAGRAPH}}
- **Em dashes:** {{GRAMMAR_EM_DASHES}}

## Before / After Examples

### Example 1: Marketing Copy

**Before:** {{BEFORE_MARKETING}}
**After:** {{AFTER_MARKETING}}

### Example 2: UI Microcopy

**Before:** {{BEFORE_UI}}
**After:** {{AFTER_UI}}

### Example 3: Error Message

**Before:** {{BEFORE_ERROR}}
**After:** {{AFTER_ERROR}}

## Context-Specific Adaptations

### When addressing users directly
- {{DIRECT_ADAPTATION_1}}
- {{DIRECT_ADAPTATION_2}}

### When describing technical features
- {{TECH_ADAPTATION_1}}
- {{TECH_ADAPTATION_2}}

### When writing for social media
- {{SOCIAL_ADAPTATION_1}}
- {{SOCIAL_ADAPTATION_2}}
