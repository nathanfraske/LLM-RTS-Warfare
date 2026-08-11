//! Fauna: per-world animal genomes and their tile populations — the mobile
//! kingdom of the living world (docs/19-ecology-and-subsistence.md).
//!
//! No authored taxonomy: a genome is a point on continuous **trait axes** —
//! `diet` (plant-eater ↔ flesh-eater) and `water` (terrestrial ↔ aquatic) —
//! plus climate tolerances. "Grazer", "predator", "fish" are descriptions of
//! trait space, not types; omnivores and semi-aquatic hunters emerge.
//! `dynamics` runs the monthly trophic tick.

pub mod dynamics;

use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use tuning::Ecology;
use world_map::{Water, WorldFields};
use world_schema::{Quantity, Tick};

const FAUNAGEN: SystemId = SystemId(10);

/// An animal genome: climate tolerances plus continuous trait axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaunaSpecies {
    pub id: u16,
    /// 0 = pure plant-eater … 1000 = pure flesh-eater.
    pub diet_milli: u16,
    /// 0 = fully terrestrial … 1000 = fully aquatic.
    pub water_milli: u16,
    pub t_opt: i16,
    pub t_width: i16,
    pub m_opt: u8,
    pub m_width: u8,
    /// Monthly intrinsic growth ×1000 (falls out of diet at generation:
    /// plant-eaters breed fast, flesh-eaters slow).
    pub repro_milli: u16,
    /// Food yielded per unit of biomass taken, ×1000.
    pub edible_milli: u16,
}

impl FaunaSpecies {
    #[must_use]
    pub fn plant_frac(&self) -> Quantity {
        Quantity::ONE - Quantity::from_num(self.diet_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn flesh_frac(&self) -> Quantity {
        Quantity::from_num(self.diet_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn land_frac(&self) -> Quantity {
        Quantity::ONE - Quantity::from_num(self.water_milli) / Quantity::from_num(1000)
    }

    #[must_use]
    pub fn water_frac(&self) -> Quantity {
        Quantity::from_num(self.water_milli) / Quantity::from_num(1000)
    }

    /// A human-readable description of where this genome sits in trait space.
    #[must_use]
    pub fn describe(&self) -> String {
        let diet = match self.diet_milli {
            0..=300 => "plant-eater",
            301..=650 => "omnivore",
            _ => "flesh-eater",
        };
        let habitat = match self.water_milli {
            0..=250 => "land",
            251..=650 => "shoreline",
            _ => "water",
        };
        format!("{habitat} {diet}")
    }
}

/// All wild populations, species-major: `pop[s * cells + tile]`.
#[derive(Debug)]
pub struct Fauna {
    pub species: Vec<FaunaSpecies>,
    pub pop: Vec<Quantity>,
    cells: usize,
}

/// Genomes sampled across trait space and climate space — every band of
/// diet × habitat gets contenders, none is guaranteed to thrive anywhere.
#[must_use]
pub fn generate_species(seed: WorldSeed, count: u16) -> Vec<FaunaSpecies> {
    const T_BINS: [i16; 4] = [-60, 60, 170, 270];
    (0..count)
        .map(|k| {
            let d = |salt: u64| rng::draw(seed, Tick::ZERO, FAUNAGEN, u64::from(k) << 8 | salt);
            // Stratify trait space: plant-eaters common, omnivores and
            // flesh-eaters rarer; a water band and a shoreline band exist.
            let diet_milli = match k % 6 {
                0 | 1 | 3 => (d(1) % 300) as u16,
                4 => (350 + d(1) % 300) as u16,
                _ => (650 + d(1) % 350) as u16,
            };
            let water_milli = match k % 12 {
                8..=11 => (750 + d(2) % 250) as u16,
                7 => (350 + d(2) % 300) as u16,
                _ => (d(2) % 250) as u16,
            };
            let t_center = T_BINS[k as usize % T_BINS.len()];
            // Breeding speed falls out of diet: grass breeds mice, meat breeds wolves.
            let repro_milli = (230 - u64::from(diet_milli) * 14 / 100 + d(5) % 60) as u16;
            FaunaSpecies {
                id: k,
                diet_milli,
                water_milli,
                t_opt: t_center + (d(3) % 80) as i16 - 40,
                t_width: 90 + (d(4) % 110) as i16,
                m_opt: (40 + (d(6) % 180)) as u8,
                m_width: (70 + (d(7) % 90)) as u8,
                repro_milli,
                edible_milli: (700 + d(8) % 400) as u16,
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
        let species = generate_species(seed, eco.fauna_species);
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

    /// Biomass a hunter can reach on this tile, weighted by how terrestrial
    /// each genome is.
    #[must_use]
    pub fn huntable(&self, tile: usize) -> Quantity {
        self.species.iter().fold(Quantity::ZERO, |acc, s| {
            acc + self.at(s.id as usize, tile) * s.land_frac()
        })
    }

    /// Water-dwelling biomass reachable from this tile (own + neighbors).
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
        self.species.iter().fold(Quantity::ZERO, |acc, s| {
            acc + self.at(s.id as usize, tile) * s.water_frac()
        })
    }

    /// Take up to `wanted` land-side biomass. Returns food (biomass × edibility).
    pub fn hunt(&mut self, tile: usize, wanted: Quantity, eco: &Ecology) -> Quantity {
        self.take(tile, wanted, false, eco)
    }

    /// Take up to `wanted` water-side biomass around the tile.
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

    /// Capture live land plant-eaters for herding; returns head captured.
    pub fn capture_grazers(&mut self, tile: usize, wanted: Quantity, eco: &Ecology) -> Quantity {
        let mut captured = Quantity::ZERO;
        for si in 0..self.species.len() {
            let s = &self.species[si];
            if captured >= wanted
                || s.plant_frac() < Quantity::from_num(0.6)
                || s.land_frac() < Quantity::from_num(0.6)
            {
                continue;
            }
            let have = self.at(si, tile);
            let take = (wanted - captured).min(have * Quantity::from_num(eco.refuge_frac));
            self.set(si, tile, have - take);
            captured += take;
        }
        captured
    }

    fn take(&mut self, tile: usize, wanted: Quantity, water_side: bool, eco: &Ecology) -> Quantity {
        let mut food = Quantity::ZERO;
        let mut left = wanted;
        for si in 0..self.species.len() {
            if left <= Quantity::ZERO {
                break;
            }
            let s = &self.species[si];
            let side = if water_side {
                s.water_frac()
            } else {
                s.land_frac()
            };
            if side < Quantity::from_num(0.05) {
                continue;
            }
            let have = self.at(si, tile);
            let reachable = have * side;
            // Never strip a stock below its refuge share in one month.
            let take = left
                .min(reachable - reachable * Quantity::from_num(eco.refuge_frac))
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

/// Habitat ceiling of a tile for one genome: where its body can live,
/// interpolated between the green land and the stocked waters, scaled down
/// for flesh-heavy diets (hunters are always rarer than what they hunt).
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
    let land_k = if fields.elevation[tile] < 0 {
        Quantity::ZERO
    } else {
        Quantity::from_num(eco.grazer_k_full) * fit * Quantity::from_num(flora_density)
            / Quantity::from_num(255)
    };
    let bounty = match fields.water[tile] {
        Water::River => eco.aquatic_k_river,
        Water::Lake => eco.aquatic_k_lake,
        Water::Ocean => eco.aquatic_k_ocean,
        Water::Dry => 0.0,
    };
    let water_k = Quantity::from_num(bounty) * fit;
    let habitat = land_k * s.land_frac() + water_k * s.water_frac();
    let hunter_scale = Quantity::from_num(eco.predator_habitat_k / eco.grazer_k_full);
    habitat * (s.plant_frac() + s.flesh_frac() * hunter_scale)
}
