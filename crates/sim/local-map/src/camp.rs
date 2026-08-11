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

/// Stand every roomed building on the ground at its true size: walls on
/// the rectangle's rim, roofed interior within, trees cleared under.
/// Slots ring the camp deterministically.
pub(crate) fn raise_buildings(
    built: &mut [u8],
    tree: &mut [bool],
    water: &[Water],
    buildings: &[Design],
    cx: u32,
    cy: u32,
) {
    const SLOTS: [(i64, i64); 6] = [
        (-16, -8),
        (10, -12),
        (-14, 10),
        (12, 9),
        (-3, -16),
        (15, -1),
    ];
    let size = i64::from(LOCAL_SIZE);
    for (slot, design) in buildings.iter().filter(|d| !d.is_groundwork()).enumerate() {
        let (w, d) = design.footprint();
        let (w, d) = (i64::from(w), i64::from(d));
        let (ox, oy) = SLOTS[slot % SLOTS.len()];
        let x0 = (i64::from(cx) + ox).clamp(1, size - w - 1);
        let y0 = (i64::from(cy) + oy).clamp(1, size - d - 1);
        let class = design.wall_class();
        for dy in 0..d {
            for dx in 0..w {
                let (x, y) = (x0 + dx, y0 + dy);
                let at = (y as usize) * LOCAL_SIZE as usize + x as usize;
                if water[at] != Water::Dry {
                    continue; // walls stop at the waterline
                }
                tree[at] = false;
                let rim = dx == 0 || dy == 0 || dx == w - 1 || dy == d - 1;
                built[at] = if rim { class } else { class + 10 };
            }
        }
    }
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
