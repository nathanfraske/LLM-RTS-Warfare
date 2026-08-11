//! The regolith's monthly movements (docs/27 §1): frost and heat break
//! rock downward; roots build soil and rot spends it; wind winnows the
//! dry; water carries the fine downtree. Every movement is a transfer
//! inside or between fixed columns — a loss is an exposure, a gain a
//! burial.

use tuning::Ground;
use world_map::WorldFields;

use crate::Regolith;

/// Move up to `amount` from part `a` to part `b` of one tile's column.
fn shift(from: &mut u8, to: &mut u8, amount: u8) {
    let moved = amount.min(*from).min(255 - *to);
    *from -= moved;
    *to += moved;
}

/// In-column movements: weathering, roots, rot, winnowing.
// Parallel component arrays share the tile index (see `lib`).
#[allow(clippy::needless_range_loop)]
pub(crate) fn weather_and_grow(
    ground: &mut Regolith,
    fields: &WorldFields,
    sky: &climate::Climate,
    flora_live: &[u8],
    month: u64,
    se: &tuning::Seasons,
    g: &Ground,
) {
    for tile in 0..ground.rock.len() {
        if fields.elevation[tile] < 0 {
            continue;
        }
        let t = climate::t_eff(fields, tile, month, se);
        let dry = sky.delivered[tile] < g.dry_water;
        let bare = flora_live[tile] < g.bare_veg;

        // Frost-shattering: a thaw month over banked snow grinds the rock.
        if sky.snowpack[tile] > 0 && sky.growth[tile] > 0 {
            let (rock, coarse) = split_rock_coarse(ground, tile);
            shift(rock, coarse, g.frost_shatter);
            let (coarse, sand) = split_coarse_sand(ground, tile);
            shift(coarse, sand, g.frost_shatter);
        }
        // Heat-cracking: hot, dry, bare stone sheds sand.
        if t > g.hot_deci && dry && bare {
            let (rock, sand) = split_rock_sand(ground, tile);
            shift(rock, sand, g.heat_crack);
        }
        // Chemical weathering: where the water runs, the clay is made.
        if !dry {
            let (rock, fines) = two(&mut ground.rock, &mut ground.fines, tile);
            shift(rock, fines, g.wet_weather);
        }
        // Roots build soil out of the mineral top; rot spends it back.
        let veg = u16::from(flora_live[tile]);
        if veg > u16::from(g.bare_veg) {
            let build = u8::try_from(u16::from(g.root_build) * veg / 255).expect("bounded");
            let donor_is_fines = ground.fines[tile] >= ground.sand[tile];
            let (mineral, organic) = if donor_is_fines {
                let (f, o) = two(&mut ground.fines, &mut ground.organic, tile);
                (f, o)
            } else {
                let (s, o) = two(&mut ground.sand, &mut ground.organic, tile);
                (s, o)
            };
            shift(mineral, organic, build);
        } else if t > g.hot_deci {
            let (organic, sand) = two(&mut ground.organic, &mut ground.sand, tile);
            shift(organic, sand, g.rot_loss);
        }
        // Winnowing: dry bare fines blow toward sand.
        if dry && bare {
            let (fines, sand) = two(&mut ground.fines, &mut ground.sand, tile);
            shift(fines, sand, g.winnow);
        }
    }
}

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
        // Receiver: silt buries whatever it lands on — coarse ground
        // first, then sand, and in the end even the living soil (floods
        // bury fields; deltas never stop rising).
        if fields.elevation[target] >= 0 {
            let candidates = [
                ground.rock[target],
                ground.coarse[target],
                ground.sand[target],
            ];
            let buried = match (0..3).max_by_key(|&i| candidates[i]) {
                Some(0) if candidates[0] > 0 => &mut ground.rock[target],
                Some(1) if candidates[1] > 0 => &mut ground.coarse[target],
                Some(2) if candidates[2] > 0 => &mut ground.sand[target],
                _ => &mut ground.organic[target],
            };
            let take = moved.min(*buried);
            *buried -= take;
            ground.fines[target] = ground.fines[target].saturating_add(take);
        }
    }
}

/// Two distinct parts of one tile, borrowed together.
fn two<'a>(a: &'a mut [u8], b: &'a mut [u8], tile: usize) -> (&'a mut u8, &'a mut u8) {
    (&mut a[tile], &mut b[tile])
}

fn split_rock_coarse(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.rock, &mut g.coarse, tile)
}

fn split_coarse_sand(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.coarse, &mut g.sand, tile)
}

fn split_rock_sand(g: &mut Regolith, tile: usize) -> (&mut u8, &mut u8) {
    two(&mut g.rock, &mut g.sand, tile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuning::{Seasons, Weather};
    use world_map::hydrology::Water as W;

    fn strip(elevations: [i32; 3], moisture: [u8; 3], temp: [i16; 3]) -> WorldFields {
        WorldFields {
            size: 3,
            elevation: vec![
                elevations[0],
                elevations[1],
                elevations[2],
                -100,
                -100,
                -100,
                -100,
                -100,
                -100,
            ],
            water: vec![W::Dry; 9],
            flow_acc: vec![1; 9],
            drains_to: vec![
                1,
                2,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
            ],
            temperature: vec![temp[0], temp[1], temp[2], 100, 100, 100, 100, 100, 100],
            moisture: vec![
                moisture[0],
                moisture[1],
                moisture[2],
                100,
                100,
                100,
                100,
                100,
                100,
            ],
            cell_fertility: vec![100; 9],
        }
    }

    fn still_sky(cells: usize, delivered: u16) -> climate::Climate {
        climate::Climate {
            wet: vec![100; cells],
            snowpack: vec![0; cells],
            growth: vec![500; cells],
            delivered: vec![delivered; cells],
        }
    }

    #[test]
    fn deserts_are_arrived_at_not_placed() {
        let fields = strip([40, 40, 40], [30, 30, 30], [300, 300, 300]);
        let g = Ground::default();
        let flora_start = vec![120u8; 9];
        let mut ground = Regolith::genesis(&fields, &flora_start, &g);
        let sand0 = ground.sand[1];
        let fert0 = ground.fertility(1);

        // The green dies; the dry heat goes to work for twenty years.
        let sky = still_sky(9, 0);
        let bare = vec![0u8; 9];
        let se = Seasons::default();
        let _ = Weather::default();
        for month in 0..240 {
            ground.tick_month(&fields, &sky, &bare, month, &se, &g);
        }
        assert!(
            ground.sand[1] > sand0,
            "bare dry heat must sand the ground ({} -> {})",
            sand0,
            ground.sand[1]
        );
        assert_eq!(ground.organic[1], 0, "the soil's life is gone");
        assert!(
            ground.fertility(1) < fert0,
            "and the fertility went with it"
        );
        assert!(ground.describe(1).contains("sand"));
    }

    #[test]
    fn silt_finds_the_floodplain() {
        // Steep wet highland draining onto a plain: the silt must arrive.
        let fields = strip([900, 200, 60], [160, 160, 160], [120, 140, 150]);
        let g = Ground::default();
        let flora = vec![140u8; 9];
        let mut ground = Regolith::genesis(&fields, &flora, &g);
        let fines_plain0 = ground.fines[2];

        let sky = still_sky(9, 30);
        let se = Seasons::default();
        for month in 0..240 {
            ground.tick_month(&fields, &sky, &flora, month, &se, &g);
        }
        assert!(
            ground.fines[2] > fines_plain0,
            "the plain must gain silt ({} -> {})",
            fines_plain0,
            ground.fines[2]
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
}
