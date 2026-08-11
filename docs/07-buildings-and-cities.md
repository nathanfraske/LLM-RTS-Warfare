# Buildings and Cities

Buildings follow the same compositional philosophy as technology ([03](03-invention-grammar.md)): a building is a **footprint plus functional modules**, and everything about it — behavior, appearance, cost — derives from the modules.

## Modules

Module families (initial): production (mill, forge, refinery mechanisms), storage, housing, research, administration, culture/faith, defense (walls, towers), and **mounts**.

**Mounts host device designs.** A fortification module with a launcher mount holding a nation's guided-munition design *is* a missile silo — no "Missile Silo" authored anywhere. Coastal batteries, AA sites, and whatever stranger things the grammar produces all emerge the same way. Process-upgrade discoveries retrofit module recipes (better smelting upgrades every forge).

## Derived form

Footprint from capacity; wall glyphs if fortified; chimney smoke if combustion industry; module decorations composed onto the base sprite. At DF-style resolution, procedural building appearance is a solved-by-construction problem.

## Who decides

- **Autopilot placement** ([04](04-institutions-directives.md)): governors respond to pressure signals — housing shortage zones districts, unworked ore zones mines, threat zones walls — within budget shares set by policy.
- **Agent decrees**: `Decree { project, location, budget }` forces specific undertakings — the fortress at the river crossing, a road link, a research campus. Decrees are the agent's only site-specific lever, by design.

## Construction is watchable

Buildings are **built by individuals, visibly**: materials hauled from storage, staged build states (site → frame → complete), workforce drawn from cohorts through the normal labor system ([02](02-simulation-core.md)). Construction sites are hydration triggers — a decree visibly becomes gangs, carts, and scaffolds. This is prime spectator content and it doubles as legible feedback to the agent's screenshot tool: you can *see* your policy happening.

## Organic city growth

No city painter. Simple local rules, compounding over sim-years:

- Roads form along repeated haul routes, then attract frontage.
- Districts cluster by function (industry near ore and water, markets at crossroads).
- Walls trace the settled perimeter when threat rises — and constrain later sprawl until they're outgrown.
- Density rises with land pressure; infrastructure networks (roads → rail → power) arrive as process techs permit.

City time-lapses across a long-running world — hamlet to walled town to sprawling industrial capital — are among the strongest artifacts this project can produce ([11 — M5](11-roadmap.md)).
