//! The turning year and the water cycle (docs/24, docs/26): seasonal
//! forcing, evaporation, rain, snow, and the sky's clock. World
//! configuration, never constants.

use serde::{Deserialize, Serialize};

/// The shape of the year: a latitude-scaled temperature swing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Seasons {
    /// Seasonal swing at the poles, deci-°C (half peak-to-peak).
    pub amplitude_polar_deci: i16,
    /// Seasonal swing at the equator, deci-°C.
    pub amplitude_equator_deci: i16,
    /// Month (0-11) at which the north is warmest.
    pub warm_month: u8,
    /// Southern hemisphere runs opposite when true.
    pub southern_flip: bool,
}

impl Default for Seasons {
    fn default() -> Self {
        Self {
            amplitude_polar_deci: 160,
            amplitude_equator_deci: 20,
            warm_month: 6,
            southern_flip: true,
        }
    }
}

/// The water cycle's monthly pass (docs/26 §2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Weather {
    /// Airborne moisture a warm water tile pushes up per month.
    pub evap_gain: u8,
    /// Share of a tile's air that mixes with neighbors each month, per mille.
    pub mix_permille: u16,
    /// Extra share blown downwind (latitude-band winds), per mille.
    pub wind_permille: u16,
    /// Base share of airborne moisture that falls as rain, per mille.
    pub rain_permille: u16,
    /// Extra rain per point of windward rise (orographic lift), per mille.
    pub orographic_permille: u16,
    /// Effective temperature at or below which rain banks as snow, deci-°C.
    pub freeze_deci: i16,
    /// Snow-water melted per month per warm deci-degree above freezing.
    pub melt_per_deci: u16,
    /// Snowpack cap per tile (water units).
    pub snow_cap: u16,
    /// Growth gate: effective temperature of zero growth, deci-°C.
    pub growth_cold_deci: i16,
    /// Growth gate: effective temperature of full warmth, deci-°C.
    pub growth_warm_deci: i16,
    /// Delivered water (rain + melt) for a full growth gate.
    pub water_full: u16,
    /// Standing water's delivered-water bonus (rivers, lakes, coasts).
    pub riverine_water: u16,
    /// Hunt yield penalty at full snowpack, per mille.
    pub hunt_snow_penalty_permille: u16,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            evap_gain: 26,
            mix_permille: 240,
            wind_permille: 180,
            rain_permille: 210,
            orographic_permille: 5,
            freeze_deci: 0,
            melt_per_deci: 3,
            snow_cap: 900,
            growth_cold_deci: -20,
            growth_warm_deci: 160,
            water_full: 26,
            riverine_water: 14,
            hunt_snow_penalty_permille: 450,
        }
    }
}

/// Fire in the world (docs/26): what catches, how it runs, what stops it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Wildfire {
    /// Vegetation density below which nothing burns.
    pub fuel_min: u8,
    /// Density above which dry lightning finds tinder.
    pub tinder_fuel: u8,
    /// Delivered water below which the land counts as parched.
    pub dry_delivered: u16,
    /// Delivered water that puts a fire out.
    pub quench_delivered: u16,
    /// Fresh-catch intensity.
    pub ignite_intensity: u8,
    /// Fuel burned per burning month, density points.
    pub burn_rate: u8,
    /// Base monthly spread chance to a burnable neighbor, per mille
    /// (doubled by drought, doubled again downwind).
    pub spread_permille: u16,
    /// Dry-lightning strikes per parched tinder tile, per ten thousand.
    pub lightning_permyriad: u16,
    /// Share of a settled tile's people a fire month takes, per mille.
    pub fire_cull_permille: u16,
}

impl Default for Wildfire {
    fn default() -> Self {
        Self {
            fuel_min: 45,
            tinder_fuel: 120,
            dry_delivered: 8,
            quench_delivered: 30,
            ignite_intensity: 180,
            burn_rate: 90,
            spread_permille: 210,
            lightning_permyriad: 4,
            fire_cull_permille: 45,
        }
    }
}

/// The sky's presentation clock (docs/26 §4): the moon and the night.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sky {
    /// Days per lunar cycle.
    pub moon_period_days: u16,
    /// How dark a new-moon midnight gets, per mille of full shade.
    pub night_depth_permille: u16,
    /// Axial tilt, deci-degrees: sets how day length swings with latitude
    /// and season (0 = eternal equinox; ~234 = Earth-like; higher = wilder
    /// polar days and nights).
    pub axial_tilt_deci: u16,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            moon_period_days: 30,
            night_depth_permille: 620,
            axial_tilt_deci: 234,
        }
    }
}
