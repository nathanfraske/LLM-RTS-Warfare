# Open Directives — No Authored Verb List

The current `Directive` enum (Name, SetStance, Settle, Commission, SetLabor) is a hand-authored list of what an overseer may do — which caps the action space at whatever we anticipated. Same disease the [invention grammar](03a-grammar-spec.md) cures for technology and [tuning](01a-foundation.md) cures for numbers; this document commits the cure for *governance*. Design agreed 2026-08-11; implementation is its own work item.

## The shape: a self-describing lever registry

Two generic directives replace the enum, operating over a registry that **sim systems populate**, not a schema we curate:

```json
{ "kind": "Set",   "key": "labor.hunt",            "value": 350 }
{ "kind": "Set",   "key": "expansion.posture",     "value": "expansive" }
{ "kind": "Enact", "action": "works.commission",   "target": 4468, "params": { "work": "farmstead" } }
{ "kind": "Enact", "action": "band.settle",        "target": 4469 }
```

- **Policy leaves** (`Set`): every per-nation behavior knob the autopilot reads — labor weights, expansion posture, future taxation, doctrine, diplomacy stances — is a leaf in a per-nation **policy tree**. Each sim crate *registers* its leaves: path, type, bounds, default, mandate cost class, and a one-line description. Nation state stops growing bespoke fields; the autopilot reads the tree.
- **Actions** (`Enact`): targeted undertakings (commission a work, decree a settlement, later: raise a levy, dispatch an envoy). Registered the same way: name, target kind, params schema, validation hook, cost class. The works catalog becomes registry data — `WorkKind` stops being an enum the same day.

## Why this solves it

1. **Depth without schema churn.** A new system ships its levers by registering them. `directive-schema` never changes again; old council logs stay replayable (unknown keys at replay = the same in-world rejection they got originally).
2. **The surface is self-describing.** Reports (and the M2 MCP `charter`/`directives` tools) *enumerate the live registry* — current values, bounds, costs, descriptions. Agents discover what they can do by reading, not by us re-teaching prompts. An overseer facing a new lever it has never seen is the intended experience.
3. **Validation and pricing stay server-side.** Registry entries carry bounds and validators; mandate ([16](16-mandate-and-works.md)) prices by cost class. Open surface, closed rules — nothing becomes free-form or nondeterministic.
4. **It composes toward the grammar.** When the invention grammar lands, buildable designs appear as actions automatically (`works.commission` params come from the design registry). Governance verbs can eventually be grammar-generated too — petition systems, offices, edicts as composed primitives.

## What stays closed on purpose

Natural language never enters the authoritative loop — an edict is a key path and a value, not prose to interpret (determinism, replay, fairness). Prose lives where it belongs: diplomacy between agents ([06](06-diplomacy-intel.md)) and the naming of things.

## Migration plan (next implementation session)

1. `policy` schema crate: `PolicyKey`, typed values, registry types; `Set`/`Enact` in directive-schema alongside the legacy variants.
2. Nations gain a policy tree; stance and labor become leaves (legacy variants forward to them, then retire).
3. Works catalog → registry data; `Commission` becomes `Enact("works.commission")`.
4. Reports/charter render from the registry; the hand-written examples block dies.
