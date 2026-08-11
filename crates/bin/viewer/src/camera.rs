//! Pan/zoom camera over cell space: drag, arrows/WASD, wheel zoom-to-cursor.

use eframe::egui::{self, Pos2, Rect, Vec2};

pub struct Camera {
    /// Cell coordinate at the viewport center.
    pub center: Vec2,
    /// Screen pixels per cell.
    pub zoom: f32,
}

impl Camera {
    #[must_use]
    pub fn fit(map_size: u32, viewport: Vec2) -> Self {
        let size = map_size as f32;
        Self {
            center: Vec2::splat(size / 2.0),
            zoom: (viewport.x.min(viewport.y) / size).max(0.5),
        }
    }

    #[must_use]
    pub fn to_screen(&self, rect: Rect, cell: Vec2) -> Pos2 {
        rect.center() + (cell - self.center) * self.zoom
    }

    #[must_use]
    pub fn to_cell(&self, rect: Rect, screen: Pos2) -> Vec2 {
        (screen - rect.center()) / self.zoom + self.center
    }

    /// Apply drag, keyboard pan, and wheel zoom for this frame.
    pub fn handle(&mut self, ui: &egui::Ui, rect: Rect, response: &egui::Response, dt: f32) {
        if response.dragged() {
            self.center -= response.drag_delta() / self.zoom;
        }
        let pan = 420.0 * dt / self.zoom;
        ui.input(|i| {
            if i.key_down(egui::Key::A) || i.key_down(egui::Key::ArrowLeft) {
                self.center.x -= pan;
            }
            if i.key_down(egui::Key::D) || i.key_down(egui::Key::ArrowRight) {
                self.center.x += pan;
            }
            if i.key_down(egui::Key::W) || i.key_down(egui::Key::ArrowUp) {
                self.center.y -= pan;
            }
            if i.key_down(egui::Key::S) || i.key_down(egui::Key::ArrowDown) {
                self.center.y += pan;
            }
        });
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.0
            && let Some(cursor) = response.hover_pos()
        {
            let before = self.to_cell(rect, cursor);
            self.zoom = (self.zoom * 1.1f32.powf(scroll / 60.0)).clamp(0.4, 48.0);
            let after = self.to_cell(rect, cursor);
            self.center += before - after;
        }
    }
}
