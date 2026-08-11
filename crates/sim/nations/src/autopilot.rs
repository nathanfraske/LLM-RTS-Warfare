//! The band autopilot: monthly splitting and frontier settlement within the
//! overseer's stance — policy in, visible expansion out
//! (docs/04-institutions-directives.md, docs/14-bands-and-councils.md).

use crate::WorldNations;
use cohorts::{CohortKey, Cohorts};
use directive_schema::Stance;
use sim_events::{Event, EventLog, WorldSeed};
use species::Species;
use world_map::{WorldFields, tiles};
use world_schema::{Quantity, Tick, TileId};

/// Stance interpretation: (split-population threshold multiplier, crowding trigger).
fn stance_params(stance: Stance) -> (Quantity, Quantity) {
    match stance {
        Stance::Consolidate => (Quantity::from_num(1.5), Quantity::from_num(1.05)),
        Stance::Steady => (Quantity::from_num(1.0), Quantity::from_num(0.85)),
        Stance::Expansive => (Quantity::from_num(0.7), Quantity::from_num(0.65)),
    }
}

/// One nation-tick per closed month: at most one settlement per nation.
pub fn tick_month(
    _seed: WorldSeed,
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    table: &[Species],
    cohorts: &mut Cohorts,
    log: &mut EventLog,
) {
    for ni in 0..world.nations.len() {
        let nation = &world.nations[ni];
        let s = &table[nation.species.0 as usize];
        let (threshold_mult, crowd_trigger) = stance_params(nation.stance);
        // Higher expansion drive lowers the split threshold.
        let split_threshold = Quantity::from_num(220) * threshold_mult * Quantity::from_num(1000)
            / Quantity::from_num(s.drive_milli);

        let owned: Vec<TileId> = world.owned_tiles(nation.id).collect();
        let decreed = nation.decreed_target;
        let mut settlement: Option<(TileId, TileId, Quantity, bool)> = None;

        for &t in &owned {
            let key = CohortKey {
                tile: t,
                species: nation.species,
            };
            let pop = cohorts.population_of(key);
            if pop < Quantity::from_num(120) {
                continue;
            }
            let cap = crate::capacity(fields, t.0 as usize, s);
            let crowd = if cap > Quantity::ZERO {
                pop / cap
            } else {
                Quantity::from_num(2)
            };

            let neighbors = tiles::land_neighbors(fields, t.0 as usize);
            // A decree overrides the pressure triggers for its target.
            let decreed_here = decreed
                .filter(|target| neighbors.contains(target))
                .filter(|target| world.owner[target.0 as usize].is_none());
            let pressured = pop > split_threshold || crowd > crowd_trigger;

            let target = if let Some(target) = decreed_here {
                Some((target, true))
            } else if pressured {
                neighbors
                    .iter()
                    .filter(|target| world.owner[target.0 as usize].is_none())
                    .map(|&target| (target, crate::fitness(fields, target.0 as usize, s)))
                    .filter(|(_, fit)| *fit > Quantity::from_num(0.1))
                    .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)))
                    .map(|(target, _)| (target, false))
            } else {
                None
            };

            if let Some((target, was_decreed)) = target {
                let settlers = pop * Quantity::from_num(0.4);
                if settlers >= Quantity::from_num(60) {
                    settlement = Some((t, target, settlers, was_decreed));
                    break; // one settlement per nation per month
                }
            }
        }

        if let Some((from, target, settlers, was_decreed)) = settlement {
            let nation_id = world.nations[ni].id;
            let species_id = world.nations[ni].species;
            let moved = cohorts.remove(
                CohortKey {
                    tile: from,
                    species: species_id,
                },
                settlers,
            );
            cohorts.add(
                CohortKey {
                    tile: target,
                    species: species_id,
                },
                moved,
            );
            world.owner[target.0 as usize] = Some(nation_id);
            if was_decreed {
                world.nations[ni].decreed_target = None;
            }
            log.push(Event::TileSettled {
                tick,
                nation: nation_id,
                from,
                tile: target,
                settlers: moved,
            });
            // First contact fires the moment territories touch (low id first).
            let (neighbors, n) = fields.grid().neighbors8(target.0 as usize);
            for &nb in &neighbors[..n] {
                if let Some(other) = world.owner[nb]
                    && other != nation_id
                {
                    let (lo, hi) = if nation_id.0 <= other.0 {
                        (nation_id, other)
                    } else {
                        (other, nation_id)
                    };
                    if world.met.insert((lo.0, hi.0)) {
                        log.push(Event::NationsMet { tick, a: lo, b: hi });
                    }
                }
            }
        }
    }
}
