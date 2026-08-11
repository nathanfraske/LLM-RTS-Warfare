//! Animal genomes: continuous trait axes (docs/19) realized as generated
//! anatomies (docs/23). A genome says where a species lives and what it
//! eats; its body plan says what it is made of, how it moves, senses,
//! speaks, and what runs in its veins — all derived, none authored.

use anatomy::{BodyPlan, PlanContext, Substance, function};
use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use tuning::Bodies;
use world_schema::{Quantity, Tick};

const FAUNAGEN: SystemId = SystemId(10);

/// An animal genome: climate tolerances, trait axes, and the body built
/// from them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaunaSpecies {
    pub id: u16,
    /// 0 = pure plant-eater … 1000 = pure flesh-eater.
    pub diet_milli: u16,
    /// 0 = fully terrestrial … 1000 = fully aquatic.
    pub water_milli: u16,
    pub t_opt: i16,
    pub t_width: i16,
    pub m_opt: u8,
    pub m_width: u8,
    /// Monthly intrinsic growth ×1000 (falls out of diet at generation:
    /// plant-eaters breed fast, flesh-eaters slow).
    pub repro_milli: u16,
    /// Food yielded per unit of biomass taken, ×1000 — derived from what
    /// the body is built of (living stone is barely a meal).
    pub edible_milli: u16,
    /// The generated anatomy (docs/23).
    pub plan: BodyPlan,
}

impl FaunaSpecies {
    #[must_use]
    pub fn plant_frac(&self) -> Quantity {
        Quantity::ONE - Quantity::from_num(self.diet_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn flesh_frac(&self) -> Quantity {
        Quantity::from_num(self.diet_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn land_frac(&self) -> Quantity {
        Quantity::ONE - Quantity::from_num(self.water_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn water_frac(&self) -> Quantity {
        Quantity::from_num(self.water_milli) / Quantity::from_num(1000)
    }

    /// Where this genome sits in trait space, and the body that lives
    /// there — one legible line (docs/21).
    #[must_use]
    pub fn describe(&self, palette: &[Substance]) -> String {
        let diet = match self.diet_milli {
            0..=300 => "plant-eater",
            301..=650 => "omnivore",
            _ => "flesh-eater",
        };
        let habitat = match self.water_milli {
            0..=250 => "land",
            251..=650 => "shoreline",
            _ => "water",
        };
        format!(
            "{habitat} {diet}: {}",
            function::describe(&self.plan, palette)
        )
    }
}

/// Genomes sampled across trait space and climate space — every band of
/// diet × habitat gets contenders, none is guaranteed to thrive anywhere.
/// Each genome's anatomy is generated from the same seed chain; its
/// edibility derives from that body's tissue.
#[must_use]
pub fn generate_species(
    seed: WorldSeed,
    count: u16,
    palette: &[Substance],
    bod: &Bodies,
) -> Vec<FaunaSpecies> {
    const T_BINS: [i16; 4] = [-60, 60, 170, 270];
    (0..count)
        .map(|k| {
            let d = |salt: u64| rng::draw(seed, Tick::ZERO, FAUNAGEN, u64::from(k) << 8 | salt);
            // Stratify trait space: plant-eaters common, omnivores and
            // flesh-eaters rarer; a water band and a shoreline band exist.
            let diet_milli = match k % 6 {
                0 | 1 | 3 => (d(1) % 300) as u16,
                4 => (350 + d(1) % 300) as u16,
                _ => (650 + d(1) % 350) as u16,
            };
            let water_milli = match k % 12 {
                8..=11 => (750 + d(2) % 250) as u16,
                7 => (350 + d(2) % 300) as u16,
                _ => (d(2) % 250) as u16,
            };
            let t_center = T_BINS[k as usize % T_BINS.len()];
            // Breeding speed falls out of diet: grass breeds mice, meat breeds wolves.
            let repro_milli = (230 - u64::from(diet_milli) * 14 / 100 + d(5) % 60) as u16;
            // Slow breeders are the big beasts; the plan is sized to match.
            let plan = anatomy::plan::generate(
                seed,
                0xFA00 | u64::from(k),
                PlanContext {
                    size_milli: (1000u64.saturating_sub(u64::from(repro_milli) * 3) as u16)
                        .clamp(120, 950),
                    water_milli,
                    hunter_milli: diet_milli,
                },
                palette,
                bod,
            );
            let edible_milli = function::edible_milli(&plan, palette, bod.mineral_inedible_floor);
            FaunaSpecies {
                id: k,
                diet_milli,
                water_milli,
                t_opt: t_center + (d(3) % 80) as i16 - 40,
                t_width: 90 + (d(4) % 110) as i16,
                m_opt: (40 + (d(6) % 180)) as u8,
                m_width: (70 + (d(7) % 90)) as u8,
                repro_milli,
                edible_milli,
                plan,
            }
        })
        .collect()
}
