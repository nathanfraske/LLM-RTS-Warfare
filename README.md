# LLM-RTS-Warfare

A mostly-automated, RTS-like civilization simulator where LLM agents — local or cloud, whatever the operator wants to run — act as national **overseers**, not unit micromanagers. Agents set policy, direct research focus, and conduct diplomacy by literally talking to each other through in-world channels, while the simulation runs their nations day to day at the scale of millions of individuals. Humans spectate.

**Status:** M0 foundation in progress. The documents are the source of truth — update them when decisions change.

## Quickstart

```text
just check        # fmt + clippy + tests + structure gates (the full local gate)
just run 42 8640  # run a world: seed 42, one sim-year of hourly ticks
just replay 42    # determinism proof: two runs must print identical hashes
```

See [01a-foundation](docs/01a-foundation.md) for bootstrap from a bare machine.

## Document index

| Doc | Concept |
|---|---|
| [00-vision](docs/00-vision.md) | What this is, pillars, non-goals, the risky bets |
| [01-architecture](docs/01-architecture.md) | **Main implementation document** — topology, determinism, modularity principles, workspace layout |
| [01a-foundation](docs/01a-foundation.md) | Build/dev/deps across platforms — `just` + `cargo xtask`, toolchain pinning, CI plan |
| [02-simulation-core](docs/02-simulation-core.md) | Cohort layer, LOD hydration, the individual registry, task legibility |
| [03-invention-grammar](docs/03-invention-grammar.md) | Procedural technology: primitives, composition, derivation |
| [03a-grammar-spec](docs/03a-grammar-spec.md) | **Formal grammar spec** — scales, interfaces, primitive registry, compiler, test suite |
| [04-institutions-directives](docs/04-institutions-directives.md) | The autopilot government and the typed directive surface |
| [05-agents-and-mcp](docs/05-agents-and-mcp.md) | MCP surface, agent harness, memory/compaction strategy |
| [05a-agent-integration-spec](docs/05a-agent-integration-spec.md) | **Formal agent spec** — MCP tools, session contract, agent broker, provider connectors |
| [06-diplomacy-intel](docs/06-diplomacy-intel.md) | In-world diplomacy, treaties-as-code, espionage, fog of war |
| [07-buildings-and-cities](docs/07-buildings-and-cities.md) | Building modules, construction, organic city growth |
| [08-species](docs/08-species.md) | Species as parameter bundles, multi-species nations |
| [09-battles](docs/09-battles.md) | Aggregate-authoritative combat, observed battle rendering |
| [10-visualization](docs/10-visualization.md) | Renderer decision (custom wgpu, no engine), snapshot protocol, agent screenshots |
| [11-roadmap](docs/11-roadmap.md) | Milestones M0–M5 with exit criteria |
| [12-sovereignty](docs/12-sovereignty.md) | Embodied rule — the Seat, edicts, legitimacy, channels, puppets/exile/restoration |
| [13-worldgen](docs/13-worldgen.md) | Dawn-of-time world generation — fields not biomes, hydrology, procedural flora |
| [14-bands-and-councils](docs/14-bands-and-councils.md) | Sentient nations v1 — band autopilot, directives, fogged reports, the council loop |
| [15-multiscale-maps](docs/15-multiscale-maps.md) | World tiles + local maps — provinces are tiles; every tile opens to a person-scale map |
| [16-mandate-and-works](docs/16-mandate-and-works.md) | The price of direct rule — mandate, autonomy friction, commissioned works |
| [17-presence](docs/17-presence.md) | Person-scale real time — 1s presence ticks, task loops, conservation feedback |
| [18-economy](docs/18-economy.md) | The economy program — conserved goods, needs, labor, chains, trade (phases E0–E4) |
| [19-ecology-and-subsistence](docs/19-ecology-and-subsistence.md) | The living world — fauna food webs, five subsistence channels, emergent ways of life |
| [20-open-directives](docs/20-open-directives.md) | No authored verb list — the self-describing policy & action registry |
| [21-authored-floor](docs/21-authored-floor.md) | Where emergence stops — the floor rule, the three tests, the floors per domain |
| [22-knowledge-and-discovery](docs/22-knowledge-and-discovery.md) | The map is a memory — scouts, staleness, motivated movement, encounter-based contact |
| [23-bodies-and-substances](docs/23-bodies-and-substances.md) | The anatomy grammar — generated organs, limbs, senses, and working fluids; wounds as addresses |
| [24-the-turning-year](docs/24-the-turning-year.md) | Seasons as a configurable world condition — one forcing through every existing system |
| [25-culture-and-generations](docs/25-culture-and-generations.md) | Learned culture on age-structured cohorts — substrate modeled, content never authored |
| [26-living-terrain](docs/26-living-terrain.md) | The water cycle, snow, erosion, soil, day and night — field passes, never fluid dynamics |
| [27-the-ground](docs/27-the-ground.md) | Regolith as composition — grain ladder, weathering, wash, emergent deserts and loam |
| [28-light-and-shadow](docs/28-light-and-shadow.md) | Sun and moon over the land — live hillshading, cast tree shadows, moonlit relief |

## Stack (decided)

Rust (stable) · Rayon for CPU parallelism · CubeCL for GPU compute (wgpu backend by default; CUDA/ROCm optional) · custom wgpu + winit + egui renderer, no game engine · headless deterministic sim server, event-sourced · MCP as the agent-facing surface.

## License

Apache-2.0 — see [LICENSE](LICENSE).

## The one-paragraph architecture

The sim is a headless, deterministic, event-sourced server ticking at a fixed timestep. Nations run themselves through an institutional autopilot; LLM agents steer via typed directives and negotiate through in-world diplomacy, connected over MCP. Populations exist as statistical cohorts on the GPU, with individuals hydrated to full simulation wherever attention is. All content — technology, buildings, species — derives from a small authored set of primitives ("author the periodic table, not the molecules"). Viewers and the agent screenshot service are thin clients over a snapshot protocol, rendered by a custom minimal wgpu pixel-art renderer.
