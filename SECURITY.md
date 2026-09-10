# Security policy

## Reporting a vulnerability

Please do not open a public issue. Use GitHub's private advisory flow:

<https://github.com/jahala/plotplot/security/advisories/new>

We acknowledge within 72 hours and coordinate disclosure with you.

## Supported versions

Only the latest tag receives security updates. Older tags do not.

## What runs

- The cargo gate on every push and pull request that touches the stem or the contracts:
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, on Ubuntu
  and macOS.
- The garden's own gate on every pull request: `plotplot check --strict`, which runs every
  judge the lockfile pins, weeder among them, on a hosted runner.
- The stem fetches judges only from the URLs `garden.lock` names and refuses bytes whose
  sha256 is not the one pinned. Nothing is run from PATH.
