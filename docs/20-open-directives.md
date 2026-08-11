# Open Directives — No Authored Verb List

A hand-authored `Directive` enum (Name, SetStance, Settle, Commission, SetLabor…) caps the action space at whatever we anticipated. Same disease the [invention grammar](03a-grammar-spec.md) cures for technology and [tuning](01a-foundation.md) cures for numbers; this document commits the cure for *governance*. **Shipped 2026-08-11**: the enum is gone — `policy` schema crate, `Set`/`Enact` directives, per-nation policy trees, registry-rendered charters.

## The shape: a self-describing lever registry

Two generic directives replace the enum, operating over a registry that **sim systems populate**, not a schema we curate:

```json
{ "kind": "Set",   "key": "labor.hunt",            "value": 350 }
{ "kind": "Set",   "key": "expansion.posture",     "value": "expansive" }
{ "kind": "Enact", "action": "works.commission",   "target": 4468, "params": { "work": "farmstead" } }
{ "kind": "Enact", "action": "band.settle",        "target": 4469 }
```

- **Policy leaves** (`Set`): every per-nation behavior knob the autopilot reads — labor weights, expansion posture, future taxation, doctrine, diplomacy stances — is a leaf in a per-nation **policy tree**. Each sim crate *registers* its leaves: path, type, bounds, default, mandate cost, and a one-line description. Nation state stops growing bespoke fields; the autopilot reads the tree. A decreed leaf is **pinned**: the autopilot may drift only leaves no council order has touched — so a council can pin `labor.hunt` and leave the rest to the return-follower, per leaf, not all-or-nothing.
- **Actions** (`Enact`): targeted undertakings (commission a work, decree a settlement, later: raise a levy, dispatch an envoy). Registered the same way: name, target kind (nation / owned tile / frontier tile), param schemas, cost. The works catalog is registry data — `WorkKind` is no longer an enum; works are keys from `works::catalog`, and adding a work is a catalog line plus its effect.

## Why this solves it

1. **Depth without schema churn.** A new system ships its levers by registering them. `directive-schema` never changes again; old council logs stay replayable (unknown keys at replay = the same in-world rejection they got originally).
2. **The surface is self-describing.** Reports (and the M2 MCP `charter`/`directives` tools) *enumerate the live registry* — current values, bounds, costs, descriptions. Agents discover what they can do by reading, not by us re-teaching prompts. An overseer facing a new lever it has never seen is the intended experience.
3. **Validation and pricing stay server-side.** Registry entries carry bounds and validators; mandate ([16](16-mandate-and-works.md)) prices by cost class. Open surface, closed rules — nothing becomes free-form or nondeterministic.
4. **It composes toward the grammar.** When the invention grammar lands, buildable designs appear as actions automatically (`works.commission` params come from the design registry). Governance verbs can eventually be grammar-generated too — petition systems, offices, edicts as composed primitives.

## What stays closed on purpose

Natural language never enters the authoritative loop — an edict is a key path and a value, not prose to interpret (determinism, replay, fairness). Prose lives where it belongs: diplomacy between agents ([06](06-diplomacy-intel.md)) and the naming of things.

## How it landed (2026-08-11)

1. `policy` schema crate: `PolicyValue`/`PolicyType` (int range, choice, text), `PolicyDef`/`ActionDef`/`Registry`, and the per-nation `PolicyTree` with per-leaf directed pins. `directive-schema` is now just `Set`/`Enact` — the legacy variants were deleted outright (the council log was empty; nothing needed forwarding).
2. Systems register: `nations::registry` contributes `expansion.posture` + the three actions; `economy::policy_defs` contributes the five `labor.*` leaves. The composition root (`sim-server::registry::assemble`) concatenates registrations — a new system joins with one line.
3. Validation, pricing, and application all read the registry (`nations::directives`): unknown keys, out-of-bounds values, bad params, and wrong targets are in-world rejections that cost nothing. The only per-action code is a one-arm dispatch to the owning system.
4. Reports render the charter *from the registry* — levers with live values (decreed marks), bounds, costs, summaries; actions with target kinds and param schemas. The hand-written examples block is gone; `PolicySet` replaced the bespoke stance/labor events.

Still open, by design: cost *classes* (prices are per-entry numbers from tuning today); grammar-generated actions when buildable designs land ([03a](03a-grammar-spec.md)); a `release` form to un-pin a decreed leaf (restraint currently means never pinning it).
