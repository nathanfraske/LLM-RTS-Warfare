//! Nations as territorial bands over world tiles: spawn placement, ownership,
//! and carrying capacity (docs/14-bands-and-councils.md, docs/15-multiscale-maps.md).
//! The monthly autopilot lives in `autopilot`; overseer directives in `directives`.

pub mod autopilot;
pub mod directives;
pub mod mandate;
pub mod works;

use directive_schema::Stance;
use sim_events::rng;
use sim_events::{Event, EventLog, SystemId, WorldSeed};
use species::Species;
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, SpeciesId, Tick, TileId};

const NATIONS: SystemId = SystemId(8);

#[derive(Debug, Clone, PartialEq)]
pub struct Nation {
    pub id: NationId,
    pub name: String,
    pub species: SpeciesId,
    pub seat: TileId,
    pub stance: Stance,
    /// Overseer-decreed settlement target, consumed when settled.
    pub decreed_target: Option<TileId>,
    /// The people's readiness to be commanded (docs/16-mandate-and-works.md).
    pub mandate: Quantity,
    /// Friction from direct rule: raises costs, slows mandate regen.
    pub autonomy: Quantity,
}

#[derive(Debug, Default)]
pub struct WorldNations {
    pub nations: Vec<Nation>,
    /// Tile owner, indexed by tile id.
    pub owner: Vec<Option<NationId>>,
    /// Unordered nation pairs that have made contact.
    pub met: std::collections::BTreeSet<(u32, u32)>,
    /// Commissioned works across all tiles.
    pub works: works::Works,
}

impl WorldNations {
    pub fn owned_tiles(&self, nation: NationId) -> impl Iterator<Item = TileId> + '_ {
        self.owner
            .iter()
            .enumerate()
            .filter(move |(_, o)| **o == Some(nation))
            .map(|(t, _)| TileId(t as u32))
    }

    #[must_use]
    pub fn borders_territory(&self, nation: NationId, fields: &WorldFields, tile: usize) -> bool {
        let (neighbors, n) = fields.grid().neighbors8(tile);
        neighbors[..n]
            .iter()
            .any(|&nb| self.owner[nb] == Some(nation))
    }
}

/// Species climate fit for a tile, straight from the fields.
#[must_use]
pub fn fitness(fields: &WorldFields, tile: usize, s: &Species) -> Quantity {
    species::tile_fitness(s, fields.temperature[tile], fields.moisture[tile])
}

/// Carrying capacity of one world tile for a species: climate fit × soil,
/// grown by completed works. Integer-exact fixed-point (sim-path authoritative).
#[must_use]
pub fn capacity(fields: &WorldFields, tile: usize, s: &Species, works: &works::Works) -> Quantity {
    let fit = fitness(fields, tile, s);
    let soil = Quantity::from_num(fields.cell_fertility[tile]) / Quantity::from_num(255);
    let base = Quantity::from_num(260)
        + Quantity::from_num(1450)
            * fit
            * (Quantity::from_num(0.35) + soil * Quantity::from_num(0.65));
    base * works.capacity_mult(tile as u32)
}

/// Spawn `count` nations at well-separated, species-fit habitable tiles.
/// Purely score-driven — no randomness, so no seed parameter.
#[must_use]
pub fn spawn(
    fields: &WorldFields,
    table: &[Species],
    count: u32,
    log: &mut EventLog,
) -> WorldNations {
    let cells = fields.grid().cells();
    let mut world = WorldNations {
        nations: Vec::new(),
        owner: vec![None; cells],
        met: std::collections::BTreeSet::new(),
        works: works::Works::default(),
    };
    let habitable: Vec<usize> = (0..cells)
        .filter(|&t| tiles::habitable(fields, t))
        .collect();
    let starting_mandate = Quantity::from_num(mandate::STARTING_MANDATE);
    for i in 0..count {
        let s = &table[(i as usize) % table.len()];
        let mut best: Option<(i128, u32)> = None;
        for &t in &habitable {
            if world.owner[t].is_some() {
                continue;
            }
            let fit = fitness(fields, t, s);
            if fit < Quantity::from_num(0.05) {
                continue;
            }
            let (x, y) = fields.grid().xy(t);
            let dist_sq = world
                .nations
                .iter()
                .map(|n| {
                    let (sx, sy) = fields.grid().xy(n.seat.0 as usize);
                    let dx = i64::from(x) - i64::from(sx);
                    let dy = i64::from(y) - i64::from(sy);
                    dx * dx + dy * dy
                })
                .min()
                .unwrap_or(1_000_000);
            let score = i128::from(fit.to_bits()) * i128::from(dist_sq.min(4_000) + 60);
            if best.is_none_or(|(b, _)| score > b) {
                best = Some((score, t as u32));
            }
        }
        let Some((_, seat)) = best else { continue };
        let id = NationId(i);
        world.owner[seat as usize] = Some(id);
        world.nations.push(Nation {
            id,
            name: format!("{} Band {}", s.name, i + 1),
            species: s.id,
            seat: TileId(seat),
            stance: Stance::Steady,
            decreed_target: None,
            mandate: starting_mandate,
            autonomy: Quantity::ZERO,
        });
        log.push(Event::NationSpawned {
            nation: id,
            species: s.id,
            seat: TileId(seat),
        });
    }
    world
}

/// Dawn-of-time founder cohorts: one small band at each nation's seat tile.
#[must_use]
pub fn found_cohorts(seed: WorldSeed, world: &WorldNations) -> cohorts::Cohorts {
    let mut founded = cohorts::Cohorts::new();
    for nation in &world.nations {
        let head = 140
            + i64::try_from(rng::draw(seed, Tick::ZERO, NATIONS, u64::from(nation.id.0)) % 160)
                .expect("bounded by modulus");
        founded.add(
            cohorts::CohortKey {
                tile: nation.seat,
                species: nation.species,
            },
            Quantity::from_num(head),
        );
    }
    founded
}
