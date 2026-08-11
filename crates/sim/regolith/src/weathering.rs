//! In-column movements of the regolith (docs/27 §1): frost and heat break
//! rock downward, water makes clay of it, roots build soil and rot spends
//! it, and the dry wind winnows fines — in place and downwind, where green
//! ground traps loess and bare ground piles dunes.

use tuning::Ground;
use world_map::WorldFields;

use crate::Regolith;
use crate::transport::{settle_burial, split_coarse_sand, split_rock_coarse, split_rock_sand, two};

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
        // Winnowing: dry bare fines blow — some in place toward sand,
        // some downwind, where green ground traps them as loess and bare
        // ground piles them as dune (docs/27 queue, landed).
        if dry && bare {
            let hop = u8::try_from(u32::from(g.winnow) * u32::from(g.winnow_hop_permille) / 1000)
                .expect("bounded");
            let stay = g.winnow - hop.min(g.winnow);
            let (fines, sand) = two(&mut ground.fines, &mut ground.sand, tile);
            shift(fines, sand, stay);
            let size = fields.size as usize;
            let (x, y) = (tile % size, tile / size);
            let dx = climate::wind_dx(y, size);
            let downwind_x = x.wrapping_add_signed(dx);
            if downwind_x < size {
                let target = y * size + downwind_x;
                if fields.elevation[target] >= 0 {
                    let moved = hop.min(ground.fines[tile]);
                    ground.fines[tile] -= moved;
                    if flora_live[target] > g.bare_veg {
                        // Loess: green ground traps the dust as new fines.
                        ground.fines[target] = ground.fines[target].saturating_add(moved);
                        settle_burial(ground, target, moved, false, false);
                    } else {
                        // Dune: bare ground is buried in marching sand.
                        ground.sand[target] = ground.sand[target].saturating_add(moved);
                        settle_burial(ground, target, moved, true, true);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{still_sky, strip};
    use tuning::Seasons;

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
    fn the_plow_eats_the_soil_and_fallow_restores_it() {
        let fields = strip([40, 40, 40], [160, 160, 160], [150, 150, 150]);
        let g = Ground::default();
        let flora = vec![160u8; 9];
        let mut ground = Regolith::genesis(&fields, &flora, &g);
        let organic0 = ground.organic[1];

        // Hard cultivation, month after month.
        for _ in 0..60 {
            ground.farm_wear(1, 800, &g);
        }
        let worn = ground.organic[1];
        assert!(worn < organic0, "sustained farming spends the soil");

        // Fallow under living green: the roots build it back.
        let sky = still_sky(9, 20);
        let se = Seasons::default();
        for month in 0..120 {
            ground.tick_month(&fields, &sky, &flora, month, &se, &g);
        }
        assert!(
            ground.organic[1] > worn,
            "fallow ground recovers ({worn} -> {})",
            ground.organic[1]
        );
    }
}
