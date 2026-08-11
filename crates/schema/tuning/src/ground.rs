//! The regolith's movements (docs/27): weathering, wash, roots, wind.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ground {
    /// Rock broken toward coarse (and coarse toward sand) per freeze-thaw
    /// month, composition points.
    pub frost_shatter: u8,
    /// Rock weathered toward sand per hot, dry, bare month.
    pub heat_crack: u8,
    /// Rock dissolved toward fines per wet month (chemical weathering —
    /// where the water is, the clay is made).
    pub wet_weather: u8,
    /// Fines carried downtree per month at full delivered water, points.
    pub wash: u8,
    /// Extra wash per 100 m of drop to the outflow neighbor, points.
    pub wash_steep: u8,
    /// Organic built per month under full vegetation, points.
    pub root_build: u8,
    /// Organic lost per hot, bare month, points.
    pub rot_loss: u8,
    /// Fines winnowed toward sand per dry, bare month, points.
    pub winnow: u8,
    /// Fertility weights per composition part, per mille of contribution.
    pub fert_organic_permille: u16,
    pub fert_fines_permille: u16,
    pub fert_sand_permille: u16,
    /// Vegetation density (0-255) counting as "bare" below this.
    pub bare_veg: u8,
    /// Delivered water counting as "dry" below this.
    pub dry_water: u16,
    /// Effective temperature counting as "hot", deci-degrees.
    pub hot_deci: i16,
}

impl Default for Ground {
    fn default() -> Self {
        Self {
            frost_shatter: 2,
            heat_crack: 1,
            wet_weather: 2,
            wash: 4,
            wash_steep: 3,
            root_build: 3,
            rot_loss: 2,
            winnow: 3,
            fert_organic_permille: 1000,
            fert_fines_permille: 900,
            fert_sand_permille: 80,
            bare_veg: 55,
            dry_water: 6,
            hot_deci: 220,
        }
    }
}
