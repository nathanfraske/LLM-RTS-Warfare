//! The ground itself (docs/27-the-ground.md): every land tile's regolith as
//! a composition over the grain ladder — rock, coarse, sand, fines,
//! organic — summing to a constant column. No soil-type enum exists:
//! "loam", "scree", and "desert" are regions of this space, and every
//! process is a movement through it. Fertility is derived, never stored
//! as a verdict. `movements` runs the monthly passes.

mod movements;

use tuning::Ground;
use world_map::{WorldFields, tiles};

/// The column total every tile's composition sums to.
pub const COLUMN: u16 = 255;

/// The living skin of the world, one composition per tile.
#[derive(Debug)]
pub struct Regolith {
    pub rock: Vec<u8>,
    pub coarse: Vec<u8>,
    pub sand: Vec<u8>,
    pub fines: Vec<u8>,
    pub organic: Vec<u8>,
    fertility: Vec<u8>,
}

impl Regolith {
    /// Genesis: read the composition off the land as formed — steep high
    /// ground rocky, watered lowland fine, green ground organic, dry heat
    /// sandy. Pure derivation, no randomness.
    #[must_use]
    // Five parallel component arrays share the tile index; zipping them
    // would obscure, not clarify.
    #[allow(clippy::needless_range_loop)]
    pub fn genesis(fields: &WorldFields, flora_density: &[u8], g: &Ground) -> Self {
        let cells = fields.grid().cells();
        let mut ground = Self {
            rock: vec![0; cells],
            coarse: vec![0; cells],
            sand: vec![0; cells],
            fines: vec![0; cells],
            organic: vec![0; cells],
            fertility: vec![0; cells],
        };
        for tile in 0..cells {
            if fields.elevation[tile] < 0 {
                continue;
            }
            let slope = slope_of(fields, tile);
            let high = (fields.elevation[tile] - 1_500).max(0) / 12;
            let rock = (slope / 3 + high).clamp(8, 200) as u32;
            let coarse = (slope / 4).clamp(4, 120) as u32;
            let wet_flat = u32::from(tiles::riverine(fields, tile)) * 90
                + u32::from(fields.water[tile] != world_map::Water::Dry) * 40
                + u32::from(fields.moisture[tile]) / 3;
            let fines = (30 + wet_flat).saturating_sub((slope / 2) as u32);
            let dry = fields.moisture[tile] < 70;
            let hot = fields.temperature[tile] > 240;
            let sand = 20 + u32::from(dry) * 80 + u32::from(hot) * 30;
            let organic = u32::from(flora_density[tile]) * 35 / 100;
            ground.set_normalized(tile, [rock, coarse, sand, fines, organic]);
        }
        ground.refresh_fertility(fields, g);
        ground
    }

    /// Scale a raw composition onto the fixed column, exactly.
    fn set_normalized(&mut self, tile: usize, parts: [u32; 5]) {
        let total: u32 = parts.iter().sum::<u32>().max(1);
        let mut scaled = [0u8; 5];
        let mut used = 0u32;
        for (i, &p) in parts.iter().enumerate() {
            let s = p * u32::from(COLUMN) / total;
            scaled[i] = u8::try_from(s.min(255)).expect("bounded");
            used += s;
        }
        // Largest part absorbs the rounding remainder.
        let biggest = (0..5).max_by_key(|&i| parts[i]).expect("five parts");
        scaled[biggest] =
            u8::try_from((u32::from(scaled[biggest]) + (u32::from(COLUMN) - used)).min(255))
                .expect("bounded");
        self.rock[tile] = scaled[0];
        self.coarse[tile] = scaled[1];
        self.sand[tile] = scaled[2];
        self.fines[tile] = scaled[3];
        self.organic[tile] = scaled[4];
    }

    /// One month of movement through composition space (docs/27 §1).
    pub fn tick_month(
        &mut self,
        fields: &WorldFields,
        sky: &climate::Climate,
        flora_live: &[u8],
        month: u64,
        se: &tuning::Seasons,
        g: &Ground,
    ) {
        movements::weather_and_grow(self, fields, sky, flora_live, month, se, g);
        movements::wash(self, fields, sky, g);
        self.refresh_fertility(fields, g);
    }

    /// Live fertility, derived from what the ground is made of.
    #[must_use]
    pub fn fertility(&self, tile: usize) -> u8 {
        self.fertility[tile]
    }

    fn refresh_fertility(&mut self, fields: &WorldFields, g: &Ground) {
        for tile in 0..self.fertility.len() {
            if fields.elevation[tile] < 0 {
                self.fertility[tile] = 0;
                continue;
            }
            let f = u32::from(self.organic[tile]) * u32::from(g.fert_organic_permille) / 1000
                + u32::from(self.fines[tile]) * u32::from(g.fert_fines_permille) / 1000
                + u32::from(self.sand[tile]) * u32::from(g.fert_sand_permille) / 1000;
            self.fertility[tile] = u8::try_from(f.min(255)).expect("clamped");
        }
    }

    /// The one legible line (docs/21): what the ground here is.
    #[must_use]
    pub fn describe(&self, tile: usize) -> String {
        let parts = [
            (self.rock[tile], "bare rock"),
            (self.coarse[tile], "gravel and scree"),
            (self.sand[tile], "sand"),
            (self.fines[tile], "silt and clay"),
            (self.organic[tile], "living soil"),
        ];
        let mut sorted = parts;
        sorted.sort_by_key(|&(amount, _)| std::cmp::Reverse(amount));
        let character = if self.organic[tile] > 90 {
            "rich ground"
        } else if self.sand[tile] > 130 {
            "shifting country"
        } else if self.rock[tile] + self.coarse[tile] > 150 {
            "hard country"
        } else {
            "workable ground"
        };
        format!("mostly {}, some {} — {character}", sorted[0].1, sorted[1].1)
    }
}

/// Steepest rise or drop to a *land* neighbor — the sea floor's depth is
/// not a cliff face on the shore.
fn slope_of(fields: &WorldFields, tile: usize) -> i32 {
    let (neighbors, n) = fields.grid().neighbors8(tile);
    neighbors[..n]
        .iter()
        .filter(|&&nb| fields.elevation[nb] >= 0)
        .map(|&nb| (fields.elevation[tile] - fields.elevation[nb]).abs())
        .max()
        .unwrap_or(0)
}
