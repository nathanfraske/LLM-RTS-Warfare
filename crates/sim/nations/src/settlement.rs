//! Crowding founds new settlements (docs/22 §3): splits go only where the
//! nation's memory says good free land lies; a decree consumes its target
//! where borders and memory allow. Need with no known outlet scouts instead.

use crate::WorldNations;
use crate::autopilot::{contact_check, posture_threshold};
use crate::relocation::remembered_free;
use cohorts::{CohortKey, Cohorts};
use knowledge::{TileMemory, WorldKnowledge};
use sim_events::{Event, EventLog};
use species::Species;
use tuning::{Exploration, Society};
use world_map::{WorldFields, tiles};
use world_schema::{Quantity, Tick, TileId};

/// Crowded settlements send two-fifths of their people to found the most
/// promising tile *they remember*. At most one settlement per nation per
/// month; a decree consumes its target only where memory and borders allow.
#[allow(clippy::too_many_arguments)]
pub(crate) fn split_crowded(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    table: &[Species],
    cohorts: &mut Cohorts,
    known: &mut WorldKnowledge,
    log: &mut EventLog,
    potential: &dyn Fn(usize) -> Quantity,
    soc: &Society,
    exp: &Exploration,
) {
    for ni in 0..world.nations.len() {
        let nation = &world.nations[ni];
        let nation_id = nation.id;
        let s = &table[nation.species.0 as usize];
        let split_threshold = Quantity::from_num(soc.split_base_pop)
            * posture_threshold(&nation.policy, soc)
            * Quantity::from_num(1000)
            / Quantity::from_num(s.drive_milli);

        let owned: Vec<TileId> = world.owned_tiles(nation_id).collect();
        let decreed = nation.decreed_target;
        let mut settlement: Option<(TileId, TileId, Quantity, bool)> = None;
        let mut pressured_dark = false;

        for &t in &owned {
            let pop = cohorts.population_of(CohortKey {
                tile: t,
                species: nation.species,
            });
            if pop < Quantity::from_num(soc.split_min_pop) {
                continue;
            }
            let neighbors = tiles::land_neighbors(fields, t.0 as usize);
            let decreed_here = decreed
                .filter(|target| neighbors.contains(target))
                .filter(|target| world.owner[target.0 as usize].is_none());
            let pressured = pop > split_threshold;

            let memory = known.of(nation_id);
            let target = if let Some(target) = decreed_here {
                Some((target, true))
            } else if pressured {
                let found = neighbors
                    .iter()
                    .filter_map(|&target| {
                        remembered_free(world, memory, target).map(|p| (target, p))
                    })
                    .filter(|(_, p)| *p > Quantity::from_num(soc.split_potential_floor))
                    .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)))
                    .map(|(target, _)| (target, false));
                if found.is_none() {
                    pressured_dark = true; // need outran knowledge
                }
                found
            } else {
                None
            };

            if let Some((target, was_decreed)) = target {
                let settlers = pop * Quantity::from_num(soc.settlers_frac);
                if settlers >= Quantity::from_num(soc.settlers_min) {
                    settlement = Some((t, target, settlers, was_decreed));
                    break;
                }
            }
        }

        if let Some((from, target, settlers, was_decreed)) = settlement {
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
            known.of_mut(nation_id).observe(
                target.0 as usize,
                TileMemory {
                    last_seen: tick,
                    potential: potential(target.0 as usize),
                    owner: Some(nation_id),
                },
            );
            log.push(Event::TileSettled {
                tick,
                nation: nation_id,
                from,
                tile: target,
                settlers: moved,
            });
            contact_check(tick, world, fields, target, nation_id, log);
        } else if pressured_dark {
            // Crowded with nowhere known to go: scout before settling.
            let seat = world.nations[ni].seat;
            known.need_scout(nation_id, seat, fields, tick, exp, log);
        }
    }
}
