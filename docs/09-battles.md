# Battles

One honest split, stated up front: **aggregate resolution is authoritative; the observed battle is a faithful, seeded performance of it.** Fairness, replay integrity, and cost all demand that observation never changes outcomes.

## Aggregate resolution (authoritative)

Runs on CPU in fixed-point ([01](01-architecture.md)). Inputs: army composition (compiled from the nations' actual device designs, [03](03-invention-grammar.md)), doctrine and stance ([04](04-institutions-directives.md)), terrain, supply state, morale, fortifications and their mounts ([07](07-buildings-and-cities.md)). Logistics constrain hard — unsupplied armies degrade before they fight, which is exactly the non-obvious consequence space agents should be reasoning about.

Outputs: casualties by cohort, ground taken, morale shifts, materiel losses, prisoners, and a **battle timeline**.

## The battle timeline artifact

Resolution emits a phased timeline — approach, skirmish, main engagement, break, rout/pursuit — with per-phase aggregate deltas. This artifact is the contract between the authoritative sim and everything downstream: the renderer performs it, readouts summarize it, the narrator retells it, replays scrub it.

## Observed battles (the performance)

When a battle is watched — by a spectator or an agent screenshot — the site hydrates ([02](02-simulation-core.md)): individual soldiers instantiated from the participating cohorts, formations moving on flow fields, volleys, morale breaks, and routs staged to realize the timeline's phases, seeded from the counter-RNG so every viewing is identical. Spectacle without authority: sharp-eyed viewers see causally convincing action, and the outcome still matches what unobserved resolution would have produced, because it *is* that resolution.

Mounted designs fire visibly — the *Spear of Dawn* rises from its silo trailing flame because propulsion, guidance, and payload are real components doing their derived jobs ([03](03-invention-grammar.md)).

## Sieges

Sieges run the same split against building modules: bombardment degrades walls and mounts, blockade starves stores, assault resolves against the remaining defense value. City walls that took sim-years to build fall in observable stages.

## Consequences flow back

Casualties debit cohorts and **stamp the registry** ([02](02-simulation-core.md)) — named individuals die, records gain battle honors, families gain grief the narrator can find. Veterancy accrues to surviving cohorts. War exhaustion feeds unrest and the agent's next hard council session.
