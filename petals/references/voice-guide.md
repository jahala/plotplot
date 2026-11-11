# Voice Audit Guide

This guide documents the structured process for auditing text content against the brand voice rules. The canonical source for voice rules is `.brand/voice.md`.

## Process

### 1. Build the Voice Rules

Read `.brand/voice.md` and extract:

- **Forbidden Terms**: From the "Forbidden Terms" table. Each entry has a term, a reason, and a preferred alternative. Store as a list of `{term, alternative}` pairs.
- **Tone Rules**: From the "Tone Spectrum" table. Note the expected formality, warmth, and directness for each context.
- **Grammar Rules**: From the "Grammar & Formatting" section. Note case conventions, voice (active/passive), contraction rules, and sentence length guidelines.

### 2. Determine the Context

Before auditing, determine what kind of text is being checked:

- **Inline text** (provided directly as an argument): Treat as marketing/UI copy unless otherwise specified.
- **UI code file** (`.tsx`, `.jsx`, `.vue`, `.svelte`): The text is UI microcopy. Apply UI-specific tone rules.
- **Documentation file** (`.md`, `.mdx`): Apply documentation tone rules.
- **Error/log file**: Apply error message tone rules.
- **Plain text file** (`.txt`): Treat as marketing copy unless context indicates otherwise.

If the voice guide has context-specific adaptations, apply the relevant ones.

### 3. Scan for Forbidden Terms

For each forbidden term in the voice rules:

1. Scan the target text **case-insensitively** for the term.
2. Match both exact words and substrings within words (e.g., "leveraging" contains "leverage").
3. For each match, record:
   - The forbidden term found
   - The line number (or position in inline text)
   - The full matched word in context

### 4. Check Tone Rules

Beyond explicitly forbidden terms, flag patterns that violate tone rules:

- **Jargon density**: If the text contains 3+ jargon or buzzword-like terms (even if not in the forbidden list), flag as a warning suggesting simpler alternatives.
- **Sentence length**: If a sentence exceeds the guideline (e.g., 25+ words), flag as a warning.
- **Passive voice**: If grammar rules require active voice, flag passive constructions.

Note: Tone rule violations are **warnings**, not errors. They suggest improvement without blocking a clean check.

### 5. Check Terminology

If `.brand/voice.md` has a "Terminology" table: for each banned synonym, search case-insensitively; a match is a **warning** naming the canonical term. A capitalized product name where the rules require lowercase is a **warning**.

### 6. Check Capitalization & Microcopy

If the rules exist in `.brand/voice.md`: flag as **warnings** headings/buttons/labels in Title Case where sentence case is required, button labels that are not verb-first or exceed the stated word limit, and exclamation marks in interface copy if forbidden.

### 7. Provide Replacements

For each forbidden term violation, provide the preferred alternative from the voice guide:

```
VIOLATION [voice] "leverage" -> use "use"
```

If the voice guide does not list a specific alternative for a flagged term, provide a plain-language suggestion:

```
WARNING [voice] "utilize" is overly formal. Consider "use".
```

### 8. Report Results

For each violation, report:
- **Dimension**: `voice`
- **Severity**: `error` (forbidden term) or `warning` (tone rule)
- **Position**: Line number or character offset
- **Found**: The offending term
- **Suggested**: The preferred alternative

Format for errors:
```
ERROR [voice] line 1: "leverage" is a forbidden term. Use "use" instead.
```

Format for warnings:
```
WARNING [voice] line 2: Sentence exceeds 25 words. Consider breaking into shorter sentences.
```

If zero violations:
```
PASS [voice] Text follows brand voice guidelines.
```

### 9. Handle Missing Context

If `.brand/voice.md` does not exist:
- Report as a setup issue: "Voice guide not found. Skipping voice audit."
- Do NOT report this as a voice violation.
- Allow other audit dimensions to proceed normally.

### Reference

- Canonical voice source: `.brand/voice.md`
- This guide is used by: `/petals voice <text>` and `/petals check <file>` (voice audit dimension)

## /petals voice `<text-or-file>`

When `/petals voice <text-or-file>` is invoked, the agent audits the provided text or file content against `.brand/voice.md` rules. This can be used standalone or as part of `/petals check <file>`.

### Guard clause

1. Check if `.brand/` directory exists. If not:
   ```
   No brand configured. Run /petals init --from <url> to connect to a central brand repo.
   ```
   Exit non-zero. Do not proceed.

2. Check if `.brand/voice.md` exists. If not:
   ```
   SKIP [voice] voice.md not found. Skipping voice audit.
   ```
   Exit non-zero (missing configuration is a setup issue).

### Determine the input

The argument can be either inline text or a file path:

- **Inline text** (no file extension, or quoted string that does not match an existing file): Audit the text directly.
- **File path** (matches an existing file, or has a known extension like `.txt`, `.md`, `.tsx`, `.jsx`, `.html`): Read the file and audit its text content.

### Process

Follow the process above, steps 1 to 9, loading only `.brand/voice.md` (and the product layer's `voice.md` if there is one). Exit 0 when there are no errors (warnings alone do not fail); exit non-zero when any forbidden term is found.
