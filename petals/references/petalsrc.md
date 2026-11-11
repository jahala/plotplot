# .petalsrc Format

`.petalsrc` is a YAML file at the project root. It records the brand source, current version, and project-level overrides.

```yaml
# .petalsrc — Project brand configuration
brand:
  # The URL or path to the central brand repository
  source: github.com/company/brand

  # The version tag currently fetched (e.g., v1.2.0)
  version: v1.2.0

  # Project-level overrides. Non-null values are preserved during /petals update.
  # Null values inherit from the central brand on each update.
  # Keys are flattened dot-separated paths into .brand/ files.
  overrides:
    # Example: override the primary color for a campaign
    # colors.primary: "#FFD700"

    # Example: track but don't override the body font (inherits from central)
    # typography.body: null
```

### Stages: mockups and production

A file is production unless `.petalsrc` declares its path a mockup:

```yaml
stages:
  mockup:
    - docs/designs/**
```

Content-truth findings (placeholder copy, 555 numbers, `example.com` addresses, John or Jane Doe, `href="#"`) are **errors** in production and **notes** in a mockup, where believable filler is how a design gets judged. Brand rules are the same in both stages. `check.sh --stage mockup|production` overrides the configured stage. Text inside `<pre>`, inline `<code>` and comments is not copy, and in a markdown file neither are fenced code blocks or inline code: no rule judges quoted output.

### Override key conventions

Override keys use dot-separated paths that map to fields in `.brand/` markdown files:

| Key | Maps to | File |
|-----|---------|------|
| `colors.primary` | Primary hex value | `.brand/colors.md` |
| `colors.accent` | Accent hex value | `.brand/colors.md` |
| `typography.heading` | Heading font family | `.brand/typography.md` |
| `typography.body` | Body font family | `.brand/typography.md` |
| `typography.mono` | Mono font family | `.brand/typography.md` |
| `voice.terms` | Project-specific forbidden terms (appended as JSON array of {term, reason, use_instead} objects) | `.brand/voice.md` |

### Value semantics

- **Non-null string**: This value is preserved during updates. The central brand's value for this key is ignored.
- **null**: This key is tracked but not overridden. The central brand's value flows through on each update.
