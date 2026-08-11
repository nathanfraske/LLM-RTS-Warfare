# Architecture — Main Implementation Document

## The two clocks

A sim ticking millions of entities runs at milliseconds per step. An LLM takes 10–120 seconds per decision, and a local 7B differs wildly from a frontier cloud model. These loops are never coupled. The sim is the fast, authoritative clock; agents are a slow policy clock; the **institutional layer** ([04](04-institutions-directives.md)) bridges them by running each nation competently between "council sessions."

By default the sim runs free — it never waits for an agent. Think-speed becomes a real strategic property of an agent setup (fast-and-shallow vs. slow-and-deep). Optional fairness modes exist for competitive play ([05](05-agents-and-mcp.md)).

## Process topology

- **`sim-server`** — headless, deterministic, authoritative. Owns world state, the event log, and all rule enforcement.
- Clients attach over the snapshot/delta protocol ([10](10-visualization.md)): the human **viewer**, the **shots** screenshot service, and the **mcp-surface** that agent harnesses connect to.
- Dev builds may embed the viewer in-process (enabling zero-copy rendering straight from sim GPU buffers), but the protocol boundary remains the contract even then.

## Determinism and event sourcing

- **Fixed timestep.** Working proposal: 1 tick = 1 sim-hour; default speed 24 ticks/sec = one sim-day per real second; adjustable and pausable. Tunable until M1 locks it.
- **World evolution = f(seed, directive log).** Every external input — directives, diplomatic messages, admin commands — enters through the logged input stream. Saves are periodic snapshots plus the log tail. Replay and time-scrubbing fall out for free.
- **Authoritative discrete state runs on CPU in integer/fixed-point math**: combat outcomes, discovery rolls, treaty compliance checks, registry stamps. Bit-exact per machine.
- **Continuous fields may run on GPU** (cohort dynamics, markets, flow fields): small float divergence is tolerated and reconciled at snapshot boundaries — snapshots are authoritative. Cross-machine bit-exactness is a non-goal.
- **RNG is counter-based** (Philox-style), keyed by `(world_seed, tick, system_id, entity_id)`. No shared mutable RNG streams; parallel-safe, replay-safe, and order-independent within a tick.

## Stack

| Layer | Choice | Notes |
|---|---|---|
| Language | Rust (stable) | Entire stack, including GPU kernels via CubeCL |
| CPU parallelism | Rayon | Data-parallel sweeps over provinces/cohorts/entities |
| GPU compute | CubeCL | wgpu backend by default (portable); CUDA/ROCm backends optional |
| Rendering | wgpu + winit + egui | Custom minimal renderer, no game engine — see [10](10-visualization.md) |
| Wire format | serde + postcard | Revisit zero-copy (rkyv) only on profiler evidence |
| Agent surface | MCP | stdio and HTTP transports; nation-scoped auth |

**GPU isolation rule:** all GPU code lives in dedicated kernel crates operating on plain SoA buffers whose layouts are defined in schema crates. CubeCL is a bet on a young project; if it becomes a liability, swapping a kernel crate to raw WGSL/wgpu is a contained change, not a rewrite.

## Modularity principles (imperative)

These are hard rules, enforceable in review and CI — not aspirations.

1. **Minimal nodes.** The workspace is many small crates. Each crate does exactly one thing and exposes a narrow, deliberate public API. The same discipline continues *inside* crates: one concept per module, one concept per file — a file is a node too. If a node's responsibility can't be stated in one sentence without an "and," split it.

2. **No grab bags, at any granularity.** A grab bag is anything — a crate, a file, a module, a function — that grows by *accumulation*: things get added to it because it's there, not because it owns them. This is banned as a pattern, not just as a name. Every concept has exactly one **direct owner** — the node named for it — and new code goes to its owner; when no owner exists yet, create the node ("nodularize out") instead of appending to a convenient neighbor. Named dumping grounds (`utils`, `common`, `helpers`, `misc`, `shared`, a `types` junk drawer) are merely the degenerate case and are forbidden outright. Review tests: *"where would a stranger look for this?"* must have exactly one answer; a file that gains a second concern splits immediately, regardless of size; a function whose description needs an "and" splits too. Soft tripwire: any file trending past ~300 lines gets audited for a hidden second concern.

3. **Explicit data contracts.** Every system declares what it reads and what it writes. Cross-node communication happens only through typed events and typed buffers defined in schema crates — never by reaching into another node's internals. If two nodes need to share a type, that type belongs in a schema crate, named for its domain.

4. **Dependency DAG.** No cycles, enforced in CI. Layering rules: schema crates sit at the bottom and depend on nothing above them; sim nodes may not depend on render, net, or MCP; render depends only on the snapshot schema; the MCP surface depends only on readout/directive schemas. The sim must build and run with zero rendering or networking code compiled in.

5. **The kernel escape hatch.** Any hot node may be given a hyper-optimized twin — a CubeCL kernel, SIMD, a cache-tuned rewrite — behind the same narrow interface. This is always on the table, with obligations:
   - The readable reference implementation stays in-tree as the correctness oracle.
   - Golden tests pin both implementations to identical outputs (tolerance-bounded for float fields).
   - A criterion benchmark demonstrates the win; profiler evidence (Tracy) precedes the work.
   - Escalation ladder: better algorithm → better data layout → Rayon → SIMD → GPU kernel. Skipping rungs requires a reason written down in the crate.

6. **Schema-first content.** All content — invention primitives, species, building modules — is data (RON files) validated against schema crates. Procedural generators emit *data*, never code. Hand-authored examples come first; generators generalize a schema that examples have already proven.

## Workspace layout (planned)

Names are illustrative; the shape and layering rules are the commitment.

```text
crates/
  schema/                # bottom layer: pure data types, no logic dependencies
    world-schema/          # terrain, provinces, entity/identity ids
    directive-schema/      # typed directives and policy parameters
    design-schema/         # invention grammar data types
    snapshot-schema/       # wire types for viewers and shots
  sim/
    sim-clock/             # fixed timestep, tick scheduling
    sim-store/             # SoA storage, archetypes
    sim-events/            # event log, event sourcing, counter-RNG
    world-map/             # terrain, provinces, flow fields
    cohorts/               # statistical population dynamics
    economy/               # production, consumption, market clearing
    invention/             # primitive registry, discovery, design compilation
    institutions/          # the autopilot government
    military/              # armies, aggregate combat resolution
    hydration/             # LOD promotion/demotion, identity registry
    individuals/           # task AI for observed individuals
    buildings/             # module containers, construction
    species/               # species parameter application
    diplomacy/             # messages, treaties, violation checks
    intel/                 # fog of war, espionage, knowledge staleness
    readouts/              # report and situation-digest generation
  gpu/
    cohorts-gpu/           # CubeCL kernels, one crate per domain
    flowfield-gpu/
    market-gpu/
  io/
    snapshot/              # interest management, delta encoding
    net/                   # transport (TCP/localhost first)
    mcp-surface/           # agent-facing MCP server, nation-scoped auth
  render/
    render-core/           # windowless wgpu tile/sprite/particle/text renderer
    viewer/                # winit + egui human spectator client
    shots/                 # offscreen PNG screenshot service
  agents/
    harness-ref/           # reference agent harness (dossier pattern)
    provider-api/          # AgentProvider trait, session/usage types
    provider-openai-compat/# Ollama, LM Studio, llama.cpp, vLLM endpoints
    provider-anthropic/    # Claude via the Anthropic API
    provider-allmyagents/  # platform-tier connector (see 05a)
    agent-broker/          # operator-facing manager: wizard, provisioning, budgets
  bin/
    sim-server/            # composes the sim headless
  tools/
    xtask/                 # structure gates + crate generator (see 01a-foundation)
```

## Testing and CI gates

- **Golden replay:** fixed seed + scripted directive log → identical event-log hash, per machine/backend. This is the determinism gate; it runs on every commit.
- **Grammar suite:** the historical designs ([03](03-invention-grammar.md)) must compile and exhibit their expected capabilities.
- **Conservation properties:** population, goods, and wealth totals conserved across hydration/dehydration and all economic flows (property tests).
- **Structure gates:** dependency-cycle check, forbidden-crate-name check (`utils`, `common`, ...), clippy, rustfmt.
- **Benches:** criterion suites tracked over time; kernel-twin swaps require a recorded before/after.
