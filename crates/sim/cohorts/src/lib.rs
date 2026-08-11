//! Statistical population cohorts and their demographic dynamics
//! (docs/02-simulation-core.md — the cohort layer; structure-of-arrays by design).

use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use tuning::Society;
use world_schema::{Quantity, SpeciesId, Tick, TileId};

const DEMOGRAPHICS: SystemId = SystemId(2);

/// Statistical bucket key. Occupation joins the key at M1
/// (docs/02-simulation-core.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CohortKey {
    pub tile: TileId,
    pub species: SpeciesId,
}

impl CohortKey {
    /// Stable RNG salt: dynamics must not depend on insertion order.
    #[must_use]
    fn rng_key(self) -> u64 {
        (u64::from(self.tile.0) << 16) | u64::from(self.species.0)
    }
}

/// Per-cohort monthly parameters, composed by the caller from species
/// modifiers and tile carrying capacity (docs/14-bands-and-councils.md).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortDrive {
    pub birth_rate: Quantity,
    pub death_rate: Quantity,
    /// Food eaten over food needed last month (1 = fed). Drives everything:
    /// births fall and deaths rise as the larder empties
    /// (docs/19-ecology-and-subsistence.md — the capacity formula is gone).
    pub nutrition: Quantity,
}

/// Aggregate demographic result of one closed month.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthDelta {
    pub births: Quantity,
    pub deaths: Quantity,
    /// Cohorts that crossed the famine threshold this month.
    pub famines: Vec<CohortKey>,
}

/// All cohorts, structure-of-arrays: `keys[i]` describes `population[i]`.
#[derive(Debug, Default)]
pub struct Cohorts {
    keys: Vec<CohortKey>,
    population: Vec<Quantity>,
}

impl Cohorts {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = (CohortKey, Quantity)> + '_ {
        self.keys
            .iter()
            .copied()
            .zip(self.population.iter().copied())
    }

    #[must_use]
    pub fn population_of(&self, key: CohortKey) -> Quantity {
        self.index_of(key)
            .map_or(Quantity::ZERO, |i| self.population[i])
    }

    #[must_use]
    pub fn total_population(&self) -> Quantity {
        self.population
            .iter()
            .fold(Quantity::ZERO, |acc, p| acc + *p)
    }

    /// Add people to a cohort, creating it if absent.
    pub fn add(&mut self, key: CohortKey, amount: Quantity) {
        if let Some(i) = self.index_of(key) {
            self.population[i] += amount;
        } else {
            self.keys.push(key);
            self.population.push(amount);
        }
    }

    /// Remove up to `amount` people; returns how many actually moved.
    pub fn remove(&mut self, key: CohortKey, amount: Quantity) -> Quantity {
        let Some(i) = self.index_of(key) else {
            return Quantity::ZERO;
        };
        let moved = self.population[i].min(amount).max(Quantity::ZERO);
        self.population[i] -= moved;
        moved
    }

    fn index_of(&self, key: CohortKey) -> Option<usize> {
        self.keys.iter().position(|&k| k == key)
    }

    /// Close a month: births and deaths per cohort, crowding-adjusted.
    /// `drives` must align with `entries()` order.
    pub fn tick_month(
        &mut self,
        seed: WorldSeed,
        tick: Tick,
        drives: &[CohortDrive],
        soc: &Society,
    ) -> MonthDelta {
        assert_eq!(
            drives.len(),
            self.keys.len(),
            "drives must align with cohorts"
        );
        let mut delta = MonthDelta {
            births: Quantity::ZERO,
            deaths: Quantity::ZERO,
            famines: Vec::new(),
        };
        for (i, pop) in self.population.iter_mut().enumerate() {
            if *pop <= Quantity::ZERO {
                continue;
            }
            let key = self.keys[i];
            let drive = drives[i];
            // Jitter in [0.75, 1.25): monthly variance around the base rates.
            let jitter = |salt: u64| {
                Quantity::from_num(0.75)
                    + rng::unit(seed, tick, DEMOGRAPHICS, key.rng_key() ^ salt)
                        * Quantity::from_num(0.5)
            };
            let birth_factor = drive.nutrition.clamp(
                Quantity::from_num(soc.birth_factor_min),
                Quantity::from_num(soc.birth_factor_max),
            );
            let death_factor = (Quantity::from_num(soc.death_offset) - drive.nutrition).clamp(
                Quantity::from_num(soc.death_factor_min),
                Quantity::from_num(soc.death_factor_max),
            );
            let births = *pop * drive.birth_rate * birth_factor * jitter(0x5EED_0000_0000_0001);
            let deaths = *pop * drive.death_rate * death_factor * jitter(0x5EED_0000_0000_0002);
            *pop = (*pop + births - deaths).max(Quantity::ZERO);
            delta.births += births;
            delta.deaths += deaths;
            if drive.nutrition < Quantity::from_num(soc.famine_nutrition) {
                delta.famines.push(key);
            }
        }
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(p: u32) -> CohortKey {
        CohortKey {
            tile: TileId(p),
            species: SpeciesId(0),
        }
    }

    fn drives(n: usize, nutrition: f64) -> Vec<CohortDrive> {
        vec![
            CohortDrive {
                birth_rate: Quantity::from_num(0.006),
                death_rate: Quantity::from_num(0.0045),
                nutrition: Quantity::from_num(nutrition),
            };
            n
        ]
    }

    #[test]
    fn population_is_conserved_exactly_including_transfers() {
        let seed = WorldSeed(42);
        let mut cohorts = Cohorts::new();
        for p in 0..12 {
            cohorts.add(key(p), Quantity::from_num(200 + i64::from(p) * 37));
        }
        let initial = cohorts.total_population();
        let mut running = initial;
        for month in 1..=24u64 {
            let delta = cohorts.tick_month(
                seed,
                Tick(month * 720),
                &drives(cohorts.len(), 1.0),
                &Society::default(),
            );
            running = running + delta.births - delta.deaths;
            // A migration mid-history must not break the ledger.
            let moved = cohorts.remove(key(0), Quantity::from_num(15));
            cohorts.add(key(99), moved);
        }
        assert_eq!(cohorts.total_population(), running);
        assert_ne!(cohorts.total_population(), initial);
    }

    #[test]
    fn hunger_turns_growth_into_famine() {
        let seed = WorldSeed(7);
        let mut cohorts = Cohorts::new();
        cohorts.add(key(0), Quantity::from_num(1_000));
        let hungry = drives(1, 0.5); // half-fed
        let delta = cohorts.tick_month(seed, Tick(720), &hungry, &Society::default());
        assert_eq!(delta.famines, vec![key(0)]);
        assert!(delta.deaths > delta.births, "hunger must cost lives");
    }

    #[test]
    fn dynamics_ignore_insertion_order() {
        let seed = WorldSeed(9);
        let mut a = Cohorts::new();
        let mut b = Cohorts::new();
        for p in 0..8 {
            a.add(key(p), Quantity::from_num(300));
        }
        for p in (0..8).rev() {
            b.add(key(p), Quantity::from_num(300));
        }
        a.tick_month(seed, Tick(720), &drives(8, 1.0), &Society::default());
        b.tick_month(seed, Tick(720), &drives(8, 1.0), &Society::default());
        assert_eq!(a.population_of(key(3)), b.population_of(key(3)));
        assert_eq!(a.total_population(), b.total_population());
    }
}
