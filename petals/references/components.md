# Component contracts

A contract says where a component is built, what it must show and which tokens it uses. Agents read it before they build UI, so a button is reused, never rebuilt. The check reads it too.

## Where contracts live

- In the project's brand: `.brand/components/<name>.md` for a component, `.brand/components/families/<family>.md` for a family.
- The blank forms are in `templates/components/component.md` and `templates/components/family.md`. Copy one, fill every `{{…}}`, and delete what does not apply.

## Before you build a control

1. Look in `.brand/components/` for a contract whose **Covers** column names the element you need (`button`, `input[type=checkbox]`, `select`).
2. If one exists, use the component from its **Home** file, with its **Class**. Do not write a new one, even a small one.
3. If none exists and the project will use the control more than once, write the contract first, then build the component at the home it names.

## Writing a contract

- **Home** has one row per place the component is built: the file, the class a use carries, and the raw elements it replaces, as CSS selectors. Keep it to one file unless the framework splits markup and style.
- **States** lists every state a user can meet: at least default, hover, focus visible, active and disabled for anything clickable. Each state names its tokens.
- **Tokens** names a token for every value. Use only tokens `tokens.css` defines; `scripts/export-tokens.sh --check` fails on a contract that cites one it does not.
- **Rules** are numbered so a test, a review or a finding can cite them.
- **Verification** gives each rule the test that holds it, or `Gap:` and the reason. Gaps are expected early; hiding them is not.

## Families

Group components when they share rules: buttons, input fields, overlays. The family holds the rules every member keeps, the membership rule that decides where a new component goes, what members may vary in, and an adoption table that names each exception and its reason. A member's contract may add rules but never break the family's.
