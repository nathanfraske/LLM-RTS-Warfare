# Living Terrain — Water, Weather, Light, and the Moving Earth

The land stops being scenery. Water cycles through sky, snow, river, and sea; day turns to night under a moon; soil is made, moved, and lost. All of it stays inside the [doc-21](21-authored-floor.md) world floor — **field passes on the fixed grid, never fluid dynamics** — and all of it is [tuning](01a-foundation.md) from birth: every rate, threshold, and period is world configuration, changeable per world, never a constant to hunt down later. This realizes and absorbs [24-the-turning-year](24-the-turning-year.md): seasons are the forcing; the water cycle is how the forcing touches the land.

## 1. Two books, strictly kept

- **Authoritative climate** (sim state, deterministic, monthly): what feeds ecology, economy, and knowledge. Integer fields only.
- **Presentation water and light** (viewer, per-frame, zero sim state): flow, waves, tides, moonlight — derived from authoritative fields and the sim clock, conserving what the ledgers say ([17](17-presence.md) discipline: presence renders truth, never invents it).

## 2. The water cycle (W0 — ships now)

One monthly pass over existing fields plus three live ones (`wet` — airborne moisture; `snowpack` — banked winter water; `growth` — the per-tile gate everything green reads):

1. **Season**: effective temperature = genesis baseline + a latitude-scaled seasonal swing (`tuning::Seasons`: equator/pole amplitudes, phase, hemisphere flip — a gentle world, a brutal one, or a strange one is config, and the swing is strongest where it should be).
2. **Evaporation**: seas, lakes, and rivers push moisture into the air above them, more when warm.
3. **Transport**: one directional mixing step — prevailing winds derive from latitude band (trades, westerlies), never per-tile authorship.
4. **Rain**: air sheds moisture at a rate that climbs with **orographic lift** — air pushed up rising ground rains out on the windward side and arrives dry beyond. Rain shadows, wet coasts, and dry interiors *emerge from the heightmap*; nobody paints them.
5. **Snow**: rain falling below freezing banks as snowpack instead of feeding growth — winter *stores* water on the heights. Warmth melts it back out: **spring flush**, downhill, on schedule, for free.
6. **The gate**: warmth × delivered water → per-tile growth factor. Flora regrowth, cultivation, and (through the plants) the herds all breathe with it; deep snow slows the hunt. Reports and the autopilot read the *seasonal* numbers, so return-following bands drift with the year — transhumance stays emergent — and every tile memory ([22](22-knowledge-and-discovery.md)) is now stamped with the season it was seen: **a summer map lies about winter**, structurally.

The drainage tree gains its missing half: `drains_to` (each cell's outflow neighbor, computed by the priority flood since genesis) is now a stored field — flow animation, discharge, and sediment all ride it.

## 3. The moving earth (W1 — designed, next)

Slow field passes along `drains_to`, all integer, all monthly-or-slower:

- **Discharge**: seasonal water actually routed downtree — rivers swell with the melt and shrink in drought; fishing and (later) navigation and floods read it.
- **Erosion and sediment**: high-discharge steep cells lose material downtree; it settles where the water slows. Elevation changes at geological gentleness; **fertility follows the silt** — floodplains and deltas *become* the best farmland because that is where the soil goes, closing the loop the economy already reads (`cell_fertility` becomes a living field).
- **Soil life**: cultivation depletes what deposition and fallow rebuild — the soil-exhaustion slot doc-21 pre-approved, landing on real sediment instead of an abstract meter.
- **Flood events**: a melt-heavy month over a swollen tree marks floodplain tiles — destructive and enriching at once, and a reason overseers care about the snowpack readout.

## 4. Light and living water (W2 — presentation, first slice ships now)

The sim clock already counts hours; the sky finally uses them. **Day and night**: a light tint driven by the hour, dawn and dusk ramps, night depth set by the **moon** — a `tuning::Sky` period, full moons silvering the map, new moons going truly dark. **Seasonal ground**: the terrain texture re-renders with the months — snowpack whitens the heights and the snowline walks the contours; browning follows the growth gate. **Rivers flow**: motes stream along `drains_to`, faster where accumulation is high. **The sea moves**: wave-pulse at the coasts, and a slow tidal breathing keyed to the moon clock — the same clock that will drive tidal mechanics if they ever earn a ledger entry. Later slices: rain and snowfall visible at both scales, floods as spectacle, local maps inheriting all of it.

## 5. The burning season (landed)

Where fuel, drought, and an ignition meet, the land **burns**: lava lights its borders, dry lightning rarely finds parched tinder, and the fire spreads month by month — twice as eagerly downwind, twice again through drought — until rain, snowpack, water, or bare ground starves it. The burn eats the green (the herds and the gatherers feel it through the ledgers that already exist), sweeps settled tiles at a cost in people, and leaves scars the regrowth reheals at the pace the growth gate allows. Flames render live on the map; the scars persist in the terrain's own colors. Deferred, slots ready: creatures with burning humors ([23](23-bodies-and-substances.md) volatile carriers) lighting the ground where they bleed, once wounds spill at world scale; fire as a weapon when war lands.

## 6. Guards

No atmosphere cells, no fluid solver, no per-drop water: fields, passes, and derived light — the same shape as everything above the floor. Every number in `tuning::{Seasons, Weather, Sky}`. Species climate *niches* stay judged against the annual baseline (a niche is a way of life, not a month); the seasons act through food, snow, and water, which is how winter actually kills. And the one-sentence test holds all the way down: "the pass flooded because the snow went early", "the delta feeds them because the mountains are washing into it."
