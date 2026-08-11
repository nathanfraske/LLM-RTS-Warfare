//! The anatomy grammar: substance palettes and plan budgets (docs/23).

use serde::{Deserialize, Serialize};

/// The anatomy grammar (docs/23): how many substances a world's palette
/// holds and how generated body plans are budgeted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bodies {
    /// Substances generated per world palette.
    pub substances: u16,
    /// Extra parts rolled beyond the construction guarantees.
    pub extra_parts_max: u16,
    /// A carrier thicker than this needs a pump to move it.
    pub pump_viscosity_need: u16,
    /// A body larger than this needs a pump regardless of carrier.
    pub pump_size_need: u16,
    /// Chance per plan of an integument shell, per mille.
    pub shell_permille: u16,
    /// Chance of redundant vitals (second pump or core), per mille.
    pub redundancy_permille: u16,
    /// Tissue mineral fraction at which flesh stops being food.
    pub mineral_inedible_floor: u16,
}

impl Default for Bodies {
    fn default() -> Self {
        Self {
            substances: 10,
            extra_parts_max: 6,
            pump_viscosity_need: 260,
            pump_size_need: 420,
            shell_permille: 380,
            redundancy_permille: 120,
            mineral_inedible_floor: 850,
        }
    }
}
