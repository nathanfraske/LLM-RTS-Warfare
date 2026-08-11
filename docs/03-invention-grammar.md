# The Invention Grammar — Procedural Technology

> **Formal spec:** [03a-grammar-spec](03a-grammar-spec.md) — authoritative where the two differ. This document is the concept overview.

**Goal:** an enormous technology space with no hand-authored content lists. We author the periodic table, not the molecules: a small primitive set with real engine semantics, composition rules, and derivation of behavior, appearance, and cost. "Missile" appears nowhere in the data files, yet missiles happen.

## Primitives (hand-authored, small — dozens, not thousands)

A **primitive** is a physical verb or material scale the engine genuinely understands. Each ships with:

- **Sim semantics** — an ECS component implementation (what it *does*).
- **A glyph language** — how it contributes to a composed sprite at 8–16px.
- **A cost model** — materials, industry capacity, expertise demanded.
- **Parameter ranges** — continuous scales that research improves.

Initial families (illustrative, to be refined by the test suite):

- *Mechanics:* structure/airframe, wheel, lever/spring, rotor, sail.
- *Energetics:* combustion, sustained combustion (rocket), explosive, energy storage.
- *Projection:* projectile, launcher, lift.
- *Information:* optics/sensor, signal/comms, computation, control.
- *Materials & knowledge scales:* metallurgy, chemistry, agronomy, medicine.
- *Process:* production-method improvements that upgrade building-module recipes.

## The type system (where design effort concentrates)

Primitives compose through typed slots with **plausibility gates** — physical-sanity rules that keep the combinatorics honest:

- Guidance requires something self-propelled or falling.
- A payload requires a delivery mechanism.
- Sustained combustion requires energy-dense chemistry above a threshold, and an airframe rated for it.

This type system is the highest-leverage, hardest-to-change artifact in the project ([00 — bets](00-vision.md)). It gets designed on paper and validated by the test suite before implementation hardens around it.

## Designs

A **design** is a typed graph of primitives with concrete parameters. Designs compile to one of three targets:

1. **Devices** — weapons, vehicles, tools → an ECS component bundle (an archetype).
2. **Building modules** — new module types for [buildings](07-buildings-and-cities.md) (a launcher module, a mill mechanism).
3. **Process upgrades** — efficiency/recipe improvements to existing modules (better smelting, crop rotation).

Because behavior compiles from components, a design *acts like* what it is with no per-item code.

## Discovery

Research is institutional ([04](04-institutions-directives.md)): the agent's directives set **focus weights** over primitive families; institutions convert budget and expertise into progress along parameter scales and into stochastic **design proposals** — weighted by focus, gated by prerequisites (parameter thresholds), rolled from the counter-RNG so replays agree. A proposal arrives as a world event: *"Your engineers propose a design: sustained-combustion airframe, sensor-control guidance, explosive payload."*

**The agent names it.** Leaders christen inventions; names flow into intel reports, diplomacy, and the narrator. Cheap to build, enormous flavor.

## Eras are emergent

There are no era pages. The same compositional slots traversed with improving parameters produce recognizable ages: mechanical launcher + projectile is a catapult; add chemistry, a cannon; add sustained combustion, a rocket; add sensor+control, a guided missile. Two nations with different focus policies, species affinities ([08](08-species.md)), and discovery rolls walk genuinely different paths — asymmetric arsenals are content, not a balance bug.

## Derivation

- **Behavior:** from the compiled component bundle. Propulsion accelerates, guidance steers toward a tracked target, payload triggers area damage on impact.
- **Appearance:** procedural sprite assembly from part glyphs — elongated body from airframe, fins from control surfaces, flame particle from combustion. The minimal pixel-art direction is load-bearing: composition is tractable at 8–16px and nowhere else.
- **Cost:** summed and scaled from component cost models — materials, industry slots, expertise. Balance is derived, never hand-tuned per item.

## The missile, end to end (canonical walkthrough)

1. Agent policy: heavy focus on chemistry and propulsion; decree funds a research campus.
2. Institutions cross metallurgy/chemistry thresholds; proposal event fires: airframe + sustained combustion + guidance + explosive payload.
3. Agent ratifies and names it — say, *"Spear of Dawn."*
4. Design compiles to an archetype; industry tools up (module recipes demand materials and capacity); mounts on fortifications can host it ([07](07-buildings-and-cities.md)) — a silo emerges, unauthored.
5. In battle, the entity launches, accelerates, tracks, and explodes — and the sprite is a 5×3 body with fins and a flame trail. It looks and acts like a missile because the engine understands the parts, not because anyone defined "missile."
6. Enemy intel reports it by observed capability ([06](06-diplomacy-intel.md)): "self-propelled guided munition, est. range …" — under whatever name *their* spies overheard.

## The grammar test suite (validation method)

Hand-write ~20 historical designs as **unit tests of the grammar** — not shipped content: sword, plow, cart, bow, ballista, galley, water mill, cannon, printing press (a process/module tech), musket, ironclad, telegraph, rifle, artillery, tank, aircraft, radar, rocket, guided missile, computer. Each test asserts the grammar can express the design and that the compiled bundle exhibits the expected capabilities. If a design can't be expressed, the primitive set is wrong — fix the table, not the test.

## Open questions

- Primitive granularity: too coarse → samey inventions; too fine → nonsense proposals. The test suite plus playtesting arbitrates.
- Proposal "sensibleness": pure weighted rolls may need a small heuristic layer so proposals feel motivated by a nation's situation (wars beget weapons, famines beget agronomy).
- Parameter-tuning workflow: derived balance still needs global knobs; keep them few and top-level.
