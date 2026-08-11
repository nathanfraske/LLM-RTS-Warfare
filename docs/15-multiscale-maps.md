# Multi-Scale Maps — World Tiles and Local Maps

The Songs of Syx / Dwarf Fortress structure: the strategic map is a grid of **world tiles**, and every tile opens into a full **local map** underneath, where individuals exist at person scale. This supersedes the province-blob partition of the original worldgen pass.

## The layers

| Layer | Grid | Cell means | Who lives here |
|---|---|---|---|
| **World** | ~192² world tiles (CLI-selectable) | ~10 km of country; **a tile is the province** — the unit of ownership, settlement, and strategy | Nations, cohorts, armies, caravans |
| **Local** | 256² local cells *per world tile* | ~40 m; a person occupies about one cell | Individuals, buildings, trees, battles |

Effective person-scale world: ~49,000² cells at defaults. The design leaves room for two more layers when needed: a **regional** middle layer (16× between world and local) if world sizes grow past what one grid handles gracefully, and **z-levels** on local maps (underground, cliffs) in the Dwarf Fortress lineage — both additive, neither blocked by today's shapes.

## Provinces are tiles now

The BFS province partition, orphan attachment, and `Province` summary struct are deleted — a world tile *carries its fields directly* (elevation, water, temperature, moisture, fertility, flora). Ownership is `owner[tile]`; adjacency is the grid; terrain labels derive per tile. This is a strict simplification: one grid, one id space (`TileId` = row-major index), no derived blobs. Nations expand tile by tile; reports and directives speak in tiles ("Settle tile 4021").

## Local maps are derived, not stored

A local map is a **pure deterministic function** of `(world seed, tile coords, surrounding world-tile fields)` — generated on demand in milliseconds, discarded freely, identical every visit ([01 determinism](01-architecture.md)):

- **Elevation**: bilinear interpolation of the surrounding tiles' elevations plus high-frequency noise keyed by *global* cell coordinates, so adjacent local maps agree at their shared edges.
- **Water**: sea where interpolated elevation is below sea level; riverine tiles carve a wobbling channel through the map along the tile's dominant flow direction.
- **Vegetation**: trees and ground cover scattered deterministically from the tile's settled flora (species, form, density) — a forest tile *is* a forest down here.
- **Settlement**: populated tiles place a camp (huts near a chosen center) sized by the cohort; individuals wander it at person scale. Buildings-proper attach here when [07](07-buildings-and-cities.md) lands.

**Persistence rule:** until construction exists, local maps are stateless projections of world state. When civilization starts *modifying* local maps (buildings, roads, clearing), those edits become per-tile overlay records on top of the same deterministic base — the base is never stored.

## What renders where (the art-scale fix)

People were oversized because individuals were drawn on the strategic map. Now:

- **World view**: terrain tiles, territory tint, settlement markers, armies/caravans as units. No individuals.
- **Local view**: open any tile (double-click; Esc/Backspace returns) and individuals appear at their true scale — one person ≈ one local cell — with the trees, water, and camp of that exact tile.

Hydration ([02](02-simulation-core.md)) maps cleanly: cohorts stay authoritative at the world layer; opening a local map is the presentation-layer hydration trigger.

## Migration notes (applied in this pass)

`TileId` replaces `ProvinceId` everywhere; `CohortKey.tile`; events and directives renamed (`Settle { tile }`); carrying capacity is computed from tile fields (climate fit × fertility) instead of blob cell counts; nation spawn scans habitable tiles with fitness × separation; reports list owned tiles and cap the frontier table at the best candidates. The old directive log referenced blob-province ids and is reset — new world, Year 1.
