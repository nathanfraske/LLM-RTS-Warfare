//! Nations as territorial bands: spawn placement, territory ownership, and
//! carrying capacity (docs/14-bands-and-councils.md). The monthly autopilot
//! lives in `autopilot`; overseer directives apply in `directives`.

pub mod autopilot;
pub mod directives;

use directive_schema::Stance;
use sim_events::rng;
use sim_events::{Event, EventLog, SystemId, WorldSeed};
use species::Species;
use world_map::Province;
use world_schema::{NationId, ProvinceId, Quantity, SpeciesId, Tick};

const NATIONS: SystemId = SystemId(8);

#[derive(Debug, Clone, PartialEq)]
pub struct Nation {
    pub id: NationId,
    pub name: String,
    pub species: SpeciesId,
    pub seat: ProvinceId,
    pub stance: Stance,
    /// Overseer-decreed settlement target, consumed when settled.
    pub decreed_target: Option<ProvinceId>,
}

#[derive(Debug, Default)]
pub struct WorldNations {
    pub nations: Vec<Nation>,
    /// Province owner, indexed by province id.
    pub owner: Vec<Option<NationId>>,
    /// Unordered nation pairs that have made contact.
    pub met: std::collections::BTreeSet<(u32, u32)>,
}

impl WorldNations {
    pub fn owned_provinces(&self, nation: NationId) -> impl Iterator<Item = ProvinceId> + '_ {
        self.owner
            .iter()
            .enumerate()
            .filter(move |(_, o)| **o == Some(nation))
            .map(|(p, _)| ProvinceId(p as u32))
    }

    #[must_use]
    pub fn borders_territory(&self, nation: NationId, province: &Province) -> bool {
        province
            .neighbors
            .iter()
            .any(|n| self.owner[n.0 as usize] == Some(nation))
    }
}

/// Carrying capacity of a province for a species: land area scaled by
/// climate fit. Integer-exact fixed-point (sim-path authoritative).
#[must_use]
pub fn capacity(province: &Province, s: &Species) -> Quantity {
    let fit = species::province_fitness(s, province);
    Quantity::from_num(province.cells) * (Quantity::from_num(0.5) + fit * Quantity::from_num(5.5))
        / Quantity::from_num(4)
}

/// Spawn `count` nations at well-separated, species-fit habitable seats.
/// Purely score-driven — no randomness, so no seed parameter.
#[must_use]
pub fn spawn(
    provinces: &[Province],
    table: &[Species],
    count: u32,
    log: &mut EventLog,
) -> WorldNations {
    let mut world = WorldNations {
        nations: Vec::new(),
        owner: vec![None; provinces.len()],
        met: std::collections::BTreeSet::new(),
    };
    for i in 0..count {
        let s = &table[(i as usize) % table.len()];
        let mut best: Option<(i128, u32)> = None;
        for p in provinces {
            if !p.habitable || world.owner[p.id.0 as usize].is_some() {
                continue;
            }
            let fit = species::province_fitness(s, p);
            if fit < Quantity::from_num(0.05) {
                continue;
            }
            let dist_sq = world
                .nations
                .iter()
                .map(|n| {
                    let seat = &provinces[n.seat.0 as usize];
                    let dx = i64::from(p.center.0) - i64::from(seat.center.0);
                    let dy = i64::from(p.center.1) - i64::from(seat.center.1);
                    dx * dx + dy * dy
                })
                .min()
                .unwrap_or(1_000_000);
            let score = i128::from(fit.to_bits()) * i128::from(dist_sq.min(40_000) + 400);
            if best.is_none_or(|(b, _)| score > b) {
                best = Some((score, p.id.0));
            }
        }
        let Some((_, seat)) = best else { continue };
        let id = NationId(i);
        world.owner[seat as usize] = Some(id);
        world.nations.push(Nation {
            id,
            name: format!("{} Band {}", s.name, i + 1),
            species: s.id,
            seat: ProvinceId(seat),
            stance: Stance::Steady,
            decreed_target: None,
        });
        log.push(Event::NationSpawned {
            nation: id,
            species: s.id,
            seat: ProvinceId(seat),
        });
    }
    world
}

/// Dawn-of-time founder cohorts: one small band at each nation's seat.
#[must_use]
pub fn found_cohorts(seed: WorldSeed, world: &WorldNations) -> cohorts::Cohorts {
    let mut founded = cohorts::Cohorts::new();
    for nation in &world.nations {
        let head = 140
            + i64::try_from(rng::draw(seed, Tick::ZERO, NATIONS, u64::from(nation.id.0)) % 160)
                .expect("bounded by modulus");
        founded.add(
            cohorts::CohortKey {
                province: nation.seat,
                species: nation.species,
            },
            Quantity::from_num(head),
        );
    }
    founded
}
