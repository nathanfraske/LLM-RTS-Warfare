//! The sky over the map (docs/26 §4): day and night from the sim clock,
//! day length from axial tilt and latitude, night depth from the moon.
//! Pure presentation — every number derives from authoritative state.

use eframe::egui::{self, Color32, Rect};
use sim_server::World;

/// Moon brightness 0..=1: a triangle over the lunar period, full at mid.
fn moon(world: &World) -> f32 {
    let period = f32::from(world.tuning.sky.moon_period_days.max(1));
    let day = (world.tick().0 / 24) as f32;
    let phase = (day / period).fract();
    1.0 - (2.0 * phase - 1.0).abs()
}

/// Row darkness 0..=1 for the current hour: inside the row's daylight span
/// it is zero; night deepens by tuning and lightens under the moon.
fn darkness(world: &World, y: u32) -> f32 {
    let tick = world.tick().0;
    let hour = (tick % 24) as f32 + 0.5;
    let month = tick / 720;
    let size = world.genesis.fields.size;
    let daylight = f32::from(climate::daylight_milli(
        y,
        size,
        month,
        &world.tuning.seasons,
        world.tuning.sky.axial_tilt_deci,
    )) / 1000.0;
    let half_day = 12.0 * daylight;
    let from_noon = (hour - 12.0).abs();
    // A 1.5-hour dawn/dusk ramp around the daylight edge.
    let light = ((half_day - from_noon) / 1.5 + 0.5).clamp(0.0, 1.0);
    let depth = f32::from(world.tuning.sky.night_depth_permille) / 1000.0;
    (1.0 - light) * depth * (1.0 - 0.55 * moon(world))
}

/// Paint the night over the world map, row by row — the terminator and the
/// polar nights are visible shapes, not a global dimmer.
pub fn draw_night(world: &World, painter: &egui::Painter, view: Rect, map_rect: Rect) {
    let size = world.genesis.fields.size;
    let row_height = map_rect.height() / size as f32;
    for y in 0..size {
        let dark = darkness(world, y);
        if dark <= 0.01 {
            continue;
        }
        let top = map_rect.top() + row_height * y as f32;
        let row = Rect::from_min_max(
            egui::pos2(map_rect.left(), top),
            egui::pos2(map_rect.right(), top + row_height + 0.5),
        )
        .intersect(view);
        if row.is_positive() {
            painter.rect_filled(
                row,
                0.0,
                Color32::from_rgba_unmultiplied(6, 9, 26, (dark * 255.0) as u8),
            );
        }
    }
}

/// The same night, as one tint for a local map (the tile's own row).
#[must_use]
pub fn local_night(world: &World, tile: u32) -> Option<Color32> {
    let y = tile / world.genesis.fields.size;
    let dark = darkness(world, y);
    (dark > 0.01).then(|| Color32::from_rgba_unmultiplied(6, 9, 26, (dark * 255.0) as u8))
}
