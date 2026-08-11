//! Climate v1: latitude + altitude temperature, distance-to-water moisture
//! (docs/13-worldgen.md — deliberately cheap; weather sim supersedes later).

use std::collections::VecDeque;

use crate::grid::Grid;
use crate::hydrology::Water;
use crate::noise::{self, Channel};
use sim_events::WorldSeed;

const MOISTURE_JITTER: Channel = Channel(192);

/// Deci-°C per cell. Equator at mid-map; -6.5°C per 1000 m lapse rate.
#[must_use]
pub fn temperature(grid: Grid, elevation: &[i32]) -> Vec<i16> {
    let mid = grid.size as f32 / 2.0;
    (0..grid.cells())
        .map(|i| {
            let (_, y) = grid.xy(i);
            let frac = ((y as f32 - mid).abs() / mid).min(1.0);
            let sea_level_c = 31.0 - 40.0 * frac - 14.0 * frac * frac;
            let altitude_m = elevation[i].max(0) as f32;
            let c = sea_level_c - 6.5 * altitude_m / 1000.0;
            (c * 10.0).clamp(-600.0, 500.0) as i16
        })
        .collect()
}

/// Moisture 0–255 from decaying BFS distance to any water, plus noise jitter.
#[must_use]
pub fn moisture(seed: WorldSeed, grid: Grid, water: &[Water]) -> Vec<u8> {
    let n = grid.cells();
    let mut dist = vec![u32::MAX; n];
    let mut queue = VecDeque::new();
    for i in 0..n {
        if water[i] != Water::Dry {
            dist[i] = 0;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        let (neighbors, count) = grid.neighbors8(i);
        for &nb in &neighbors[..count] {
            if dist[nb] == u32::MAX {
                dist[nb] = dist[i] + 1;
                queue.push_back(nb);
            }
        }
    }

    let scale = 14.0 / grid.size as f32;
    (0..n)
        .map(|i| {
            let d = dist[i].min(10_000) as f32;
            // Rational decay instead of exp() — transcendental-free rule.
            let base = 255.0 * 22.0 / (22.0 + d * 1.6);
            let (x, y) = grid.xy(i);
            let jitter =
                noise::fbm(seed, MOISTURE_JITTER, x as f32 * scale, y as f32 * scale, 3) * 34.0;
            (base + jitter).clamp(0.0, 255.0) as u8
        })
        .collect()
}
