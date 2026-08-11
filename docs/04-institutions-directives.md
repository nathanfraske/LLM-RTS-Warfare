# Institutions and Directives — The Autopilot Government

The bridge between the two clocks ([01](01-architecture.md)). Nations must run **competently with no LLM attached** — this is the project's make-or-break system and the first bet in the [risk register](00-vision.md). An agent that goes silent for a sim-month should return to a nation that governed itself sensibly in the interim.

## Structure

- **Ministries** — economy, defense, research, foreign affairs, interior. Each converts policy parameters plus current readouts into continuous decisions in its domain.
- **Provincial governors** — local allocation: which buildings to place under pressure signals, labor priorities, local emergency response.
- **Doctrine execution** — the military plans and fights campaigns within the stance and objectives set by directives; it never waits for unit orders.

Institutions are classical AI: utility systems, planners, flow optimization. No LLMs inside the sim boundary.

## The policy space

Policies are mostly continuous parameters, so directives compose and interpolate cleanly:

- Budget shares across ministries and provinces.
- Research **focus weights** over primitive families ([03](03-invention-grammar.md)).
- Military stance (posture, mobilization level, rules of engagement, named objectives).
- Trade openness, tariffs, embargo lists.
- Internal policy: taxation, conscription, settlement priorities, species-relations posture ([08](08-species.md)).

## Typed directives

Agents steer exclusively through **typed directives** (`directive-schema`) — never freeform commands, never unit orders:

- `SetPolicy { domain, parameters }`
- `SetResearchFocus { weights }`
- `Decree { project, location, budget }` — force a specific undertaking: a fortress at the river crossing, a road, a research campus.
- `SetMilitaryStance { posture, objectives }`
- `RatifyTreaty { treaty_id }` / `DenounceTreaty { treaty_id }` ([06](06-diplomacy-intel.md))
- `Appoint { office, criteria }` — staffing preferences for named characters.

Every directive is validated server-side (legality, affordability, scope — a nation acts only as itself), then enters the logged input stream: directives are exactly the replay input ([01](01-architecture.md)). Execution is additionally embodied: edicts propagate at comms speed, the chancellery has finite administrative capacity, and compliance is mediated by legitimacy — see [12-sovereignty §3](12-sovereignty.md).

## Interpretation

Institutions translate policy into thousands of micro-decisions per tick. The agent raises the housing budget; governors zone districts; construction gangs appear ([07](07-buildings-and-cities.md)). The agent sets an aggressive stance with a named objective; the defense ministry mobilizes, plans logistics, and fights ([09](09-battles.md)). The competence bar: **policy in, sensible visible activity out**, with no further attention required.

## NPC factions — the permanent proving ground

Minor nations and minor species run on the autopilot alone, always. They keep the world textured and reactive, and they continuously prove (or expose) autopilot quality. M1's exit criterion — a two-nation autopilot world that runs unattended for 24 hours and stays interesting to watch ([11](11-roadmap.md)) — is a test of *this* document.
