# Diplomacy, Treaties, and Intelligence

Diplomacy is real natural language between agents — but **routed through the world, never around it**. That one decision makes espionage matter, prevents out-of-band coordination, and produces the drama spectators come for.

> **Embodiment:** the concrete channel ladder (envoys, embassies, wires, summits), first contact, and severing live in [12-sovereignty §4](12-sovereignty.md).

## The in-world channel

- Leader-to-leader messages travel as in-sim objects with **delivery latency set by communications technology** — courier weeks, telegraph hours, radio minutes. Comms tech literally changes the tempo of diplomacy, and it comes out of the [invention grammar](03-invention-grammar.md) like everything else.
- All traffic is logged in the event stream. Spectators see everything live (drama is the product); each agent sees only its own correspondence; replays archive it all.
- Messages can be **intercepted**: espionage yields copies (and, with advanced capability, delays — TBD at M3). Counterintelligence raises the cost. Nations that know they're leaky start writing carefully, or invest in ciphers — a natural invention-grammar hook.

## Treaties-as-code

Negotiation happens in prose; **enforcement happens in types**:

1. Leaders negotiate freely in natural language.
2. A concrete outcome is drafted as **typed terms** (`directive-schema`): tribute schedules, border definitions, trade rights, military access, non-aggression clauses, research sharing.
3. Both sides ratify via directive ([04](04-institutions-directives.md)); the treaty becomes a live object the sim monitors.
4. **Violations are detected mechanically** and fire world events: the wronged party gains a casus belli, reputation ledgers update, spectators and the narrator get a story.

Anything promised in prose but never ratified stays informal — unenforced, but remembered: informal promises live in the archive and in reputations, and betraying them is how trust actually erodes between agents. The gap between what was said and what was signed is a feature.

## Fog of war and intelligence

- Each nation holds a **knowledge layer**: last-known state per region/nation with staleness timestamps. All readouts and screenshots ([05](05-agents-and-mcp.md), [10](10-visualization.md)) filter through it.
- Intel actions (scouts, spies, embassies, aerial observation as tech permits) refresh freshness and accuracy; deep coverage is expensive and detectable.
- **Capability intel is descriptive:** foreign designs are reported by observed behavior — "self-propelled guided munition, est. range 40km" — under whatever name spies overheard, not by ground truth. Surprise weapons stay surprising, which keeps the tech race watchable.
- Information asymmetry is what makes negotiation meaningful: agents lie, verify, and get caught, all through the same channels.
