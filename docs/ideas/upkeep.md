# upkeep (working name), keeping shipped products from rotting

2026-10-26. A draft, not a loop. Infrastructure (`docs/decisions/2026-10-26-layers.md`).

## Who it is for, and their pain

An owner with several shipped products, each of which decays without anyone touching it:
libraries fall behind and collect security advisories, a framework version stops getting fixes,
a model or an API it calls is retired, a domain or a certificate expires, a key runs out. With one
product a person notices. With ten, keeping them alive is most of the work, and the first sign of
a missed one is a broken product in front of its users.

## What it owns, and what it does not

upkeep owns turning decay into scheduled work across every shipped product. It does not detect
dependency lag in one tree (weeder's `scan` already carries that rule, law §6a), fetch release
notes and advisories (mull fetches them), plan the work (tend2) or run it (pleach).

## How it works

1. **Collect.** For each product: weeder `scan`'s dependency findings; advisories and end-of-support
   notices that mull fetched for the product's dependencies and platforms; retirement notices for
   the models and APIs the product calls; renewal and expiry dates from transplant and well.
2. **Decide.** Each item becomes one of: a deadline in almanac (renew the domain by this date), a
   proposed loop for the product's map (upgrade this framework, with its checks), or an alert to
   the owner through gate (a security advisory with a known exploit).
3. **Batch.** Small upgrades of the same kind across products are proposed together, so one plan
   runs them all and lands each on its own product's checks.

It wraps the existing open dependency-update tools and advisory databases rather than rebuilding
them.

## What code decides

Which items are due, which pass a severity line, which batch together, and whether an upgrade
landed (the product's own checks). A typed judge may read a free-text retirement notice for its
date and scope, in a shape code can refuse.

## Alone, and with the garden

Alone, it reads a set of repositories and writes a list of what is decaying, with dates. With the
garden, the list becomes almanac entries and loops the gardener runs.

## Data and its lifetime

Each run's collected items are kept until the work they caused lands or is dismissed, then
compacted to one line per item in a yearly archive.

## Metric and kill criterion

Metric: decay that reached users first (an expired domain, an exploited advisory, a retired API
called). The target is none.

Kill criterion: if, across the first three products, upkeep's output is only weeder's scan
findings plus what an update tool already proposes, it is not a tool. It folds into weeder `scan`
and almanac entries.

## Open questions

- Whether upkeep is its own bed or the gardener's maintenance rules. Detection, fetching, planning
  and running all belong to other beds, so what upkeep keeps is routing and batching. Asked on
  2026-10-27; the owner kept it a bed for now. It earns its own bed by holding logic of its own,
  such as judging which upgrades are safe to batch; if it never does, it folds into the gardener.
- Its name.
- How far upkeep goes on its own: whether a patch-level upgrade that passes every check may land
  without a yes.
