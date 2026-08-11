//! The three monthly passes of the water cycle (docs/26 §2): evaporation
//! off the waters, wind-and-mix transport, and rain that banks as snow in
//! the cold, melts in the warm, and gates all growth. Integer field math
//! on the fixed grid — the doc-21 world floor holds.

use tuning::{Seasons, Weather};
use world_map::{Water, WorldFields};

use crate::{Climate, t_eff};

/// Warm water breathes moisture into the air above it; all air leaks a
/// little, which keeps the cycle bounded.
pub(crate) fn evaporate(
    climate: &mut Climate,
    fields: &WorldFields,
    month: u64,
    wx: &Weather,
    se: &Seasons,
) {
    for (tile, wet) in climate.wet.iter_mut().enumerate() {
        let mut air = u16::from(*wet);
        air -= air / 16; // the leak
        if fields.water[tile] != Water::Dry {
            let warm = i32::from(t_eff(fields, tile, month, se)).clamp(0, 300);
            air += u16::from(wx.evap_gain) * u16::try_from(warm).expect("clamped") / 150;
        }
        *wet = u8::try_from(air.min(255)).expect("clamped");
    }
}

/// Prevailing winds by latitude band — trades, westerlies, polar
/// easterlies, derived from row position, never authored per tile — plus
/// an isotropic mixing share.
pub(crate) fn transport(climate: &mut Climate, fields: &WorldFields, wx: &Weather) {
    let size = fields.size as usize;
    let old = climate.wet.clone();
    for y in 0..size {
        let dx = wind_dx(y, size);
        for x in 0..size {
            let tile = y * size + x;
            let mut air = u32::from(old[tile]) * (1000 - u32::from(wx.mix_permille)) / 1000;
            // Mix with the four neighbors.
            let mut mixed = 0u32;
            let mut count = 0u32;
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx < size && ny < size {
                    mixed += u32::from(old[ny * size + nx]);
                    count += 1;
                }
            }
            if let Some(neighborhood) = mixed.checked_div(count) {
                air += neighborhood * u32::from(wx.mix_permille) / 1000;
            }
            // The wind: take an extra share from the upwind neighbor.
            let upwind_x = x.wrapping_add_signed(-dx);
            if upwind_x < size {
                let from = u32::from(old[y * size + upwind_x]);
                air = air.saturating_sub(u32::from(old[tile]) * u32::from(wx.wind_permille) / 1000)
                    + from * u32::from(wx.wind_permille) / 1000;
            }
            climate.wet[tile] = u8::try_from(air.min(255)).expect("clamped");
        }
    }
}

/// Air sheds rain — more where the wind pushes it uphill. Below freezing
/// it banks as snow; warmth melts the bank back out; delivered water and
/// warmth together set the growth gate.
pub(crate) fn rain_snow_and_gate(
    climate: &mut Climate,
    fields: &WorldFields,
    month: u64,
    wx: &Weather,
    se: &Seasons,
) {
    let size = fields.size as usize;
    for tile in 0..climate.wet.len() {
        if fields.elevation[tile] < 0 {
            climate.snowpack[tile] = 0;
            climate.growth[tile] = 0;
            climate.delivered[tile] = 0;
            continue;
        }
        let (x, y) = (tile % size, tile / size);
        let dx = wind_dx(y, size);
        // Orographic lift: air about to climb rains out here.
        let downwind_x = x.wrapping_add_signed(dx);
        let rise = if downwind_x < size {
            (fields.elevation[y * size + downwind_x] - fields.elevation[tile]).max(0)
        } else {
            0
        };
        let rate = u32::from(wx.rain_permille)
            + (u32::try_from(rise).expect("non-negative") / 25) * u32::from(wx.orographic_permille);
        let rain = u32::from(climate.wet[tile]) * rate.min(900) / 1000;
        climate.wet[tile] =
            u8::try_from((u32::from(climate.wet[tile]) - rain).min(255)).expect("clamped");

        let t = t_eff(fields, tile, month, se);
        let mut delivered = 0u32;
        if t <= wx.freeze_deci {
            climate.snowpack[tile] = u16::try_from(
                (u32::from(climate.snowpack[tile]) + rain).min(u32::from(wx.snow_cap)),
            )
            .expect("capped");
        } else {
            delivered += rain;
            let warm = u32::try_from(i32::from(t) - i32::from(wx.freeze_deci)).expect("positive");
            let melt = (u32::from(wx.melt_per_deci) * warm).min(u32::from(climate.snowpack[tile]));
            climate.snowpack[tile] -= u16::try_from(melt).expect("bounded by pack");
            delivered += melt;
        }
        if fields.water[tile] != Water::Dry || world_map::tiles::riverine(fields, tile) {
            delivered += u32::from(wx.riverine_water);
        }

        climate.delivered[tile] = u16::try_from(delivered.min(60_000)).expect("capped");
        let span = i32::from(wx.growth_warm_deci) - i32::from(wx.growth_cold_deci);
        let warmth =
            ((i32::from(t) - i32::from(wx.growth_cold_deci)) * 1000 / span.max(1)).clamp(0, 1000);
        let water = (delivered * 1000 / u32::from(wx.water_full.max(1))).min(1000);
        climate.growth[tile] =
            u16::try_from(u32::try_from(warmth).expect("clamped") * water / 1000).expect("bounded");
    }
}

/// Wind direction for a row: trades blow one way, westerlies the other,
/// polar easterlies again — thirds of each hemisphere's latitude.
#[must_use]
pub fn wind_dx(y: usize, size: usize) -> isize {
    let half = size / 2;
    let from_equator = y.abs_diff(half) * 3 / half.max(1);
    if from_equator == 1 { 1 } else { -1 }
}
