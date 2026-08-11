//! Fauna: per-world animal genomes and their tile populations — the mobile
//! kingdom of the living world (docs/19-ecology-and-subsistence.md).
//! Grazers eat flora, predators eat grazers, aquatic species fill the waters;
//! humans hunt, fish, and capture from all of it. `dynamics` runs the monthly
//! trophic tick.

pub mod dynamics;

use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use tuning::Ecology;
use world_map::{Water, WorldFields};
use world_schema::{Quantity, Tick};

const FAUNAGEN: SystemId = SystemId(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trophic {
    Grazer,
    Predator,
    Aquatic,
}

/// An animal genome: climate tolerances plus a trophic role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaunaSpecies {
    pub id: u16,
    pub trophic: Trophic,
    pub t_opt: i16,
    pub t_width: i16,
    pub m_opt: u8,
    pub m_width: u8,
    /// Monthly intrinsic growth ×1000.
    pub repro_milli: u16,
    /// Food yielded per unit of biomass taken, ×1000.
    pub edible_milli: u16,
}

/// All wild populations, species-major: `pop[s * cells + tile]`.
#[derive(Debug)]
pub struct Fauna {
    pub species: Vec<FaunaSpecies>,
    pub pop: Vec<Quantity>,
    cells: usize,
}

pub const DEFAULT_SPECIES: u16 = 12;

/// Stratified genomes: five grazers, three predators, four aquatic.
#[must_use]
pub fn generate_species(seed: WorldSeed, count: u16) -> Vec<FaunaSpecies> {
    const T_BINS: [i16; 4] = [-60, 60, 170, 270];
    (0..count)
        .map(|k| {
            let d = |salt: u64| rng::draw(seed, Tick::ZERO, FAUNAGEN, u64::from(k) << 8 | salt);
            let trophic = match k % 12 {
                0..=4 => Trophic::Grazer,
                5..=7 => Trophic::Predator,
                _ => Trophic::Aquatic,
            };
            let t_center = T_BINS[k as usize % T_BINS.len()];
            FaunaSpecies {
                id: k,
                trophic,
                t_opt: t_center + (d(1) % 80) as i16 - 40,
                t_width: 90 + (d(2) % 110) as i16,
                m_opt: (40 + (d(3) % 180)) as u8,
                m_width: (70 + (d(4) % 90)) as u8,
                repro_milli: match trophic {
                    Trophic::Grazer => (140 + d(5) % 120) as u16,
                    Trophic::Predator => (60 + d(5) % 60) as u16,
                    Trophic::Aquatic => (160 + d(5) % 140) as u16,
                },
                edible_milli: (700 + d(6) % 400) as u16,
            }
        })
        .collect()
}

impl Fauna {
    /// Generate species and seed their populations across the world.
    #[must_use]
    pub fn genesis(
        seed: WorldSeed,
        fields: &WorldFields,
        flora_density: &[u8],
        eco: &Ecology,
    ) -> Self {
        let species = generate_species(seed, DEFAULT_SPECIES);
        let cells = fields.grid().cells();
        let mut pop = vec![Quantity::ZERO; species.len() * cells];
        for s in &species {
            for t in 0..cells {
                let k = carrying(s, fields, flora_density[t], t, eco);
                if k > Quantity::from_num(4) {
                    pop[s.id as usize * cells + t] = k * Quantity::from_num(eco.genesis_fill);
                }
            }
        }
        Self {
            species,
            pop,
            cells,
        }
    }

    #[must_use]
    pub fn at(&self, species: usize, tile: usize) -> Quantity {
        self.pop[species * self.cells + tile]
    }

    pub fn set(&mut self, species: usize, tile: usize, value: Quantity) {
        self.pop[species * self.cells + tile] = value.max(Quantity::ZERO);
    }

    #[must_use]
    pub fn cells(&self) -> usize {
        self.cells
    }

    /// Land biomass a hunter can reach on this tile (grazers + predators).
    #[must_use]
    pub fn huntable(&self, tile: usize) -> Quantity {
        self.species
            .iter()
            .filter(|s| s.trophic != Trophic::Aquatic)
            .fold(Quantity::ZERO, |acc, s| acc + self.at(s.id as usize, tile))
    }

    /// Aquatic biomass reachable from this tile (own water + neighbors).
    #[must_use]
    pub fn fishable(&self, fields: &WorldFields, tile: usize) -> Quantity {
        let mut total = self.aquatic_at(tile);
        let (neighbors, n) = fields.grid().neighbors8(tile);
        for &nb in &neighbors[..n] {
            total += self.aquatic_at(nb);
        }
        total
    }

    fn aquatic_at(&self, tile: usize) -> Quantity {
        self.species
            .iter()
            .filter(|s| s.trophic == Trophic::Aquatic)
            .fold(Quantity::ZERO, |acc, s| acc + self.at(s.id as usize, tile))
    }

    /// Take up to `wanted` biomass from land species, largest stocks first.
    /// Returns food gained (biomass × edibility).
    pub fn hunt(&mut self, tile: usize, wanted: Quantity, eco: &Ecology) -> Quantity {
        self.take(tile, wanted, false, eco)
    }

    /// Take up to `wanted` biomass from aquatic species around the tile.
    pub fn fish(
        &mut self,
        fields: &WorldFields,
        tile: usize,
        wanted: Quantity,
        eco: &Ecology,
    ) -> Quantity {
        let mut food = self.take(tile, wanted, true, eco);
        if food < wanted {
            let (neighbors, n) = fields.grid().neighbors8(tile);
            for &nb in &neighbors[..n] {
                if food >= wanted {
                    break;
                }
                food += self.take(nb, wanted - food, true, eco);
            }
        }
        food
    }

    /// Capture live grazers for herding; returns head captured.
    pub fn capture_grazers(&mut self, tile: usize, wanted: Quantity, eco: &Ecology) -> Quantity {
        let mut captured = Quantity::ZERO;
        for si in 0..self.species.len() {
            if self.species[si].trophic != Trophic::Grazer || captured >= wanted {
                continue;
            }
            let have = self.at(si, tile);
            let take = (wanted - captured).min(have * Quantity::from_num(eco.refuge_frac));
            self.set(si, tile, have - take);
            captured += take;
        }
        captured
    }

    fn take(&mut self, tile: usize, wanted: Quantity, aquatic: bool, eco: &Ecology) -> Quantity {
        let mut food = Quantity::ZERO;
        let mut left = wanted;
        for si in 0..self.species.len() {
            let is_aquatic = self.species[si].trophic == Trophic::Aquatic;
            if is_aquatic != aquatic || left <= Quantity::ZERO {
                continue;
            }
            let have = self.at(si, tile);
            // Never strip a stock below its refuge share in one month.
            let take = left
                .min(have - have * Quantity::from_num(eco.refuge_frac))
                .max(Quantity::ZERO);
            self.set(si, tile, have - take);
            let edible =
                Quantity::from_num(self.species[si].edible_milli) / Quantity::from_num(1000);
            food += take * edible;
            left -= take;
        }
        food
    }
}

/// Carrying capacity of a tile for one species.
#[must_use]
pub fn carrying(
    s: &FaunaSpecies,
    fields: &WorldFields,
    flora_density: u8,
    tile: usize,
    eco: &Ecology,
) -> Quantity {
    let fit = species::bump_q(
        i32::from(fields.temperature[tile]),
        i32::from(s.t_opt),
        i32::from(s.t_width),
    ) * species::bump_q(
        i32::from(fields.moisture[tile]),
        i32::from(s.m_opt),
        i32::from(s.m_width),
    );
    match s.trophic {
        Trophic::Grazer => {
            if fields.elevation[tile] < 0 {
                Quantity::ZERO
            } else {
                Quantity::from_num(eco.grazer_k_full) * fit * Quantity::from_num(flora_density)
                    / Quantity::from_num(255)
            }
        }
        // Predator K comes from prey in `dynamics`; this is a habitat ceiling.
        Trophic::Predator => {
            if fields.elevation[tile] < 0 {
                Quantity::ZERO
            } else {
                Quantity::from_num(eco.predator_habitat_k) * fit
            }
        }
        Trophic::Aquatic => {
            let bounty = match fields.water[tile] {
                Water::River => eco.aquatic_k_river,
                Water::Lake => eco.aquatic_k_lake,
                Water::Ocean => eco.aquatic_k_ocean,
                Water::Dry => 0.0,
            };
            Quantity::from_num(bounty) * fit
        }
    }
}
