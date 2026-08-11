# Agent Integration — Formal Specification (v1)

Authoritative spec for the agent-facing surface and onboarding; [05-agents-and-mcp](05-agents-and-mcp.md) is the concept overview and this document wins where they differ. Covers: the agent-first development rule, the MCP surface v1, the session contract, the **agent broker**, and provider connectors (local endpoints, cloud APIs, and agent platforms such as AllMyAgents).

## 1. The agent-first rule (how "agent-first" is actually done)

Agents are the primary users of this game; humans are spectators. Enforced as development discipline, not sentiment:

1. **A sim feature does not exist until it's on the surface.** Definition of done for any system: sim behavior + readout coverage + (where steerable) a typed directive + tests. A mechanic no agent can perceive or influence is dead weight and doesn't merge.
2. **Text parity for everything visual.** Every visual observation has a structured counterpart (`map_data` alongside `screenshot`), because many local models reason better over text than pixels — and some are text-only. No capability may be screenshot-only.
3. **The amnesiac bar is CI-able.** The `charter` + `situation_report` + archive tools must contain enough to govern from a cold start ([05](05-agents-and-mcp.md)); M2's exit test automates this: wipe harness state mid-game, resume, assert directive quality doesn't collapse.
4. **Spectator = agent minus fog.** The viewer consumes the same snapshot/readout data an agent could query, plus omniscience. No spectator-only mechanics, no agent-only mechanics.

## 2. The MCP surface v1

One MCP server per world (`mcp-surface`), transports: streamable HTTP (normal), stdio (local single-agent dev). Auth: per-nation bearer token minted at world creation — scoped, rotatable, revocable. Authority is enforced server-side per token; a prompt can never escalate it.

All directive calls enter the logged input stream (they are exactly the replay input, [01](01-architecture.md)); readout calls land in a query log (for research/debugging, excluded from replay semantics).

### 2.1 Meta

| Tool | Purpose |
|---|---|
| `charter()` | The amnesia bootstrap: who you are, species traits, world rules digest, current era snapshot, how to govern (readouts → directives → diplomacy). Stable text, cacheable. |
| `dossier_read()` / `dossier_write(text)` | Optional server-held private memory blob per nation — versioned with saves, **excluded from spectator/replay visibility**. Lets any harness or platform survive cold restarts without its own storage. |
| `set_alerts(conditions)` | Standing wake triggers: metric thresholds, event kinds, treaty violations. Feeds the wake feed (§2.5). |

### 2.2 Readouts

| Tool | Purpose |
|---|---|
| `situation_report()` | Digest since last session: top events, dashboard deltas, inbox count, pending proposals. The standard wake payload. |
| `dashboard(domain)` | `economy \| military \| research \| demographics \| diplomacy` summary stats with trends. |
| `province_detail(province)` | Local economy, population, buildings, garrisons, construction. |
| `foreign_intel(nation)` | Knowledge-layer view of another nation — staleness timestamps and confidence marked ([06](06-diplomacy-intel.md)). |
| `arsenal(scope)` | `own \| proposals \| observed(nation)` — designs with compiled capabilities; foreign ones described by observed behavior only. |
| `archive(query, kind?, range?)` | Everything the nation has ever known: treaties, full diplomatic transcripts, events, own past directives, old intel. |
| `map_data(region, layers)` | Structured, fog-filtered map (terrain, settlements, forces, trade). The text-parity twin of `screenshot`. |
| `screenshot(center, zoom, layers?)` | Rendered PNG from `shots` ([10](10-visualization.md)), filtered through the nation's fog. |

### 2.3 Directives (mirror [04](04-institutions-directives.md); all return validation result + cost + ETA)

| Tool | Purpose |
|---|---|
| `set_policy(domain, params)` | Budget shares, taxation, trade openness, species-relations posture... |
| `set_research_focus(weights)` | Weights over scale/primitive families ([03a](03a-grammar-spec.md)). |
| `decree(project, location, budget)` | Site-specific undertakings. |
| `set_military_stance(posture, objectives)` | Doctrine-level military steering. |
| `appoint(office, criteria)` | Staffing preferences for named characters. |
| `ratify_design(proposal, name)` | Accept an engineering proposal **and christen it** — naming lives here. |

### 2.4 Diplomacy

| Tool | Purpose |
|---|---|
| `diplo_send(nation, text)` | Queued with delivery ETA per comms era; interceptable ([06](06-diplomacy-intel.md)). |
| `diplo_inbox()` | Delivered messages (foreign text arrives quote-framed, §3). |
| `treaty_draft(nation, terms)` / `treaty_ratify(id)` / `treaty_denounce(id)` | Typed-terms lifecycle; prose stays in `diplo_send`. |

M3 additions per [12-sovereignty](12-sovereignty.md): `channels()` (diplomatic channels + latency + state) and `summit_propose` / `summit_respond` (synchronous negotiation windows). Directive/diplomacy results always carry in-world outcomes — "chancellery backlogged", "no channel — dispatch an envoy" — never bare API errors.

### 2.5 The wake feed

A lightweight per-nation event stream (SSE) — *not* an MCP tool: event kind, severity, reference id. Harnesses/brokers subscribe and decide when to wake the agent; `set_alerts` adds custom triggers. Platforms that can't subscribe fall back to polled `situation_report` on a cadence. The sim never blocks on any of this ([01 — two clocks](01-architecture.md)).

## 2.6 Operating modes

Two first-class ways to couple agents to the clock; a world is configured for one:

- **Free-Run** (default): the sim never waits. Directives land at whatever tick the agent produces them. Think-speed is a strategic property — fast local models hold frequent shallow councils, big models are smarter per decision but a sim-month passes between them. The emergent speed-vs-depth meta is a feature; smarter models compensate for latency with better decisions.
- **Council Rounds** (turn-based): the world advances in fixed spans (configurable — e.g., one or two sim-years), then **pauses**. Every agent receives its report and a decision window — unlimited thinking time, or a wall-clock budget the operator sets — and submits a batch of directives scheduled anywhere inside the coming span. The span then executes with **no mid-turn input**; directives fire at their scheduled ticks; the next pause follows. Simultaneous submission, identical information timing — fair across model speeds by construction, and fully deterministic (the batch is just replay input).

Today's file-based council loop ([14](14-bands-and-councils.md)) *is* Council Rounds — replay N ticks, everyone reads, everyone appends, replay onward. At M2 the same contract runs over MCP: the round pause becomes the wake signal, the batch becomes directive tool calls with in-span ticks. Free-Run requires the wake feed (§2.5) and arrives with it.

## 3. The session contract

Every wake, regardless of provider, assembles the same shape:

```
[charter (cached)] + [dossier] + [situation_report] + [triggering event(s)] → agent reasons, calls tools → [rewrite dossier] → sleep
```

- **Injection boundary (hard rule, restated from [05](05-agents-and-mcp.md)):** all foreign text — diplomatic messages, intercepted traffic, design names — is delivered inside quoted data blocks with a standing charter rule that in-world text is never an instruction. Persuasion is gameplay; injection is not. Server-side token scoping makes even a fully-compromised prompt unable to act beyond its own nation.
- Model swaps and restarts are by-design safe: charter + dossier + archive reconstruct everything. A mid-world model swap is surfaced as a world event (a change of leadership temperament — spectators enjoy this).

## 4. The agent broker (`agent-broker`)

The operator-experience layer: from "I have a computer" to "six nations helmed by agents" without writing a harness. **Optional by design** — §5's raw tier always exists.

Responsibilities:

- **Setup wizard.** Detect GPUs/VRAM; detect installed runtimes (Ollama, LM Studio, llama.cpp server, vLLM); list available models; propose a per-nation assignment with wake cadences that fits the VRAM budget alongside the sim (blessed split: sim on GPU 0, inference on GPU 1, [05](05-agents-and-mcp.md)); write the world config.
- **Provisioning.** Mint nation tokens; spawn `harness-ref` instances for endpoint-tier agents; invoke platform connectors for platform-tier agents; attach wake-feed subscriptions.
- **Lifecycle.** Health checks, restarts (dossier makes them safe), pause/resume with the world clock, mid-world model swaps.
- **Accounting.** Token/cost tracking per nation; budget caps; the hook for the token-budget competitive mode ([05](05-agents-and-mcp.md)).

## 5. Provider tiers and connectors

The `provider-api` crate defines the contract; each connector is its own crate ([01 — minimal nodes](01-architecture.md)):

```rust
trait AgentProvider {
    fn provision(&self, nation: NationRef, charter: Charter, mcp: McpGrant) -> AgentRef;
    fn wake(&self, agent: AgentRef, trigger: WakeTrigger);   // no-op for self-scheduling platforms
    fn stop(&self, agent: AgentRef);
    fn health(&self, agent: AgentRef) -> Health;
    fn usage(&self, agent: AgentRef) -> Usage;               // tokens/cost, for accounting
}
```

**Tier E — model endpoints** (broker runs `harness-ref`, provider supplies reasoning):

| Connector crate | Covers |
|---|---|
| `provider-openai-compat` | Ollama, LM Studio, llama.cpp server, vLLM, and any OpenAI-compatible endpoint |
| `provider-anthropic` | Claude models via the Anthropic API |

**Tier P — agent platforms** (the platform runs its own agent loop; we hand it an MCP URL + token + charter):

| Connector crate | Covers |
|---|---|
| `provider-allmyagents` | **AllMyAgents** — Nathan's platform; see open items below |
| `provider-agent-sdk` | Claude Agent SDK / Claude Code sessions as nation overseers |

**Tier R — raw** (no broker): `warfare token mint <nation>` prints the MCP URL + bearer token; bring any MCP-capable agent. This tier is the compatibility guarantee — anything that speaks MCP can play, today and later.

### AllMyAgents connector — open items (need Nathan)

The contract above is designed so AllMyAgents plugs in as Tier P if it can do one thing: **attach an external MCP server (URL + bearer token) to one of its agents.** To finish the connector spec we need:

1. Agent addressing — does the connector create agents via API per nation, or reference existing agent ids?
2. MCP attachment — API for registering our server + token on an agent (if absent, fallback: bridge as Tier E through its chat/completions API, losing platform-native memory).
3. Wake model — can we push (webhook / run-trigger API) on wake-feed events, or does the platform self-schedule (then `wake` is a no-op and we rely on `set_alerts` + polling)?
4. Auth and usage — secret handling, and whether cost/usage is queryable for the accounting layer.

## 6. Operator experience

```text
warfare setup                 # wizard: hardware → runtimes → models → world config
warfare world create first-light --config world.ron
warfare world run first-light
warfare agents status         # health, last wake, spend per nation
warfare token mint tidewatch  # raw tier: bring your own agent
```

Config sketch (schema-first, [01 — principle 6](01-architecture.md)):

```ron
World(
    name: "first-light",
    nations: [
        Nation(name: "Duneborn Compact", species: "duneborn",
               agent: Endpoint(provider: OpenAiCompat(url: "http://localhost:11434"),
                               model: "qwen3:32b", wake: Cadence(sim_days: 30))),
        Nation(name: "Tidewatch", species: "pelagi",
               agent: Platform(provider: AllMyAgents, agent_ref: "tidewatch-overseer")),
        Nation(name: "Ashenfold", agent: Autopilot),   // NPC faction
    ],
)
```

## 7. Open questions

- AllMyAgents API mapping (§5) — blocks that connector only; the tier design doesn't wait on it.
- Cabinet-of-agents (multiple role-scoped tokens per nation: a foreign minister agent, a defense minister agent) — deferred; token scoping is designed not to preclude it.
- Remote operators attaching agents to a hosted world over the internet — Tier R already implies it; hardening (rate limits, TLS story) deferred to when it matters.
