# trellis, the shape agents build into

2026-10-26. A draft, not a loop. Layer: architecture, structure, data model, APIs
(`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner whose products are built by agents. An agent writes code that works today and quietly
breaks the shape: a module reaches into another it should not, the data drifts from what was
designed, an API field disappears and a client breaks, a store keeps personal data forever because
nobody said how long. Architecture written as prose is read once and then ignored.

## What it owns, and what it does not

trellis owns the product's shape as files a person approves: module rules (which part may use
which), the data model, the API specification, the analytics events, and how long each store keeps
what it holds. It does not judge the honesty of a diff (weeder does), read code (tilth does) or
decide features (graft does).

## How it works

1. **Draft.** Skills draft the rules, the schema and the API specification from graft's features
   and flows. Every entity points at the feature it serves; every endpoint at the job or flow step
   it serves; every analytics event at the outcome harvest measures with it.
2. **Approve.** A person approves each file. Only approved files are checked against.
3. **Check, on every change:** the code respects the module rules, read through tilth's dependency
   reader; the code's data matches the schema; the code matches the API specification; no breaking
   change slips through (a removed field, a changed type); every store declares its lifetime and
   what removes old data. It also flags an entity no feature needs and a job no endpoint serves.

It wraps existing open checkers for API specifications, breaking API changes and database
migrations. What trellis adds is the module rules, the lifetime rule and the trace back to
features and jobs.

## What code decides

Every check above is mechanical. Findings are SARIF, and the checks run inside weeder's gate, so
a product has one gate.

## Alone, and with the garden

Alone, trellis checks any repository against rule, schema and API files a person wrote. With
graft, its files are drafted from the features and flows, and its trace finds what no job needs.

## Data and its lifetime

The shape files live in the product's repository, versioned with the code. A change to stored
data's shape is a new migration, never an edit to an applied one.

## Metric and kill criterion

Metric: drift caught before merge, against drift found after.

Kill criterion: if module rules, schema conformance and API conformance cannot be checked
mechanically on the garden's own repositories, trellis stays a document. graft's
`docs/architecture.md` has a "may depend on" table and is the first test case.

## Open questions

- The format of module rules; a small table in the shape graft already uses is the first
  candidate.
