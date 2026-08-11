//! The spectator's window into docs/22: a per-nation fog overlay (cycle
//! with F) showing the world as one nation actually knows it, and the gold
//! dots of scout parties walking the real map.

use eframe::egui::{self, Rect, TextureHandle, TextureOptions};
use sim_server::World;

use crate::camera::Camera;
use crate::layers;

/// Which nation's knowledge the spectator is looking through, if any.
#[derive(Default)]
pub struct FogView {
    view: Option<usize>,
    texture: Option<TextureHandle>,
}

impl FogView {
    /// F cycles: omniscient, then each nation in turn, then back.
    pub fn cycle(&mut self, nations: usize) {
        self.view = match self.view {
            None if nations > 0 => Some(0),
            Some(i) if i + 1 < nations => Some(i + 1),
            _ => None,
        };
    }

    /// The fog tracks a living map: rebuild its texture each frame.
    pub fn refresh(&mut self, ctx: &egui::Context, world: &World) {
        match self.view {
            Some(i) => {
                let id = world.nations.nations[i].id;
                let image = layers::fog_image(world, id);
                match &mut self.texture {
                    Some(handle) => handle.set(image, TextureOptions::NEAREST),
                    None => {
                        self.texture =
                            Some(ctx.load_texture("fog", image, TextureOptions::NEAREST));
                    }
                }
            }
            None => self.texture = None,
        }
    }

    #[must_use]
    pub fn label(&self, world: &World) -> Option<String> {
        self.view.map(|i| world.nations.nations[i].name.clone())
    }

    pub fn draw(&self, painter: &egui::Painter, map_rect: Rect, uv: Rect) {
        if let Some(fog) = &self.texture {
            painter.image(fog.id(), map_rect, uv, egui::Color32::WHITE);
        }
    }
}

/// Parties afield: gold running dots, the world's first honest movers.
pub fn draw_scouts(world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
    let grid = world.genesis.fields.grid();
    for party in &world.knowledge.parties {
        let (x, y) = grid.xy(party.tile.0 as usize);
        let at = cam.to_screen(rect, egui::Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
        if rect.contains(at) {
            let r = (cam.zoom * 0.3).clamp(2.5, 7.0);
            painter.circle_filled(at, r, egui::Color32::from_rgb(240, 205, 90));
            painter.circle_stroke(at, r, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
    }
}
