//! Continental heightfield assembly: warped continents + ridged mountain
//! chains + edge seas, quantized to integer meters (docs/13-worldgen.md).

use crate::grid::Grid;
use crate::noise::{self, Channel};
use sim_events::WorldSeed;

const CONTINENTS: Channel = Channel(0);
const WARP: Channel = Channel(64);
const RIDGES: Channel = Channel(128);

const OCEAN_FRACTION: f32 = 0.58;
const MAX_LAND_M: f32 = 4_500.0;
const MAX_DEPTH_M: f32 = 3_500.0;

/// Elevation in meters per cell; `< 0` is below sea level.
#[must_use]
pub fn generate(seed: WorldSeed, grid: Grid) -> Vec<i32> {
    let size = grid.size as f32;
    let continent_scale = 2.6 / size;
    let ridge_scale = 9.0 / size;
    let mut surface = vec![0.0f32; grid.cells()];

    for (i, h) in surface.iter_mut().enumerate() {
        let (x, y) = grid.xy(i);
        let (fx, fy) = (x as f32, y as f32);
        let cont = noise::warped_fbm(
            seed,
            CONTINENTS,
            WARP,
            fx * continent_scale,
            fy * continent_scale,
            5,
            0.9,
        );
        let ridge = noise::ridged(seed, RIDGES, fx * ridge_scale, fy * ridge_scale, 4);
        // Mountains only rise from continental interiors.
        let interior = (cont * 1.6).clamp(0.0, 1.0);
        let raw = cont * 0.8 + ridge * 0.65 * interior;
        *h = raw * edge_falloff(fx, fy, size) - (1.0 - edge_falloff(fx, fy, size));
    }

    let sea = quantile(&surface, OCEAN_FRACTION);
    let top = surface.iter().copied().fold(f32::MIN, f32::max);
    let bottom = surface.iter().copied().fold(f32::MAX, f32::min);
    let land_span = (top - sea).max(1e-6);
    let sea_span = (sea - bottom).max(1e-6);

    surface
        .iter()
        .map(|&h| {
            if h >= sea {
                ((h - sea) / land_span * MAX_LAND_M) as i32
            } else {
                -(((sea - h) / sea_span * MAX_DEPTH_M) as i32) - 1
            }
        })
        .collect()
}

/// Smoothly sinks the map borders so edges are always ocean.
fn edge_falloff(x: f32, y: f32, size: f32) -> f32 {
    let margin = size * 0.14;
    let d = x.min(y).min(size - 1.0 - x).min(size - 1.0 - y);
    let t = (d / margin).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Value below which `fraction` of samples fall (deterministic total order).
fn quantile(values: &[f32], fraction: f32) -> f32 {
    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let k = ((sorted.len() - 1) as f32 * fraction) as usize;
    sorted[k]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn land_fraction_is_reasonable_and_edges_are_ocean() {
        let grid = Grid { size: 128 };
        let elev = generate(WorldSeed(42), grid);
        let land = elev.iter().filter(|&&e| e >= 0).count();
        let fraction = land as f32 / elev.len() as f32;
        assert!(
            (0.25..=0.55).contains(&fraction),
            "land fraction {fraction}"
        );
        for (i, &e) in elev.iter().enumerate() {
            if grid.on_border(i) {
                assert!(e < 0, "border cell {i} must be ocean");
            }
        }
    }
}
