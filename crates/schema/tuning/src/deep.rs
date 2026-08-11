//! The underground (docs/29): mineral palette size, the geologic event
//! budget, cave and vent formation, and the fire's schedule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Deep {
    /// Mineral species generated per world.
    pub minerals: u16,
    /// Great uplifts read off the heightfield's peaks.
    pub uplifts: u8,
    /// Basins read off the interior lowlands.
    pub basins: u8,
    /// Metal-bearing intrusions drawn near uplifts and faults.
    pub intrusions: u8,
    /// Mantle plumes — the vents.
    pub plumes: u8,
    /// Vein halo radius around an intrusion, tiles.
    pub vein_radius: u16,
    /// Bedrock solubility above which water can open caves, milli.
    pub cave_solubility: u16,
    /// Moisture (0-255) needed for cave formation.
    pub cave_moisture: u8,
    /// Genesis warmth at a vent, deci-degrees, fading over its radius.
    pub geothermal_deci: i16,
    pub geothermal_radius: u16,
    /// Eruption period bounds, months.
    pub eruption_min_months: u16,
    pub eruption_max_months: u16,
    /// Lava run length at full vent strength, tiles.
    pub lava_reach: u16,
    /// Share of a settled tile's people a lava run takes, per mille.
    pub lava_cull_permille: u16,
    /// Ash fall radius at full vent strength, tiles (stretched downwind).
    pub ash_radius: u16,
    /// Fines the heaviest ash lays on a tile, composition points.
    pub ash_fines: u8,
    /// Flora smothered under the heaviest ash, density points.
    pub ash_smother: u8,
    /// Quake epicenters seeded on the faults.
    pub quakes: u8,
    /// Quake period bounds, months.
    pub quake_min_months: u16,
    pub quake_max_months: u16,
    /// Shake radius, tiles.
    pub quake_radius: u16,
    /// Share of a settled tile's people a shake takes, per mille.
    pub quake_cull_permille: u16,
    /// Coarse a shaken slope sheds downtree, points.
    pub quake_slide: u8,
}

impl Default for Deep {
    fn default() -> Self {
        Self {
            minerals: 12,
            uplifts: 5,
            basins: 4,
            intrusions: 10,
            plumes: 3,
            vein_radius: 4,
            cave_solubility: 620,
            cave_moisture: 90,
            geothermal_deci: 90,
            geothermal_radius: 3,
            eruption_min_months: 360,
            eruption_max_months: 1_800,
            lava_reach: 5,
            lava_cull_permille: 550,
            ash_radius: 6,
            ash_fines: 30,
            ash_smother: 200,
            quakes: 4,
            quake_min_months: 240,
            quake_max_months: 1_200,
            quake_radius: 4,
            quake_cull_permille: 60,
            quake_slide: 8,
        }
    }
}
