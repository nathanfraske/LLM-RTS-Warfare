//! Fauna: per-world animal genomes and their tile populations — the mobile
//! kingdom of the living world (docs/19-ecology-and-subsistence.md).
//!
//! No authored taxonomy: a genome is a point on continuous **trait axes** —
//! `diet` (plant-eater ↔ flesh-eater) and `water` (terrestrial ↔ aquatic) —
//! plus climate tolerances. "Grazer", "predator", "fish" are descriptions of
//! trait space, not types; omnivores and semi-aquatic hunters emerge.
//! `dynamics` runs the monthly trophic tick.

pub mod dynamics;
mod genomes;

pub use genomes::{FaunaSpecies, generate_species};

use anatomy::Substance;
use sim_events::WorldSeed;
use tuning::{Bodies, Ecology};
use world_map::{Water, WorldFields};
use world_schema::Quantity;

/// All wild populations, species-major: `pop[s * cells + tile]`.
#[derive(Debug)]
pub struct Fauna {
    pub species: Vec<FaunaSpecies>,
    /// The world's substance palette (docs/23) — what bodies are built
    /// from and what runs in them.
    pub substances: Vec<Substance>,
    pub pop: Vec<Quantity>,
    cells: usize,
}

impl Fauna {
    /// Generate the substance palette, the species with their bodies, and
    /// seed populations across the world.
    #[must_use]
    pub fn genesis(
        seed: WorldSeed,
        fields: &WorldFields,
        flora_density: &[u8],
        eco: &Ecology,
        bod: &Bodies,
    ) -> Self {
        let substances = anatomy::substances(seed, bod.substances);
        let species = generate_species(seed, eco.fauna_species, &substances, bod);
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
            substances,
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

    /// The most numerous species on a tile, for inspection and describes.
    #[must_use]
    pub fn top_species_at(&self, tile: usize) -> Option<&FaunaSpecies> {
        self.species
            .iter()
            .filter(|s| self.at(s.id as usize, tile) > Quantity::from_num(2))
            .max_by_key(|s| self.at(s.id as usize, tile).to_bits())
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
