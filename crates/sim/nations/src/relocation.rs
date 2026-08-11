//! Hunger on the move (docs/22 §3): a starving band relocates to the best
//! tile its memory offers — or, when memory offers nothing, gambles blindly
//! into adjacent unwalked land. Desperate people do not wait for surveys.

use crate::WorldNations;
use crate::autopilot::contact_check;
use cohorts::{CohortKey, Cohorts};
use knowledge::{TileMemory, WorldKnowledge};
use sim_events::rng;
use sim_events::{Event, EventLog, SystemId, WorldSeed};
use tuning::Society;
use world_map::{WorldFields, tiles};
use world_schema::{Quantity, Tick, TileId};

const NATIONS: SystemId = SystemId(8);

/// A remembered candidate: known to this nation, remembered free, and
/// actually still free (claims collide in the world, not in memory).
pub(crate) fn remembered_free(
    world: &WorldNations,
    memory: &knowledge::NationKnowledge,
    tile: TileId,
) -> Option<Quantity> {
    let m = memory.known(tile.0 as usize)?;
    if m.owner.is_some() || world.owner[tile.0 as usize].is_some() {
        return None;
    }
    Some(m.potential)
}

/// A band hungry too long moves: to the best *remembered* better tile if
/// its map offers one, else blindly into adjacent unknown land — desperate
/// people do not wait for surveys. Staying is for when there is nowhere at
/// all to go.
#[allow(clippy::too_many_arguments)]
pub(crate) fn relocate_starving(
    tick: Tick,
    seed: WorldSeed,
    world: &mut WorldNations,
    fields: &WorldFields,
    cohorts: &mut Cohorts,
    known: &mut WorldKnowledge,
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
        let neighbors = tiles::land_neighbors(fields, from.0 as usize);

        // Informed: the best tile memory says is better than staying.
        let memory = known.of(nation_id);
        let informed = neighbors
            .iter()
            .filter_map(|&t| remembered_free(world, memory, t).map(|p| (t, p)))
            .filter(|(_, promise)| *promise >= here * Quantity::from_num(soc.relocate_gain))
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.0.cmp(&a.0.0)))
            .map(|(t, _)| t);

        // Blind: any adjacent land nobody has ever walked. A pure gamble.
        let target = informed.or_else(|| {
            let unknown: Vec<TileId> = neighbors
                .iter()
                .copied()
                .filter(|t| {
                    memory.known(t.0 as usize).is_none() && world.owner[t.0 as usize].is_none()
                })
                .collect();
            if unknown.is_empty() {
                None
            } else {
                let roll = rng::draw(seed, tick, NATIONS, u64::from(from.0) << 8 | 0x51);
                Some(unknown[usize::try_from(roll).unwrap_or(0) % unknown.len()])
            }
        });
        let Some(to) = target else {
            continue; // nowhere known, nowhere new — endure or dwindle
        };
        let blind = informed.is_none();

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
        // They learn where they landed the hard way.
        known.of_mut(nation_id).observe(
            to.0 as usize,
            TileMemory {
                last_seen: tick,
                potential: potential(to.0 as usize),
                owner: Some(nation_id),
            },
        );
        log.push(Event::BandMoved {
            tick,
            nation: nation_id,
            from,
            to,
            blind,
        });
        contact_check(tick, world, fields, to, nation_id, log);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use policy::{PolicyTree, Registry};
    use world_map::hydrology::Water;
    use world_schema::NationId;

    /// Desperate bands do not wait for surveys (docs/22 §3): with nothing
    /// better remembered, a starving band gambles into unwalked land.
    #[test]
    fn a_starving_band_with_no_known_exit_moves_blind() {
        let fields = WorldFields {
            size: 3,
            elevation: vec![10; 9],
            water: vec![Water::Dry; 9],
            flow_acc: vec![1; 9],
            temperature: vec![200; 9],
            moisture: vec![150; 9],
            cell_fertility: vec![120; 9],
        };
        let soc = Society::default();
        let registry = Registry {
            policies: crate::registry::policy_defs(&soc),
            actions: Vec::new(),
        };
        let table = species::archetypes();
        let home = TileId(4); // center of the 3x3
        let mut world = WorldNations {
            nations: vec![crate::Nation {
                id: NationId(0),
                name: "The Test Band".into(),
                species: table[0].id,
                seat: home,
                decreed_target: None,
                mandate: Quantity::from_num(soc.starting_mandate),
                autonomy: Quantity::ZERO,
                policy: PolicyTree::from_defaults(&registry),
            }],
            owner: vec![None; 9],
            met: std::collections::BTreeSet::new(),
            works: crate::works::Works::default(),
        };
        world.owner[4] = Some(NationId(0));
        let mut cohorts = Cohorts::new();
        cohorts.add(
            CohortKey {
                tile: home,
                species: table[0].id,
            },
            Quantity::from_num(100),
        );
        // The nation remembers only home; every neighbor is unwalked.
        let mut known = WorldKnowledge::new(9, [NationId(0)].into_iter());
        let mut log = EventLog::new();

        relocate_starving(
            Tick(720),
            WorldSeed(7),
            &mut world,
            &fields,
            &mut cohorts,
            &mut known,
            &mut log,
            &|_| Quantity::ONE,
            &[home],
            &soc,
        );

        let blind_move = log.iter().find_map(|e| match e {
            Event::BandMoved { blind, to, .. } => Some((*blind, *to)),
            _ => None,
        });
        let (blind, to) = blind_move.expect("the band must move somewhere");
        assert!(blind, "with nothing remembered, the move is a gamble");
        assert_eq!(world.owner[4], None, "home was abandoned");
        assert_eq!(world.owner[to.0 as usize], Some(NationId(0)));
        assert!(
            known.of(NationId(0)).known(to.0 as usize).is_some(),
            "they learn where they landed the hard way"
        );
    }
}
