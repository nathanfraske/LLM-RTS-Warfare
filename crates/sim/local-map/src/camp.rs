//! The settled overlay at person scale: where the camp sits, the plots
//! it clears, and the raised buildings standing at their true size
//! (docs/30 — footprints are real ground).

use structures::Design;
use world_map::Water;

use crate::LOCAL_SIZE;

/// First dry cell spiralling out from the center; clears trees around it.
pub(crate) fn place_camp(water: &[Water], tree: &mut [bool]) -> (u32, u32) {
    let size = i64::from(LOCAL_SIZE);
    let center = size / 2;
    let mut best = (LOCAL_SIZE / 2, LOCAL_SIZE / 2);
    'search: for radius in 0..size / 2 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let (x, y) = (center + dx, center + dy);
                if x < 0 || y < 0 || x >= size || y >= size {
                    continue;
                }
                let i = (y as usize) * LOCAL_SIZE as usize + x as usize;
                if water[i] == Water::Dry {
                    best = (x as u32, y as u32);
                    break 'search;
                }
            }
        }
    }
    let (cx, cy) = (i64::from(best.0), i64::from(best.1));
    for dy in -7i64..=7 {
        for dx in -7i64..=7 {
            let (x, y) = (cx + dx, cy + dy);
            if x >= 0 && y >= 0 && x < size && y < size {
                tree[(y as usize) * LOCAL_SIZE as usize + x as usize] = false;
            }
        }
    }
    best
}

/// Builders choose their ground (docs/30): every raising seeks the best
/// site its people can find — flat, dry, near the camp, costing the least
/// clearing — and never stands on another's floor. No slot table exists;
/// the terrain decides, in the order the buildings were raised.
#[allow(clippy::too_many_arguments)]
pub(crate) fn raise_buildings(
    built: &mut [u8],
    tree: &mut [bool],
    water: &[Water],
    elevation: &[i32],
    buildings: &[Design],
    cx: u32,
    cy: u32,
) {
    for design in buildings.iter().filter(|d| !d.is_groundwork()) {
        let (w, d) = design.footprint();
        let Some((x0, y0)) = best_site(
            built,
            tree,
            water,
            elevation,
            i64::from(w),
            i64::from(d),
            cx,
            cy,
        ) else {
            continue; // nowhere fit to build this month's ambition
        };
        let class = design.wall_class();
        for dy in 0..i64::from(d) {
            for dx in 0..i64::from(w) {
                let at = ((y0 + dy) as usize) * LOCAL_SIZE as usize + (x0 + dx) as usize;
                tree[at] = false;
                let rim = dx == 0 || dy == 0 || dx == i64::from(w) - 1 || dy == i64::from(d) - 1;
                built[at] = if rim { class } else { class + 10 };
            }
        }
    }
}

/// The cheapest buildable rectangle: dry and unclaimed throughout, then
/// scored by felling work, ground roughness, and the walk from camp.
/// Deterministic: rings scan outward in fixed order, first-best wins ties.
#[allow(clippy::too_many_arguments)]
fn best_site(
    built: &[u8],
    tree: &[bool],
    water: &[Water],
    elevation: &[i32],
    w: i64,
    d: i64,
    cx: u32,
    cy: u32,
) -> Option<(i64, i64)> {
    let size = i64::from(LOCAL_SIZE);
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    let mut best: Option<(i64, i64, i64)> = None;
    for radius in (6..64).step_by(2) {
        // Once something is found, nearby rings can still undercut it on
        // ground quality, but far ones cannot beat the walk.
        if let Some((cost, _, _)) = best
            && radius * 2 > cost
        {
            break;
        }
        for dy in (-radius..=radius).step_by(2) {
            for dx in (-radius..=radius).step_by(2) {
                if dx.abs().max(dy.abs()) != radius && dx.abs().max(dy.abs()) != radius - 1 {
                    continue;
                }
                let x0 = cx + dx - w / 2;
                let y0 = cy + dy - d / 2;
                if x0 < 1 || y0 < 1 || x0 + w >= size - 1 || y0 + d >= size - 1 {
                    continue;
                }
                let Some(ground_cost) = rect_cost(built, tree, water, elevation, x0, y0, w, d)
                else {
                    continue;
                };
                let cost = ground_cost + radius * 2;
                if best.is_none_or(|(b, _, _)| cost < b) {
                    best = Some((cost, x0, y0));
                }
            }
        }
    }
    best.map(|(_, x, y)| (x, y))
}

/// A rectangle's price to build on, or `None` if water or another floor
/// forbids it: two per tree felled, three per unit of unevenness.
#[allow(clippy::too_many_arguments)]
fn rect_cost(
    built: &[u8],
    tree: &[bool],
    water: &[Water],
    elevation: &[i32],
    x0: i64,
    y0: i64,
    w: i64,
    d: i64,
) -> Option<i64> {
    let mut trees = 0i64;
    let mut lo = i32::MAX;
    let mut hi = i32::MIN;
    for dy in 0..d {
        for dx in 0..w {
            let at = ((y0 + dy) as usize) * LOCAL_SIZE as usize + (x0 + dx) as usize;
            if water[at] != Water::Dry || built[at] != 0 {
                return None;
            }
            trees += i64::from(tree[at]);
            lo = lo.min(elevation[at]);
            hi = hi.max(elevation[at]);
        }
    }
    Some(trees * 2 + i64::from(hi - lo) * 3)
}

/// Tilled fields need open ground east of the camp.
pub(crate) fn clear_farm_plot(tree: &mut [bool], cx: u32, cy: u32) {
    let size = i64::from(LOCAL_SIZE);
    for dy in -10i64..=10 {
        for dx in 9i64..=30 {
            let (x, y) = (i64::from(cx) + dx, i64::from(cy) + dy);
            if x >= 0 && y >= 0 && x < size && y < size {
                tree[(y as usize) * LOCAL_SIZE as usize + x as usize] = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structures::{Aspects, Material};

    fn a_building(area_milli: u16) -> Design {
        let stone = Material {
            word: "stone-walled",
            hardness: 600,
            binding: 300,
            mass: 900,
            supply: 120,
        };
        Design {
            wall: stone,
            roof: stone,
            footing_milli: 500,
            aspects: Aspects {
                capacity_milli: 500,
                ..Aspects::default()
            },
            area_milli,
            name: "test raising".into(),
            integrity_milli: 500,
            months: 3,
        }
    }

    #[test]
    fn builders_pick_their_ground_and_never_stand_on_water_or_each_other() {
        let cells = (LOCAL_SIZE * LOCAL_SIZE) as usize;
        let mut water = vec![Water::Dry; cells];
        let mut elevation = vec![100i32; cells];
        let mut tree = vec![false; cells];
        let size = LOCAL_SIZE as usize;
        // A lake just east of camp, a sharp ridge just south.
        for y in 0..size {
            for x in 0..size {
                if (135..190).contains(&x) && (100..160).contains(&y) {
                    water[y * size + x] = Water::Lake;
                }
                if (140..150).contains(&y) && x < 135 {
                    elevation[y * size + x] = 100 + i32::try_from((x * 7) % 40).expect("small");
                }
            }
        }
        let mut built = vec![0u8; cells];
        let designs = vec![a_building(400), a_building(150), a_building(250)];
        raise_buildings(
            &mut built, &mut tree, &water, &elevation, &designs, 128, 128,
        );

        let mut placed = 0usize;
        for (at, &b) in built.iter().enumerate() {
            if b == 0 {
                continue;
            }
            placed += 1;
            assert_eq!(water[at], Water::Dry, "no wall stands in the lake");
            let y = at / size;
            let x = at % size;
            assert!(
                !((140..150).contains(&y) && x < 135) || elevation[at] == 100,
                "rough ground was priced, flat was chosen"
            );
        }
        let expected: usize = designs
            .iter()
            .map(|d| {
                let (w, h) = d.footprint();
                usize::from(w) * usize::from(h)
            })
            .sum();
        assert_eq!(placed, expected, "every raising found ground, none overlap");

        // Determinism: same land, same builders, same village.
        let mut built2 = vec![0u8; cells];
        let mut tree2 = vec![false; cells];
        raise_buildings(
            &mut built2,
            &mut tree2,
            &water,
            &elevation,
            &designs,
            128,
            128,
        );
        assert_eq!(built, built2);
    }
}
