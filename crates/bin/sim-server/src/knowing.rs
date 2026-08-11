//! How the world's nations come to know it (docs/22): parties afield step
//! every hour, and home ground is re-observed every month. Split from
//! `world` — this is the perception plumbing, not the monthly order.

use world_schema::{Quantity, Tick};

use crate::World;

impl World {
    /// Scout parties walk every hour; knowledge lands when they come home.
    pub(crate) fn step_scouts(&mut self, tick: Tick) {
        if self.knowledge.parties.is_empty() {
            return;
        }
        let species_of: std::collections::BTreeMap<u32, world_schema::SpeciesId> = self
            .nations
            .nations
            .iter()
            .map(|n| (n.id.0, n.species))
            .collect();
        let fields = &self.genesis.fields;
        let wild = &self.fauna;
        let flora_live = &self.flora_live;
        let sky = &self.climate;
        let ground = &self.regolith;
        let sub = &self.tuning.subsistence;
        let wx = &self.tuning.weather;
        let owner = &self.nations.owner;
        let table = self.table;
        let hostile_fit = Quantity::from_num(self.tuning.exploration.hostile_fit);
        let sample = |t: usize| {
            (
                economy::potential(fields, wild, flora_live, sky, ground, t, sub, wx),
                owner[t],
            )
        };
        let hostile = |nation: world_schema::NationId, t: usize| {
            species_of
                .get(&nation.0)
                .is_none_or(|s| nations::fitness(fields, t, &table[s.0 as usize]) < hostile_fit)
        };
        knowledge::scouts::tick(
            &mut self.knowledge,
            fields,
            owner,
            &mut self.nations.met,
            self.seed,
            tick,
            &self.tuning.exploration,
            &sample,
            &hostile,
            &mut self.log,
        );
    }

    /// Home is always fresh: settled tiles and their surroundings.
    pub(crate) fn refresh_home_knowledge(&mut self, tick: Tick) {
        let fields = &self.genesis.fields;
        let wild = &self.fauna;
        let flora_live = &self.flora_live;
        let sky = &self.climate;
        let ground = &self.regolith;
        let sub = &self.tuning.subsistence;
        let wx = &self.tuning.weather;
        let owner = &self.nations.owner;
        let sample = |t: usize| {
            (
                economy::potential(fields, wild, flora_live, sky, ground, t, sub, wx),
                owner[t],
            )
        };
        for nation in &self.nations.nations {
            let home_tiles: Vec<world_schema::TileId> = self
                .nations
                .owner
                .iter()
                .enumerate()
                .filter(|(_, o)| **o == Some(nation.id))
                .map(|(t, _)| world_schema::TileId(t as u32))
                .collect();
            self.knowledge
                .refresh_home(nation.id, &home_tiles, fields, &sample, tick);
        }
    }
}
