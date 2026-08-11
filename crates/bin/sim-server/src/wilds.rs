//! The wild world's monthly turn: sky, ground, green, and beasts, in
//! causal order — climate first (docs/26), the regolith it wets and
//! freezes (docs/27), the flora both gate, and the fauna the flora feeds.

use sim_events::Event;
use world_schema::{Quantity, Tick, TileId};

use crate::World;

impl World {
    pub(crate) fn breathe(&mut self, tick: Tick) {
        self.eruptions(tick);
        self.climate.tick_month(
            &self.genesis.fields,
            tick.0 / 720,
            &self.tuning.weather,
            &self.tuning.seasons,
        );
        self.regolith.tick_month(
            &self.genesis.fields,
            &self.climate,
            &self.flora_live,
            tick.0 / 720,
            &self.tuning.seasons,
            &self.tuning.ground,
        );
        flora::regrow_month(
            &mut self.flora_live,
            &self.genesis.flora.density,
            self.tuning.ecology.regrow_divisor,
            &self.climate.growth,
        );
        fauna::dynamics::tick_month(
            &mut self.fauna,
            &self.genesis.fields,
            &mut self.flora_live,
            &self.tuning.ecology,
        );
    }

    /// The fire below (docs/29 §3): due vents pour lava down the drainage
    /// tree. The green burns, the herds are culled, settled people die,
    /// and the ground is buried in fresh rock — which the ordinary
    /// weathering rules will turn, in a generation, into the richest soil
    /// in the region.
    fn eruptions(&mut self, tick: Tick) {
        let month = tick.0 / 720;
        let due = geology::fire::due(&self.genesis.geology, month);
        for (vent, strength) in due {
            let path = geology::fire::lava_path(
                &self.genesis.fields,
                vent,
                strength,
                self.tuning.deep.lava_reach,
            );
            for &t in &path {
                let tile = t as usize;
                self.flora_live[tile] = 0;
                self.regolith.bury_in_rock(tile);
                for si in 0..self.fauna.species.len() {
                    self.fauna.set(si, tile, Quantity::ZERO);
                }
                if let Some(owner) = self.nations.owner[tile] {
                    let species = self
                        .nations
                        .nations
                        .iter()
                        .find(|n| n.id == owner)
                        .map(|n| n.species);
                    if let Some(species) = species {
                        let key = cohorts::CohortKey {
                            tile: TileId(t),
                            species,
                        };
                        let pop = self.cohorts.population_of(key);
                        let taken = pop * Quantity::from_num(self.tuning.deep.lava_cull_permille)
                            / Quantity::from_num(1000);
                        let _ = self.cohorts.remove(key, taken);
                    }
                }
            }
            self.log.push(Event::VolcanoErupted {
                tick,
                tile: TileId(vent),
                reach: u32::try_from(path.len()).expect("short path"),
            });
        }
    }
}
