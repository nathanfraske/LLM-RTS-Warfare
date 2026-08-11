//! Structures at world scale (docs/30): a settled tile's buildings show
//! as small material-colored blocks beside the camp dot, sized by their
//! true footprints — a roomy long-store reads bigger than a hearth-house.

use eframe::egui::{self, Color32, Rect, Vec2};
use sim_server::World;

use crate::camera::Camera;

pub fn draw_world(world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
    if cam.zoom < 2.4 {
        return;
    }
    let grid = world.genesis.fields.grid();
    for (tile, owner) in world.nations.owner.iter().enumerate() {
        if owner.is_none() {
            continue;
        }
        let standing = world
            .nations
            .works
            .completed(u32::try_from(tile).expect("tile fits"));
        if standing.is_empty() {
            continue;
        }
        let (x, y) = grid.xy(tile);
        for (i, b) in standing.iter().take(3).enumerate() {
            let (w, d) = b.design.footprint();
            let scale = (f32::from(w) * f32::from(d)).sqrt() / 14.0;
            let side = (cam.zoom * (0.16 + 0.22 * scale)).clamp(1.5, 7.0);
            let offset = Vec2::new(0.22 + 0.3 * i as f32 - 0.3, 0.3 - 0.25 * i as f32);
            let at = cam.to_screen(rect, Vec2::new(x as f32 + 0.5, y as f32 + 0.5) + offset);
            if !rect.contains(at) {
                continue;
            }
            let color = match b.design.wall_class() {
                2 => Color32::from_rgb(150, 148, 154),
                3 => Color32::from_rgb(124, 88, 50),
                _ => Color32::from_rgb(163, 120, 70),
            };
            painter.rect_filled(
                Rect::from_center_size(at, Vec2::new(side, side * 0.8)),
                1.0,
                color,
            );
        }
    }
}
