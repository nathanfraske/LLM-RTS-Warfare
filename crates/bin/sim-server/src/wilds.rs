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
        // The burning season: fire eats fuel before the regrowth answers.
        let burn = self.blaze.tick_month(
            self.seed,
            tick,
            &self.genesis.fields,
            &self.climate,
            &mut self.flora_live,
            &self.tuning.seasons,
            &self.tuning.wildfire,
        );
        for &tile in &burn.burning {
            if let Some(owner) = self.nations.owner[tile.0 as usize] {
                self.cull_settled(tile.0, self.tuning.wildfire.fire_cull_permille);
                self.log.push(Event::Wildfire { tick, tile });
                for work in self.nations.works.scorch(tile.0, &self.tuning.structures) {
                    self.log.push(Event::WorkToppled {
                        tick,
                        nation: owner,
                        tile,
                        work,
                    });
                }
            }
        }
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
                self.cull_settled(t, self.tuning.deep.lava_cull_permille);
                // Molten rock lights whatever green borders the run.
                let (neighbors, n) = self.genesis.fields.grid().neighbors8(tile);
                for &nb in &neighbors[..n] {
                    self.blaze
                        .ignite(nb, &self.flora_live, &self.tuning.wildfire);
                }
            }
            // The ejecta: ash rides the wind far past the lava, smothering
            // the green by heaviness and laying fines the weathering will
            // turn to farmland.
            let ash = geology::fire::ash_fall(
                &self.genesis.fields,
                vent,
                strength,
                self.tuning.deep.ash_radius,
            );
            for &(t, heaviness) in &ash {
                let tile = t as usize;
                let smother = u8::try_from(
                    u32::from(self.tuning.deep.ash_smother) * u32::from(heaviness) / 255,
                )
                .expect("bounded");
                self.flora_live[tile] = self.flora_live[tile].saturating_sub(smother);
                self.regolith
                    .ash_fall(tile, heaviness, self.tuning.deep.ash_fines);
                if heaviness > 170
                    && let Some(owner) = self.nations.owner[tile]
                {
                    for work in self.nations.works.ash_load(t, &self.tuning.structures) {
                        self.log.push(Event::WorkToppled {
                            tick,
                            nation: owner,
                            tile: TileId(t),
                            work,
                        });
                    }
                }
            }
            self.log.push(Event::VolcanoErupted {
                tick,
                tile: TileId(vent),
                reach: u32::try_from(path.len()).expect("short path"),
                ash_tiles: u32::try_from(ash.len()).expect("bounded footprint"),
            });
        }
        self.quakes(tick, month);
    }

    /// The faults slip on their own clocks (docs/29): the shake topples
    /// finished works, takes a small toll of the settled, and breaks the
    /// scree loose on every slope it touches.
    fn quakes(&mut self, tick: Tick, month: u64) {
        let due = geology::fire::due_quakes(&self.genesis.geology, month);
        for epicenter in due {
            let radius = i64::from(self.tuning.deep.quake_radius);
            let (ex, ey) = self.genesis.fields.grid().xy(epicenter as usize);
            let size = i64::from(self.genesis.fields.size);
            let mut shaken = 0u32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let x = i64::from(ex) + dx;
                    let y = i64::from(ey) + dy;
                    if x < 0 || y < 0 || x >= size || y >= size {
                        continue;
                    }
                    let tile = (y as usize) * self.genesis.fields.size as usize + x as usize;
                    if self.genesis.fields.elevation[tile] < 0 {
                        continue;
                    }
                    shaken += 1;
                    self.regolith
                        .shake(&self.genesis.fields, tile, self.tuning.deep.quake_slide);
                    let t32 = u32::try_from(tile).expect("tile fits");
                    self.cull_settled(t32, self.tuning.deep.quake_cull_permille);
                    if let Some(owner) = self.nations.owner[tile] {
                        for work in self.nations.works.shake(t32, &self.tuning.structures) {
                            self.log.push(Event::WorkToppled {
                                tick,
                                nation: owner,
                                tile: TileId(t32),
                                work,
                            });
                        }
                    }
                }
            }
            self.log.push(Event::Earthquake {
                tick,
                tile: TileId(epicenter),
                reach: shaken,
            });
        }
    }

    /// A calamity's toll on whoever holds the tile.
    fn cull_settled(&mut self, tile: u32, permille: u16) {
        if let Some(owner) = self.nations.owner[tile as usize] {
            let species = self
                .nations
                .nations
                .iter()
                .find(|n| n.id == owner)
                .map(|n| n.species);
            if let Some(species) = species {
                let key = cohorts::CohortKey {
                    tile: TileId(tile),
                    species,
                };
                let pop = self.cohorts.population_of(key);
                let taken = pop * Quantity::from_num(permille) / Quantity::from_num(1000);
                let _ = self.cohorts.remove(key, taken);
            }
        }
    }
}
