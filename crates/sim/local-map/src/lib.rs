//! Local maps: the person-scale map under every world tile
//! (docs/15-multiscale-maps.md). A pure deterministic function of
//! `(seed, tile, surrounding world fields)` — generated on demand,
//! discarded freely, identical every visit. Adjacent tiles agree at their
//! edges because detail noise is keyed by global cell coordinates.

use flora::{FloraMap, NO_FLORA};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_map::noise::{self, Channel};
use world_map::{Water, WorldFields};
use world_schema::{Tick, TileId};

const LOCALGEN: SystemId = SystemId(9);
const DETAIL: Channel = Channel(224);
const MEANDER: Channel = Channel(240);

pub const LOCAL_SIZE: u32 = 256;

/// The person-scale map of one world tile. One cell ≈ one person's footprint.
#[derive(Debug)]
pub struct LocalMap {
    pub size: u32,
    pub elevation: Vec<i32>,
    pub water: Vec<Water>,
    /// Ground vegetation density 0–255.
    pub veg: Vec<u8>,
    /// Baked tree scatter.
    pub tree: Vec<bool>,
    /// Camp center when the tile is settled.
    pub camp: Option<(u32, u32)>,
    /// Completed works on this tile, by registry key, for rendering.
    pub works: Vec<String>,
}

/// Generate the local map for `tile`. `populated` places the camp.
#[must_use]
pub fn generate(
    seed: WorldSeed,
    fields: &WorldFields,
    flora: &FloraMap,
    tile: TileId,
    populated: bool,
    works: &[String],
    density_live: &[u8],
) -> LocalMap {
    let world = fields.grid();
    let t = tile.0 as usize;
    let (tx, ty) = world.xy(t);
    let n = (LOCAL_SIZE * LOCAL_SIZE) as usize;

    let mut elevation = Vec::with_capacity(n);
    for ly in 0..LOCAL_SIZE {
        for lx in 0..LOCAL_SIZE {
            let gx = tx as f32 + (lx as f32 + 0.5) / LOCAL_SIZE as f32;
            let gy = ty as f32 + (ly as f32 + 0.5) / LOCAL_SIZE as f32;
            let base = sample_world_elevation(fields, gx, gy);
            let detail = noise::fbm(seed, DETAIL, gx * 40.0, gy * 40.0, 4);
            elevation.push(base + (detail * 70.0) as i32);
        }
    }

    let mut water: Vec<Water> = elevation
        .iter()
        .map(|&e| if e < 0 { Water::Ocean } else { Water::Dry })
        .collect();
    match fields.water[t] {
        Water::River => carve_river(seed, fields, t, &mut water, &mut elevation),
        Water::Lake => flood_basin(&mut water, &elevation),
        _ => {}
    }

    let density = density_live[t];
    let tree_chance: u64 = match flora.occupant[t] {
        o if o == NO_FLORA => 0,
        o => 8 + u64::from(flora.species[o as usize].woodiness_milli) * 150 / 1000,
    };
    let mut veg = Vec::with_capacity(n);
    let mut tree = Vec::with_capacity(n);
    for ly in 0..LOCAL_SIZE {
        for lx in 0..LOCAL_SIZE {
            let i = (ly * LOCAL_SIZE + lx) as usize;
            let gx = u64::from(tx) * u64::from(LOCAL_SIZE) + u64::from(lx);
            let gy = u64::from(ty) * u64::from(LOCAL_SIZE) + u64::from(ly);
            let key = (gx << 24) | gy;
            let patchiness =
                noise::fbm(seed, Channel(232), gx as f32 / 34.0, gy as f32 / 34.0, 2) * 0.5 + 0.75;
            let local_density = ((f32::from(density) * patchiness).clamp(0.0, 255.0)) as u8;
            let grounded = water[i] == Water::Dry;
            veg.push(if grounded { local_density } else { 0 });
            let roll = rng::draw(seed, Tick::ZERO, LOCALGEN, key) % 1000;
            tree.push(grounded && roll < tree_chance * u64::from(local_density) / 255);
        }
    }

    let camp = populated.then(|| place_camp(&water, &mut tree));
    // Presentation matches works by their registry key (docs/20).
    if let Some((cx, cy)) = camp
        && works.iter().any(|w| w.contains("field-works"))
    {
        clear_farm_plot(&mut tree, cx, cy);
    }

    LocalMap {
        size: LOCAL_SIZE,
        elevation,
        water,
        veg,
        tree,
        camp,
        works: works.to_vec(),
    }
}

/// Tilled fields need open ground east of the camp.
fn clear_farm_plot(tree: &mut [bool], cx: u32, cy: u32) {
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

/// Bilinear world elevation at fractional tile coordinates (tile centers).
fn sample_world_elevation(fields: &WorldFields, gx: f32, gy: f32) -> i32 {
    let world = fields.grid();
    let max = (world.size - 1) as f32;
    let x = (gx - 0.5).clamp(0.0, max);
    let y = (gy - 0.5).clamp(0.0, max);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = ((x0 + 1).min(world.size - 1), (y0 + 1).min(world.size - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |cx: u32, cy: u32| fields.elevation[world.idx(cx, cy)] as f32;
    let top = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
    let bottom = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
    (top * (1.0 - fy) + bottom * fy) as i32
}

/// A wobbling channel across the map along the tile's dominant flow axis.
fn carve_river(
    seed: WorldSeed,
    fields: &WorldFields,
    tile: usize,
    water: &mut [Water],
    elevation: &mut [i32],
) {
    let world = fields.grid();
    let (neighbors, count) = world.neighbors8(tile);
    let downstream = neighbors[..count]
        .iter()
        .copied()
        .max_by_key(|&nb| (fields.flow_acc[nb], usize::MAX - nb))
        .unwrap_or(tile);
    let (tx, ty) = world.xy(tile);
    let (dx, dy) = world.xy(downstream);
    let dir = (i64::from(dx) - i64::from(tx), i64::from(dy) - i64::from(ty));
    let vertical = dir.1.abs() >= dir.0.abs();

    let half = i64::from(LOCAL_SIZE / 2);
    for along in 0..LOCAL_SIZE {
        let g = (u64::from(tx) * 31 + u64::from(ty) * 17) as f32 + along as f32 / 48.0;
        let wobble = (noise::fbm(seed, MEANDER, g, 0.37, 3) * 46.0) as i64;
        let across = (half + wobble).clamp(3, i64::from(LOCAL_SIZE) - 4);
        for w in -2i64..=2 {
            let (x, y) = if vertical {
                (across + w, i64::from(along))
            } else {
                (i64::from(along), across + w)
            };
            if x >= 0 && y >= 0 && x < i64::from(LOCAL_SIZE) && y < i64::from(LOCAL_SIZE) {
                let i = (y as usize) * LOCAL_SIZE as usize + x as usize;
                if water[i] == Water::Dry {
                    water[i] = Water::River;
                    elevation[i] = elevation[i].min(1);
                }
            }
        }
    }
}

/// Lake tiles pool water in their lower basin.
fn flood_basin(water: &mut [Water], elevation: &[i32]) {
    let mut sorted = elevation.to_vec();
    sorted.sort_unstable();
    let level = sorted[sorted.len() / 3];
    for (i, w) in water.iter_mut().enumerate() {
        if *w == Water::Dry && elevation[i] <= level {
            *w = Water::Lake;
        }
    }
}

/// First dry cell spiralling out from the center; clears trees around it.
fn place_camp(water: &[Water], tree: &mut [bool]) -> (u32, u32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_maps_are_deterministic_and_grounded() {
        let seed = WorldSeed(42);
        let fields = WorldFields::generate(seed, 96);
        let flora = flora::settle::settle(seed, &fields, 24);
        let tile = (0..fields.grid().cells())
            .find(|&t| world_map::tiles::habitable(&fields, t))
            .expect("habitable tile exists");
        let a = generate(
            seed,
            &fields,
            &flora,
            TileId(tile as u32),
            true,
            &[],
            &flora.density,
        );
        let b = generate(
            seed,
            &fields,
            &flora,
            TileId(tile as u32),
            true,
            &[],
            &flora.density,
        );
        assert_eq!(a.elevation, b.elevation);
        assert_eq!(a.water, b.water);
        assert_eq!(a.tree, b.tree);
        assert!(a.camp.is_some());
        let dry = a.water.iter().filter(|w| **w == Water::Dry).count();
        assert!(dry > (a.water.len() / 4), "a habitable tile is mostly land");
    }

    #[test]
    fn river_tiles_carry_a_channel() {
        let seed = WorldSeed(42);
        let fields = WorldFields::generate(seed, 96);
        let flora = flora::settle::settle(seed, &fields, 24);
        if let Some(tile) = (0..fields.grid().cells()).find(|&t| fields.water[t] == Water::River) {
            let map = generate(
                seed,
                &fields,
                &flora,
                TileId(tile as u32),
                false,
                &[],
                &flora.density,
            );
            let river = map.water.iter().filter(|w| **w == Water::River).count();
            assert!(
                river > 200,
                "river channel must cross the local map: {river}"
            );
        }
    }
}
