# Roadmap

Each milestone has an exit criterion — a demo, not a feature list. LLMs enter at M2, not before; scale work waits until M4 by design ([00 — bets](00-vision.md)).

## M0 — Deterministic skeleton

Workspace scaffolded per the [planned layout](01-architecture.md) with CI gates live from day one (determinism gate, cycle gate, forbidden-name gate). `sim-clock`, `sim-store`, `sim-events` (event log + counter-RNG), cohort arrays ticking on CPU. Single-threaded and correct first. Worldgen v1 per [13-worldgen](13-worldgen.md): heightfield, hydrology (rivers/lakes), climate fields, procedural flora settling, derived provinces, dawn-of-time founders — with the `map-export` BMP eyeball loop.

**Exit:** golden replay test green — fixed seed + scripted directives → identical event-log hash; 100k pops ticking with conserved totals.

## M0.5 — Sight ✓ (viewer v0)

Live in-process viewer on the egui shell ([10](10-visualization.md)): terrain + territory textures, villagers/caravans, pan/zoom/inspect camera, timestep controls, spectator chronicle with overseer actions highlighted. The wgpu `render-core` passes land later, with pixel-art sprites.

**Exit met:** you can watch the world run, steer time, and follow the overseers.

> **Post-M0.5 course:** the next arc is the [economy program](18-economy.md) (E0–E4) interleaved with [presence](17-presence.md) phases (P1–P4) — E0 then P1 first. The M1 text below predates tiles/works and stands as the original intent; its "fun to watch unattended" exit criterion still governs.

## M1 — The autopilot nation

Economy with clearing markets; buildings v1 (modules, visible construction); institutions v1 (ministries, governors); typed directives issued by scripts; simple military + battles v1 (aggregate resolution + basic timeline rendering); hydration v1 with task-atom AI. The [invention grammar](03-invention-grammar.md) lands on paper here — type system + historical test suite designed before implementation hardens.

**Exit:** a two-nation autopilot world runs 24h unattended, stays coherent (no death spirals), and is genuinely interesting to watch. This is the make-or-break demo.

## M2 — First agent

`mcp-surface` with readouts, situation reports, directives ([05a](05a-agent-integration-spec.md)); `shots` screenshot service; `harness-ref` with the dossier pattern; `provider-api` + the endpoint tier (one local model via an OpenAI-compatible runtime); one LLM nation vs. scripted nations. Invention grammar implemented; discovery events + agent naming live.

**Exit:** the amnesiac test — cold-restart the harness mid-game and governance stays sensible from readouts + dossier alone.

## M3 — Society of agents

Diplomacy channel with era-dependent latency; treaties-as-code with mechanical violation detection; fog of war + intel layer; interception; multiple heterogeneous agents (local + cloud mixed); `agent-broker` v1 with the setup wizard and the first platform-tier connector ([05a](05a-agent-integration-spec.md) — AllMyAgents if its API answers land by then).

**Exit:** 4+ agents run a week-long world unattended; at least one treaty is made and one broken *with visible consequences*, without human intervention.

## M4 — Scale

Rayon sweeps; CubeCL kernels for cohorts, flow fields, markets (kernel-twin discipline per [01 — principle 5](01-architecture.md)); full hydration promotion/demotion under interest management; profiling with Tracy driving the kernel list.

**Exit:** 1M+ individuals per nation × 8 nations at target tick rate on one GPU, with the sim workload leaving VRAM headroom for local inference on the second GPU.

## M5 — Spectacle

Replay scrubbing, timelines, event ticker, narrator hooks (an LLM retelling the event stream as newsreels), battle performance polish, city time-lapse export.

**Exit:** a stranger watches for 10 minutes and understands the war without reading the docs.

## Deliberately deferred

Multi-GPU sim sharding (region partitioning); browser spectator gateway; competitive fairness modes and leaderboards; procedural species *generator* (archetype files come first, [08](08-species.md)); human advisor mode.
