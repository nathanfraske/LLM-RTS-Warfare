# The Underground — Generated Geology, Never an Ore List

The world gets a third dimension the same way it got everything else: **author the property axes, generate the periodic table per world, and let everything above bind to properties, never names.** There is no `Ore::Iron`. A world generates *mineral species* — points in property space — and its deposits, caves, faults, and volcanoes fall out of a generated **geologic history**, not painted veins. The underground is the material half of the [invention grammar's](03a-grammar-spec.md) future: when designs need "something hard that takes an edge" or "something that burns hot", they will bid against properties, and whatever this world's ground actually holds either answers or doesn't. The missile test, for metallurgy.

## 1. Mineral species: the generated periodic table

Per world, a palette of mineral species over terminal axes (the [23](23-bodies-and-substances.md) substance discipline, underground): `hardness` (talc … adamant), `metal` (earthy … native lustre — smeltability), `solubility` (inert … karst-former), `energy` (dead stone … burns hot). "Iron", "coal", "limestone", and "marble" are *descriptions of regions*; a given world may hold something iron-ish, something stranger, or a gap where agents expected a metal — scarcity as world character. Every species self-describes ("a hard bright metal-stone", "a soft black rock that burns") — the doc-21 duty, three floors down.

## 2. History makes the map

Geology is generated as a handful of **events**, not as painted deposits (counts in `tuning::Deep`):

- **Uplifts** read straight off the heightfield's great peaks — the mountains that exist *are* the record of them. Their bedrock samples hard.
- **Basins** read off the interior lowlands; their bedrock samples soft, soluble, energy-rich — the coal-and-limestone region of the space, unnamed.
- **Intrusions** are drawn points seeded near uplifts and faults: mineralization halos — **veins** — sampling metal-rich, at depths that get shallower where erosion has stripped the cover. Prospecting will therefore read the *landscape*: metals near old mountains, fuels under old lowlands — honest geology an overseer can learn.
- **Faults** run between the uplift roots: lines of weakness that seat springs and vents, guide later earthquakes, and thin the roof over the deep.
- **Plumes** are the fire below: vent points whose neighborhoods run warm.

Per tile this compiles to a compact column: bedrock species, at most a notable vein (species, depth, richness), fault presence, cave size, vent strength. Columns deepen (multiple strata) when digging exists to care.

## 3. Caves, fire, and the warm ground

- **Caves** form where soluble bedrock meets water — karst by derivation. A cave is a *place*: local maps show its mouth, and the multi-scale principle ([15](15-multiscale-maps.md)) eventually extends **down** — a cave tile opening onto an underground local map is the same move as a world tile opening onto a surface one.
- **Volcanism is live.** Vents erupt on long deterministic schedules; an eruption pours **lava down the drainage tree** — lava is just a molten mineral substance ([23]'s space contains it; the rock beast and the mountain bleed from the same palette) — burning the green to nothing, culling herds and bands in its path, and burying the ground in fresh rock. Then the *existing* weathering rules take over: wet rock sheds fines, roots rebuild organic — **volcanic soils emerge from rules already shipped**, and in a generation the flank of the mountain is the richest ground in the region, which is why people will keep living there. Nobody codes that tradeoff; it composes.
- **Geothermal warmth** bakes into the temperature field at genesis around vents: warm pockets in cold country — springs where a band can winter, green anomalies a scout can find. Habitable islands in the waste, by derivation.

## 4. Knowledge and extraction (the interfaces)

The spectator sees the underground; **agents must learn it**. Surface signs (vents, cave mouths, exposed veins where cover is thin) ride the existing [knowledge layer](22-knowledge-and-discovery.md); deep truth will need prospectors and diggings — scouts' machinery pointed down. Extraction, when works or the grammar demand it, bids against the column: hardness prices the digging, depth prices the reach, richness pays it. The regolith ([27](27-the-ground.md)) is the top of the same stack — its rock fraction takes its character from the bedrock beneath.

**Ships now (U0 + first fire):** mineral palette, event history, per-tile columns (bedrock, veins, faults, caves, vents), geothermal genesis warmth, live eruptions with lava runs and their full surface consequences, inspector lines, events in feed and chronicle. **Landed since:** volcanic **ejecta** — ash rides the latitude winds far past the lava, downwind-stretched, smothering the green by heaviness and laying fines the weathering turns to farmland (light ash enriches, heavy ash near the vent buries even the soil); **earthquakes** on the faults — epicenters with their own long clocks whose shakes topple finished works, take a small toll of the settled, and break the scree loose on every slope in reach; and the **layer camera** — G cycles the spectator's view down through the columns (surface → under light cover → the deep), bedrock colored by its mineral's axes, veins glinting in their depth band, caves as hollows, faults as cracks, vents as fire. **Queued:** underground local maps; prospecting via the knowledge layer; extraction verbs and grammar binding (U2); hydrothermal vein growth along faults; aftershocks and quake-driven river rerouting when elevation goes live.
