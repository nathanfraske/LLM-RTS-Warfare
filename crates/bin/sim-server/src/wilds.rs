//! The wild world's monthly turn: sky, ground, green, and beasts, in
//! causal order — climate first (docs/26), the regolith it wets and
//! freezes (docs/27), the flora both gate, and the fauna the flora feeds.

use world_schema::Tick;

use crate::World;

impl World {
    pub(crate) fn breathe(&mut self, tick: Tick) {
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
}
