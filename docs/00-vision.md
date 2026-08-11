# Vision

LLM agents helm nations in a persistent, real-time, massively populated world. They do not click units. They read reports, take map screenshots, set budgets and doctrines, direct research focus, decree projects, and — centrally — talk to each other: trade deals, alliances, threats, betrayals, all in natural language routed through the world itself. The simulation does the rest, and the pleasure of the thing is watching it: individuals farming and hauling and drinking at taverns, cities growing organically, armies breaking on walls, strange weapons appearing from a procedural technology space and getting names from the leaders who commissioned them.

## Pillars

1. **Agents are overseers.** The interface between an agent and its nation is policy, not control. If a design pressure ever pushes toward per-unit orders, the design is wrong — fix the institutional layer instead.
2. **The world is watchable.** Legibility of individual behavior is a core feature, not decoration. Every system should ask: what does this look like from above at 16px?
3. **Worlds outlive context windows.** Agents must be able to govern well despite compaction, restarts, and model swaps. The world itself is the memory; readouts must pass the amnesiac-leader test ([05](05-agents-and-mcp.md)).
4. **Millions through level of detail.** The bulk of the population is statistical; individuals are hydrated where attention is. Aggregates are authoritative; individuals are faithful views ([02](02-simulation-core.md)).
5. **Nothing is a content list.** Technology, buildings, and species derive from a small authored primitive set and composition rules. We author the periodic table, not the molecules ([03](03-invention-grammar.md)).
6. **Everything happens in-world.** Diplomacy has delivery time and can be intercepted. Intelligence has staleness. Screenshots are fog-of-war views. No out-of-band channels.

## Non-goals

- **Not a physics-level emergent simulator.** That is the separate "deep emergent civ sim" project. Here, emergence is welcome but secondary; systemic grand-strategy-style simulation (cohorts, markets, institutions) is the model.
- **Not a human-playable RTS.** Spectator-first. A human "advisor mode" (issuing directives by hand) may exist later as a sandbox tool, but no human-facing control UI drives design.
- **Not an esport.** Perfect balance is a non-goal; asymmetry between nations, species, and tech paths is content. Fairness matters only in optional competitive modes ([05](05-agents-and-mcp.md)).
- **Not cross-machine bit-determinism.** Replay is guaranteed per machine/backend; GPU float fields are reconciled at snapshots ([01](01-architecture.md)).

## The risky bets (ranked)

1. **The institutional autopilot** must run a nation competently with no LLM attached. If it can't, agents get dragged into micromanagement the interface doesn't support, and everything collapses. This is the make-or-break system, and it's why NPC factions run on it permanently as a proving ground.
2. **The invention grammar's type system** is where design effort concentrates. Everything compiles against it; it's expensive to change once code exists.
3. **Long-horizon agent coherence** is a genuine research problem even with the dossier pattern. Expect iteration.
4. **Simulation depth** must make policy consequences non-obvious, or this becomes LLMs playing a spreadsheet where negotiation is the only interesting layer. Markets must actually clear; logistics must actually constrain.

Scale is deliberately *not* on this list — LOD plus data-oriented design is well-trodden ground. Optimization must not eat the early schedule.
