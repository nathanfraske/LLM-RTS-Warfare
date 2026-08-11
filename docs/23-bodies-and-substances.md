# Bodies and Substances — The Anatomy Grammar

Dwarf-Fortress depth and past it: creatures and peoples with organs, limbs, senses, and working fluids that can drain away and kill them — **without authoring a single body plan**. A rock-fleshed beast with molten ichor for blood must be able to exist because the generator's space contains it, not because anyone wrote "RockBeast". This is the [invention grammar's](03a-grammar-spec.md) move applied to biology: author the periodic table of parts and substances; every anatomy is a generated molecule.

## 1. The floor moves down one level — by its own rule

[Doc 21](21-authored-floor.md) says the floor descends only when something watchable or governable cannot be expressed above it. Wounds are that something: "the spear took its pump and it bled its heat out on the ice" cannot be expressed at trait-axis level, and wound stories are half of what makes DF worlds worth telling. So the ecology floor moves from *trait axes* down to *part and substance primitives* — and stops. The guards that keep this from becoming the CMB project:

- **The periodic table stays small and authored**: ~10 part roles, a handful of substance property axes. Everything above — every anatomy, every monster — is generated composition.
- **No chemistry below substances.** A substance is a terminal parameter bundle (heat, viscosity, volatility, mineral fraction). *Why* lava burns is not simulated; *that* hot spills ignite is a property.
- **Cohorts stay authoritative.** Bodies are species-level state plus event-time sampling plus presence-time hydration ([17](17-presence.md) conservation rule). No per-individual organs at world scale, ever — a million-person world carries a few dozen body plans, not a million livers.
- **Every plan self-describes** in one line, or it's below the floor.

## 2. Substances — what flows and what it's made of

Generated per world (count in tuning), each a point in property space, never a named list: `heat` (frigid ichor … blood-warm … molten), `viscosity` (thin spray … tar), `volatility` (inert … ignites when spilled), `mineral` (organic … living stone). "Blood", "sap", and "lava" are *descriptions of regions* in this space, the way "grazer" describes trait space. Two substances matter per plan: the **carrier** (the working fluid whose loss is death) and the **tissue** (what the body is built from — flesh to stone; stone flesh is barely edible, which the food web then honors).

## 3. Parts — the authored periodic table

Roles, each with the interfaces it offers and needs: **Core** (seat of coordination — lose it, lose the creature), **Pump** (drives the carrier; plans whose carrier is thin and small may do without), **Conduit** (carries flow; breaches drain), **Processor** (turns food into life), **Reservoir** (stores against lean hours), **Sensor** (a *medium* axis — mechanical/vibration … chemical … radiant — plus range and acuity: eyes, whiskers, heat-pits, and stranger organs are all points on the same axes), **Emitter** (the sensor axes mirrored outward — voice, glow, scent: a people communicates however its emitters and its listeners' sensors overlap), **Locomotor** (a medium axis again — substrate legs, fluid fins, later air), **Manipulator** (grasping), **Shell** (integument, hide, plate). Parts carry size, symmetry pairing, and armor. That's the whole authored vocabulary.

## 4. Plans — generated wholes, derived function

A body plan is a generated graph: parts sampled and paired under construction guarantees (a core, a processor, sensing, a way to move; a pump whenever the carrier's mass and viscosity demand one), sized against the species' scale, constrained by its ecological traits (aquatic species roll fluid-medium locomotors; hunters roll far sensors). Everything a body *can do* is **derived, never stored**: movement modes from locomotor media, senses from sensors, communication channels from emitter–sensor overlap, food value from tissue, and the **vulnerability profile** from the graph itself — which functions degrade when a part is lost, how fast the carrier drains through a breach (viscosity), what a spilled volatile carrier does to the ground it lands on. Redundancy is real: a two-pump beast survives what kills a one-pump one, and nobody authored that — it's connectivity.

## 5. Wounds (B1+): damage is an address, not a number

A wound names a part, not a hit-point pool. Severity is derived: a breached conduit starts carrier loss; a lost locomotor halves a movement mode; a lost core ends the story. Healing is derived the same way — clotting from carrier viscosity, regrowth from tissue properties, treatment from what care is available (and later, medicine the [grammar](03a-grammar-spec.md) invents; weapons name the parts they seek and armor argues with shells). At cohort scale, combat and beast-danger resolve *statistically but against the real plans* — casualties are sampled from actual vulnerability profiles, so a stone-fleshed enemy genuinely blunts spears. At presence scale, the same wounds hydrate visibly: the limp is a lost locomotor, the red trail is the carrier, and both books balance ([17] P2 — presence earns, never invents).

## 6. Phases

- **B0 (now)**: `anatomy` crate — substance palettes, part roles, plan generation, derived function and describe lines; every fauna species gets a real body plan; edibility derives from tissue. Peoples get plans when they become generated genomes (next wave).
- **B1**: wounds and mortality-by-anatomy at event scale — hunting accidents, beast dangers, the first anatomically honest deaths; healing rates.
- **B2**: presence hydration — visible wounds, limps, spills; battle scenes resolve against plans (P3).
- **B3**: grammar coupling — weapons target parts, armor meets shells, medicine as invented technology.

**Deferred, slots ready:** growth and metamorphosis (plans changing over a life); part-level speciation drift; substance interactions beyond spill effects; hybrid vigor and chimeras.
