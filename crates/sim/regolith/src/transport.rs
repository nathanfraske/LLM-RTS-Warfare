//! Cross-column transport (docs/27 §1): water washes fines down the
//! drainage tree, oversteep scree slides, and every arrival settles into
//! the receiving column by the burial rules — silt spares the living
//! soil, rubble and sand do not.

use tuning::Ground;
use world_map::WorldFields;

use crate::Regolith;

/// Cross-column movement: delivered water carries fines down the drainage
/// tree. The donor's surface coarsens toward what lies beneath; the
/// receiver is buried in silt.
pub(crate) fn wash(
    ground: &mut Regolith,
    fields: &WorldFields,
    sky: &climate::Climate,
    g: &Ground,
) {
    for tile in 0..ground.rock.len() {
        if fields.elevation[tile] < 0 {
            continue;
        }
        let target = fields.drains_to[tile];
        if target == u32::MAX {
            continue;
        }
        let target = target as usize;
        let water = u32::from(sky.delivered[tile]);
        if water == 0 {
            continue;
        }
        let drop = (fields.elevation[tile] - fields.elevation[target]).max(0);
        let carry = (u32::from(g.wash) * water.min(40) / 40
            + u32::from(g.wash_steep) * u32::try_from(drop).expect("non-negative") / 100)
            .min(30);
        let moved = u8::try_from(carry).expect("capped").min(ground.fines[tile]);
        if moved == 0 {
            continue;
        }
        // Donor: fines leave; the column refills from the rock beneath.
        ground.fines[tile] -= moved;
        ground.rock[tile] = ground.rock[tile].saturating_add(moved);
        // Receiver: silt settles into the mineral ground — it displaces
        // rock, scree, and sand, but never the living soil (silt enriches;
        // rubble and lava are what bury fields). A column already pure
        // loam takes no more.
        if fields.elevation[target] >= 0 {
            let mineral = u16::from(ground.rock[target])
                + u16::from(ground.coarse[target])
                + u16::from(ground.sand[target]);
            let take = moved.min(u8::try_from(mineral.min(255)).expect("clamped"));
            ground.fines[target] = ground.fines[target].saturating_add(take);
            settle_burial(ground, target, take, false, false);
        }
    }
}

/// Two distinct parts of one tile, borrowed together.
pub(crate) fn two<'a>(a: &'a mut [u8], b: &'a mut [u8], tile: usize) -> (&'a mut u8, &'a mut u8) {
    (&mut a[tile], &mut b[tile])
}

pub(crate) fn split_rock_coarse(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.rock, &mut g.coarse, tile)
}

pub(crate) fn split_coarse_sand(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.coarse, &mut g.sand, tile)
}

pub(crate) fn split_rock_sand(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.rock, &mut g.sand, tile)
}

/// Loose scree on oversteep ground slides downtree, burying what lies
/// below in coarse rubble and baring the slope to stone.
pub(crate) fn slides(ground: &mut Regolith, fields: &WorldFields, g: &Ground) {
    for tile in 0..ground.rock.len() {
        if fields.elevation[tile] < 0
            || crate::slope_of(fields, tile) <= g.slide_slope
            || ground.coarse[tile] < 20
        {
            continue;
        }
        let target = fields.drains_to[tile];
        if target == u32::MAX || fields.elevation[target as usize] < 0 {
            continue;
        }
        let target = target as usize;
        let moved = g.slide.min(ground.coarse[tile]);
        ground.coarse[tile] -= moved;
        ground.rock[tile] = ground.rock[tile].saturating_add(moved);
        ground.coarse[target] = ground.coarse[target].saturating_add(moved);
        settle_burial(ground, target, moved, true, true);
    }
}

/// Ash settling (docs/29): the burial rules, entered from outside.
pub(crate) fn settle_ash(ground: &mut Regolith, tile: usize, arrived: u8, heavy: bool) {
    settle_burial(ground, tile, arrived, false, heavy);
}

/// A quake shakes the slope loose (docs/29): a chunk of coarse and rock
/// goes downtree regardless of the usual slope threshold.
pub(crate) fn shake_loose(ground: &mut Regolith, fields: &WorldFields, tile: usize, strength: u8) {
    if fields.elevation[tile] < 0 {
        return;
    }
    let target = fields.drains_to[tile];
    if target == u32::MAX || fields.elevation[target as usize] < 0 {
        return;
    }
    let target = target as usize;
    let moved = strength.min(ground.coarse[tile]);
    ground.coarse[tile] -= moved;
    ground.rock[tile] = ground.rock[tile].saturating_add(moved);
    ground.coarse[target] = ground.coarse[target].saturating_add(moved);
    settle_burial(ground, target, moved, true, true);
}

/// Keep a receiving column at its fixed total: arriving material displaces
/// the largest mineral part. `displace_fines` is false when fines are what
/// arrived; `bury_soil` lets rubble and sand take the living soil at the
/// last — silt never does.
pub(crate) fn settle_burial(
    ground: &mut Regolith,
    tile: usize,
    arrived: u8,
    displace_fines: bool,
    bury_soil: bool,
) {
    let mut left = arrived;
    while left > 0 {
        let rock = ground.rock[tile];
        let coarse = ground.coarse[tile];
        let sand = ground.sand[tile];
        let fines = if displace_fines {
            ground.fines[tile]
        } else {
            0
        };
        let top = rock.max(coarse).max(sand).max(fines);
        let slot: &mut u8 = if top == 0 {
            if bury_soil && ground.organic[tile] > 0 {
                &mut ground.organic[tile]
            } else {
                break;
            }
        } else if rock == top {
            &mut ground.rock[tile]
        } else if coarse == top {
            &mut ground.coarse[tile]
        } else if sand == top {
            &mut ground.sand[tile]
        } else {
            &mut ground.fines[tile]
        };
        let take = left.min(*slot);
        *slot -= take;
        left -= take;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{still_sky, strip};
    use tuning::Seasons;

    #[test]
    fn silt_finds_the_floodplain() {
        // Steep wet highland draining onto a plain: the silt must arrive.
        let fields = strip([900, 200, 60], [160, 160, 160], [120, 140, 150]);
        let g = Ground::default();
        let flora = vec![140u8; 9];
        let mut ground = Regolith::genesis(&fields, &flora, &g);
        let organic_plain0 = ground.organic[2];
        let fert_plain0 = ground.fertility(2);

        let sky = still_sky(9, 30);
        let se = Seasons::default();
        for month in 0..240 {
            ground.tick_month(&fields, &sky, &flora, month, &se, &g);
        }
        assert!(
            ground.organic[2] > organic_plain0,
            "the arriving silt must feed the soil ({} -> {})",
            organic_plain0,
            ground.organic[2]
        );
        assert!(
            ground.fertility(2) > fert_plain0,
            "and the floodplain must grow richer ({} -> {})",
            fert_plain0,
            ground.fertility(2)
        );
        assert!(
            ground.fines[0] < 20,
            "the hill exports the silt it makes — nothing settles on the steep"
        );
        assert!(
            ground.fertility(2) > ground.fertility(0),
            "fertility follows the silt downhill"
        );
    }

    #[test]
    fn dunes_march_downwind() {
        // A dry, hot, bare strip: row 1 of a 3x3 world, wind blowing west.
        let fields = strip([40, 40, 40], [30, 30, 30], [300, 300, 300]);
        let g = Ground::default();
        let seeded = vec![120u8; 9];
        let mut ground = Regolith::genesis(&fields, &seeded, &g);
        // Give the middle tile something to lose.
        ground.fines[1] = 90;
        ground.sand[1] = 40;
        let downwind_sand0 = ground.sand[0];

        let sky = still_sky(9, 0);
        let bare = vec![0u8; 9];
        let se = Seasons::default();
        for month in 0..120 {
            ground.tick_month(&fields, &sky, &bare, month, &se, &g);
        }
        assert!(
            ground.sand[0] > downwind_sand0,
            "sand must pile downwind ({downwind_sand0} -> {})",
            ground.sand[0]
        );
        assert!(ground.fines[1] < 90, "the donor's fines blew away");
    }
}
