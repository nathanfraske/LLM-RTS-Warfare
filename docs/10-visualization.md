# Visualization

## Decision: no game engine — a custom thin renderer in raw Rust

**Stack: wgpu + winit + egui**, structured as a windowless `render-core` crate consumed by two clients: the human `viewer` and the headless `shots` screenshot service.

### Why custom is the right call here

- **The art direction shrinks the renderer to almost nothing.** DF-style top-down 2.5D pixel art needs ~four passes: tile chunks, instanced sprites from an atlas, particles, bitmap glyph text — plus an egui overlay. That's a small, very optimizable surface, and it fits the minimal-nodes rule ([01 — modularity](01-architecture.md)).
- **Headless-first is a hard requirement, not an afterthought.** Agent screenshots mean offscreen render-to-texture → PNG on a server with no window. Raw wgpu does this natively and cleanly.
- **Zero-copy potential.** Sim state already lives in GPU buffers (CubeCL). In co-located dev mode the renderer can draw *directly from sim compute buffers* — no CPU roundtrip. An engine's ownership of the device and frame graph would fight this.
- **No engine churn or ECS overlap.** Bevy brings its own ECS/scheduler (we have our own sim store) and breaking releases we'd track forever, to use a sliver of its features.

### Alternatives considered

| Option | Verdict |
|---|---|
| **Bevy** | The fallback if we ever genuinely need engine machinery (3D, asset pipelines, animation, audio graphs). Not now: ECS overlap, version churn, headless/offscreen is second-class for our purposes. |
| macroquad / ggez | Too limited for the scale ambitions; weaker fit for offscreen service use. |
| Godot (gdext) | Splits the stack across languages and runtimes. Against the whole ethos. |

## Viewer v0 (shipped at M0.5)

The first live viewer (`crates/bin/viewer`) runs the sim **in-process** (the sanctioned dev mode) on the eframe shell: the terrain palette rendered to a nearest-filtered texture, a territory-tint overlay refreshed on ownership changes, presentation-layer villagers and settler caravans ([02](02-simulation-core.md) hydration, visual-only), pan/zoom/inspect camera, timestep controls (pause, 1 day/s → 1 year/s), and the color-coded spectator chronicle with overseer actions highlighted. `just view 42`.

This deliberately uses egui's painter for marks — adequate at current scale, and egui stays in the final stack for all UI chrome regardless. The custom `render-core` wgpu passes below arrive when pixel-art sprite fidelity and entity counts demand them; the viewer's map/overlay/marks split maps 1:1 onto those future passes.

## `render-core` scope

- Tile-chunk pass (terrain, floors), instanced sprite pass (individuals, buildings, devices — composed procedural sprites from [03](03-invention-grammar.md)/[07](07-buildings-and-cities.md)), particle pass (smoke, fire, rocket trails), bitmap-font glyph pass.
- **Zoom LOD:** near zoom is full sprite view; far zoom degrades gracefully into a symbolic glyph map (armies as banners, cities as sigils) — the DF map lineage, and it doubles as the natural rendering of unhydrated aggregate data. The zoom threshold and the [hydration](02-simulation-core.md) threshold are the same design object.
- Palette/theming as data; colorblind-safe defaults.
- egui for all UI chrome: inspector panels, charts (`egui_plot`), timelines, event ticker. Everything stays Rust.

## Clients

- **`viewer`** — winit window, camera, entity inspection, dashboards, live event ticker; on replays, a scrub timeline ([01 — event sourcing](01-architecture.md)). This grows into the spectator product ([11 — M5](11-roadmap.md)).
- **`shots`** — offscreen render → PNG for the MCP screenshot tool. Renders from a snapshot stream **filtered through the requesting nation's knowledge layer** ([06](06-diplomacy-intel.md)): stale regions render stale, unknown regions render unknown. Agents see their fog, not our truth.

## The snapshot protocol

- Clients subscribe with a viewport + LOD level; the server's **interest management** sends only relevant entities at the right detail — the same machinery as hydration triggers, reused.
- Delta-encoded streams over postcard; TCP/localhost transport first, a WebSocket/WebTransport gateway later if browser spectating happens.
- In-process dev mode bypasses the socket but **not the schema**: the protocol boundary is the contract even when zero-copy.
