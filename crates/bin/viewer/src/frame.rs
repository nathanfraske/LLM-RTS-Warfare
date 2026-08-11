//! Per-frame upkeep: advancing the sim clock, draining fresh events into
//! the feed, and repainting the textures whose sources moved — territory
//! on events, terrain with the month, relief with the hour.

use eframe::egui::TextureOptions;
use sim_events::Event;

use crate::app::App;
use crate::{feed, layers};

const MAX_TICKS_PER_FRAME: u64 = 30_000;
const FEED_CAP: usize = 400;

impl App {
    pub(crate) fn advance(&mut self, dt: f64) {
        if self.paused {
            return;
        }
        self.tick_debt += self.ticks_per_sec * dt;
        let mut steps = self.tick_debt.floor() as u64;
        self.tick_debt -= steps as f64;
        steps = steps.min(MAX_TICKS_PER_FRAME);
        for _ in 0..steps {
            self.world.step();
        }
        if steps > 0 {
            self.drain_events();
        }
    }

    /// Pull new events into the feed, caravans, and overlay refresh.
    pub(crate) fn drain_events(&mut self) {
        let fresh: Vec<Event> = self
            .world
            .log
            .iter()
            .skip(self.seen_events)
            .cloned()
            .collect();
        self.seen_events = self.world.log.len();
        for event in fresh {
            match &event {
                Event::TileSettled {
                    nation, from, tile, ..
                } => {
                    self.folk
                        .spawn_caravan(&self.world, from.0, tile.0, nation.0);
                    self.territory_dirty = true;
                }
                Event::NationSpawned { .. } => self.territory_dirty = true,
                _ => {}
            }
            if let Some(line) = feed::describe(&event, &self.world) {
                self.feed.push(line);
            }
        }
        if self.feed.len() > FEED_CAP {
            self.feed.drain(..self.feed.len() - FEED_CAP);
        }
    }

    pub(crate) fn refresh_textures(&mut self) {
        if self.territory_dirty {
            self.territory.set(
                layers::territory_image(&self.world),
                TextureOptions::NEAREST,
            );
            self.territory_dirty = false;
        }
        let month = self.world.tick().0 / 720;
        if month != self.terrain_month {
            self.terrain
                .set(layers::terrain_image(&self.world), TextureOptions::NEAREST);
            self.terrain_month = month;
        }
        let hour = self.world.tick().0 % 24;
        if hour != self.shade_hour {
            self.shade
                .set(crate::sky::shade_image(&self.world), TextureOptions::LINEAR);
            self.shade_hour = hour;
        }
    }
}
