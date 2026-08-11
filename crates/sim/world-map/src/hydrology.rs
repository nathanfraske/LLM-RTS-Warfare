//! Water from height: priority-flood depression filling, drainage,
//! flow accumulation, lakes and rivers (docs/13-worldgen.md).
//!
//! The flood's pop order doubles as a drainage tree: each cell drains via the
//! cell that first reached it, guaranteeing a monotone path to the ocean —
//! flats and lake spill paths handled with no special cases.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::grid::Grid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Water {
    Dry,
    Ocean,
    Lake,
    River,
}

#[derive(Debug)]
pub struct Hydrology {
    pub water: Vec<Water>,
    /// Cells drained through each cell (rain = 1 per land cell).
    pub flow_acc: Vec<u32>,
    /// Water surface height (differs from ground inside lakes).
    pub filled: Vec<i32>,
    /// Each cell's outflow neighbor in the drainage tree; `u32::MAX` at
    /// the border sinks. Flow animation, discharge, and sediment ride it.
    pub drains_to: Vec<u32>,
}

#[must_use]
pub fn compute(grid: Grid, elevation: &[i32]) -> Hydrology {
    let n = grid.cells();
    let mut filled = elevation.to_vec();
    let mut visited = vec![false; n];
    let mut drains_to = vec![usize::MAX; n];
    let mut pop_order = Vec::with_capacity(n);

    // Priority flood seeded from the border (all ocean by construction).
    let mut heap: BinaryHeap<Reverse<(i32, usize)>> = BinaryHeap::new();
    for i in 0..n {
        if grid.on_border(i) {
            visited[i] = true;
            heap.push(Reverse((filled[i], i)));
        }
    }
    while let Some(Reverse((level, i))) = heap.pop() {
        pop_order.push(i);
        let (neighbors, count) = grid.neighbors8(i);
        for &nb in &neighbors[..count] {
            if !visited[nb] {
                visited[nb] = true;
                filled[nb] = elevation[nb].max(level);
                drains_to[nb] = i;
                heap.push(Reverse((filled[nb], nb)));
            }
        }
    }

    // Rain accumulates down the drainage tree in reverse pop (highest-first) order.
    let mut flow_acc = vec![0u32; n];
    for &i in pop_order.iter().rev() {
        if elevation[i] >= 0 {
            flow_acc[i] += 1;
        }
        let parent = drains_to[i];
        if parent != usize::MAX {
            flow_acc[parent] += flow_acc[i];
        }
    }

    // Classify. River threshold scales with map area.
    let river_threshold = (n as u32 / 640).max(24);
    let water = (0..n)
        .map(|i| {
            if elevation[i] < 0 {
                Water::Ocean
            } else if filled[i] > elevation[i] {
                Water::Lake
            } else if flow_acc[i] >= river_threshold {
                Water::River
            } else {
                Water::Dry
            }
        })
        .collect();

    let drains_to = drains_to
        .iter()
        .map(|&d| if d == usize::MAX { u32::MAX } else { d as u32 })
        .collect();
    Hydrology {
        water,
        flow_acc,
        filled,
        drains_to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heightfield;
    use sim_events::WorldSeed;

    #[test]
    fn rivers_exist_and_drainage_reaches_the_sea() {
        let grid = Grid { size: 128 };
        let elev = heightfield::generate(WorldSeed(42), grid);
        let hydro = compute(grid, &elev);
        let rivers = hydro.water.iter().filter(|w| **w == Water::River).count();
        assert!(rivers > 0, "a 128² world should carve at least one river");
        // Total rain equals land cells; it all ends up accumulated at border cells.
        let land = elev.iter().filter(|&&e| e >= 0).count() as u64;
        let border_acc: u64 = (0..grid.cells())
            .filter(|&i| grid.on_border(i))
            .map(|i| u64::from(hydro.flow_acc[i]))
            .sum();
        assert_eq!(border_acc, land, "all rain must drain off the map");
    }
}
