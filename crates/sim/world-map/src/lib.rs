//! The physical world map: field generation, hydrology, climate, and the
//! province partition (docs/13-worldgen.md — fields, not biomes).

pub mod climate;
pub mod fertility;
pub mod grid;
pub mod heightfield;
pub mod hydrology;
pub mod noise;
pub mod provinces;
pub mod terrain;

pub use grid::Grid;
pub use hydrology::Water;
pub use provinces::{NO_PROVINCE, Province};
pub use terrain::Terrain;

use sim_events::WorldSeed;

/// The generated physical fields, flat row-major structure-of-arrays layers.
#[derive(Debug)]
pub struct WorldFields {
    pub size: u32,
    /// Meters; `< 0` is below sea level.
    pub elevation: Vec<i32>,
    pub water: Vec<Water>,
    pub flow_acc: Vec<u32>,
    /// Deci-°C.
    pub temperature: Vec<i16>,
    /// 0–255.
    pub moisture: Vec<u8>,
    /// 0–255, climate-derived (flora enriches at province level).
    pub cell_fertility: Vec<u8>,
}

impl WorldFields {
    /// Run the full field pipeline: height → water → climate → fertility.
    #[must_use]
    pub fn generate(seed: WorldSeed, size: u32) -> Self {
        let grid = Grid { size };
        let elevation = heightfield::generate(seed, grid);
        let hydro = hydrology::compute(grid, &elevation);
        let temperature = climate::temperature(grid, &elevation);
        let moisture = climate::moisture(seed, grid, &hydro.water);
        let cell_fertility =
            fertility::cell_fertility(&elevation, &hydro.water, &temperature, &moisture);
        Self {
            size,
            elevation,
            water: hydro.water,
            flow_acc: hydro.flow_acc,
            temperature,
            moisture,
            cell_fertility,
        }
    }

    #[must_use]
    pub fn grid(&self) -> Grid {
        Grid { size: self.size }
    }

    #[must_use]
    pub fn land_cells(&self) -> u32 {
        self.elevation.iter().filter(|&&e| e >= 0).count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_is_deterministic() {
        let a = WorldFields::generate(WorldSeed(7), 96);
        let b = WorldFields::generate(WorldSeed(7), 96);
        assert_eq!(a.elevation, b.elevation);
        assert_eq!(a.water, b.water);
        assert_eq!(a.temperature, b.temperature);
        assert_eq!(a.moisture, b.moisture);
        let c = WorldFields::generate(WorldSeed(8), 96);
        assert_ne!(a.elevation, c.elevation);
    }

    #[test]
    fn provinces_cover_all_land_contiguously() {
        let fields = WorldFields::generate(WorldSeed(7), 96);
        let flora = vec![0u8; fields.grid().cells()];
        let (provinces, of_cell) = provinces::partition(WorldSeed(7), &fields, &flora, 24);
        assert_eq!(provinces.len(), 24);
        for (i, &assigned) in of_cell.iter().enumerate() {
            let is_land = fields.elevation[i] >= 0 && fields.water[i] != Water::Lake;
            assert_eq!(assigned != NO_PROVINCE, is_land, "cell {i}");
        }
        assert!(provinces.iter().any(|p| p.habitable));
    }
}
