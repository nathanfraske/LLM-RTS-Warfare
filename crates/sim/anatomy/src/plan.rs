//! Generating a whole body (docs/23 §4): parts sampled and paired under
//! construction guarantees, a carrier chosen to suit the tissue, everything
//! deterministic from `(seed, key)`. No plan is authored; every plan works.

use serde::{Deserialize, Serialize};
use sim_events::WorldSeed;
use sim_events::rng;
use tuning::Bodies;
use world_schema::Tick;

use crate::parts::{Part, Role};
use crate::{BODYGEN, Substance};

/// Ecological hints the generator honors: how big, how aquatic, how much
/// of a hunter. The plan realizes the genome's way of life in organs.
#[derive(Debug, Clone, Copy)]
pub struct PlanContext {
    pub size_milli: u16,
    pub water_milli: u16,
    pub hunter_milli: u16,
}

/// A generated anatomy: what the body is built from, what flows in it,
/// and the parts it is composed of. Function is derived in `function`,
/// never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyPlan {
    /// Substance id of the working fluid — its loss is death.
    pub carrier: u16,
    /// Substance id the body is built from.
    pub tissue: u16,
    pub size_milli: u16,
    pub parts: Vec<Part>,
}

/// Generate the plan for one species. Guarantees: a core, a processor, a
/// way to sense, a way to move; a pump wherever carrier and bulk demand
/// one; conduits wherever there are pumps.
#[must_use]
pub fn generate(
    seed: WorldSeed,
    key: u64,
    ctx: PlanContext,
    palette: &[Substance],
    bod: &Bodies,
) -> BodyPlan {
    let d = |salt: u64| rng::draw(seed, Tick::ZERO, BODYGEN, key << 16 | 0xB0D << 4 | salt);
    let pick = |salt: u64, n: usize| (d(salt) as usize) % n.max(1);

    // Tissue: mostly organic builds; a real minority of mineral ones.
    let organic: Vec<&Substance> = palette.iter().filter(|s| s.mineral_milli < 500).collect();
    let mineral: Vec<&Substance> = palette.iter().filter(|s| s.mineral_milli >= 500).collect();
    let tissue = if d(1) % 1000 < 130 && !mineral.is_empty() {
        mineral[pick(2, mineral.len())]
    } else if organic.is_empty() {
        &palette[pick(2, palette.len())]
    } else {
        organic[pick(2, organic.len())]
    };

    // Carrier: prefer a fluid whose mineral nature suits the tissue.
    let suited: Vec<&Substance> = palette
        .iter()
        .filter(|s| s.mineral_milli.abs_diff(tissue.mineral_milli) <= 400)
        .collect();
    let carrier = if suited.is_empty() {
        &palette[pick(3, palette.len())]
    } else {
        suited[pick(3, suited.len())]
    };

    let mut parts = Vec::new();
    vitals(&mut parts, &d, carrier, ctx, bod);
    faculties(&mut parts, &d, ctx);
    adornments(&mut parts, &d, tissue, bod);

    BodyPlan {
        carrier: carrier.id,
        tissue: tissue.id,
        size_milli: ctx.size_milli,
        parts,
    }
}

type Draw<'a> = &'a dyn Fn(u64) -> u64;

fn sized(d: Draw, base: u64, spread: u64, salt: u64) -> u16 {
    (base + d(salt) % spread) as u16
}

/// The guarantees: a core, a processor, pumping wherever carrier and bulk
/// demand it, conduits wherever there are pumps.
fn vitals(parts: &mut Vec<Part>, d: Draw, carrier: &Substance, ctx: PlanContext, bod: &Bodies) {
    parts.push(Part {
        role: Role::Core,
        medium_milli: 0,
        size_milli: sized(d, 120, 100, 4),
        count: 1,
        armor_milli: 0,
    });
    parts.push(Part {
        role: Role::Processor,
        medium_milli: 0,
        size_milli: sized(d, 150, 150, 5),
        count: 1,
        armor_milli: 0,
    });
    let needs_pump =
        carrier.viscosity_milli > bod.pump_viscosity_need || ctx.size_milli > bod.pump_size_need;
    if needs_pump {
        let pumps = if d(6) % 1000 < u64::from(bod.redundancy_permille) {
            2
        } else {
            1
        };
        parts.push(Part {
            role: Role::Pump,
            medium_milli: 0,
            size_milli: sized(d, 80, 90, 7),
            count: pumps,
            armor_milli: 0,
        });
    }
    parts.push(Part {
        role: Role::Conduit,
        medium_milli: 0,
        size_milli: sized(d, 60, 60, 8),
        count: if needs_pump { 2 } else { 1 },
        armor_milli: 0,
    });
}

/// Locomotion realizes the genome's habitat — substrate limbs, fluid fins,
/// or both for the shoreline-livers — and everyone senses; hunters far.
fn faculties(parts: &mut Vec<Part>, d: Draw, ctx: PlanContext) {
    let limb_pairs = 1 + (d(9) % 3) as u8; // 2, 4, or 6 of a kind
    if ctx.water_milli < 700 {
        parts.push(Part {
            role: Role::Locomotor,
            medium_milli: (d(10) % 300) as u16,
            size_milli: sized(d, 120, 140, 11),
            count: limb_pairs * 2,
            armor_milli: 0,
        });
    }
    if ctx.water_milli > 300 {
        parts.push(Part {
            role: Role::Locomotor,
            medium_milli: (700 + d(12) % 300) as u16,
            size_milli: sized(d, 120, 140, 13),
            count: 2 + 2 * (d(14) % 2) as u8,
            armor_milli: 0,
        });
    }
    parts.push(Part {
        role: Role::Sensor,
        medium_milli: (d(15) % 1000) as u16,
        size_milli: sized(d, 40, 70, 16),
        count: 2,
        armor_milli: 0,
    });
    if ctx.hunter_milli > 550 {
        parts.push(Part {
            role: Role::Sensor,
            medium_milli: (600 + d(17) % 400) as u16,
            size_milli: sized(d, 140, 120, 18),
            count: 2,
            armor_milli: 0,
        });
    }
}

/// The optional rolls: voice, storage, grasp, armor.
fn adornments(parts: &mut Vec<Part>, d: Draw, tissue: &Substance, bod: &Bodies) {
    let extras = d(19) % u64::from(bod.extra_parts_max + 1);
    if extras > 0 || d(20) % 1000 < 700 {
        parts.push(Part {
            role: Role::Emitter,
            medium_milli: (d(21) % 1000) as u16,
            size_milli: sized(d, 30, 60, 22),
            count: 1,
            armor_milli: 0,
        });
    }
    if extras > 1 {
        parts.push(Part {
            role: Role::Reservoir,
            medium_milli: 0,
            size_milli: sized(d, 90, 120, 23),
            count: 1,
            armor_milli: 0,
        });
    }
    if extras > 2 && d(24) % 1000 < 350 {
        parts.push(Part {
            role: Role::Manipulator,
            medium_milli: 0,
            size_milli: sized(d, 60, 80, 25),
            count: 2,
            armor_milli: 0,
        });
    }
    if d(26) % 1000 < u64::from(bod.shell_permille) {
        parts.push(Part {
            role: Role::Shell,
            medium_milli: 0,
            size_milli: sized(d, 150, 200, 27),
            count: 1,
            // Stone-fleshed builds wear heavier plate — same roll, honest scale.
            armor_milli: (200 + d(28) % 500) as u16 + tissue.mineral_milli / 3,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substances;

    fn any_ctx(k: u16) -> PlanContext {
        PlanContext {
            size_milli: 200 + k * 37 % 800,
            water_milli: (k * 211) % 1000,
            hunter_milli: (k * 97) % 1000,
        }
    }

    #[test]
    fn plans_are_deterministic_guaranteed_viable_and_varied() {
        let seed = WorldSeed(42);
        let bod = Bodies::default();
        let palette = substances(seed, bod.substances);
        let mut shelled = 0;
        let mut media = std::collections::BTreeSet::new();
        for k in 0..64u16 {
            let plan = generate(seed, u64::from(k), any_ctx(k), &palette, &bod);
            let again = generate(seed, u64::from(k), any_ctx(k), &palette, &bod);
            assert_eq!(plan, again, "same seed and key, same body");
            let has = |r: Role| plan.parts.iter().any(|p| p.role == r);
            assert!(has(Role::Core) && has(Role::Processor), "vitals guaranteed");
            assert!(has(Role::Sensor), "everything senses somehow");
            assert!(has(Role::Locomotor), "everything moves somehow");
            for p in plan.parts.iter().filter(|p| p.role == Role::Locomotor) {
                media.insert(p.medium_milli / 350);
            }
            if has(Role::Shell) {
                shelled += 1;
            }
        }
        assert!(shelled > 5, "some bodies armor themselves");
        assert!(media.len() > 1, "bodies move by more than one medium");
    }
}
