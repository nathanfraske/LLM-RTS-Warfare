# Sovereignty and Embodiment

How an agent is *allowed* to interact with its nation and with other nations — and what happens when its nation is conquered, subsumed, or destroyed. Complements [04](04-institutions-directives.md) (institutions), [05a](05a-agent-integration-spec.md) (surface), and [06](06-diplomacy-intel.md) (diplomacy).

## 1. The principle: power is simulated

The agent is not a floating admin console; it is the mind of a **government that exists in the world** — with a seat, communication lines, legitimacy, and enemies. MCP token scoping is only the outer wall (you can never act as another nation). The *living* constraint is in-world: every tool call is executed by the nation's actual apparatus, and everything the agent knows arrived through it. "What is the agent allowed to do?" always has a diegetic answer, never a permissions-table answer.

## 2. The Seat of Power

Each agent-led nation has a **Seat** — a capital administrative complex ([07](07-buildings-and-cities.md) admin modules) housing the court/council staffed by named characters ([04](04-institutions-directives.md) appointments). The Seat is where edicts originate and reports terminate. It is a physical object: it can be expanded, moved (expensive, slow), besieged, evacuated, captured.

## 3. Governing through the world

- **Readouts have internal fog.** Dashboards aggregate at the Seat over the nation's *internal* comms network: at courier tech, provincial figures arrive days or weeks stale; telegraph and radio tighten the loop. `dashboard`/`map_data` freshness reflects this — fog of war applies to your own nation. Comms technology is therefore a *domestic governance* upgrade, not just a diplomacy-latency upgrade ([06](06-diplomacy-intel.md)).
- **Directives are edicts.** Issued at the Seat, they propagate outward at comms speed and take effect province by province as they arrive. Distant provinces lag policy; severed provinces (cut comms, occupation) run on last-received policy under governor discretion until reconnected. Newly captured territory cannot be administered until connected.
- **Administrative capacity is the rate limit.** The court processes a bounded number of edicts per unit time, scaling with admin modules, staff quality, and computation tech. The MCP layer surfaces it diegetically: a directive can return *"the chancellery is backlogged — queued, ETA 3 weeks."* Agents that spam directives govern worse — by simulation, not by API throttle.
- **Legitimacy mediates compliance.** A simulated quantity fed by unrest, war exhaustion, species tension ([08](08-species.md)), prosperity, and victories. High legitimacy: institutions execute crisply. Low: sluggish compliance, provinces defying edicts, and ultimately coup or civil-war risk (§5). The agent can spend on legitimacy (ceremony, welfare, propaganda — policy domains) but cannot decree it.

## 4. Speaking to other nations

All leader-to-leader contact runs on in-world **channels** ([06](06-diplomacy-intel.md)); no channel, no conversation.

| Channel | Established by | Latency | Notes |
|---|---|---|---|
| Envoy | Dispatch (travel time, route risk) | Weeks–months | First contact; interceptable en route |
| Embassy | Mutual agreement; resident ambassador | Days–weeks | Persistent channel; expellable (severs it); espionage surface **both ways** |
| Wire/radio | Both ends have the tech + agreement | Hours–minutes | Interceptable; `cipher_practice` ([03a](03a-grammar-spec.md)) hardens |
| **Summit** | Agreed meeting (leaders travel — time and risk) | Live | Grants a synchronous multi-round agent-to-agent conversation window "at the table"; high stakes, high bandwidth |

First contact with an unknown nation requires an envoy expedition to somewhere its writ runs. Severing: expel ambassadors, cut wires, or simply let war close the routes — going dark is itself a diplomatic signal spectators can watch.

## 5. The sovereignty lifecycle

```
Sovereign ⇄ Vassal/Puppet → Occupied → Ended
     ↘        ↙        ↘
      Exile ─────────→ Restoration → Sovereign
```

- **Vassal/puppet** — created by treaty or imposed by conquest ([06](06-diplomacy-intel.md) typed terms). The agent *keeps governing*, but sovereignty clamps are mechanical: treaty terms restrict the directive space (foreign policy locked or proxied through the overlord, tribute auto-deducted, military caps, no embassies). The charter updates to list the clamps explicitly. Puppet play is scheming within bounds: secret rearmament, independence intrigue, petitioning the overlord — all through in-world means. The overlord gets levers: demands, garrisons, and a **resident** at the puppet's Seat (a standing intel feed, and a visible insult).
- **Occupation and ending** — losing all territory plus the Seat, with no escape, ends the nation. Ending is always a world event with narrative weight.
- **Government-in-exile** — if the court evacuates to a willing host before the Seat falls, the agent persists at drastically reduced capacity: intrigue-only existence — stoking revolt in the occupied homeland (the population keeps its identity and culture in the cohort layer, [02](02-simulation-core.md); occupation breeds unrest), diplomacy for restoration, funding partisans. Exile lasts while the host tolerates it.
- **Restoration** — reclaim and hold territory → re-found the Seat → resume sovereign play. The full arc (fall → exile → restoration) is deliberately supported because it is the best story the system can produce.
- **Coup / civil war / succession** — legitimacy collapse can replace the government. A coup hands the nation to autopilot or to a successor agent (operator policy). Civil war splits the nation; the agent retains the faction holding the Seat, the rest become NPC factions — or new agent slots.
- **Elimination policy (operator-configurable, world config):** an eliminated operator's agent goes to spectator, or into a **standby pool** — new nations that arise mid-world (revolts, colonies, civil-war splinters) are offered to the pool. Long-running worlds stay populated with players without resets.

## 6. Surface mapping ([05a](05a-agent-integration-spec.md))

No new subsystem — this doc binds existing surfaces: charter updates on every lifecycle transition (with explicit clamp lists for puppets); directives return in-world results ("province unreachable", "forbidden by Treaty of Reeds §3", "chancellery backlogged"); `diplo_send` returns channel + ETA or "no channel — dispatch an envoy"; wake-feed events for lifecycle transitions. Additions at M3: `channels()` (list channels + latency + state) and `summit_propose(nation, location)` / `summit_respond(id)`.

## 7. Open questions

- Exile expiry: indefinite-while-hosted vs. a decay clock — decide after watching one real fall.
- Standby-pool mechanics: how new-nation offers are sequenced among waiting operators.
- Whether an overlord agent can *delegate* directive subsets to a puppet agent (a token scope carve-out — the cabinet-of-agents machinery from [05a §7](05a-agent-integration-spec.md) would cover it).
