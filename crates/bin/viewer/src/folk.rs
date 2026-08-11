//! World-layer moving units: settler caravans traveling between tiles on
//! real settlement events. Individuals live on local maps (`localfolk`),
//! never on the strategic map (docs/15-multiscale-maps.md).

use eframe::egui::{self, Rect, Vec2};
use map_export::palette;
use sim_server::World;

use crate::camera::Camera;

const CARAVAN_SPEED: f32 = 2.4; // world tiles per real second

struct Caravan {
    from: Vec2,
    to: Vec2,
    progress: f32,
    nation: u32,
}

#[derive(Default)]
pub struct Folk {
    caravans: Vec<Caravan>,
}

impl Folk {
    /// Called on `TileSettled` events: a caravan sets out.
    pub fn spawn_caravan(&mut self, world: &World, from: u32, to: u32, nation: u32) {
        let grid = world.genesis.fields.grid();
        let centre = |t: u32| {
            let (x, y) = grid.xy(t as usize);
            Vec2::new(x as f32 + 0.5, y as f32 + 0.5)
        };
        self.caravans.push(Caravan {
            from: centre(from),
            to: centre(to),
            progress: 0.0,
            nation,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for caravan in &mut self.caravans {
            let journey = (caravan.to - caravan.from).length().max(0.5);
            caravan.progress += CARAVAN_SPEED * dt / journey;
        }
        self.caravans.retain(|c| c.progress < 1.0);
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect) {
        let radius = (cam.zoom * 0.22).clamp(1.5, 5.0);
        for caravan in &self.caravans {
            let head = caravan.from + (caravan.to - caravan.from) * caravan.progress.min(1.0);
            let at = cam.to_screen(rect, head);
            if rect.contains(at) {
                painter.circle_filled(at, radius, egui::Color32::from_rgb(240, 205, 90));
                let (r, g, b) = palette::id_color(caravan.nation);
                painter.circle_stroke(
                    at,
                    radius * 1.7,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(r, g, b)),
                );
            }
        }
    }
}
