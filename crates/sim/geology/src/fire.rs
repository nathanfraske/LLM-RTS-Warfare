//! The fire below (docs/29 §3): each vent erupts on its own long clock,
//! and lava runs down the same drainage tree as everything else — a molten
//! mineral flood whose surface consequences the composition root applies.

use world_map::WorldFields;

use crate::Geology;

/// Vents due to erupt this month: (vent tile, strength).
#[must_use]
pub fn due(geology: &Geology, month: u64) -> Vec<(u32, u8)> {
    geology
        .schedules
        .iter()
        .filter(|(_, period, phase)| {
            let p = u64::from((*period).max(1));
            month % p == u64::from(*phase) % p && month > 0
        })
        .map(|&(tile, _, _)| (tile, geology.vents[tile as usize]))
        .collect()
}

/// Epicenters due to slip this month: (fault tile, radius-scale strength).
#[must_use]
pub fn due_quakes(geology: &Geology, month: u64) -> Vec<u32> {
    geology
        .quake_clocks
        .iter()
        .filter(|(_, period, phase)| {
            let p = u64::from((*period).max(1));
            month % p == u64::from(*phase) % p && month > 0
        })
        .map(|&(tile, _, _)| tile)
        .collect()
}

/// The ash footprint: every land tile within the fall, downwind-stretched,
/// with heaviness 0..=255 fading from the vent.
#[must_use]
pub fn ash_fall(fields: &WorldFields, vent: u32, strength: u8, radius: u16) -> Vec<(u32, u8)> {
    let size = fields.size as usize;
    let r = i64::from(radius) * i64::from(strength) / 255;
    if r == 0 {
        return Vec::new();
    }
    let (vx, vy) = fields.grid().xy(vent as usize);
    let dx_wind = climate::wind_dx(vy as usize, size);
    let mut out = Vec::new();
    for dy in -r..=r {
        for dx in -(r * 2)..=(r * 2) {
            // Downwind reach doubles; upwind halves.
            let windward = dx * dx_wind as i64;
            let stretch = if windward >= 0 { 2 } else { 1 };
            let dist = (dx.abs() / stretch).max(dy.abs());
            if dist > r {
                continue;
            }
            let x = i64::from(vx) + dx;
            let y = i64::from(vy) + dy;
            if x < 0 || y < 0 || x >= i64::from(fields.size) || y >= i64::from(fields.size) {
                continue;
            }
            let tile = (y as usize) * size + x as usize;
            if fields.elevation[tile] < 0 {
                continue;
            }
            let heaviness =
                u8::try_from(((r + 1 - dist) * 255 / (r + 1)).clamp(1, 255)).expect("bounded");
            out.push((tile as u32, heaviness));
        }
    }
    out
}

/// The lava's run: downtree from the vent, as far as its strength carries.
#[must_use]
pub fn lava_path(fields: &WorldFields, vent: u32, strength: u8, reach: u16) -> Vec<u32> {
    let len = (u32::from(reach) * u32::from(strength) / 255).max(1);
    let mut path = vec![vent];
    let mut at = vent;
    for _ in 0..len {
        let next = fields.drains_to[at as usize];
        if next == u32::MAX || fields.elevation[next as usize] < 0 {
            break;
        }
        path.push(next);
        at = next;
    }
    path
}
