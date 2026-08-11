//! The living sky over the fields (docs/26-living-terrain.md W0): seasonal
//! forcing and the monthly water cycle — evaporation, wind-blown moisture,
//! orographic rain, snowpack, melt, and the growth gate everything green
//! reads. Field passes on the fixed grid, all integer, never a fluid
//! solver. Every rate lives in `tuning::{Seasons, Weather}`.

mod cycle;

pub use cycle::wind_dx;

use tuning::{Seasons, Weather};
use world_map::WorldFields;
use world_schema::Quantity;

/// The dynamic sky-and-snow state over a world.
#[derive(Debug)]
pub struct Climate {
    /// Airborne moisture per tile — the traveling half of the cycle.
    pub wet: Vec<u8>,
    /// Banked winter water per tile.
    pub snowpack: Vec<u16>,
    /// This month's growth gate per tile, 0..=1000 (warmth × water).
    pub growth: Vec<u16>,
    /// This month's delivered ground water per tile (rain + melt) — what
    /// the wash reads (docs/27) and discharge will read (docs/26 W1).
    pub delivered: Vec<u16>,
}

/// Seasonal temperature offset for a row, deci-°C: a latitude-scaled
/// swing on a twelve-month triangle wave. The poles feel it hardest.
#[must_use]
pub fn season_offset_deci(y: u32, size: u32, month: u64, se: &Seasons) -> i16 {
    // Triangle wave over the year: +1 at warm_month, -1 opposite.
    let phase = (month + 12 - u64::from(se.warm_month)) % 12;
    let swing_milli: i64 = match phase {
        0..=6 => 1000 - i64::try_from(phase).expect("small") * 333,
        _ => -1000 + (12 - i64::try_from(phase).expect("small")) * 333,
    };
    // Latitude 0 at equator (mid-row), 1000 at the poles (edges).
    let half = i64::from(size) / 2;
    let lat_milli = ((i64::from(y) - half).abs() * 1000 / half.max(1)).min(1000);
    let amplitude = i64::from(se.amplitude_equator_deci)
        + (i64::from(se.amplitude_polar_deci) - i64::from(se.amplitude_equator_deci)) * lat_milli
            / 1000;
    let flip = if se.southern_flip && i64::from(y) > half {
        -1
    } else {
        1
    };
    i16::try_from(amplitude * swing_milli * flip / 1000).unwrap_or(0)
}

/// Daylight fraction (0..=1000) for a row this month: axial tilt swings
/// day length with latitude on the same triangle wave as the seasons —
/// midnight sun and polar night emerge at high tilt × high latitude.
/// Piecewise-linear on purpose (docs/13: no transcendentals).
#[must_use]
pub fn daylight_milli(y: u32, size: u32, month: u64, se: &Seasons, tilt_deci: u16) -> u16 {
    let phase = (month + 12 - u64::from(se.warm_month)) % 12;
    let swing_milli: i64 = match phase {
        0..=6 => 1000 - i64::try_from(phase).expect("small") * 333,
        _ => -1000 + (12 - i64::try_from(phase).expect("small")) * 333,
    };
    let half = i64::from(size) / 2;
    let signed_lat = (half - i64::from(y)) * 1000 / half.max(1); // +north, -south
    let decl_milli = i64::from(tilt_deci) * 1000 / 900 * swing_milli / 1000;
    u16::try_from((500 + 2 * signed_lat * decl_milli / 1000).clamp(0, 1000)).expect("clamped")
}

/// Effective temperature of a tile this month, deci-°C.
#[must_use]
pub fn t_eff(fields: &WorldFields, tile: usize, month: u64, se: &Seasons) -> i16 {
    let y = (tile / fields.size as usize) as u32;
    fields.temperature[tile].saturating_add(season_offset_deci(y, fields.size, month, se))
}

impl Climate {
    /// Genesis: calm air seeded from the baseline, no snow, and the first
    /// month's gate computed.
    #[must_use]
    pub fn genesis(fields: &WorldFields, wx: &Weather, se: &Seasons) -> Self {
        let cells = fields.grid().cells();
        let mut climate = Self {
            wet: fields.moisture.clone(),
            snowpack: vec![0; cells],
            growth: vec![0; cells],
            delivered: vec![0; cells],
        };
        climate.tick_month(fields, 0, wx, se);
        climate
    }

    /// One month of the cycle (docs/26 §2): evaporate, blow, rain, bank,
    /// melt, and gate. Deterministic, integer, no randomness at all.
    pub fn tick_month(&mut self, fields: &WorldFields, month: u64, wx: &Weather, se: &Seasons) {
        cycle::evaporate(self, fields, month, wx, se);
        cycle::transport(self, fields, wx);
        cycle::rain_snow_and_gate(self, fields, month, wx, se);
    }

    /// Snow cover as a 0..=1 fraction of the cap — hunting and rendering
    /// both read this.
    #[must_use]
    pub fn snow_frac(&self, tile: usize, wx: &Weather) -> Quantity {
        Quantity::from_num(self.snowpack[tile]) / Quantity::from_num(wx.snow_cap.max(1))
    }

    /// The growth gate as a 0..=1 factor.
    #[must_use]
    pub fn growth_frac(&self, tile: usize) -> Quantity {
        Quantity::from_num(self.growth[tile]) / Quantity::from_num(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_map::WorldFields;

    #[test]
    fn the_year_turns_and_the_snow_banks_and_melts() {
        let fields = WorldFields::generate(sim_events::WorldSeed(77), 64);
        let wx = Weather::default();
        let se = Seasons::default();
        let mut climate = Climate::genesis(&fields, &wx, &se);

        // A mid-latitude land tile with real winters and real summers:
        // warm enough to live, cold enough to freeze.
        let probe = (4..30)
            .flat_map(|y| (0..64).map(move |x| y * 64 + x))
            .find(|&t| {
                fields.elevation[t] >= 0
                    && t_eff(&fields, t, 0, &se) <= wx.freeze_deci
                    && t_eff(&fields, t, u64::from(se.warm_month), &se) > 60
            })
            .expect("a land tile that freezes in winter and thaws in summer");
        let mut peak_snow = 0u16;
        let mut summer_snow = u16::MAX;
        let mut summer_growth = 0u16;
        let mut winter_growth = u16::MAX;
        for month in 1..=24u64 {
            climate.tick_month(&fields, month, &wx, &se);
            peak_snow = peak_snow.max(climate.snowpack[probe]);
            // The probe is northern: its summer is the warm month's phase.
            let phase = month % 12;
            if (5..=7).contains(&phase) {
                summer_snow = summer_snow.min(climate.snowpack[probe]);
                summer_growth = summer_growth.max(climate.growth[probe]);
            }
            if phase <= 1 || phase == 11 {
                winter_growth = winter_growth.min(climate.growth[probe]);
            }
        }
        assert!(peak_snow > 0, "a freezing winter must bank snow");
        assert!(
            summer_snow < peak_snow,
            "warmth must take snow back ({summer_snow} vs peak {peak_snow})"
        );
        assert!(
            summer_growth > winter_growth,
            "the gate must breathe with the year ({summer_growth} vs {winter_growth})"
        );

        // Determinism: the same months replayed give the same sky.
        let mut again = Climate::genesis(&fields, &wx, &se);
        for month in 1..=24u64 {
            again.tick_month(&fields, month, &wx, &se);
        }
        assert_eq!(climate.wet, again.wet);
        assert_eq!(climate.snowpack, again.snowpack);
        assert_eq!(climate.growth, again.growth);
    }
}
