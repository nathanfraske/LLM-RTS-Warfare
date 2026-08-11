//! Living water (docs/26 §4): motes streaming down the drainage tree,
//! wave-pulse at the coasts, and a tidal breathing keyed to the moon
//! clock. Presentation only — the rivers flow because `drains_to` says
//! where, and the tide swells because the moon says when.

use eframe::egui::{self, Color32, Rect, Vec2};
use sim_server::World;
use world_map::Water;

use crate::camera::Camera;

/// Precomputed water geometry: river segments and wave-washed coast tiles.
pub struct Waters {
    rivers: Vec<(u32, u32)>,
    coasts: Vec<u32>,
}

fn hash01(v: u32) -> f32 {
    let h = v.wrapping_mul(2_654_435_761);
    ((h >> 16) & 0xFFFF) as f32 / 65_536.0
}

fn center(world: &World, tile: u32) -> Vec2 {
    let (x, y) = world.genesis.fields.grid().xy(tile as usize);
    Vec2::new(x as f32 + 0.5, y as f32 + 0.5)
}

impl Waters {
    #[must_use]
    pub fn new(world: &World) -> Self {
        let fields = &world.genesis.fields;
        let cells = fields.grid().cells();
        let rivers = (0..cells)
            .filter(|&t| fields.water[t] == Water::River)
            .filter_map(|t| {
                let next = fields.drains_to[t];
                (next != u32::MAX).then_some((t as u32, next))
            })
            .collect();
        let coasts = (0..cells)
            .filter(|&t| {
                fields.water[t] == Water::Ocean && {
                    let (neighbors, n) = fields.grid().neighbors8(t);
                    neighbors[..n].iter().any(|&nb| fields.elevation[nb] >= 0)
                }
            })
            .map(|t| t as u32)
            .collect();
        Self { rivers, coasts }
    }

    /// Rivers run and the sea breathes; still under pause, because the sim
    /// clock is the only clock here.
    pub fn draw(&self, world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
        if cam.zoom < 2.2 {
            return;
        }
        let t = world.tick().0 as f32;
        let radius = (cam.zoom * 0.12).clamp(0.8, 2.6);

        for &(tile, next) in &self.rivers {
            let phase = (t * 0.11 + hash01(tile)).fract();
            let a = center(world, tile);
            let b = center(world, next);
            let at = a + (b - a) * phase;
            let screen = cam.to_screen(rect, at);
            if rect.contains(screen) {
                painter.circle_filled(
                    screen,
                    radius,
                    Color32::from_rgba_unmultiplied(170, 210, 250, 140),
                );
            }
        }

        // Two tides per lunar day, gently scaling the surf.
        let period = f32::from(world.tuning.sky.moon_period_days.max(1)) * 24.0;
        let tide_phase = (t * 2.0 / period).fract();
        let tide = 0.6 + 0.4 * (1.0 - (2.0 * tide_phase - 1.0).abs());
        for &tile in &self.coasts {
            let pulse = (t * 0.06 + hash01(tile) * 3.0).fract();
            let swell = 1.0 - (2.0 * pulse - 1.0).abs();
            let screen = cam.to_screen(rect, center(world, tile));
            if rect.contains(screen) {
                let alpha = (26.0 + 66.0 * swell * tide) as u8;
                painter.circle_stroke(
                    screen,
                    radius * (1.2 + swell),
                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(210, 235, 255, alpha)),
                );
            }
        }
    }
}
