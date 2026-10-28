# Rulings: the owner's standing decisions on using sources

A capture plan is per project, but an owner's rulings hold for every project. Each owner keeps one rulings file, and every tool that writes a capture plan copies each entry's `ruling` from it, so no tool asks the owner again and no plan carries a ruling the owner did not make. The owner's file lives with their data, at `~/.local/share/plotplot/<portfolio>/rulings.json`.

Schema: `contracts/rulings.schema.json`. Fixtures: `contracts/fixtures/rulings/`. Test: `contracts/test/rulings.test.sh`.

| Field | Required | Meaning |
|---|---|---|
| `rulings` | yes | the format version, `"1"` |
| `owner` | yes | whose rulings these are |
| `note` | no | what the file is for |
| `all` | yes | the ruling for every adapter, present and future |
| `by_adapter` | yes, may be empty | rulings for single adapters by id, each overriding `all` for that adapter |

A ruling has the capture plan's ruling shape, defined once in `contracts/capture-plan.schema.json`: `use` (`allowed` or `refused`), `by`, `on` and `says`, the owner's own words.

## Rules a reader may rely on

- **Copy, never compose.** A plan writer copies `by_adapter[<adapter id>]` when it exists, otherwise `all`, into the entry's `ruling`, unchanged.
- **One file per owner.** A ruling changes in this file, and the next plan written carries it; plans already written keep the ruling they were written with until they are written again.
- **A ruling decides use, not limits.** No ruling changes a source's rate limit or deletion requirements, or lets a tool work around a captcha, two-factor or a login it was not given (`contracts/capture-plan.md`).
