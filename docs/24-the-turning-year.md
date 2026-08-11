# The Turning Year — Seasons as a World Condition

The calendar has twelve months ([01](01-architecture.md): 720 ticks each) but the world doesn't know it: fields are frozen in an eternal noon and yields never move. Seasons make time a *force* — and they are an **input condition, not a constant**: the operator sets the year's shape per world and can reshape it at will.

## 1. One forcing, everywhere at once

A single seasonal factor per tile per month, derived from latitude and the month's phase (the temperature field already knows latitude — the year just swings it). Amplitude, phase, and hemisphere behavior live in `tuning::Seasons`, carried by `RunConfig` like every other world condition: a gentle world, a brutal one, a tide-locked strange one — all one config away, changeable between runs or (later) mid-run as a live condition. No weather simulation, no atmosphere — a field modulation, exactly the doc-21-admissible form.

The one factor flows through everything that already exists, which is the point — no new systems, only existing dynamics finally hearing the clock:

- **Flora** regrowth pulses and fades; the green retreats in winter and the baseline breathes.
- **Fauna** growth follows the plants down and up; lean months thin the herds.
- **Channels** ([19](19-ecology-and-subsistence.md)): cultivation gets a growing season, fishing gets runs, hunting gets winter-lean game and snow-slowed range. Per-channel seasonal response derives from what the channel draws on — nothing is scripted per month.
- **Stores**: spoilage and the granary stop being bookkeeping and become survival. A people that ignores Month 10 dies in Month 12.

## 2. What emerges free of charge

Return-following bands oscillate between summer and winter grounds — **transhumance, unauthored**. The [knowledge layer](22-knowledge-and-discovery.md) gets its best trick without a line of code: *a tile scouted in summer lies about winter*. Memory already stamps when a tile was seen; once yields swing, a July memory of a valley is a beautiful, dangerous falsehood, and settling on it is a new emergent way to die. Overseers must learn to ask *when* their map was made — and the charter's levers (labor, settlement timing) acquire an annual rhythm the autopilot control group will also exhibit, honestly.

## 3. It must be seen

The year changes the *picture*, not just the ledgers ([10](10-visualization.md)): terrain tint follows the season factor (snowline walking down the mountains, the plains browning, ice on the high lakes), and local maps inherit it (bare trees, dulled grass at person scale). The spectator should know the month at a glance from the land alone; the fog view composes with it (a stale memory renders in the season it was *seen*, not the season it is — the map literally shows you your own outdated summer).

**Ships as:** `tuning::Seasons` + the per-tile monthly factor; flora/fauna/channel/spoilage modulation; viewer seasonal terrain tint at both scales; the season stamp on tile memories. **Deferred, slots ready:** weather events as short-lived field perturbations (storms, droughts) once seasons prove the plumbing; mid-run condition changes as logged world events; season-aware works (icehouses).
