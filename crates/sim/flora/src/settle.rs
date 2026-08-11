//! Ecological settling: species spread from origin sites and compete for
//! cells; incumbents hold with a defender's bonus (docs/13-worldgen.md).
//! Produces contiguous ranges, frontiers, endemism. Upgrade path: sim-time
//! ecology, then mutation/speciation.

use crate::{FloraSpecies, generate_species};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_map::{Water, WorldFields};
use world_schema::Tick;

const SETTLE: SystemId = SystemId(6);
const SPREAD: SystemId = SystemId(7);

pub const NO_FLORA: u16 = u16::MAX;
const ORIGIN_PROBES: u64 = 160;
const MIN_FITNESS: f32 = 0.06;
const DEFENDER_BONUS: f32 = 1.15;

/// Per-round claim chance in percent — trees creep, grass races. The jitter
/// also breaks up square Chebyshev wavefronts into organic frontiers.
fn spread_chance(form: crate::GrowthForm) -> u64 {
    match form {
        crate::GrowthForm::Grass => 80,
        crate::GrowthForm::Shrub => 64,
        crate::GrowthForm::Tree => 48,
    }
}

/// The settled vegetation layers.
#[derive(Debug)]
pub struct FloraMap {
    pub species: Vec<FloraSpecies>,
    /// Dominant species index per cell, `NO_FLORA` if bare.
    pub occupant: Vec<u16>,
    /// Vegetation density 0–255 (winner's fitness, scaled).
    pub density: Vec<u8>,
}

/// Generate a world's species and settle them onto the fields.
#[must_use]
pub fn settle(seed: WorldSeed, fields: &WorldFields, species_count: u16) -> FloraMap {
    let species = generate_species(seed, species_count);
    let grid = fields.grid();
    let n = grid.cells();
    let mut occupant = vec![NO_FLORA; n];
    let mut score = vec![0.0f32; n];

    let fitness = |s: &FloraSpecies, i: usize| {
        if fields.water[i] == Water::Dry || fields.water[i] == Water::River {
            s.fitness(
                fields.elevation[i],
                fields.temperature[i],
                fields.moisture[i],
            )
        } else {
            0.0
        }
    };

    // Origins scale with land area so big continents get reached.
    let origins_per_species =
        (fields.land_cells() as usize / (usize::from(species_count.max(1)) * 400)).clamp(4, 32);
    let mut frontiers: Vec<Vec<usize>> = Vec::with_capacity(species.len());
    for s in &species {
        let mut probes: Vec<(usize, f32)> = (0..ORIGIN_PROBES)
            .map(|p| {
                let cell = (rng::draw(seed, Tick::ZERO, SETTLE, u64::from(s.id.0) << 32 | p)
                    % n as u64) as usize;
                (cell, fitness(s, cell))
            })
            .collect();
        probes.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut frontier = Vec::new();
        for &(cell, fit) in probes.iter().take(origins_per_species) {
            if fit > MIN_FITNESS && fit > score[cell] {
                occupant[cell] = s.id.0;
                score[cell] = fit;
                frontier.push(cell);
            }
        }
        frontiers.push(frontier);
    }

    // Competitive spread, species in id order each round — deterministic.
    // Rounds scale with map size so ranges can cross continents.
    let rounds = (fields.size / 5).max(48);
    for round in 0..rounds {
        for (si, s) in species.iter().enumerate() {
            let chance = spread_chance(s.form);
            let mut next = Vec::new();
            for &cell in &frontiers[si] {
                let mut blocked_by_roll = false;
                let (neighbors, count) = grid.neighbors8(cell);
                for &nb in &neighbors[..count] {
                    if occupant[nb] == s.id.0 {
                        continue;
                    }
                    let fit = fitness(s, nb);
                    if fit > MIN_FITNESS && fit > score[nb] * DEFENDER_BONUS {
                        let roll = rng::draw(
                            seed,
                            Tick(u64::from(round)),
                            SPREAD,
                            (nb as u64) << 16 | u64::from(s.id.0),
                        ) % 100;
                        if roll < chance {
                            occupant[nb] = s.id.0;
                            score[nb] = fit;
                            next.push(nb);
                        } else {
                            blocked_by_roll = true;
                        }
                    }
                }
                // A failed roll keeps the frontier alive to retry next round.
                if blocked_by_roll {
                    next.push(cell);
                }
            }
            frontiers[si] = next;
        }
    }

    let density = score
        .iter()
        .map(|&f| (f.clamp(0.0, 1.2) / 1.2 * 255.0) as u8)
        .collect();
    FloraMap {
        species,
        occupant,
        density,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settling_is_deterministic_and_covers_fertile_land() {
        let fields = WorldFields::generate(WorldSeed(42), 96);
        let a = settle(WorldSeed(42), &fields, 24);
        let b = settle(WorldSeed(42), &fields, 24);
        assert_eq!(a.occupant, b.occupant);
        assert_eq!(a.density, b.density);

        let fertile: Vec<usize> = (0..fields.grid().cells())
            .filter(|&i| fields.cell_fertility[i] > 60)
            .collect();
        assert!(!fertile.is_empty());
        let vegetated = fertile
            .iter()
            .filter(|&&i| a.occupant[i] != NO_FLORA)
            .count();
        let coverage = vegetated as f32 / fertile.len() as f32;
        assert!(
            coverage > 0.6,
            "fertile land should mostly vegetate: {coverage}"
        );

        let distinct: std::collections::BTreeSet<u16> = a
            .occupant
            .iter()
            .copied()
            .filter(|&o| o != NO_FLORA)
            .collect();
        assert!(
            distinct.len() >= 6,
            "expect real diversity: {}",
            distinct.len()
        );
    }
}
