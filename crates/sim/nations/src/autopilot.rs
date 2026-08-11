//! The band autopilot: monthly splitting and frontier settlement within the
//! overseer's stance — policy in, visible expansion out
//! (docs/04-institutions-directives.md, docs/14-bands-and-councils.md).

use crate::WorldNations;
use cohorts::{CohortKey, Cohorts};
use directive_schema::Stance;
use sim_events::{Event, EventLog, WorldSeed};
use species::Species;
use world_map::Province;
use world_schema::{ProvinceId, Quantity, Tick};

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
    provinces: &[Province],
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

        let owned: Vec<ProvinceId> = world.owned_provinces(nation.id).collect();
        let decreed = nation.decreed_target;
        let mut settlement: Option<(ProvinceId, ProvinceId, Quantity, bool)> = None;

        for &p in &owned {
            let province = &provinces[p.0 as usize];
            let key = CohortKey {
                province: p,
                species: nation.species,
            };
            let pop = cohorts.population_of(key);
            if pop < Quantity::from_num(120) {
                continue;
            }
            let cap = crate::capacity(province, s);
            let crowd = if cap > Quantity::ZERO {
                pop / cap
            } else {
                Quantity::from_num(2)
            };

            // A decree overrides the pressure triggers for its target.
            let decreed_here = decreed
                .filter(|t| province.neighbors.contains(t))
                .filter(|t| world.owner[t.0 as usize].is_none());
            let pressured = pop > split_threshold || crowd > crowd_trigger;

            let target = if let Some(t) = decreed_here {
                Some((t, true))
            } else if pressured {
                province
                    .neighbors
                    .iter()
                    .filter(|t| world.owner[t.0 as usize].is_none())
                    .map(|&t| (t, species::province_fitness(s, &provinces[t.0 as usize])))
                    .filter(|(_, fit)| *fit > Quantity::from_num(0.1))
                    .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)))
                    .map(|(t, _)| (t, false))
            } else {
                None
            };

            if let Some((t, was_decreed)) = target {
                let settlers = pop * Quantity::from_num(0.4);
                if settlers >= Quantity::from_num(60) {
                    settlement = Some((p, t, settlers, was_decreed));
                    break; // one settlement per nation per month
                }
            }
        }

        if let Some((from, target, settlers, was_decreed)) = settlement {
            let nation_id = world.nations[ni].id;
            let species_id = world.nations[ni].species;
            let moved = cohorts.remove(
                CohortKey {
                    province: from,
                    species: species_id,
                },
                settlers,
            );
            cohorts.add(
                CohortKey {
                    province: target,
                    species: species_id,
                },
                moved,
            );
            world.owner[target.0 as usize] = Some(nation_id);
            if was_decreed {
                world.nations[ni].decreed_target = None;
            }
            log.push(Event::ProvinceSettled {
                tick,
                nation: nation_id,
                from,
                province: target,
                settlers: moved,
            });
            // First contact fires the moment territories touch (low id first).
            for &nb in &provinces[target.0 as usize].neighbors {
                if let Some(other) = world.owner[nb.0 as usize]
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
