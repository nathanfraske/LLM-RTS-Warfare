# Presence — Person-Scale Real Time

The want: stand in a tile at **one second per second** and watch it be real — people walking around, talking, chopping trees that actually fall, tending fields, and battles fought live. Not a screensaver over statistics: a simulation you can be present in. This document commits the architecture for that without breaking the aggregate-authoritative core ([02](02-simulation-core.md)).

## One time dial, two granularities

The world clock stays authoritative at 1 tick = 1 sim-hour. **Presence time** subdivides it: 1 presence tick ≈ 1 sim-second (3600 per world tick), simulated only on *hydrated* local maps — the tiles someone (a spectator, an agent screenshot, a battle) is actually watching.

The viewer's speed dial becomes one continuous range. At `1s/s`–`1min/s` you are *in* presence time: individuals act per-second, world ticks crawl. From `1h/s` upward presence collapses back to sparse performance (today's wanderers) and the strategic layer carries the story. Same world, same clock — zoom in time the way you zoom in space.

## What presence simulates (phased)

- **P0 — shipped:** decorative wanderers and camps on generated local maps.
- **P1 — legible task loops:** individuals get jobs derived from tile state — chop (the tree visibly falls and is removed), haul to the camp, tend farmstead plots, build in-progress works (scaffolds + builders instead of nothing until completion), rest, and socialize with floating **barks** (short chatter lines — "talking to each other" starts as flavor with personality, not language simulation).
- **P2 — conservation feedback:** presence work becomes real. The rule that keeps determinism and fairness intact: **presence never invents resources; it *earns against* the tile's aggregate budget.** The month says this tile produces N wood; watched chopping visibly claims pieces of that N; unwatched tiles just get N. Feedback enters the event log at world-tick boundaries like any other event — replay-exact, observed and unobserved tiles identical in outcome. (Same reconciliation philosophy as the GPU field rule in [01](01-architecture.md).)
- **P3 — battles in presence time:** [09](09-battles.md)'s battle timeline performed at per-second granularity when watched — formations, volleys, routs live. Resolution stays aggregate-authoritative; a later *opt-in* mode may resolve observed battles at presence scale, which trades observer-independence for spectacle and gets decided then, not smuggled in.
- **P4 — hot tiles:** a budget of always-presence-simulated tiles (capitals, frontlines, wherever the narrator's attention is), LRU-evicted, everything else statistical. This is the scaling knob — and where presence crowds eventually justify GPU behavior kernels ([11 — M4](11-roadmap.md)).

## Determinism stance

A tile's presence activity is a pure function of `(seed, tile, world-tick window, tile state)` — like local-map generation itself. Watching changes *when* the pretty part happens, never *what* the ledgers say. The amnesiac rule gets a twin: **the unobserved world must be indistinguishable in outcome from the observed one.**

## Operating modes interact cleanly

In Free-Run mode ([05a](05a-agent-integration-spec.md)) presence viewing at 1s/s effectively pauses strategy — a month of council silence passes in twenty real minutes, which is exactly the DF-style tradeoff and fine. In Council Rounds mode, presence time is pure spectating between decision windows. Neither mode changes what presence may simulate.
