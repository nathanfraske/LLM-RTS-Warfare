//! Sentient species as parameter bundles (docs/08-species.md): climate
//! tolerances and demographic modifiers feeding the shared cohort systems.
//!
//! v1 archetypes are hand-authored constants; the RON data-file form arrives
//! with world config (schema-first rule, docs/01-architecture.md §6).

use serde::{Deserialize, Serialize};
use world_map::Province;
use world_schema::{Quantity, SpeciesId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
    pub id: SpeciesId,
    pub name: &'static str,
    /// Climate optimum/tolerance: temperature in deci-°C, moisture 0–255.
    pub t_opt: i16,
    pub t_width: i16,
    pub m_opt: u8,
    pub m_width: u8,
    /// Demographic modifiers ×1000.
    pub birth_mod_milli: u16,
    pub death_mod_milli: u16,
    /// Expansion drive ×1000 — scales the autopilot's willingness to split.
    pub drive_milli: u16,
}

/// The four v1 archetypes, indexed by `SpeciesId`.
#[must_use]
pub fn archetypes() -> &'static [Species] {
    const ARCHETYPES: [Species; 4] = [
        Species {
            id: SpeciesId(0),
            name: "Duneborn",
            t_opt: 260,
            t_width: 150,
            m_opt: 70,
            m_width: 90,
            birth_mod_milli: 1050,
            death_mod_milli: 1000,
            drive_milli: 1150,
        },
        Species {
            id: SpeciesId(1),
            name: "Rivermarsh",
            t_opt: 210,
            t_width: 120,
            m_opt: 210,
            m_width: 80,
            birth_mod_milli: 1100,
            death_mod_milli: 1050,
            drive_milli: 950,
        },
        Species {
            id: SpeciesId(2),
            name: "Northkin",
            t_opt: 40,
            t_width: 160,
            m_opt: 140,
            m_width: 110,
            birth_mod_milli: 950,
            death_mod_milli: 900,
            drive_milli: 1000,
        },
        Species {
            id: SpeciesId(3),
            name: "Valewrought",
            t_opt: 180,
            t_width: 140,
            m_opt: 150,
            m_width: 100,
            birth_mod_milli: 1000,
            death_mod_milli: 950,
            drive_milli: 1050,
        },
    ];
    &ARCHETYPES
}

/// A ×1000 modifier as an exact fixed-point multiplier.
#[must_use]
pub fn milli(m: u16) -> Quantity {
    Quantity::from_num(m) / Quantity::from_num(1000)
}

/// Integer-exact quadratic tolerance bump in `[0, 1]` (fixed-point,
/// sim-path safe — the f32 twin in `world_map::fertility` is genesis-only).
#[must_use]
pub fn bump_q(value: i32, opt: i32, width: i32) -> Quantity {
    let d = value - opt;
    let w2 = width * width;
    let num = (w2 - d * d).max(0);
    Quantity::from_num(num) / Quantity::from_num(w2)
}

/// How well `species` lives in `province`: climate fit in `[0, 1]`.
#[must_use]
pub fn province_fitness(species: &Species, province: &Province) -> Quantity {
    let t = bump_q(
        i32::from(province.mean_temperature),
        i32::from(species.t_opt),
        i32::from(species.t_width),
    );
    let m = bump_q(
        i32::from(province.mean_moisture),
        i32::from(species.m_opt),
        i32::from(species.m_width),
    );
    t * m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_is_exact_and_bounded() {
        assert_eq!(bump_q(100, 100, 50), Quantity::ONE);
        assert_eq!(bump_q(150, 100, 50), Quantity::ZERO);
        assert_eq!(bump_q(200, 100, 50), Quantity::ZERO);
        let half = bump_q(125, 100, 50);
        assert!(half > Quantity::ZERO && half < Quantity::ONE);
    }
}
