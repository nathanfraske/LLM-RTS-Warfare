# Species

Species follow the compositional rule ([00 — pillar 5](00-vision.md)): a species is a **parameter bundle** flowing into systems that already exist — never bespoke code.

## The bundle

- **Physiology:** size, lifespan, birth rate, diet, habitat affinities (mountain, marsh, coast...), environmental tolerances.
- **Cognition:** learning rate, expertise depth vs. breadth.
- **Culture seeds:** priors on aggression, trade openness, hierarchy, faith — starting biases for institutions, not destiny.

## Where the parameters flow

- **Cohort dynamics** ([02](02-simulation-core.md)): cohorts are keyed by species; birth/death/needs/migration parameters vary per species.
- **Tech affinities** ([03](03-invention-grammar.md)): habitat and physiology weight research focus naturally — an aquatic species drifts toward hulls and nets, a montane one toward metallurgy. Divergent arsenals emerge without authored tech trees per race.
- **Institutions** ([04](04-institutions-directives.md)): culture seeds bias autopilot tendencies and set the default posture agents inherit and can fight against.
- **Appearance:** glyph derivation — palette and body shape from physiology, at pixel-art cost.

## Multi-species nations

Because cohorts key by species, mixed nations come for free — and **species tension becomes internal politics**: differing needs, loyalty responses to policy, migration pressures. Governing a two-species nation is a genuinely different problem for an agent, and species-relations posture is a policy domain ([04](04-institutions-directives.md)).

## NPC minor races

Minor species/factions run entirely on the institutional autopilot — world texture, trade partners, invasion threats, and the autopilot's permanent proving ground ([04](04-institutions-directives.md)).

## Authoring plan

Schema-first ([01 — principle 6](01-architecture.md)): hand-author 3–4 archetype species as RON data files *first*; write the procedural generator only after the archetypes prove the schema. Sketch:

```ron
Species(
    name: "Duneborn",
    physiology: (size: 0.9, lifespan_y: 70, birth_rate: 1.2,
                 diet: [Grain, Meat], habitat: {Desert: 1.4, Steppe: 1.1, Marsh: 0.5}),
    cognition: (learning: 1.0, breadth: 1.2),
    culture_seeds: (aggression: 0.4, trade: 1.3, hierarchy: 0.8, faith: 1.1),
    glyph: (palette: Ochre, build: Lean),
)
```
