//! The generated periodic table (docs/29 §1): mineral species as points on
//! terminal property axes. "Iron", "coal", and "limestone" are regions of
//! this space; a world holds what it rolled and nothing else.

use sim_events::WorldSeed;

use crate::draw;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mineral {
    pub id: u16,
    /// Talc … adamant.
    pub hardness_milli: u16,
    /// Earthy … native lustre (smeltability).
    pub metal_milli: u16,
    /// Inert … karst-former (water eats it).
    pub solubility_milli: u16,
    /// Dead stone … burns hot.
    pub energy_milli: u16,
}

impl Mineral {
    /// One legible line — hardness, character, and what it promises.
    #[must_use]
    pub fn describe(&self) -> String {
        let hard = match self.hardness_milli {
            0..=300 => "soft",
            301..=650 => "firm",
            _ => "hard",
        };
        let character = if self.metal_milli > 550 {
            "bright metal-stone"
        } else if self.energy_milli > 550 {
            "black rock that burns"
        } else if self.solubility_milli > 550 {
            "pale water-eaten stone"
        } else {
            "grey country stone"
        };
        format!("{hard} {character}")
    }
}

/// Stratified sampling so every world's table spans the space: hard barren
/// stones, soft soluble ones, burning ones, and at least a chance of real
/// metal — with the rolls deciding how generous this world came out.
pub(crate) fn generate(seed: WorldSeed, count: u16) -> Vec<Mineral> {
    (0..count)
        .map(|k| {
            let d = |salt: u64| draw(seed, u64::from(k) << 8 | salt);
            let (hardness, metal, solubility, energy) = match k % 4 {
                // Country stones: the bulk of any crust.
                0 => (300 + d(1) % 500, d(2) % 250, d(3) % 400, d(4) % 200),
                // Soluble soft rock: cave country, lime and salt analogues.
                1 => (100 + d(1) % 350, d(2) % 150, 500 + d(3) % 500, d(4) % 250),
                // The burning strata.
                2 => (100 + d(1) % 300, d(2) % 200, d(3) % 300, 450 + d(4) % 550),
                // The metal-bearing line.
                _ => (400 + d(1) % 600, 400 + d(2) % 600, d(3) % 250, d(4) % 150),
            };
            Mineral {
                id: k,
                hardness_milli: hardness as u16,
                metal_milli: metal as u16,
                solubility_milli: solubility as u16,
                energy_milli: energy as u16,
            }
        })
        .collect()
}
