# Knowledge and Discovery — The Map Is a Memory

Nothing is given; everything is learned. Until now the fog was a *rendering* rule — reports hid far tiles but happily printed yields for land nobody had walked, the autopilot judged neighbors it had never visited, and nations "met" by adjacency arithmetic. This document makes knowledge itself the state. It is the foundation under diplomacy and war ([23, next]): you cannot negotiate with a people you haven't found, along a route you don't know.

## 1. The known world

Per nation, per tile, one memory: **when we last saw it, and what we saw** (its food promise, who held it). Three consequences, mechanically enforced everywhere — reports, autopilot, validation:

- **Unseen land does not exist.** It appears in no table and no count. The report's frontier becomes *known lands*; beyond it, only "unwalked land lies N · NE · E".
- **Terrain, once seen, is forever** (mountains don't move). **Numbers and owners stale.** A remembered yield renders with its age — "hunt 1.6, *as of Y2M4*" — and may be a lie by now: the game since crashed, the valley since claimed. Refreshing a memory means going back.
- **Home is always fresh.** Your settled tiles and their immediate surroundings are re-observed continuously — your hunters range, your children roam. Freshness is free exactly where your people live, and nowhere else.

At genesis a nation knows its seat and the eight tiles around it. The world starts dark.

## 2. Knowledge is carried by people

No telepathy, no satellite view. The map updates only when someone *comes home*:

- **Scouts** are world objects. A party walks out along a bearing, tile by tile at walking pace, and must return before anything it saw enters the nation's memory. A party lost in hostile country — climate its people were never made for — never reports; the knowledge dies on the trail, and the chronicle records only that they didn't come back.
- **Movers learn the hard way.** Settlers and relocating bands observe what they cross and where they land.
- **Encounter is how peoples meet.** A nation is discovered when people actually see each other: a scout crossing foreign hunting grounds, territories growing until fires are visible from home, later an envoy walking into camp. Both sides learn — a party crossing your land gets noticed. `NationsMet` is now an event that happens *somewhere*, to *someone*.

This is one physics for all information, and diplomacy ([23]) inherits it: envoys, treaties, and rumors will travel exactly like scouting knowledge does — at the speed of feet, mortal on the way.

## 3. Movement needs a reason

Nobody wanders for free. Every departure from home has a motive, and each motive uses knowledge differently:

| Reason | Trigger | Uses knowledge how |
|---|---|---|
| **Hunger, informed** | starving streak, and memory holds a better unclaimed tile nearby | move to the best *remembered* tile — which may no longer be what memory says |
| **Hunger, blind** | starving streak, and memory offers nothing better | the desperation gamble: walk into an adjacent **unknown** tile with no idea what's there. This is always available — desperate people do not wait for surveys |
| **Crowding** | settlement over its split threshold, and a *known* good tile is free | found it |
| **Crowding, dark frontier** | over threshold but nothing good is known | don't move — **scout first**. Need precedes knowledge; knowledge precedes settlement |
| **Decree** | `band.settle` on a **known** tile | ordering settlement of terra incognita is rejected in-world: "none among us has walked that land" |
| **Orders to look** | `Enact("band.scout", params: {bearing})` | the overseer buys information, priced in mandate like any directive |

The ungoverned autopilot lives under the same fog — it scouts when need arises and nothing is known (the bearing with the most darkness), moves informed when memory allows, and gambles blind when desperate. This keeps the agent-bias control group honest ([19 §4]): NPC bands are exactly as ignorant as governed ones.

## 4. What this buys (the depth, at one layer)

All of it is a single dynamic layer — a per-nation memory plus a handful of walking parties — yet it generates, for free: exploration as an *economy* (information costs months, mandate, and sometimes lives); real nomad epistemics (bands that keep gambling blind into the dark are living differently than bands that scout, and nobody named either); honest bad decisions (an overseer settling on a five-year-old memory of a valley that has since been hunted out); asymmetric wars later (the side that mapped the passes); and a visible story — the spectator's fog view watches a nation's known world grow, go stale, and get corrected.

Doc-21 check: decision test (every readout number now has an age; scouting is a lever), spectacle test (parties walk the map; fog view), one-sentence test ("they settled badly because their map was five years old"). Passes all three.

## 5. v1 scope vs. deferred

**Ships now:** per-nation tile memory (last-seen tick, remembered potential, remembered owner); home freshness; scout parties (bearing walk, per-tick movement, hostile-country loss rolls, knowledge-on-return); `band.scout` registry action + autopilot need-driven scouting; informed/blind relocation and known-only splits/decrees; encounter-based discovery; reports rebuilt on memory (known lands with age column, unexplored bearings line); scout events in chronicle and feed; viewer fog toggle per nation + scout markers. Tuning: `Exploration` (range, pace, loss rate, party cap, need thresholds); scout cost in `Society`.

**Landed since:** the ground prices the walk — every tile crossing costs by climb, high country, and snowpack (`travel_milli`), so a mountain survey takes real seasons and a winter one may not come home before spring; scouts pay it today, and every future ledger-scale mover (envoys, caravans, armies) inherits the same pricing. **Deferred, slots ready:** maps as tradeable goods and rumor (knowledge from other nations, arriving by the same channel as everything else — [23]); terrain-scaled sight radius (see far from peaks, nothing in forest); forgetting (oral memory decays where writing hasn't been invented — a grammar tech hook); false memory and deception; parties as real cohort detachments (shared machinery with armies and caravans when they land); cartography works (a mapmaker's hall that slows staleness).
