//! The people build unbidden (docs/30): when stores overflow, families
//! crowd, or worked fields want working, a settled tile raises what it
//! needs from its own ground — no decree, no mandate. Whether they *may*
//! is itself a policy leaf (`building.initiative`): the council can claim
//! the sole right to build, and the people will wait on its word.

use sim_events::Event;
use world_schema::{Quantity, Tick, TileId};

use crate::World;

impl World {
    /// One self-build at most per nation per month, by the loudest need.
    pub(crate) fn initiative(&mut self, tick: Tick) {
        let st = self.tuning.structures.clone();
        let mut raises: Vec<(u32, usize, world_schema::NationId)> = Vec::new();
        for nation in &self.nations.nations {
            if nation.policy.text(nations::registry::BUILDING_INITIATIVE)
                != nations::registry::INITIATIVE_UNBIDDEN
            {
                continue; // the council keeps the sole right to build
            }
            let mut chosen: Option<(u32, usize)> = None;
            for tile in self.nations.owned_tiles(nation.id) {
                let t = tile.0;
                let pop = self.cohorts.population_of(cohorts::CohortKey {
                    tile,
                    species: nation.species,
                });
                if pop < Quantity::from_num(st.initiative_pop)
                    || self.nations.works.load(t) >= usize::from(st.max_per_tile)
                    || !self.nations.works.in_progress(t).is_empty()
                {
                    continue;
                }
                let emphasis = self.wanted_emphasis(t, &st);
                if let Some(emphasis) = emphasis {
                    chosen = Some((t, emphasis));
                    break;
                }
            }
            if let Some((t, emphasis)) = chosen {
                raises.push((t, emphasis, nation.id));
            }
        }
        for (t, emphasis, nation_id) in raises {
            let design = structures::design(
                emphasis,
                &self.regolith,
                &self.genesis.geology,
                &self.flora_live,
                &self.genesis.fields,
                t as usize,
                &st,
            );
            let name = design.name.clone();
            self.nations.works.commission(t, design);
            self.log.push(Event::PeopleRaised {
                tick,
                nation: nation_id,
                tile: TileId(t),
                work: name,
            });
        }
    }

    /// The loudest need on a tile, if any: overflowing stores want room,
    /// crowded families want cover, established fields want working.
    fn wanted_emphasis(&self, tile: u32, st: &tuning::Structures) -> Option<usize> {
        let works = &self.nations.works;
        let cap =
            Quantity::from_num(self.tuning.subsistence.store_base) + works.store_bonus(tile, st);
        if let Some(te) = self.economy.tile(tile) {
            let full =
                cap * Quantity::from_num(st.initiative_store_permille) / Quantity::from_num(1000);
            if te.stock >= full {
                return Some(0); // roomy
            }
            let establish = te.establishment * Quantity::from_num(1000)
                >= Quantity::from_num(st.initiative_establish_permille);
            if establish && works.cultivation_mult(tile, st) == Quantity::ONE {
                return Some(2); // ground-working
            }
        }
        let sheltered = works.birth_mult(tile, st) > Quantity::ONE;
        if !sheltered {
            let pop = self
                .nations
                .owner
                .get(tile as usize)
                .copied()
                .flatten()
                .and_then(|owner| {
                    self.nations
                        .nations
                        .iter()
                        .find(|n| n.id == owner)
                        .map(|n| {
                            self.cohorts.population_of(cohorts::CohortKey {
                                tile: TileId(tile),
                                species: n.species,
                            })
                        })
                });
            if pop.is_some_and(|p| p >= Quantity::from_num(u32::from(st.initiative_pop) * 2)) {
                return Some(1); // sheltering
            }
        }
        None
    }
}
