# World Generation — Dawn of Time

> **Scale update:** the world grid's cells are now **world tiles** — the strategic/province unit — and every tile opens into a person-scale local map ([15-multiscale-maps](15-multiscale-maps.md)). The field pipeline below is unchanged; it simply runs at tile resolution (default 192²), and `local-map` derives the detail underneath.

The world starts primordial: land, water, climate, and wild flora — no civilization. Founder bands (tiny populations) appear at habitable sites and everything else emerges in sim time. All start parameters are tunable; "dawn of time" is the default framing, not a hard rule.

## Principles

1. **Fields, not biomes.** Generation produces physical fields — elevation, water, flow, temperature, moisture — and everything else *derives*: terrain labels are display summaries, fertility is a formula over fields, vegetation is a competition over fields. No authored biome painting, matching the no-content-lists rule ([00 — pillar 5](00-vision.md)).
2. **Genesis floats, integer aftermath.** Noise runs in `f32` but is **transcendental-free by rule** (no `sin`/`exp`/`powf` — polynomial fades, hashed gradients, `sqrt` only), which keeps it deterministic across platforms in practice. The float surface is then quantized to integer layers (elevation in meters `i32`, temperature in deci-°C `i16`, moisture `u8`), and *all* downstream work — hydrology, provinces, flora, everything sim-time — is integer/fixed-point and bulletproof-deterministic ([01](01-architecture.md)).
3. **Square map, edge seas.** v1 is a single square grid (default 512², CLI-selectable) with soft edge falloff so borders are ocean. Massive scale is a data-layout problem we already solve (flat SoA vectors); 2048² works today on CPU, and erosion/weather land on GPU later.

## Pipeline

`WorldFields::generate` (crate `world-map`), then flora, then provinces:

1. **Heightfield** — domain-warped fractal noise for continents plus ridged noise for mountain chains, masked to continental interiors; edge falloff; sea level picked by target ocean fraction (~58%); quantize to meters.
2. **Hydrology** — priority-flood depression filling (deterministic total order). Cells whose filled level exceeds their ground level become **lakes**; the flood's pop order yields a drainage tree (every cell drains monotonically to ocean/edge), and accumulating rain down that tree gives flow volume. High-flow cells become **rivers**, which terminate in lakes or ocean. Mountains, valleys, watersheds all fall out of the heightfield.
3. **Climate v1** — temperature from latitude (equator at mid-map) plus lapse rate with altitude; moisture from decaying distance-to-water plus noise jitter. Deliberately cheap; the upgrade path is wind-advected moisture, rain shadow, and seasons as a sim-time weather system (GPU, [11 — M4](11-roadmap.md)+).
4. **Flora** (crate `flora`) — see below.
5. **Provinces** — derived last: seeds spread over land by best-candidate sampling weighted by fertility, multi-source BFS partitions the landmass into contiguous provinces; each summarizes its cells (dominant terrain label, mean fertility including vegetation, coastal/river access, habitability).
6. **Founders** — dawn-of-time bands (~80–400 people) seed only habitable provinces (fertile + water access). The rest of the map is empty frontier for migration to claim.

## Procedural flora (the diversity engine)

No plant list. Each world generates its own flora as **genomes** — parameter bundles in the same philosophy as [08-species](08-species.md):

- Growth form (grass/shrub/tree), temperature optimum + tolerance width, moisture optimum + width, altitude ceiling, competitive vigor, and a glyph seed for the viewer.
- ~24 species per world (configurable), **stratified across climate space** so deserts and tundra get contenders too, not just the temperate sweet spot.

Diversity comes from **ecological settling**, not painting: each species gets origin sites where its fitness is high, then spreads outward round by round, claiming cells where its fitness beats the incumbent's (incumbents get a defender's bonus). The result is contiguous ranges, competition frontiers, endemism, and tree lines — evolutionary-*shaped* geography from pure competition.

**Upgrade path (in order):**
1. **Sim-time ecology** — monthly spread/dieback responding to climate shifts, fire, and civilization (logging, farming, grazing). Deforestation becomes real and visible.
2. **True evolution** — mutation on spread, speciation events when ranges fragment. The genome struct is designed so this is additive.
3. **Fauna** — mobile genomes over the same fields (grazers, predators, fish), feeding the food economy and hunting. Sapient species ([08](08-species.md)) remain a separate concept.

## Erosion and weather (deferred, planned)

Erosion is explicitly wanted and explicitly *not* v1. The plan: thermal + hydraulic (stream-power) erosion as GPU passes over the same integer/SoA layers, run either as extra genesis iterations ("age the world 10k years") or slowly in sim time. Weather likewise: a coarse GPU advection field driving moisture/temperature anomalies, which the flora ecology then responds to. Both slot into the existing field layout without schema changes — that's why fields-first matters.

## Seeing it

`map-export` (bin crate) renders any layer to BMP with zero dependencies: `terrain` (composed relief: water depth, vegetation, deserts, snowline), `height`, `flora` (species ranges), `provinces`. Recipe: `just map 42` → `maps/terrain-42.bmp`. This is the pre-viewer eyeball loop; the real viewer arrives at M0.5 ([11](11-roadmap.md)).

## Invariants under test

Same seed ⇒ identical field vectors; land fraction within band; every land cell's drainage path terminates (acyclic by construction); flora covers the habitable majority; province partition covers all land, contiguously, at the requested count.
