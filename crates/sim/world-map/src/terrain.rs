//! Display/summary terrain labels derived from the physical fields.
//! Labels never drive simulation — the fields do (docs/13-worldgen.md).

use crate::hydrology::Water;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Ocean,
    Lake,
    Mountain,
    Hills,
    Tundra,
    Desert,
    Plains,
}

#[must_use]
pub fn label(elevation: i32, water: Water, temperature_dc: i16, moisture: u8) -> Terrain {
    match water {
        Water::Ocean => Terrain::Ocean,
        Water::Lake => Terrain::Lake,
        Water::River | Water::Dry => {
            if elevation > 2_200 {
                Terrain::Mountain
            } else if temperature_dc < -60 {
                Terrain::Tundra
            } else if moisture < 58 && temperature_dc > 150 {
                Terrain::Desert
            } else if elevation > 1_100 {
                Terrain::Hills
            } else {
                Terrain::Plains
            }
        }
    }
}
