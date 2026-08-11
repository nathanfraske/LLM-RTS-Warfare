//! The band autopilot: monthly splitting, frontier settlement, and — when
//! the land stops feeding a band — relocation. Policy in, visible movement
//! out (docs/04-institutions-directives.md, docs/19-ecology-and-subsistence.md).
//!
//! Destinations are judged by *food potential* (a closure supplied by the
//! composition root, so this crate needs no ecology dependency). Bands that
//! keep moving are nomads; bands that invest stay — nobody names either.

use crate::{WorldNations, registry};
use cohorts::{CohortKey, Cohorts};
use policy::PolicyTree;
use sim_events::{Event, EventLog};
use species::Species;
use tuning::Society;
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, Tick, TileId};

/// Posture interpretation: split-population threshold multiplier. Unknown
/// text can't get in (the registry bounds the leaf) but reads as steady.
fn posture_threshold(tree: &PolicyTree, soc: &Society) -> Quantity {
    match tree.text(registry::POSTURE) {
        registry::POSTURE_CONSOLIDATE => Quantity::from_num(soc.stance_consolidate_mult),
        registry::POSTURE_EXPANSIVE => Quantity::from_num(soc.stance_expansive_mult),
        _ => Quantity::from_num(soc.stance_steady_mult),
    }
}

/// One nation-tick per closed month: starving bands move, crowded bands split.
#[allow(clippy::too_many_arguments)]
pub fn tick_month(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    table: &[Species],
    cohorts: &mut Cohorts,
    log: &mut EventLog,
    potential: &dyn Fn(usize) -> Quantity,
    starving: &[TileId],
    soc: &Society,
) {
    relocate_starving(tick, world, fields, cohorts, log, potential, starving, soc);
    split_crowded(tick, world, fields, table, cohorts, log, potential, soc);
}

/// A band hungry too long abandons its tile for the best free neighbor —
/// if anywhere nearby actually promises more food.
#[allow(clippy::too_many_arguments)]
fn relocate_starving(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    cohorts: &mut Cohorts,
    log: &mut EventLog,
    potential: &dyn Fn(usize) -> Quantity,
    starving: &[TileId],
    soc: &Society,
) {
    for &from in starving {
        let Some(nation_id) = world.owner[from.0 as usize] else {
            continue;
        };
        let ni = world
            .nations
            .iter()
            .position(|n| n.id == nation_id)
            .expect("owner exists");
        let here = potential(from.0 as usize);
        let target = tiles::land_neighbors(fields, from.0 as usize)
            .into_iter()
            .filter(|t| world.owner[t.0 as usize].is_none())
            .map(|t| (t, potential(t.0 as usize)))
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)));
        let Some((to, promise)) = target else {
            continue;
        };
        if promise < here * Quantity::from_num(soc.relocate_gain) {
            continue; // nowhere better within reach — endure or dwindle
        }
        let species_id = world.nations[ni].species;
        // The whole band leaves: take everything the cohort has.
        let moved = cohorts.remove(
            CohortKey {
                tile: from,
                species: species_id,
            },
            Quantity::MAX,
        );
        cohorts.add(
            CohortKey {
                tile: to,
                species: species_id,
            },
            moved,
        );
        world.owner[from.0 as usize] = None;
        world.owner[to.0 as usize] = Some(nation_id);
        if world.nations[ni].seat == from {
            world.nations[ni].seat = to;
        }
        log.push(Event::BandMoved {
            tick,
            nation: nation_id,
            from,
            to,
        });
        contact_check(tick, world, fields, to, nation_id, log);
    }
}

/// Crowded settlements send two-fifths of their people to found the most
/// promising free neighbor. At most one settlement per nation per month.
#[allow(clippy::too_many_arguments)]
fn split_crowded(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    table: &[Species],
    cohorts: &mut Cohorts,
    log: &mut EventLog,
    potential: &dyn Fn(usize) -> Quantity,
    soc: &Society,
) {
    for ni in 0..world.nations.len() {
        let nation = &world.nations[ni];
        let s = &table[nation.species.0 as usize];
        let split_threshold = Quantity::from_num(soc.split_base_pop)
            * posture_threshold(&nation.policy, soc)
            * Quantity::from_num(1000)
            / Quantity::from_num(s.drive_milli);

        let owned: Vec<TileId> = world.owned_tiles(nation.id).collect();
        let decreed = nation.decreed_target;
        let mut settlement: Option<(TileId, TileId, Quantity, bool)> = None;

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

            let target = if let Some(target) = decreed_here {
                Some((target, true))
            } else if pressured {
                neighbors
                    .iter()
                    .filter(|target| world.owner[target.0 as usize].is_none())
                    .map(|&target| (target, potential(target.0 as usize)))
                    .filter(|(_, p)| *p > Quantity::from_num(soc.split_potential_floor))
                    .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)))
                    .map(|(target, _)| (target, false))
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
            contact_check(tick, world, fields, target, nation_id, log);
        }
    }
}

/// First contact fires the moment territories touch (low id first).
fn contact_check(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    tile: TileId,
    nation_id: NationId,
    log: &mut EventLog,
) {
    let (neighbors, n) = fields.grid().neighbors8(tile.0 as usize);
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
