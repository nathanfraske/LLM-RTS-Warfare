//! The trodden ways (docs/30): feet make the paths — from the camp to
//! every doorstep, to the worked fields, to the water's edge — and their
//! total length is the settlement's layout number, the ground good
//! planning earns on.

use world_map::Water;

use crate::LOCAL_SIZE;

/// Feet make the paths (docs/30): from the camp to every doorstep, to the
/// worked fields, to the water's edge — the routes people actually walk,
/// worn into the ground. Returns the trodden mask and the layout's mean
/// walk in cells ×10 per destination: the number good planning earns on.
pub(crate) fn tread_paths(
    built: &[u8],
    water: &[Water],
    placed: &[(i64, i64, i64, i64)],
    has_plot: bool,
    cx: u32,
    cy: u32,
) -> (Vec<bool>, u16) {
    let size = i64::from(LOCAL_SIZE);
    let mut paths = vec![false; (LOCAL_SIZE * LOCAL_SIZE) as usize];
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    let mut total_steps = 0i64;
    let mut destinations = 0i64;

    let walk_to = |paths: &mut Vec<bool>, tx: i64, ty: i64| -> i64 {
        let (mut x, mut y) = (cx, cy);
        let mut steps = 0i64;
        while (x, y) != (tx, ty) && steps < size {
            let mut best: Option<(i64, i64, i64)> = None;
            for (dx, dy) in [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= size || ny >= size {
                    continue;
                }
                let at = (ny as usize) * LOCAL_SIZE as usize + nx as usize;
                if water[at] != Water::Dry || built[at] != 0 {
                    continue;
                }
                let dist = (tx - nx).abs().max((ty - ny).abs());
                if best.is_none_or(|(b, _, _)| dist < b) {
                    best = Some((dist, nx, ny));
                }
            }
            let Some((_, nx, ny)) = best else { break };
            if (nx, ny) == (x, y) {
                break;
            }
            x = nx;
            y = ny;
            steps += 1;
            let at = (y as usize) * LOCAL_SIZE as usize + x as usize;
            if (x, y) != (tx, ty) {
                paths[at] = true;
            }
        }
        steps
    };

    for &(x0, y0, w, d) in placed {
        // The doorstep: the rim cell facing camp, one step outside.
        let door_x = cx.clamp(x0, x0 + w - 1);
        let door_y = if cy < y0 { y0 - 1 } else { y0 + d };
        let steps = walk_to(&mut paths, door_x, door_y.clamp(0, size - 1));
        total_steps += steps;
        destinations += 1;
    }
    if has_plot {
        total_steps += walk_to(&mut paths, (cx + 19).min(size - 2), cy);
        destinations += 1;
    }
    // The water's edge, if any lies near: the drawing-and-fishing walk.
    'water: for radius in 2..70i64 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let (x, y) = (cx + dx, cy + dy);
                if x < 1 || y < 1 || x >= size - 1 || y >= size - 1 {
                    continue;
                }
                let at = (y as usize) * LOCAL_SIZE as usize + x as usize;
                if water[at] == Water::Dry
                    && [
                        at - 1,
                        at + 1,
                        at - LOCAL_SIZE as usize,
                        at + LOCAL_SIZE as usize,
                    ]
                    .iter()
                    .any(|&n| water[n] != Water::Dry)
                {
                    total_steps += walk_to(&mut paths, x, y);
                    destinations += 1;
                    break 'water;
                }
            }
        }
    }

    let layout = if destinations == 0 {
        0
    } else {
        u16::try_from((total_steps * 10 / destinations).clamp(0, 1000)).expect("clamped")
    };
    (paths, layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camp::raise_buildings;
    use structures::{Aspects, Design, Material};

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
    fn feet_make_the_paths_and_never_cross_wall_or_water() {
        let cells = (LOCAL_SIZE * LOCAL_SIZE) as usize;
        let mut water = vec![Water::Dry; cells];
        let elevation = vec![100i32; cells];
        let mut tree = vec![false; cells];
        let size = LOCAL_SIZE as usize;
        for y in 100..160 {
            for x in 135..190 {
                water[y * size + x] = Water::Lake;
            }
        }
        let mut built = vec![0u8; cells];
        let designs = vec![a_building(400), a_building(150)];
        let placed = raise_buildings(
            &mut built, &mut tree, &water, &elevation, &designs, 128, 128,
        );

        // Feet make the paths: trodden ground reaches the doorsteps and
        // the lake shore, and never crosses water or a wall.
        let (paths, layout) = tread_paths(&built, &water, &placed, false, 128, 128);
        let trodden = paths.iter().filter(|&&p| p).count();
        // A compact village wears short ways — that is the reward.
        assert!(trodden > 4, "a village wears its ways, got {trodden}");
        assert!(layout > 0, "the walk has a length");
        for (at, &p) in paths.iter().enumerate() {
            if p {
                assert_eq!(water[at], Water::Dry, "no path over water");
                assert_eq!(built[at], 0, "no path through a wall");
            }
        }
    }
}
