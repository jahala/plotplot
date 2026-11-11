# {{Component}}: contract

{{One sentence: what this component is for, in the product's words.}}

## Home

The one place this component is built. Every other file uses it from here.

| File | Class | Covers |
|------|-------|--------|
| {{src/components/Button.tsx}} | {{.btn}} | {{button, a.btn}} |

- **File** is the path from the project root.
- **Class** is the class (or component name) a use of it carries.
- **Covers** lists the raw elements it replaces, as CSS selectors (`button`, `input[type=checkbox]`). A raw one of these outside the home is a rebuild.

## Owns and does not own

- Owns: {{what this component decides: its padding, its focus ring, its states}}.
- Does not own: {{what the caller decides: its width, its placement, its copy}}.

## States

Every state the component must show. A preview shows each one.

| State | What changes | Tokens |
|-------|--------------|--------|
| default | {{}} | {{--pp-primary}} |
| hover | {{}} | {{}} |
| focus visible | {{}} | {{}} |
| active | {{}} | {{}} |
| disabled | {{}} | {{}} |

## Tokens

Every value comes from a token the brand defines in `tokens.css`. Name each one; never write a raw value.

| Use | Token |
|-----|-------|
| {{fill}} | {{--pp-primary}} |
| {{radius}} | {{--pp-radius-md}} |

## Rules

Numbered, so a test and a review can cite them.

1. {{A rule that holds for every use, citing a token where one applies.}}

## Verification

Each rule names the test that holds it, or says "Gap:" and why nothing does yet. A gap is a visible task, never a silent one.

| Rule | Held by |
|------|---------|
| 1 | {{tests/button.spec.sh}} or Gap: {{why}} |
