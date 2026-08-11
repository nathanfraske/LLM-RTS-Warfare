# The Authored Floor — Where Emergence Stops

Every "no authored lists" decision ([03a](03a-grammar-spec.md) tech, [19](19-ecology-and-subsistence.md) species, [20](20-open-directives.md) governance) pushes authorship downward. This document is the hard stop, so the push has somewhere to end. The failure mode it guards against is real and has a name here: **grounding regress** — every layer feels like it *should* emerge from a deeper one, so the authored floor sinks (biology from chemistry, chemistry from physics) until you are simulating a universe from the CMB and nothing above the floor is legible, tractable, or fun.

## The resolution

There is no such thing as a fully emergent simulation — even a physics engine authors its laws. The design choice is never *whether* to have an authored floor, only *where it sits*. And the two appetites that get conflated pull in different directions:

- **Emergence-depth**: simulating ever-lower levels so higher ones "fall out." This is what killed the physics-sim project. Depth buys almost no gameplay per unit of complexity, and it destroys legibility, because outcomes become long causal chains of variables nobody can see.
- **Possibility-width**: more distinct things that can exist and happen *at the same level*. This is what the project actually wants, and every mechanism we've shipped buys width without depth: trait axes make omnivores exist without simulating metabolism; the grammar makes missiles exist without combustion chemistry; the registry makes unanticipated policies exist without simulating bureaucrats.

**The rule: the floor sits at the highest level that still generates the wanted variety. It moves down one level only when a phenomenon someone can watch or govern cannot be expressed above it — never because a layer "ought to" emerge from something deeper.** Emergence is an instrument (for honesty, for width, for surprise); it is never the goal. In the CMB project emergence *was* the goal, so no floor could hold.

## The three tests

A quantity deserves its own dynamics (state that evolves) only if it passes at least one:

1. **The decision test** — an overseer's choice can change it, or it appears in a readout and changes overseer choices.
2. **The spectacle test** — a spectator can watch it change within a session and read the story off the screen.
3. **The one-sentence test** — every outcome it produces can be explained in one sentence of in-world vocabulary ("the predators starved because the herds were overhunted"). If explaining needs a paragraph of intermediate variables, the variables are below the floor.

Fails all three → it is a **coefficient** ([tuning](01a-foundation.md)) or a pure derivation, not a simulated layer. This is also the legibility guarantee Nathan asked for: everything above the floor ships with its own `describe()` — trait genomes name themselves, channels report per-worker returns, registry entries carry summaries. If a generated thing cannot render itself in one legible line, it is below the floor by definition.

## The floors as they stand

| Domain | The floor (authored) | Above it (emergent/derived) | Never below it |
|---|---|---|---|
| Time | 1-hour tick; presence renders ~1s but **earns against aggregates** ([17](17-presence.md)) | months, famines, campaigns | sub-second physics |
| People | cohort statistics ([02](02-simulation-core.md)) | bands, moves, splits, ways of life | persistent simulated individuals |
| Ecology | trait axes ([19](19-ecology-and-subsistence.md)) + part/substance primitives ([23](23-bodies-and-substances.md)) | food webs, crashes, anatomies, wounds | genetics, metabolism, chemistry below substances |
| Economy | five channels, conserved stocks ([18](18-economy.md)) | portfolios, nomadism, famine | calories, enzymes, soil chemistry |
| World | quantized field layers ([13](13-worldgen.md)) | rivers, climates, later erosion as field passes | fluid dynamics, particles |
| Governance | registry leaves & actions ([20](20-open-directives.md)) | policy regimes, doctrines | simulated clerks and parliaments |
| Technology | ~45 grammar primitives ([03a](03a-grammar-spec.md)) | designs, doctrines, missiles | reaction kinetics, materials science |

The floor has moved exactly once by this rule: wounds and generated anatomies could not be expressed at trait-axis level, so [23](23-bodies-and-substances.md) lowered the ecology floor to part/substance primitives — with its own hard guards (small authored periodic table, no chemistry below substances, cohorts stay authoritative, every plan self-describes). That is what a legitimate descent looks like; it is also the last one currently foreseen.

## Ruling the deferred queue

The tests decide what the "deferred, slots ready" lists may ever contain. Nutrient cycling: admissible as a visible soil-exhaustion field with a lever and a report line; inadmissible as decomposer chemistry. Weather: admissible as seasonal yield fields a spectator sees and an overseer plans around; inadmissible as atmospheric simulation. Speciation: admissible as slow trait drift that changes what `describe()` says about a region's animals; inadmissible as genomes inside individuals. Disease: admissible as outbreak dynamics over cohorts; inadmissible as virology. When a future feature argument reaches for "but real ecosystems/economies/atmospheres actually work by…", that is the regress talking — the answer is the floor.
