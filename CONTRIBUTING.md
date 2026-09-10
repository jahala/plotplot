# Contributing

Thanks for your interest. plotplot is the root of a garden of small tools, and this
repository holds the law, the contracts and the stem. Read `docs/building-the-garden.md`
first: it is what every change here is judged against.

## Workflow

1. Fork, branch, change.
2. Run the gates locally:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```
3. Open a pull request. Say what changed and how to test it.

CI runs the same three commands on every push, and `plotplot check --strict` on every
pull request.

## What helps

- Small pull requests. Easier to review, easier to merge.
- A test first. A bug fix carries a regression test; a feature carries at least one.
- A loop. Planned work is a loop in `docs/tend2/` with its checks written before the code;
  only `tend2 verify` marks a check passed. Unplanned work is an issue.
- Surgical edits. Leave surrounding code alone.
- Commit bodies that say why. The log is the reasoning trail.

## Code style

Rust, edition 2024, `rustfmt` and `clippy` with no warnings. Library code never panics on
input; failure lives in the return type. Side effects stay at the edge, in the callers.
