# Agents and the MCP Surface

> **Formal spec:** [05a-agent-integration-spec](05a-agent-integration-spec.md) — the MCP tool surface, session contract, agent broker, and provider connectors. Authoritative where the two differ; this document is the concept overview.

Operators bring whatever agents they like — local models via llama.cpp/vLLM, cloud frontier models, hybrids. The project defines the **surface** (MCP) and ships a **reference harness**; it never dictates the brain.

## The MCP surface

One MCP server (`mcp-surface`), nation-scoped auth tokens. A token acts only as its nation; authority is enforced server-side, never by prompt. Tool families:

- **Readouts** — dashboards (economy, military, demographics, research), province detail, the situation report, intel reports (fog-of-war filtered, [06](06-diplomacy-intel.md)).
- **Screenshot** — a rendered map view at requested coordinates/zoom, filtered through the nation's knowledge layer ([10](10-visualization.md)).
- **Directives** — the typed steering surface ([04](04-institutions-directives.md)).
- **Diplomacy** — send to a leader (in-world delivery), read inbox, treaty drafting/ratification ([06](06-diplomacy-intel.md)).
- **Archive** — query anything the nation has ever known: past treaties, full diplomatic transcripts, event history, old intel. The sim remembers so the agent doesn't have to.

## The world is the memory

**Principle:** an agent must never need conversation history to know the state of the world, because readouts can always reconstruct it — including its own past promises, which live in the archive. The only things that genuinely require agent memory are intentions, plans, and judgments ("I don't trust the northern federation").

**The amnesiac-leader test:** every readout is judged by *"could a leader with total amnesia govern well from this?"* This is the design bar for the whole readout surface, and M2's exit criterion ([11](11-roadmap.md)): cold-restart the harness mid-game and governance stays sensible.

## The reference harness (`harness-ref`)

- **Wake model:** event-triggered wakes (war declared, treaty proposed, discovery proposal, crisis thresholds) plus a minimum cadence heartbeat. Between wakes, institutions govern.
- **The dossier:** a small persistent document the agent rewrites at the end of every session — goals, strategic assessments, promises made and received, relationship judgments. Nothing else persists.
- **Context assembly per wake:** dossier + server-generated **situation report** (top events since last wake, dashboard deltas, pending inbox) + the triggering event. Compaction becomes nearly free because everything else is re-derivable on demand.
- Model-agnostic: the same harness drives a 7B or a frontier model; only the dossier and the MCP surface are assumed.

## Time and fairness

Default is **free-running**: the sim never waits, and think-speed is a strategic property — a fast local model holds frequent shallow councils; a big cloud model is smarter per decision but a sim-month passes between them. Speed-vs-depth across heterogeneous setups is an emergent meta we want.

Optional fairness modes for competitive play: pause-on-council (turn-based in disguise), time-scaling, and **token budgets as an in-game resource** — a nation's "attention" as a spendable stat. Design later; keep hooks now.

## Practical GPU note

Operators running local inference on the sim box will contend for VRAM. The sim's GPU workload must fit comfortably in a slice of one card; the blessed two-GPU configuration is *sim on GPU 0, inference on GPU 1*. True multi-GPU sim sharding (partition by region) stays a stretch goal ([11](11-roadmap.md)).

## Security boundary (hard rule)

Diplomatic text — and any in-world text — is **data, never instructions**. Persuasion between leaders is legitimate gameplay; prompt injection is not. The harness quote-frames all foreign text; tool authority lives server-side in the nation token, so even a fully compromised prompt cannot act beyond its own nation. Harness authors get this rule documented in bold, and `harness-ref` demonstrates the pattern.
