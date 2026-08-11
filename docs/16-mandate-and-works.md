# Mandate and Works — The Price of Direct Rule

Overseers can reach below policy and do things *directly* — commission a farmstead, decree a settlement site — but direct rule is priced, and the price compounds. This implements [12-sovereignty's](12-sovereignty.md) "power is simulated" principle as a spendable resource, and opens the control spectrum Nathan asked for: from pure oversight down to Songs-of-Syx-style project micromanagement, with the sim itself pushing back the further down you reach.

## Mandate

Each nation holds **mandate** — the people's readiness to be commanded (cap 10, regenerating ~1.2/month). Directives cost mandate:

| Directive | Base cost |
|---|---|
| `Name` | free (a gift, not an order) |
| `SetStance` | 1 |
| `Settle { tile }` | 2 |
| `Commission { tile, work }` | 3 |

Insufficient mandate → the directive is **rejected in-world** ("the council lacks the mandate: need 3.4, have 1.9"), logged, and visible in the chronicle like any other rejection.

## Autonomy — the compounding friction

Every paid intervention raises the nation's **autonomy** (+6, cap 100): people commanded often start deciding for themselves. Autonomy does two things:

- **Costs scale**: effective cost = base × (1 + autonomy/60). At autonomy 60, everything costs double.
- **Regen slows**: mandate regen × (1 − autonomy/200). At autonomy 100, half speed.

Autonomy decays ~5% per month of restraint. The loop is self-balancing: burst micromanagement gets expensive fast, forcing a return to policy-level rule while the people cool off. No hard forbiddance, no free lunch — a dial the overseer *feels*.

All of it is ordinary sim state mutated by logged directives and monthly ticks — fully deterministic and replayable.

## Works v1 (the first direct projects)

`Commission { tile, work }` starts a project on an owned tile; **institutions build it over months** (the overseer buys the decision, not the labor), completion is a world event, and finished works appear physically on the tile's local map ([15](15-multiscale-maps.md) — the first per-tile overlay content):

| Work | Build time | Effect | On the local map |
|---|---|---|---|
| `Farmstead` | 6 months | tile capacity ×1.35 | tilled field rows beside the camp |
| `Granary` | 8 months | famine threshold 1.15 → 1.40 | a stout store-house |
| `Dwellings` | 5 months | births ×1.12 | extra huts around the ring |

One work of each kind per tile. When the M1 economy lands, works become real production buildings ([07](07-buildings-and-cities.md)) and these effects are replaced by actual output — the commissioning/mandate layer is unchanged by that swap.

## Who spends the mandate: control modes

Mandate is agnostic about *which mind* spends it — that's operator configuration ([05a](05a-agent-integration-spec.md)):

- **Overseer-direct** — the main agent does everything, strategy and projects, budgeting its own mandate.
- **Steward-delegated** — a lightweight sub-agent (small local model) receives a mandate budget and standing priorities from the overseer and handles project-level work; the overseer stays at policy scale. Implemented today by whichever agent writes the `Commission` entries in the council log; at M2 this becomes a role-scoped token (the cabinet-of-agents carve-out in [05a §7](05a-agent-integration-spec.md)).
- **Hands-off** — nobody spends; the autopilot alone governs, and mandate just accrues as unused potential.

The interesting emergent tradeoff: a cheap steward can burn mandate on marginal projects and inflate autonomy, degrading the *overseer's* ability to intervene when it matters. Delegation is itself a strategic risk.

## Upgrade paths

- **Legitimacy** ([12](12-sovereignty.md)) folds in later: regen scales with legitimacy, and autonomy feeds unrest — over-commanded peoples become coup-prone.
- **Work catalogs** grow from the [invention grammar](03a-grammar-spec.md): commissionable works = designs the nation actually knows.
- **Chancellery capacity** ([12 §3](12-sovereignty.md)) remains the *throughput* limit (queue delay); mandate is the *consent* limit. Both are diegetic answers to "what may the agent do?"
