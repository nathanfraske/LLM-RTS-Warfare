# Invention Grammar — Formal Specification (v1)

This is the authoritative specification of the grammar system; [03-invention-grammar](03-invention-grammar.md) is the concept overview, and this document wins where they differ. Everything here is designed to compile against `design-schema` when implementation starts; the historical test suite (§10) is the acceptance criterion for the registry.

## 1. Glossary and data model

| Term | Definition |
|---|---|
| **Knowledge scale** | A per-nation axis of understanding (metallurgy, chemistry, ...), tiered 0–5 with continuous progress within tiers. Scales gate primitives and set parameter quality. Scales are *state of a nation*, not parts. |
| **Primitive** | An engine-understood physical verb or mechanism, with sim semantics, an interface signature, gates, parameters, a glyph contribution, and a cost profile. The authored "periodic table" (§5). |
| **Interface** | A typed capability slot in the composition vocabulary (§4): what a part *provides* or *requires*. |
| **Part** | An instantiation of a primitive with concrete parameter values inside a design. |
| **Design** | A typed graph of parts connected by interface edges, plus optional ammo references and mounted designs. The unit of discovery, naming, production, and compilation. |
| **Compilation** | The pure, deterministic function `Design → (validation, capabilities, ECS bundle, sprite, cost)`. Runs once at ratification; runtime only reads outputs. |
| **Capability** | A derived, engine-meaningful property of a compiled design (Mobile(air), Guided, Producer(recipe), ...) (§7). |
| **Method** | A part-less process design (crop rotation, sanitation): pure scale-gated recipe/behavior modifier compiled to a process upgrade. |

Rust sketch for `design-schema` (illustrative, not final):

```rust
struct ScaleId(u8);           // 9 scales, §2
struct PrimitiveId(u16);      // registry index, §5
enum Interface { /* §4 */ }

struct Primitive {
    id: PrimitiveId,
    family: Family,
    provides: Vec<Provision>,          // interface + magnitude formula over params/scales
    requires: Vec<Requirement>,        // interface + arity (slots to fill)
    gates: Vec<(ScaleId, u8)>,         // min tier per scale
    params: Vec<ParamDef>,             // ranges widen with scale tiers
    glyph: GlyphSpec,
    cost: CostProfile,
}

struct Design {
    root: PartIdx,
    parts: Vec<Part>,                  // Part { primitive, params: SmallVec<Fixed> }
    edges: Vec<(PartIdx, ReqSlot, PartIdx)>,
    ammo: Vec<DesignId>,               // compatible projectile designs (§6.3)
    mounted: Vec<DesignId>,            // designs hosted in Mount slots (§6.2)
}
```

## 2. Knowledge scales (9)

`metallurgy` · `materials` (wood, textile, ceramic, composite) · `mechanics` · `chemistry` · `agronomy` · `medicine` · `optics` · `electrics` · `computation`

- **Growth:** research budget × agent focus weights × institution quality × species affinity ([04](04-institutions-directives.md), [08](08-species.md)). Tiers 0–5; progress within a tier is continuous and improves parameter quality of everything gated on that scale (a tier-3.8 metallurgy cannon outranges a tier-3.1 one — continuous improvement without new designs).
- **Diffusion:** scales leak through trade intensity, espionage, and conquest ([06](06-diplomacy-intel.md)). Designs can be captured and reverse-engineered only up to the captor's scale tiers.
- Scales are the *only* research state. There is no tech tree; there are scales, primitives gated by them, and designs composed from primitives.

## 3. Design targets

The root part determines the compilation target:

| Root | Target | Examples |
|---|---|---|
| `frame` / `hull` / `airframe` | **Device** — ECS archetype | weapons, vehicles, tools |
| `module_shell` | **Building module** ([07](07-buildings-and-cities.md)) | mill, forge, radar station, silo |
| *(no parts — Method)* | **Process upgrade** | crop rotation, assembly line, sanitation |

## 4. Interfaces (the composition vocabulary)

| Interface | Meaning | Provided by (examples) | Required by (examples) |
|---|---|---|---|
| `Mount(size)` | Physical attachment point | frame, hull, airframe, module_shell | nearly everything |
| `Structure(hp, mass)` | Load-bearing body | frames, armor | — (root property) |
| `Support(terrain)` | Ground contact for movement | wheel, track | land mobility (§7) |
| `Buoyancy` | Floats | hull | water mobility |
| `Lift` | Counters gravity | wing (needs Thrust on host), rotor | air mobility |
| `Thrust` | Accelerating force | sail, oar_bank, rocket_motor, propeller-coupled engines | air/water mobility, munitions |
| `Pull` | External traction | draft_harness | carts, towed guns |
| `Power` | Shaft/electric energy | muscle_crew, water_wheel, wind_mill, steam/combustion engine, electric_drive | rotor, wheel (self-propelled), machinery, compute, signal |
| `Launch(class)` | Imparts initial velocity to ammo | bow_stave, torsion_spring, counterweight, barrel(+propellant) | ranged weapons |
| `Projectile(class)` | Is launchable ammo | ammo designs (arrow, bolt, shot, shell) | Launch consumers |
| `Effect(kind)` | Happens on trigger: kinetic, explosive, incendiary | kinetic_head, energetic_compound (fast regime) | payload completeness (§6.4) |
| `Trigger` | Activation condition | impact/timer/proximity_fuze, command_link | any Effect |
| `Sensing(domain)` | Information acquisition: visual, air/sea | lens_optics, radar_array | Guided (§7), sensor platforms |
| `Control(domain)` | Steering authority | control_surfaces (air), steering_gear (land/sea) | Guided, piloted vehicles |
| `Compute` | Information processing | mech_computer, electronic_computer | Guided, radar, process boosts |
| `Signal` | Info at distance | telegraph_set (wired), radio_set | command_link, comms doctrine |
| `Containment(kind)` | Holds things: cargo, fuel, magazine, crew | frame/hull volumes, warhead_casing | payloads, transports, crewed vehicles |

## 5. Primitive registry v1 (~45)

Parameter quality always scales with the governing gate scales; params listed are the design-time choices. Tier notation: `met2` = metallurgy tier ≥ 2.

### 5.1 Structure

| Primitive | Provides | Gates | Params / notes |
|---|---|---|---|
| `frame` | Mounts×n, Structure, Containment(cargo/crew) opt | mat1 | size XS–XL; material grade (wood→iron→steel→composite via met/mat) |
| `hull` | Mounts, Structure, Buoyancy, Containment | mat2 | displacement; iron hulls at met4 |
| `airframe` | Mounts, Structure (low mass) | mat3 | expendable flag (munition airframes need no crew containment) |
| `module_shell` | module Mounts, Structure | mat1 | footprint tiles; the building-module root |
| `armor_plating` | Structure+ (armor rating) | met2 | thickness → rating & mass penalty |

### 5.2 Locomotion & control

| Primitive | Provides | Gates | Params / notes |
|---|---|---|---|
| `wheel` | Support(road/flat) | mech1 | count; speed on roads, poor off-road |
| `track` | Support(rough) | met3+mech3 | slower, all-terrain, heavy loads |
| `sail` | Thrust (wind-scaled) | mat2 | area; requires Buoyancy host |
| `oar_bank` | Thrust (crew-scaled) | mat1 | crew cost; inland/coastal |
| `rotor` | Lift+Thrust from Power | mech4 | helicopters emerge; that's fine |
| `wing` | Lift (needs host Thrust) | mech3+mat3 | span → lift; drag penalty |
| `control_surfaces` | Control(air) | mech3 | authority → turn rate |
| `steering_gear` | Control(land/sea) | mech2 | — |

### 5.3 Power & traction

| Primitive | Provides | Gates | Params / notes |
|---|---|---|---|
| `muscle_crew` | Power (low) | — | crew count; baseline everything |
| `draft_harness` | Pull | agr1 | consumes livestock (economy good) |
| `water_wheel` | Power | mech2 | module-only; placement constraint: river |
| `wind_mill` | Power (variable) | mech2 | module-only |
| `steam_engine` | Power | met3+mech3 | fuel consumption (economy hook); heavy |
| `combustion_engine` | Power (dense) | chem3+met3+mech3 | fuel: refined (chem) |
| `electric_drive` | Power ↔ electric | elec2 | enables grid modules; quiet, no smoke glyph |
| `battery` | Containment(energy) | elec2+chem2 | capacity; enables untethered electrics |

### 5.4 Energetics & weapons

| Primitive | Provides | Gates | Params / notes |
|---|---|---|---|
| `bow_stave` | Launch(S) | mat1 | muscle-drawn; crossbow via mech1 lock param |
| `torsion_spring` | Launch(M) | mech2 | ballista/onager class |
| `counterweight` | Launch(L, lobbed) | mech2 | trebuchet class; siege arcs |
| `energetic_compound` | *regime:* slow burn → Launch-boost / sustained Thrust feed; fast → Effect(explosive) | chem2 (slow), chem3 (fast/dense) | burn rate is the param; one primitive, propellant *and* explosive — context decides |
| `barrel` | Launch(class by caliber) w/ compound | met2 | caliber; rifling param at met3 → accuracy |
| `rocket_motor` | Thrust (sustained, expendable) | chem3+mat3 | burns energetic_compound (slow regime); no barrel needed |
| `kinetic_head` | Effect(kinetic) | met1 | form param: blade / point / shot / shell-body |
| `warhead_casing` | Containment(payload), Mount(Trigger) | met2 | pairs compound(fast) + fuze into a deliverable payload |

### 5.5 Triggers

| Primitive | Provides | Gates | Notes |
|---|---|---|---|
| `impact_fuze` | Trigger | mech2 | — |
| `timer_fuze` | Trigger | mech3 | delay param |
| `proximity_fuze` | Trigger | elec3 | vs air/naval |
| `command_link` | Trigger | elec3 | requires Signal path; enables demolitions, remote weapons |

### 5.6 Information

| Primitive | Provides | Gates | Notes |
|---|---|---|---|
| `lens_optics` | Sensing(visual) | opt2 | range ×; spyglasses, gunsights (accuracy bonus) |
| `telegraph_set` | Signal (wired) | elec1 | network via wire process; diplomacy latency era ([06](06-diplomacy-intel.md)) |
| `radio_set` | Signal | elec2 | untethered; doctrine coordination bonuses ([09](09-battles.md)) |
| `radar_array` | Sensing(air/sea) | elec3+comp1 | long range, weather-proof |
| `mech_computer` | Compute (low) | mech4 | fire control era |
| `electronic_computer` | Compute | elec3+comp2 | guidance, radar, process boosts |

### 5.7 Civil machinery (module parts)

| Primitive | Provides | Gates | Recipe |
|---|---|---|---|
| `millstones` | production w/ Power | mech1 | grain → flour |
| `forge_hearth` | production w/ fuel | met1 | ore → metal; upgraded by Methods |
| `loom` | production w/ Power | mech2 | fiber → cloth |
| `printing_mechanism` | production | mech3 | → print good; boosts scale diffusion & culture |
| `plow_rig` | *(device used by process)* | agr1+mech1 | farming output modifier |

### 5.8 Methods (process-only designs)

| Method | Gates | Effect |
|---|---|---|
| `crop_rotation` | agr2 | farm output; famine resistance |
| `sanitation` | med2 | urban health cap; plague resistance |
| `assembly_line` | mech4 | industry throughput; unit cost ↓ |
| `cipher_practice` | comp1 | interception resistance ([06](06-diplomacy-intel.md)) |

## 6. Composition and validation rules

1. **Slot filling.** Edges connect a part's `Requirement` slot to a part providing that interface, type- and size-checked. Unfilled *required* slots fail validation; unfilled optional Mounts are fine.
2. **Recursive mounting.** A `Mount` slot may host an entire **design** (not just a part): cannon designs on an ironclad's mounts, a guided munition in a launcher module. Mounted designs keep their own identity, ammo, and upgrade path. This rule is how turrets, silos, coastal batteries, and carrier decks all exist without being authored.
3. **Ammo references.** A `Launch(class)` provider names compatible `Projectile(class)` designs. Ammo is produced and stockpiled separately — an economy good with logistics weight ([09 — supply](09-battles.md)).
4. **Payload completeness.** Any `Effect` must reach a `Trigger` (direct or via casing); any `Trigger` must have an `Effect`. No half-armed payloads.
5. **Physics-lite checks** (fixed-point, at compile time): air mobility needs `(Lift+Thrust)/mass ≥ 1`; water needs `Buoyancy ≥ mass`; land speed = `f(Power-or-Pull / mass, Support type)`; armor adds rating and mass, slowing the host through the same formula. Cheap ratios, honest consequences.
6. **Guidance rule.** `Sensing + Compute + Control` on a self-propelled or falling host ⇒ Guided. On anything else, invalid (no guided pickaxes).
7. **Gate check.** At discovery time, every part's gates must be within the nation's scale tiers. Captured foreign designs compile only up to the captor's tiers (degraded params) — reverse-engineering pressure, not free tech.
8. **Regime splits.** Parameters can select qualitative regimes (energetic_compound burn rate: propellant vs. explosive). Regimes are validated like types.

## 7. The capability compiler

Compilation emits static stats + an ECS component bundle; runtime systems (in `military`, `economy`, `individuals`) implement behavior **per component, never per design**. The grammar adds zero per-design code — that's the modularity payoff ([01 — principles](01-architecture.md)).

| Capability | Condition | Emitted |
|---|---|---|
| Mobile(land) | Support + (Power \| Pull) | `Mobility{domain, speed, terrain profile}` |
| Mobile(water) | Buoyancy ≥ mass, Thrust | `Mobility{water}` |
| Mobile(air) | Lift+Thrust ratio ≥ 1 | `Mobility{air}` |
| RangedWeapon | Launch + ammo ref | `Weapon{range, rof, accuracy(+optics), ammo}` |
| MeleeWeapon | Effect(kinetic), handheld frame | `Weapon{melee}` |
| Munition | Thrust + payload, expendable airframe | `Munition{burn, payload}` |
| Guided | §6.6 satisfied | `Guidance{tracking quality ← sensing×compute×control}` |
| Armored | armor_plating present | `Armor{rating}`, mass → Mobility |
| Sensor | Sensing | `Sensor{domain, range}` |
| Comms | Signal | `Comms{latency class, range, wired?}` |
| Transport | Containment(cargo/crew) + Mobile | `Transport{capacity}` |
| Producer (module) | machinery + Power + recipe | `Production{recipe, rate}` |
| Launcher (module/mount) | Mount hosting Munition | `Launcher{reload, magazine}` |
| ProcessUpgrade | Method | recipe/behavior modifiers on existing modules |

## 8. Sprite derivation

Sprite = pure function of `(design graph, nation palette)`; deterministic, cached at compile.

- Base silhouette from root: frame aspect/size box, hull boat-form, airframe dart, module_shell tile block (buildings on tile multiples; devices 8/16px grid).
- Slot positions map to composition offsets: fore (barrel), aft (fins, wake), dorsal (rotor, antenna), lateral (wings, oars), underslung (wheels, tracks).
- Part glyphs: wheels = circles, tracks = ladder strip, sail = triangle, barrel = fore line, control_surfaces = aft chevrons, rotor = overhead cross, radar = fan arc, armor = heavier outline, sensor = stalk dot.
- Material tier tints the ramp (wood browns → iron greys → steel blues → composite whites); nation color on trim.
- Runtime FX bind to component state: Thrust active → flame/wake trail; combustion Power → smoke puffs; Launch firing → flash; explosive Effect → expanding ring.
- **Legibility override:** if the silhouette is ambiguous at 16px, exaggerate the highest-signal part (barrel length, wingspan). Readability beats proportion, always.

## 9. Cost derivation

- Per primitive: `{materials: class × qty, industry: class-hours, expertise: scale tiers}`. Design cost = size-scaled sum of parts; mounted designs and ammo cost separately.
- Production time = industry-hours ÷ assigned module capacity; upkeep = maintenance fraction + crew + fuel/ammo consumption.
- Balance is derived. Global tuning knobs are few and top-level (per-family cost multipliers), never per-design.

## 10. The historical test suite (registry acceptance)

Each row must validate, compile, and exhibit exactly the listed capabilities. If one can't be expressed, **fix the registry, not the test**.

| # | Design | Grammar expression | Key gates | Capabilities |
|---|---|---|---|---|
| 1 | Sword | frame(XS) + kinetic_head(blade) | met1 | MeleeWeapon |
| 2 | Plow | plow_rig | agr1+mech1 | ProcessUpgrade(farming) |
| 3 | Cart | frame(M) + wheel×2 + draft_harness | mech1 | Mobile(land), Transport |
| 4 | Bow (+arrow) | bow_stave; ammo: frame(XS)+kinetic_head(point) | mat1 | RangedWeapon(S) |
| 5 | Ballista | frame(M) + torsion_spring; ammo: bolt | mech2 | RangedWeapon(M) |
| 6 | Galley | hull + oar_bank + sail | mat2 | Mobile(water), Transport |
| 7 | Water mill | module_shell + water_wheel + millstones | mech2 | Producer(flour), site: river |
| 8 | Cannon | frame(M) + barrel(L) + compound(slow); ammo: shot | chem2+met3 | RangedWeapon(siege) |
| 9 | Printing press | module_shell + printing_mechanism + muscle_crew | mech3 | Producer(print), diffusion boost |
| 10 | Musket | frame(S) + barrel(S) + compound(slow); ammo: ball | chem2+met2 | RangedWeapon(infantry) |
| 11 | Ironclad | hull(iron) + steam_engine + armor + Mounts ← cannon designs | met4+mech3 | Mobile(water), Armored, weapon platform |
| 12 | Telegraph | telegraph_set + wire-network Method | elec1 | Comms(wired) — diplomacy latency drops |
| 13 | Rifle | musket + rifling param | met3 | RangedWeapon(accuracy↑) — *derivative, no new graph* |
| 14 | Field artillery | cannon + frame(carriage) + wheel×2 + lens_optics | met3+opt2 | RangedWeapon(indirect), towed |
| 15 | Tank | frame(L) + track + combustion_engine + armor + Mount ← cannon | met4+chem3+mech4 | Mobile(land), Armored, weapon platform |
| 16 | Aircraft | airframe + wing + combustion_engine + control_surfaces + Mounts | mech4+mat3 | Mobile(air) |
| 17 | Radar station | module_shell + radar_array + electronic_computer + Power | elec3+comp2 | Sensor(air/sea, long) |
| 18 | Rocket | airframe(expendable) + rocket_motor + compound(slow) | chem3+mat3 | Munition(unguided) |
| 19 | **Guided missile** | rocket + warhead_casing(compound-fast + impact_fuze) + radar/lens Sensing + electronic_computer + control_surfaces | chem3+elec3+comp2 | Munition, **Guided** — the canonical walkthrough |
| 20 | Computer hall | module_shell + electronic_computer×n + Power | elec3+comp3 | Producer(computation), scale growth boost |

## 11. Discovery mechanics

- **Proposal generation:** per-nation cadence on the counter-RNG. Candidate graphs sampled with bias = focus weights × situation pressure (at war → weapons; famine → agronomy; blockade → logistics; rival's observed capability → counters) × novelty bonus for newly-gated primitives. Candidates are validated (§6) and cost-estimated before proposal; nonsense dies silently, the agent only ever sees buildable proposals.
- **Ratify → name → prototype → production.** The agent christens the design ([03](03-invention-grammar.md)); a prototype field-trial event may adjust params (or embarrass everyone); production unlocks module tooling and stockpiles.
- **Derivative designs:** institutions autonomously propose param bumps and part swaps of existing designs (rifle from musket). Doctrine can auto-adopt derivatives under a budget cap so arsenals stay current without agent attention; NPC factions auto-adopt by autopilot heuristics.

## 12. Resolved decisions and open questions

**Resolved here:** scales are nation-state, separate from primitives · one energetic compound with burn-rate regimes covers propellant/explosive · recursive design mounting · ammo as separate economy-borne designs · guidance is composed (Sensing+Compute+Control), never a primitive · compilation is a pure one-shot function; runtime is per-component only.

**Open (owner: this doc, revisit at M1/M2):**
- Registry granularity trims after the first playtest — candidates for merging flagged in §5.
- Proposal "sensibleness" heuristics beyond situation pressure — may need a small curated combination-blocklist if sampling produces legal-but-silly designs too often.
- Naval depth (keels, broadside arcs) and logistics vehicles (fuel trucks) — likely v1.1 primitives; the interface vocabulary already covers them.
- Whether `medicine` earns device primitives (ambulance, field hospital module) or stays Method-only in v1.
