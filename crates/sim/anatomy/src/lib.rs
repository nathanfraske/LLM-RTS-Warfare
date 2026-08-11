//! The anatomy grammar (docs/23-bodies-and-substances.md): a small authored
//! periodic table — part roles and substance property axes — from which
//! every creature's body is a generated composition. Nothing above this
//! floor is authored: "blood", "lava", and "rock beast" are descriptions of
//! regions in the space, never entries in a list.

pub mod function;
pub mod parts;
pub mod plan;

pub use parts::{Part, Role};
pub use plan::{BodyPlan, PlanContext};

use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_schema::Tick;

pub(crate) const BODYGEN: SystemId = SystemId(12);

/// A working fluid or building material: a terminal parameter bundle.
/// No chemistry lives below these axes (docs/23 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Substance {
    pub id: u16,
    /// Deci-degrees: frigid ichor … blood-warm … molten.
    pub heat_deci: i16,
    /// Thin spray … tar. Thick carriers clot fast but need pumping.
    pub viscosity_milli: u16,
    /// Inert … ignites or scalds where it spills.
    pub volatile_milli: u16,
    /// Organic … living stone.
    pub mineral_milli: u16,
}

impl Substance {
    /// One legible line — the doc-21 self-description duty.
    #[must_use]
    pub fn describe(&self) -> String {
        let heat = match self.heat_deci {
            i16::MIN..=-1 => "frigid",
            0..=200 => "cool",
            201..=450 => "warm",
            451..=2000 => "hot",
            _ => "molten",
        };
        let body = match self.viscosity_milli {
            0..=250 => "thin",
            251..=600 => "flowing",
            _ => "tarry",
        };
        let stone = match self.mineral_milli {
            0..=300 => "",
            301..=650 => "half-mineral ",
            _ => "mineral ",
        };
        format!("{heat} {body} {stone}ichor")
    }

    /// The word for flesh built from this.
    #[must_use]
    pub fn tissue_word(&self) -> &'static str {
        match self.mineral_milli {
            0..=250 => "flesh",
            251..=500 => "gristle",
            501..=750 => "half-stone flesh",
            _ => "living stone",
        }
    }
}

/// The world's substance palette: stratified so the space always admits
/// both the familiar (warm thin organic carriers) and the strange (molten
/// mineral ones) — the rock beast is reachable in every world.
#[must_use]
pub fn substances(seed: WorldSeed, count: u16) -> Vec<Substance> {
    (0..count)
        .map(|k| {
            let d = |salt: u64| rng::draw(seed, Tick::ZERO, BODYGEN, u64::from(k) << 8 | salt);
            let (heat, mineral) = match k % 5 {
                // The common strata: life-warm and organic.
                0 | 1 => ((250 + d(1) % 250) as i16, (d(2) % 250) as u16),
                2 => ((-150 + (d(1) % 350) as i16), (d(2) % 400) as u16),
                // The strange strata: stone-cold minerals and molten ones.
                3 => ((d(1) % 400) as i16, (500 + d(2) % 500) as u16),
                _ => ((4000 + d(1) % 9000) as i16, (600 + d(2) % 400) as u16),
            };
            Substance {
                id: k,
                heat_deci: heat,
                viscosity_milli: (80 + d(3) % 850) as u16,
                volatile_milli: if heat > 2000 {
                    (500 + d(4) % 500) as u16
                } else {
                    (d(4) % 350) as u16
                },
                mineral_milli: mineral,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_always_admits_the_strange() {
        for seed in [1u64, 7, 42, 1000] {
            let palette = substances(WorldSeed(seed), 10);
            assert!(
                palette
                    .iter()
                    .any(|s| s.heat_deci > 2000 && s.mineral_milli > 500),
                "every world can bleed something molten and mineral"
            );
            assert!(
                palette
                    .iter()
                    .any(|s| (201..=450).contains(&s.heat_deci) && s.mineral_milli < 300),
                "every world can bleed something like blood"
            );
            for s in &palette {
                assert!(!s.describe().is_empty());
            }
        }
    }
}
