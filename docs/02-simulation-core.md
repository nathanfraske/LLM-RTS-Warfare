# Simulation Core — Cohorts, Hydration, Individuals

The scale target (millions of individuals per nation) is met by simulating **persistent identities at varying level of detail**, not millions of full agents. Aggregates are authoritative; individuals are faithful, legible views.

## The cohort layer (authoritative, mass-scale)

A **cohort** is a statistical bucket, keyed roughly by `province × species × occupation` (final key TBD at M1). Cohorts live in SoA buffers and carry counts plus distributions: age structure, wealth, needs satisfaction, health, morale/loyalty.

Cohort dynamics are continuous-field updates and the primary GPU workload ([`cohorts-gpu`](01-architecture.md)):

- Demographics: births, deaths, aging.
- Employment: assignment against building-module labor demand.
- Needs and consumption: food, goods, housing, culture — feeding market demand.
- Migration: gradient flows toward opportunity/safety, along the same flow fields used for movement.
- Unrest and loyalty: responses to policy, war, scarcity, species tension.

## The individual registry (persistent identity, near-zero cost)

Every individual is a permanent, cheap **record**: id, name, family links, species, home, occupation, traits, notable-history stamps. Records exist for the whole population — identity is never invented on the spot twice.

While unobserved, individuals do not tick. Life events (marriage, death, migration) are resolved statistically at the cohort level and **stamped** onto sampled records so the registry stays consistent with aggregates — the Dwarf Fortress offscreen pattern. Watch villager #4823 today, come back tomorrow: she still exists, and may be married now.

Named individuals — leaders, generals, heroes, criminals — are pinned: always hydrated or on a cheap always-simulated path, since narrative hangs off them.

## Hydration (LOD promotion/demotion)

**Triggers:** a viewer camera over the area, an agent screenshot request, an active battle, construction sites, pinned individuals.

**Promotion:** individuals are instantiated by sampling consistently with cohort statistics (a city that is 40% farmers visibly empties toward fields at dawn), binding sampled records so identity persists across observations.

**Conservation:** hydrated individuals are debited from cohort aggregates; demotion re-credits and writes back a state summary to the record. Property tests enforce that totals are conserved exactly ([01 — Testing](01-architecture.md)).

**Bubbling:** notable hydrated events (a duel, a murder, a heroic stand) emit world events upward — feeding the narrator, readouts, and sometimes politics.

## Task legibility (the watchable part)

Observed individuals run a small utility-AI over **task atoms** with visible intermediate states: walk-to, carry, work-at, queue, eat, socialize, sleep, flee, fight. The storytelling lives in movement patterns and task rhythms — dawn commutes, market-day crowds, construction gangs hauling stone — not in animation fidelity. At 8–16px, a villager carrying grain is a villager sprite plus a grain-sack glyph, and that's enough.

Daily rhythm derives from cohort schedules so the hydrated view always matches the aggregate truth: hydration renders the statistics, it never contradicts them.

## Movement

Mass movement uses **flow fields** computed per goal/region on the GPU (`flowfield-gpu`) — no per-unit pathfinding at scale. Hydrated individuals follow the same fields (plus local avoidance), so observed motion is consistent with aggregate flows. Armies, migrations, and trade caravans all ride this system.

## Determinism split (restating [01](01-architecture.md))

Cohort fields and flow fields run on GPU with tolerance-bounded divergence, reconciled at snapshots. Everything discrete and consequential — who died by name, battle outcomes, registry stamps — resolves on CPU in fixed-point from the counter-RNG, so replays agree about every fact anyone could quote.
