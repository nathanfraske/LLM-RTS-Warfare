//! Scout parties: mortal couriers of knowledge (docs/22). A party walks out
//! along its bearing at foot pace, remembers what it crosses, and must come
//! home before any of it enters the nation's map. Hostile country can
//! swallow a party whole — the knowledge dies on the trail.

use std::collections::BTreeSet;

use sim_events::rng;
use sim_events::{Event, EventLog, SystemId, WorldSeed};
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Tick, TileId};

use crate::{BEARING_DELTAS, TileMemory, WorldKnowledge};

const SCOUTS: SystemId = SystemId(11);

#[derive(Debug)]
pub struct Party {
    pub nation: NationId,
    pub home: TileId,
    pub tile: TileId,
    bearing: usize,
    /// Ticks of walking accumulated toward the next tile crossing.
    progress: u16,
    tiles_out: u16,
    outbound: bool,
    /// What the party has seen, carried until it gets home.
    learned: Vec<(usize, TileMemory)>,
}

impl Party {
    #[must_use]
    pub fn new(nation: NationId, home: TileId, bearing: usize, _now: Tick) -> Self {
        Self {
            nation,
            home,
            tile: home,
            bearing: bearing % 8,
            progress: 0,
            tiles_out: 0,
            outbound: true,
            learned: Vec::new(),
        }
    }
}

/// One tick for every party afield. `sample` reads the world as it stands;
/// `hostile` judges a tile against the party's people; `owner` and `met`
/// drive encounter discovery.
#[allow(clippy::too_many_arguments)]
pub fn tick(
    world: &mut WorldKnowledge,
    fields: &WorldFields,
    owner: &[Option<NationId>],
    met: &mut BTreeSet<(u32, u32)>,
    seed: WorldSeed,
    now: Tick,
    exp: &tuning::Exploration,
    sample: &dyn Fn(usize) -> (world_schema::Quantity, Option<NationId>),
    hostile: &dyn Fn(NationId, usize) -> bool,
    log: &mut EventLog,
) {
    let mut i = 0;
    while i < world.parties.len() {
        let advance = {
            let party = &mut world.parties[i];
            party.progress += 1;
            party.progress >= exp.scout_ticks_per_tile
        };
        if !advance {
            i += 1;
            continue;
        }
        world.parties[i].progress = 0;
        match step(
            world, i, fields, owner, met, seed, now, exp, sample, hostile, log,
        ) {
            Outcome::Walking => i += 1,
            Outcome::Done => {
                let party = world.parties.swap_remove(i);
                let learned = u32::try_from(party.learned.len()).unwrap_or(u32::MAX);
                let memory = world.of_mut(party.nation);
                for (tile, seen) in party.learned {
                    memory.observe(tile, seen);
                }
                log.push(Event::ScoutReturned {
                    tick: now,
                    nation: party.nation,
                    tiles_learned: learned,
                });
            }
            Outcome::Lost => {
                let party = world.parties.swap_remove(i);
                log.push(Event::ScoutLost {
                    tick: now,
                    nation: party.nation,
                });
            }
        }
    }
}

enum Outcome {
    Walking,
    Done,
    Lost,
}

/// Move one party one tile; observe, encounter, and roll for the country.
#[allow(clippy::too_many_arguments)]
fn step(
    world: &mut WorldKnowledge,
    idx: usize,
    fields: &WorldFields,
    owner: &[Option<NationId>],
    met: &mut BTreeSet<(u32, u32)>,
    seed: WorldSeed,
    now: Tick,
    exp: &tuning::Exploration,
    sample: &dyn Fn(usize) -> (world_schema::Quantity, Option<NationId>),
    hostile: &dyn Fn(NationId, usize) -> bool,
    log: &mut EventLog,
) -> Outcome {
    let (nation, from, target_delta) = {
        let party = &world.parties[idx];
        let delta = if party.outbound {
            BEARING_DELTAS[party.bearing]
        } else {
            let grid = fields.grid();
            let (x, y) = grid.xy(party.tile.0 as usize);
            let (hx, hy) = grid.xy(party.home.0 as usize);
            (
                (i64::from(hx) - i64::from(x)).signum(),
                (i64::from(hy) - i64::from(y)).signum(),
            )
        };
        (party.nation, party.tile, delta)
    };

    let Some(next) = walkable_toward(fields, from.0 as usize, target_delta) else {
        // Boxed in by water or the world's edge: nothing more to do out here.
        return if world.parties[idx].outbound {
            world.parties[idx].outbound = false;
            Outcome::Walking
        } else {
            Outcome::Lost
        };
    };

    let party = &mut world.parties[idx];
    party.tile = TileId(next as u32);
    if party.outbound {
        party.tiles_out += 1;
        if party.tiles_out >= exp.scout_range {
            party.outbound = false;
        }
    }

    // See the ground underfoot.
    let (potential, seen_owner) = sample(next);
    party.learned.push((
        next,
        TileMemory {
            last_seen: now,
            potential,
            owner: seen_owner,
        },
    ));

    // Encounter: strangers on someone's land, or in sight of it, are seen
    // and do the seeing — both peoples discover each other.
    let (neighbors, n) = fields.grid().neighbors8(next);
    for &t in std::iter::once(&next).chain(&neighbors[..n]) {
        if let Some(other) = owner[t]
            && other != nation
        {
            let (lo, hi) = if nation.0 <= other.0 {
                (nation, other)
            } else {
                (other, nation)
            };
            if met.insert((lo.0, hi.0)) {
                log.push(Event::NationsMet {
                    tick: now,
                    a: lo,
                    b: hi,
                });
            }
        }
    }

    // The country itself: a party in land its people were never made for
    // can vanish, and everything it carried vanishes with it.
    if hostile(nation, next) {
        let roll = rng::draw(seed, now, SCOUTS, u64::from(nation.0) << 32 | next as u64) % 1000;
        if roll < u64::from(exp.scout_loss_permille) {
            return Outcome::Lost;
        }
    }

    if !world.parties[idx].outbound && world.parties[idx].tile == world.parties[idx].home {
        return Outcome::Done;
    }
    Outcome::Walking
}

/// The land neighbor closest to the wanted direction, or `None` if every
/// candidate is water or off-map. Deterministic: candidates are ranked by
/// alignment, ties by index.
fn walkable_toward(fields: &WorldFields, from: usize, (dx, dy): (i64, i64)) -> Option<usize> {
    let grid = fields.grid();
    let (x, y) = grid.xy(from);
    let mut best: Option<(i64, usize)> = None;
    for (i, (nx, ny)) in BEARING_DELTAS.iter().enumerate() {
        let cx = i64::from(x) + nx;
        let cy = i64::from(y) + ny;
        if cx < 0 || cy < 0 || cx >= i64::from(fields.size) || cy >= i64::from(fields.size) {
            continue;
        }
        let tile = (cy as usize) * fields.size as usize + cx as usize;
        if !tiles::is_land(fields, tile) {
            continue;
        }
        let align = nx * dx + ny * dy;
        if align < 0 {
            continue; // never walk backwards to dodge water — turn around instead
        }
        let score = align * 8 - i64::try_from(i).expect("small index");
        if best.is_none_or(|(b, _)| score > b) {
            best = Some((score, tile));
        }
    }
    best.map(|(_, tile)| tile)
}
