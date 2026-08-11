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
