//! Province partition: best-candidate seeds over habitable land, multi-source
//! BFS growth into contiguous provinces, per-province summaries.

use std::collections::VecDeque;

use crate::WorldFields;
use crate::hydrology::Water;
use crate::terrain::{self, Terrain};
use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_schema::{ProvinceId, Quantity, Tick};

const PARTITION: SystemId = SystemId(4);

/// Sentinel in `province_of_cell` for water/unassigned cells.
pub const NO_PROVINCE: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Province {
    pub id: ProvinceId,
    pub center: (u32, u32),
    pub cells: u32,
    pub terrain: Terrain,
    /// Demographic growth multiplier, roughly `[0.5, 1.3]`.
    pub fertility: Quantity,
    /// Mean climate over the province's cells (deci-°C / 0–255).
    pub mean_temperature: i16,
    pub mean_moisture: u8,
    /// Land-adjacent provinces, sorted by id.
    pub neighbors: Vec<ProvinceId>,
    pub coastal: bool,
    pub riverine: bool,
    /// Fertile enough, with water access, to host dawn-of-time founders.
    pub habitable: bool,
}

/// Partition land into `count` contiguous provinces.
/// Returns provinces (indexed by id) and the per-cell assignment.
#[must_use]
pub fn partition(
    seed: WorldSeed,
    fields: &WorldFields,
    flora_density: &[u8],
    count: u32,
) -> (Vec<Province>, Vec<u32>) {
    let grid = fields.grid();
    let land: Vec<usize> = (0..grid.cells())
        .filter(|&i| fields.elevation[i] >= 0 && fields.water[i] != Water::Lake)
        .collect();
    let seeds = pick_seeds(seed, fields, &land, count);

    // Multi-source BFS: FIFO order makes the partition deterministic.
    let mut province_of_cell = vec![NO_PROVINCE; grid.cells()];
    let mut queue = VecDeque::new();
    for (p, &cell) in seeds.iter().enumerate() {
        province_of_cell[cell] = p as u32;
        queue.push_back(cell);
    }
    while let Some(i) = queue.pop_front() {
        let (neighbors, n) = grid.neighbors8(i);
        for &nb in &neighbors[..n] {
            if province_of_cell[nb] == NO_PROVINCE
                && fields.elevation[nb] >= 0
                && fields.water[nb] != Water::Lake
            {
                province_of_cell[nb] = province_of_cell[i];
                queue.push_back(nb);
            }
        }
    }

    attach_orphans(fields, &seeds, &mut province_of_cell);

    let provinces = summarize(fields, flora_density, &province_of_cell, &seeds);
    (provinces, province_of_cell)
}

/// Landmasses without a seed (islands) join the province of the nearest seed,
/// so every land cell is governed. Deterministic: index order, distance ties
/// break toward the lower province id.
fn attach_orphans(fields: &WorldFields, seeds: &[usize], province_of_cell: &mut [u32]) {
    if seeds.is_empty() {
        return;
    }
    let grid = fields.grid();
    let is_land = |i: usize| fields.elevation[i] >= 0 && fields.water[i] != Water::Lake;
    for i in 0..grid.cells() {
        if province_of_cell[i] != NO_PROVINCE || !is_land(i) {
            continue;
        }
        let (x, y) = grid.xy(i);
        let mut best = 0u32;
        let mut best_d = u64::MAX;
        for (p, &s) in seeds.iter().enumerate() {
            let (sx, sy) = grid.xy(s);
            let dx = i64::from(x) - i64::from(sx);
            let dy = i64::from(y) - i64::from(sy);
            let d = (dx * dx + dy * dy) as u64;
            if d < best_d {
                best_d = d;
                best = p as u32;
            }
        }
        let mut queue = VecDeque::from([i]);
        province_of_cell[i] = best;
        while let Some(c) = queue.pop_front() {
            let (neighbors, n) = grid.neighbors8(c);
            for &nb in &neighbors[..n] {
                if province_of_cell[nb] == NO_PROVINCE && is_land(nb) {
                    province_of_cell[nb] = best;
                    queue.push_back(nb);
                }
            }
        }
    }
}

/// Greedy best-candidate sampling: spread seeds apart, biased to fertile land.
fn pick_seeds(seed: WorldSeed, fields: &WorldFields, land: &[usize], count: u32) -> Vec<usize> {
    let grid = fields.grid();
    let mut seeds: Vec<usize> = Vec::new();
    let mut draw_index = 0u64;
    for _ in 0..count.min(land.len() as u32) {
        let mut best = land[0];
        let mut best_score = -1.0f32;
        for _ in 0..96 {
            let cell = land
                [(rng::draw(seed, Tick::ZERO, PARTITION, draw_index) % land.len() as u64) as usize];
            draw_index += 1;
            let (x, y) = grid.xy(cell);
            let spread = seeds
                .iter()
                .map(|&s| {
                    let (sx, sy) = grid.xy(s);
                    let (dx, dy) = (f64::from(x) - f64::from(sx), f64::from(y) - f64::from(sy));
                    (dx * dx + dy * dy) as f32
                })
                .fold(f32::MAX, f32::min)
                .min(1e12);
            let fert = 0.35 + f32::from(fields.cell_fertility[cell]) / 255.0;
            let score = spread.sqrt() * fert;
            if score > best_score {
                best_score = score;
                best = cell;
            }
        }
        seeds.push(best);
    }
    seeds
}

/// `Terrain` discriminant order, for vote tallies.
const LABELS: [Terrain; 7] = [
    Terrain::Ocean,
    Terrain::Lake,
    Terrain::Mountain,
    Terrain::Hills,
    Terrain::Tundra,
    Terrain::Desert,
    Terrain::Plains,
];

fn summarize(
    fields: &WorldFields,
    flora_density: &[u8],
    province_of_cell: &[u32],
    seeds: &[usize],
) -> Vec<Province> {
    let grid = fields.grid();
    let mut provinces: Vec<Province> = seeds
        .iter()
        .enumerate()
        .map(|(p, &cell)| Province {
            id: ProvinceId(p as u32),
            center: grid.xy(cell),
            cells: 0,
            terrain: Terrain::Plains,
            fertility: Quantity::ZERO,
            mean_temperature: 0,
            mean_moisture: 0,
            neighbors: Vec::new(),
            coastal: false,
            riverine: false,
            habitable: false,
        })
        .collect();

    let mut fert_sum = vec![0u64; provinces.len()];
    let mut flora_sum = vec![0u64; provinces.len()];
    let mut temp_sum = vec![0i64; provinces.len()];
    let mut moist_sum = vec![0u64; provinces.len()];
    let mut terrain_votes = vec![[0u32; 7]; provinces.len()];
    let mut adjacency = std::collections::BTreeSet::new();
    for i in 0..grid.cells() {
        let p = province_of_cell[i];
        if p == NO_PROVINCE {
            continue;
        }
        let p = p as usize;
        provinces[p].cells += 1;
        fert_sum[p] += u64::from(fields.cell_fertility[i]);
        flora_sum[p] += u64::from(flora_density[i]);
        temp_sum[p] += i64::from(fields.temperature[i]);
        moist_sum[p] += u64::from(fields.moisture[i]);
        let label = terrain::label(
            fields.elevation[i],
            fields.water[i],
            fields.temperature[i],
            fields.moisture[i],
        );
        terrain_votes[p][label as usize] += 1;
        if fields.water[i] == Water::River {
            provinces[p].riverine = true;
        }
        let (neighbors, n) = grid.neighbors8(i);
        for &nb in &neighbors[..n] {
            match fields.water[nb] {
                Water::Ocean => provinces[p].coastal = true,
                Water::Lake => provinces[p].riverine = true,
                _ => {}
            }
            let q = province_of_cell[nb];
            if q != NO_PROVINCE && q != p as u32 {
                adjacency.insert(((p as u32).min(q), (p as u32).max(q)));
            }
        }
    }
    for &(a, b) in &adjacency {
        provinces[a as usize].neighbors.push(ProvinceId(b));
        provinces[b as usize].neighbors.push(ProvinceId(a));
    }
    for province in &mut provinces {
        province.neighbors.sort_unstable();
        province.neighbors.dedup();
    }

    for (p, province) in provinces.iter_mut().enumerate() {
        let cells = u64::from(province.cells.max(1));
        province.mean_temperature = (temp_sum[p] / cells.cast_signed()) as i16;
        province.mean_moisture = (moist_sum[p] / cells) as u8;
        let fert = fert_sum[p] / cells;
        let flora = flora_sum[p] / cells;
        // Base fertility from climate, enriched by settled vegetation.
        let combined = fert * (70 + flora * 30 / 255) / 100;
        province.fertility = Quantity::from_num(0.5)
            + Quantity::from_num(combined) * Quantity::from_num(0.8) / Quantity::from_num(255);
        let vote = terrain_votes[p]
            .iter()
            .enumerate()
            .max_by_key(|(label, votes)| (**votes, usize::MAX - label))
            .map_or(6, |(label, _)| label);
        province.terrain = LABELS[vote];
        province.habitable =
            (province.coastal || province.riverine) && combined > 40 && province.cells >= 12;
    }
    provinces
}
