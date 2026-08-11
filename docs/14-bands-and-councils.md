# Bands and Councils — Sentient Nations v1 and the Overseer Loop

> **Scale update:** "province" below now means a **world tile** ([15-multiscale-maps](15-multiscale-maps.md)): ownership, capacity, reports, and `Settle` directives all operate on tile ids, and the blob-province partition no longer exists.

The first playable slice of the overseer architecture: sentient-species bands as nations, a band autopilot, typed directives, and fogged council reports — governable today by Claude (or any agent) through files, upgradeable to MCP at M2 without changing any semantics.

## Spawning

Genesis places `--nations N` bands (default 4), one species archetype each ([08](08-species.md); v1 archetypes are constants in the `species` crate — Duneborn hot/dry, Rivermarsh wet/fecund, Northkin cold-hardy, Valewrought temperate). Seats are chosen by **climate fitness × separation** over habitable provinces; founders are 140–300 people at the seat. A bad draw is a real story, not a bug: a species can land somewhere marginal and start life in a famine trap the overseer must solve.

## The band autopilot ([04](04-institutions-directives.md) in miniature)

Monthly, per nation, at most one settlement:

- **Carrying capacity** per (province, species): `cells × (0.5 + fitness × 5.5) / 4`, integer-exact. Crowding beyond capacity suppresses births, raises deaths, and past 115% fires famine events.
- **Splitting:** a settlement splits when population exceeds the stance-scaled threshold (~220 people, scaled by species drive) or crowding passes the stance trigger. 40% leave for the best-fit unclaimed neighbor — but only if its fitness clears 0.1; the autopilot refuses land the people can't live on.
- **Stances** (`Consolidate | Steady | Expansive`) move both thresholds — the overseer's broad lever.
- **Decrees** (`Settle { province }`) are the sharp lever: they bypass the fitness filter (adjacency and vacancy still validated server-side) and are executed as soon as a bordering settlement can spare ≥60 settlers — a decree issued in Year 8 may be carried out in Year 10, and that gap is [embodiment](12-sovereignty.md) already emerging.
- **First contact** fires the moment territories touch, and is mutual.

## The council loop (event-sourced, no save files)

Per [01](01-architecture.md), world state = `f(seed, directive log)`. A council session is:

1. `just council <seed> <ticks>` — replays genesis + `directives.json` to the target tick, writes `reports/nation-<id>.md` (fogged) and `reports/world.md` (omniscient).
2. Each overseer reads **only its own report** and appends directives to `directives.json` (`{ tick, nation, directive }`; ticks at or after the current tick).
3. Re-run with a later tick target. History extends; the past replays bit-identically because the same directives fire at the same ticks.

Replays are ~seconds, so no save format exists yet by design. Fog rules in reports: own territory in full; the frontier one province deep; other nations only after contact; the chronicle lists only events the nation witnessed.

## Overseers today: Claude and subagents

The pre-MCP agent interface is the report/directive file pair. Demonstrated in-session (seed 42): Claude read The Emberfast's famine-trap report and decreed the two-province dispersal that ended the famines; a spawned subagent read the Rivermarsh report cold and returned a named nation ("The Reedcrown Confederacy"), an Expansive stance, and a staged three-settlement campaign — which made it the largest nation on the map by Year 13, ahead of the two ungoverned control nations. One overseer per nation, each seeing only its own report, composes to any number of agents.

**Upgrade path (M2, [05a](05a-agent-integration-spec.md)):** the report becomes `situation_report`/`dashboard` tool results, the JSON entries become directive tool calls, and the file becomes the MCP layer's logged input stream. Nothing in the sim changes.

## Directive schema (`directive-schema`)

Two generic forms over the registry ([20](20-open-directives.md)) — the schema never grows another verb:

```json
{ "tick": 43200, "nation": 0, "directive": { "kind": "Set",   "key": "expansion.posture", "value": "expansive" } }
{ "tick": 43200, "nation": 0, "directive": { "kind": "Enact", "action": "band.settle", "target": 4431 } }
```

Validation is server-side and in-world against the live registry; rejections are logged events that appear in the nation's chronicle ("a decree failed: tile does not border your territory").

## Deliberately deferred

Internal migration between owned provinces; inter-nation anything (war, trade, talk — [06](06-diplomacy-intel.md) lands after M1 economy); multi-species nations; species as RON data files; occupation splits in the cohort key; population display on the map export (worth adding soon for spectator value).
