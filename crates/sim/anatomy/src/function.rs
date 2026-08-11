//! Everything a body can do, derived from its plan — never stored
//! (docs/23 §4). Movement from locomotors, senses from sensors, food value
//! from tissue, and the describe line that is every plan's doc-21 duty.

use crate::parts::{Role, emit_word, sense_word};
use crate::{BodyPlan, Substance};

/// How well this body moves over land and through water, 0..=1000 each,
/// summed from its locomotor sets.
#[must_use]
pub fn move_media(plan: &BodyPlan) -> (u16, u16) {
    let mut land = 0u32;
    let mut water = 0u32;
    for p in plan.parts.iter().filter(|p| p.role == Role::Locomotor) {
        let power = u32::from(p.size_milli) * u32::from(p.count) / 2;
        if p.medium_milli < 500 {
            land += power;
        } else {
            water += power;
        }
    }
    (land.min(1000) as u16, water.min(1000) as u16)
}

/// Food per unit of biomass taken, ×1000 — living stone is barely a meal.
#[must_use]
pub fn edible_milli(plan: &BodyPlan, palette: &[Substance], inedible_floor: u16) -> u16 {
    let tissue = &palette[plan.tissue as usize];
    if tissue.mineral_milli >= inedible_floor {
        return 60;
    }
    let loss = u32::from(tissue.mineral_milli) * 9 / 10;
    (1080u32.saturating_sub(loss)).max(120) as u16
}

/// The one legible line (docs/21): what it is, how it moves, how it
/// senses and speaks, and what runs in its veins.
#[must_use]
pub fn describe(plan: &BodyPlan, palette: &[Substance]) -> String {
    let tissue = &palette[plan.tissue as usize];
    let carrier = &palette[plan.carrier as usize];

    let mover = {
        let (land, water) = move_media(plan);
        let legs = plan
            .parts
            .iter()
            .find(|p| p.role == Role::Locomotor && p.medium_milli < 500)
            .map_or(0, |p| p.count);
        match (land > 0, water > 0) {
            (true, true) => format!("{legs}-limbed wader"),
            (true, false) => format!("{legs}-legged strider"),
            (false, true) => "finned swimmer".to_string(),
            (false, false) => "rooted thing".to_string(),
        }
    };

    let keenest = plan
        .parts
        .iter()
        .filter(|p| p.role == Role::Sensor)
        .max_by_key(|p| p.size_milli)
        .map_or("senseless", |p| sense_word(p.medium_milli));

    let voice = plan
        .parts
        .iter()
        .find(|p| p.role == Role::Emitter)
        .map(|p| format!("; speaks by {}", emit_word(p.medium_milli)));

    let shell = plan
        .parts
        .iter()
        .any(|p| p.role == Role::Shell)
        .then_some(", shelled");

    let spill = if carrier.heat_deci > 2000 || carrier.volatile_milli > 600 {
        " that burns where it spills"
    } else if carrier.viscosity_milli > 600 {
        " that clots fast"
    } else {
        ""
    };

    format!(
        "{mover} of {}{}; senses by {keenest}{}; {} for blood{spill}",
        tissue.tissue_word(),
        shell.unwrap_or(""),
        voice.unwrap_or_default(),
        carrier.describe(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{PlanContext, generate};
    use crate::substances;
    use sim_events::WorldSeed;
    use tuning::Bodies;

    #[test]
    fn every_plan_self_describes_and_stone_is_no_meal() {
        let seed = WorldSeed(7);
        let bod = Bodies::default();
        let palette = substances(seed, bod.substances);
        for k in 0..48u64 {
            let plan = generate(
                seed,
                k,
                PlanContext {
                    size_milli: 400,
                    water_milli: (k * 173 % 1000) as u16,
                    hunter_milli: (k * 311 % 1000) as u16,
                },
                &palette,
                &bod,
            );
            let line = describe(&plan, &palette);
            assert!(line.len() > 20, "a plan must say what it is: {line}");
            let (land, water) = move_media(&plan);
            assert!(land > 0 || water > 0, "everything moves somehow");
            let tissue = &palette[plan.tissue as usize];
            let food = edible_milli(&plan, &palette, bod.mineral_inedible_floor);
            if tissue.mineral_milli >= bod.mineral_inedible_floor {
                assert!(food < 100, "living stone is barely food");
            } else if tissue.mineral_milli < 200 {
                assert!(food > 800, "soft flesh feeds well");
            }
        }
    }
}
