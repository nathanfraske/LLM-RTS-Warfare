//! The sky over the map (docs/26 §4): day and night from the sim clock,
//! day length from axial tilt and latitude, night depth from the moon.
//! Pure presentation — every number derives from authoritative state.

use eframe::egui::{self, Color32, ColorImage, Rect, Vec2};
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

/// The light over the land right now: direction it shines FROM (unit-ish),
/// altitude 0..=1, and strength 0..=1 (sun by day, phase-lit moon by night).
#[must_use]
pub fn light(world: &World) -> (Vec2, f32, f32) {
    let tick = world.tick().0;
    let hour = (tick % 24) as f32 + 0.5;
    let day = (6.0..18.0).contains(&hour);
    // Sun sweeps east to west over the day; the moon walks the same road
    // twelve hours behind.
    let sweep = if day {
        (hour - 6.0) / 12.0
    } else {
        (((hour + 12.0) % 24.0) - 6.0) / 12.0
    };
    let azimuth = sweep * std::f32::consts::PI;
    // Light FROM the east at dawn (dir points west-to-east reversed).
    let dir = Vec2::new(-azimuth.cos(), -0.45);
    let altitude = (sweep * std::f32::consts::PI).sin().max(0.05);
    let strength = if day { 1.0 } else { 0.35 * moon(world) };
    (dir.normalized(), altitude, strength)
}

/// Relief against the live sun: slopes toward the light brighten, slopes
/// away fall into shade — and the whole picture turns with the hour.
#[must_use]
pub fn shade_image(world: &World) -> ColorImage {
    let fields = &world.genesis.fields;
    let size = fields.size as usize;
    let (dir, altitude, strength) = light(world);
    // Low sun, long shadows: contrast rises as altitude falls.
    let contrast = (34.0 + 66.0 * (1.0 - altitude)) * strength;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let i = y * size + x;
            if fields.elevation[i] < 0 {
                continue;
            }
            let e = |xx: usize, yy: usize| fields.elevation[yy * size + xx].max(0) as f32;
            let gx = e((x + 1).min(size - 1), y) - e(x.saturating_sub(1), y);
            let gy = e(x, (y + 1).min(size - 1)) - e(x, y.saturating_sub(1));
            // Facing the light > 0, facing away < 0.
            let facing = ((-gx * dir.x - gy * dir.y) / 900.0).clamp(-1.0, 1.0);
            let at = i * 4;
            if facing >= 0.0 {
                rgba[at] = 255;
                rgba[at + 1] = 250;
                rgba[at + 2] = 235;
                rgba[at + 3] = (facing * contrast * 0.8) as u8;
            } else {
                rgba[at + 3] = (-facing * contrast) as u8;
            }
        }
    }
    ColorImage::from_rgba_unmultiplied([size, size], &rgba)
}

/// Shadow cast on the ground by a standing thing at person scale:
/// direction away from the light and length from its altitude, or `None`
/// in the black of a new-moon night.
#[must_use]
pub fn cast(world: &World) -> Option<(Vec2, f32, u8)> {
    let (dir, altitude, strength) = light(world);
    if strength < 0.08 {
        return None;
    }
    let length = (1.2 + 3.5 * (1.0 - altitude)) * (0.6 + 0.4 * strength);
    let alpha = (26.0 + 44.0 * strength) as u8;
    Some((Vec2::new(dir.x, 0.6), length, alpha))
}
