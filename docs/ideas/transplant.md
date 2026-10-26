# transplant, moving a built product into the world

2026-10-26. A draft, not a loop. Layer: ship (`docs/decisions/2026-10-26-layers.md`). A
transplant moves a seedling from the nursery into open ground.

## Who it is for, and their pain

An owner whose agents can build a product and cannot put it in front of people without a person
clicking through a hosting dashboard, a domain registrar, an email provider and a billing
provider. Each setup step is done once by hand, nobody records how to undo it, and when production
breaks at night nothing notices until a user complains.

## What it owns, and what it does not

transplant owns everything between a green build and a running product: hosting, domains and
DNS, email sending, billing setup, analytics wiring, preview environments, monitoring, incidents
and rollback, and the legal pages a product must have before it goes live. It does not build the
product (pleach does), decide prices (the owner does, from cultivate's evidence), or announce it
(pollen and harvest do).

## How it works

1. **Desired state.** A file in the product's repository says what should exist: the hosting
   target, the domain, the email sender, the billing products and prices, the analytics events,
   the checks a live product must pass.
2. **Plan.** `transplant plan` is a dry run. It prints every step it would take, what each costs,
   and how to undo each one. Nothing changes.
3. **Apply.** `transplant apply` takes the steps in order and stops at the first failure. Steps
   the compass lists (the first deploy to a domain, anything that costs money, turning on billing)
   wait for a yes in gate, and every cost is reserved in well first. Each step's undo is recorded
   before the step runs.
4. **Watch.** Uptime and error-rate checks on the live product. A breach opens an incident: roll
   back to the last good release with the recorded undo, record what happened, tell the owner
   through gate.
5. **Before going live**, a checklist code can check: terms and privacy pages present, the
   product's data stores each declare a lifetime (trellis), a way for users to delete their data.

It wraps the hosting platform's own command-line tool and the owner's existing deploy, email and
billing skills. It is not a platform of its own.

## What code decides

The plan, the order, the stop at first failure, the recorded undo, the cost reservation, the
yes check, the monitoring thresholds and the rollback.

## Alone, and with the garden

Alone, it is one gated command that ships a repository with a dry run and an undo for every step.
With the garden, its yeses come from gate, its costs from well, its dates (certificates, domain
renewals) go into almanac.

## Data and its lifetime

The applied-state record and each step's undo are kept for the life of the product. Monitoring
samples are kept 30 days; incidents are kept for the life of the product.

## Metric and kill criterion

Metric: ship steps that needed a person, per product; time from a breach to rollback.

Kill criterion: if a product's ship cannot be written as a plan with an undo for every step on
the owner's hosting platform, transplant is a checklist skill, not a tool.

## Open questions

- Which legal pages are generated and which the owner must write.
- How preview environments are named and removed, so none are left behind.
