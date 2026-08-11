//! The wild world: flora regrowth, fauna growth, trophic pressure.

use serde::{Deserialize, Serialize};

/// The wild world: flora regrowth, fauna growth, trophic pressure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ecology {
    /// Grazer carrying capacity on a fully vegetated, perfectly fit tile.
    pub grazer_k_full: f64,
    /// Predator habitat ceiling at perfect fit.
    pub predator_habitat_k: f64,
    /// Predator K as a fraction of local prey biomass.
    pub predator_k_prey_frac: f64,
    /// Monthly predator food demand as a fraction of predator biomass.
    pub predator_demand_frac: f64,
    /// At most this share of prey can be eaten in one month.
    pub predation_max_frac: f64,
    /// Aquatic K by water body at perfect fit.
    pub aquatic_k_river: f64,
    pub aquatic_k_lake: f64,
    pub aquatic_k_ocean: f64,
    /// Die-back keep-fraction when habitat has collapsed.
    pub collapse_keep: f64,
    /// Vegetation points a full grazer load wears off per month (cap 8).
    pub grazing_pressure: f64,
    /// Vegetation never wears below this.
    pub flora_floor: u8,
    /// Share of an overcrowded stock that migrates to a better neighbor.
    pub diffusion_frac: f64,
    /// No harvest strips a wild stock below this refuge share in a month.
    pub refuge_frac: f64,
    /// Fraction of K that populations seed at genesis.
    pub genesis_fill: f64,
    /// Monthly regrowth: gap-to-baseline divided by this (min 1 point).
    pub regrow_divisor: u8,
    /// How many genomes each kingdom generates per world.
    pub fauna_species: u16,
    pub flora_species: u16,
}

impl Default for Ecology {
    fn default() -> Self {
        Self {
            grazer_k_full: 380.0,
            predator_habitat_k: 60.0,
            predator_k_prey_frac: 0.125,
            predator_demand_frac: 0.333,
            predation_max_frac: 0.2,
            aquatic_k_river: 240.0,
            aquatic_k_lake: 320.0,
            aquatic_k_ocean: 180.0,
            collapse_keep: 0.7,
            grazing_pressure: 3.0,
            flora_floor: 10,
            diffusion_frac: 0.0625,
            refuge_frac: 0.25,
            genesis_fill: 0.5,
            regrow_divisor: 10,
            fauna_species: 12,
            flora_species: 24,
        }
    }
}
