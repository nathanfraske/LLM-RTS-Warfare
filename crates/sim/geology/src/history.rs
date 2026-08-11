//! The geologic history (docs/29 §2): a handful of generated events —
//! uplifts read off the peaks, basins off the lowlands, intrusions drawn
//! near the mountains, plumes wherever the deep fire rose — compiled into
//! per-tile columns. Deposits are consequences, never placements.

use sim_events::WorldSeed;
use tuning::Deep;
use world_map::WorldFields;

use crate::{Geology, Mineral, Vein, draw};

/// Run the history and compile the columns.
pub(crate) fn compile(
    seed: WorldSeed,
    fields: &WorldFields,
    minerals: &[Mineral],
    deep: &Deep,
) -> Geology {
    let cells = fields.grid().cells();
    let uplift_centers = peaks(fields, deep.uplifts);
    let basin_centers = interior_lowlands(fields, deep.basins);

    // Bedrock: nearest event decides the region's country rock, its
    // character biased by what kind of event it was.
    let hard: Vec<u16> = pick_by(minerals, |m| m.hardness_milli);
    let soft: Vec<u16> = pick_by(minerals, |m| m.solubility_milli.max(m.energy_milli));
    let mut bedrock = vec![hard[0]; cells];
    for (tile, bed) in bedrock.iter_mut().enumerate() {
        let (x, y) = fields.grid().xy(tile);
        let du = nearest(&uplift_centers, fields, x, y);
        let db = nearest(&basin_centers, fields, x, y);
        let (bank, which) = if du <= db { (&hard, du) } else { (&soft, db) };
        // Regions rotate through their bank so neighboring provinces of
        // the same family still differ.
        *bed = bank[(which as usize / 9 + tile / (cells / 8 + 1)) % bank.len().max(1)];
    }

    // Faults: lines between successive uplift roots.
    let mut faults = vec![false; cells];
    for pair in uplift_centers.windows(2) {
        mark_line(fields, &mut faults, pair[0], pair[1]);
    }

    // Intrusions: metal halos drawn near uplifts and faults; the vein
    // surfaces where the cover is thin (high ground, stripped by erosion).
    let metallic: Vec<u16> = pick_by(minerals, |m| m.metal_milli);
    let mut veins: Vec<Option<Vein>> = vec![None; cells];
    for i in 0..deep.intrusions {
        let anchor = uplift_centers[usize::try_from(draw(seed, 0x1000 | u64::from(i)))
            .unwrap_or(0)
            % uplift_centers.len().max(1)];
        let (ax, ay) = fields.grid().xy(anchor);
        let jx = i64::from(ax)
            + i64::try_from(draw(seed, 0x2000 | u64::from(i)) % 17).expect("small")
            - 8;
        let jy = i64::from(ay)
            + i64::try_from(draw(seed, 0x3000 | u64::from(i)) % 17).expect("small")
            - 8;
        let mineral = metallic[usize::try_from(draw(seed, 0x4000 | u64::from(i))).unwrap_or(0)
            % metallic.len().max(1)];
        halo(fields, &mut veins, (jx, jy), mineral, seed, i, deep);
    }

    // Caves: soluble bedrock plus water.
    let caves = (0..cells)
        .map(|tile| {
            if fields.elevation[tile] < 0 {
                return 0;
            }
            let bed = &minerals[bedrock[tile] as usize];
            if bed.solubility_milli >= deep.cave_solubility
                && fields.moisture[tile] >= deep.cave_moisture
            {
                let size = u32::from(bed.solubility_milli - deep.cave_solubility) / 2
                    + u32::from(fields.moisture[tile]) / 4;
                u8::try_from(size.min(255)).expect("clamped")
            } else {
                0
            }
        })
        .collect();

    // Plumes: the vents and their eruption clocks.
    let mut vents = vec![0u8; cells];
    let mut schedules = Vec::new();
    for p in 0..deep.plumes {
        let tile = land_draw(seed, fields, 0x5000 | u64::from(p));
        let strength =
            u8::try_from(120 + draw(seed, 0x6000 | u64::from(p)) % 136).expect("bounded");
        vents[tile] = strength;
        let span = u64::from(deep.eruption_max_months - deep.eruption_min_months).max(1);
        let period = u16::try_from(
            u64::from(deep.eruption_min_months) + draw(seed, 0x7000 | u64::from(p)) % span,
        )
        .expect("bounded");
        let phase = u16::try_from(draw(seed, 0x8000 | u64::from(p)) % u64::from(period.max(1)))
            .expect("bounded");
        schedules.push((tile as u32, period, phase));
    }
    schedules.sort_unstable();

    // Quakes: epicenters seeded along the faults, each with its own clock.
    let fault_tiles: Vec<usize> = (0..cells)
        .filter(|&t| faults[t] && fields.elevation[t] >= 0)
        .collect();
    let mut quake_clocks = Vec::new();
    if !fault_tiles.is_empty() {
        for q in 0..deep.quakes {
            let tile = fault_tiles[usize::try_from(draw(seed, 0xA000 | u64::from(q))).unwrap_or(0)
                % fault_tiles.len()];
            let span = u64::from(deep.quake_max_months - deep.quake_min_months).max(1);
            let period = u16::try_from(
                u64::from(deep.quake_min_months) + draw(seed, 0xB000 | u64::from(q)) % span,
            )
            .expect("bounded");
            let phase = u16::try_from(draw(seed, 0xC000 | u64::from(q)) % u64::from(period.max(1)))
                .expect("bounded");
            quake_clocks.push((tile as u32, period, phase));
        }
    }
    quake_clocks.sort_unstable();

    Geology {
        minerals: minerals.to_vec(),
        bedrock,
        veins,
        faults,
        caves,
        vents,
        schedules,
        quake_clocks,
    }
}

/// The K highest land tiles, well separated — the record of the uplifts.
fn peaks(fields: &WorldFields, count: u8) -> Vec<usize> {
    ranked(fields, count, |t, f| {
        if f.elevation[t] < 0 {
            i64::MIN
        } else {
            i64::from(f.elevation[t])
        }
    })
}

/// The K lowest interior land tiles — the record of the basins.
fn interior_lowlands(fields: &WorldFields, count: u8) -> Vec<usize> {
    ranked(fields, count, |t, f| {
        if f.elevation[t] < 0 || world_map::tiles::coastal(f, t) {
            i64::MIN
        } else {
            -i64::from(f.elevation[t])
        }
    })
}

/// Greedy best-K with a minimum separation so events spread out.
fn ranked(
    fields: &WorldFields,
    count: u8,
    score: impl Fn(usize, &WorldFields) -> i64,
) -> Vec<usize> {
    let cells = fields.grid().cells();
    let min_sep = i64::from(fields.size / 8).max(4);
    let mut order: Vec<usize> = (0..cells).collect();
    order.sort_by_key(|&t| std::cmp::Reverse(score(t, fields)));
    let mut chosen: Vec<usize> = Vec::new();
    for t in order {
        if score(t, fields) == i64::MIN || chosen.len() >= usize::from(count) {
            break;
        }
        let (x, y) = fields.grid().xy(t);
        let clear = chosen.iter().all(|&c| {
            let (cx, cy) = fields.grid().xy(c);
            (i64::from(x) - i64::from(cx))
                .abs()
                .max((i64::from(y) - i64::from(cy)).abs())
                >= min_sep
        });
        if clear {
            chosen.push(t);
        }
    }
    if chosen.is_empty() {
        chosen.push(0);
    }
    chosen
}

fn nearest(centers: &[usize], fields: &WorldFields, x: u32, y: u32) -> i64 {
    centers
        .iter()
        .map(|&c| {
            let (cx, cy) = fields.grid().xy(c);
            (i64::from(x) - i64::from(cx))
                .abs()
                .max((i64::from(y) - i64::from(cy)).abs())
        })
        .min()
        .unwrap_or(i64::MAX)
}

fn pick_by(minerals: &[Mineral], key: impl Fn(&Mineral) -> u16) -> Vec<u16> {
    let mut ids: Vec<u16> = (0..minerals.len() as u16).collect();
    ids.sort_by_key(|&i| std::cmp::Reverse(key(&minerals[i as usize])));
    ids.truncate((minerals.len() / 3).max(1));
    ids
}

/// Mark the fault line between two roots.
fn mark_line(fields: &WorldFields, faults: &mut [bool], a: usize, b: usize) {
    let (ax, ay) = fields.grid().xy(a);
    let (bx, by) = fields.grid().xy(b);
    let steps = (i64::from(ax) - i64::from(bx))
        .abs()
        .max((i64::from(ay) - i64::from(by)).abs())
        .max(1);
    for s in 0..=steps {
        let x = i64::from(ax) + (i64::from(bx) - i64::from(ax)) * s / steps;
        let y = i64::from(ay) + (i64::from(by) - i64::from(ay)) * s / steps;
        faults[(y as usize) * fields.size as usize + x as usize] = true;
    }
}

/// A vein halo around an intrusion point: richest at heart, surfacing
/// where the ground is high (the cover long since stripped).
fn halo(
    fields: &WorldFields,
    veins: &mut [Option<Vein>],
    (cx, cy): (i64, i64),
    mineral: u16,
    seed: WorldSeed,
    salt: u8,
    deep: &Deep,
) {
    let r = i64::from(deep.vein_radius);
    for dy in -r..=r {
        for dx in -r..=r {
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= i64::from(fields.size) || y >= i64::from(fields.size) {
                continue;
            }
            let tile = (y as usize) * fields.size as usize + x as usize;
            if fields.elevation[tile] < 0 {
                continue;
            }
            let dist = dx.abs().max(dy.abs());
            let roll = draw(seed, 0x9000 | u64::from(salt) << 8 | (tile as u64 & 0xFF)) % 100;
            if roll > 55 {
                continue; // veins are patchy, not discs
            }
            let richness =
                u8::try_from(((r + 1 - dist) * 200 / (r + 1)).clamp(30, 220)).expect("bounded");
            let depth = u8::try_from((2_600 - fields.elevation[tile]).clamp(20, 240) / 10)
                .expect("bounded");
            let candidate = Vein {
                mineral,
                depth,
                richness,
            };
            let better = veins[tile].is_none_or(|v| v.richness < candidate.richness);
            if better {
                veins[tile] = Some(candidate);
            }
        }
    }
}

/// A deterministic land tile for a plume.
fn land_draw(seed: WorldSeed, fields: &WorldFields, salt: u64) -> usize {
    let cells = fields.grid().cells();
    let mut tile = usize::try_from(draw(seed, salt)).unwrap_or(0) % cells;
    for bump in 0..cells {
        if fields.elevation[tile] >= 0 {
            return tile;
        }
        tile = (usize::try_from(draw(seed, salt ^ (bump as u64) << 16)).unwrap_or(0)) % cells;
    }
    tile
}
