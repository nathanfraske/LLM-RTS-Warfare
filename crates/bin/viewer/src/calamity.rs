//! The fire made visible (docs/28, docs/29): fresh lava glows along its
//! run and cools over months, ash hangs as a drifting haze over its
//! footprint and settles out, and every vent holds a standing ember.
//! Footprints are recomputed deterministically from the eruption events —
//! presentation reads the same functions the sim used.

use eframe::egui::{self, Color32, Rect, Vec2};
use sim_server::World;

use crate::camera::Camera;

const GLOW_TICKS: u64 = 720 * 2;
const HAZE_TICKS: u64 = 720 * 6;

struct Fall {
    start: u64,
    lava: Vec<u32>,
    ash: Vec<(u32, u8)>,
}

/// The recent eruptions still burning or hanging in the air.
#[derive(Default)]
pub struct Calamities {
    falls: Vec<Fall>,
}

impl Calamities {
    /// Feed every drained `VolcanoErupted` here; the footprint is
    /// recomputed exactly as the sim computed it.
    pub fn record(&mut self, world: &World, vent: u32, tick: u64) {
        let strength = world.genesis.geology.vents[vent as usize];
        self.falls.push(Fall {
            start: tick,
            lava: geology::fire::lava_path(
                &world.genesis.fields,
                vent,
                strength,
                world.tuning.deep.lava_reach,
            ),
            ash: geology::fire::ash_fall(
                &world.genesis.fields,
                vent,
                strength,
                world.tuning.deep.ash_radius,
            ),
        });
    }

    pub fn draw(&mut self, world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
        let now = world.tick().0;
        self.falls
            .retain(|f| now.saturating_sub(f.start) < HAZE_TICKS);
        let half_tile = cam.zoom * 0.5;
        for fall in &self.falls {
            let age = now.saturating_sub(fall.start);
            // Ash: a warm gray veil, thickest where the fall was heavy,
            // thinning as the months settle it out.
            let hang = 1.0 - age as f32 / HAZE_TICKS as f32;
            for &(tile, heaviness) in &fall.ash {
                let at = center(world, cam, rect, tile);
                if rect.contains(at) {
                    let alpha = (f32::from(heaviness) * 0.5 * hang) as u8;
                    painter.rect_filled(
                        Rect::from_center_size(at, Vec2::splat(half_tile * 2.2)),
                        0.0,
                        Color32::from_rgba_unmultiplied(120, 110, 104, alpha),
                    );
                }
            }
            // Lava: white-orange cooling to dull red before it fades.
            if age < GLOW_TICKS {
                let heat = 1.0 - age as f32 / GLOW_TICKS as f32;
                for &tile in &fall.lava {
                    let at = center(world, cam, rect, tile);
                    if rect.contains(at) {
                        let color = Color32::from_rgba_unmultiplied(
                            230,
                            (60.0 + 140.0 * heat) as u8,
                            (20.0 + 40.0 * heat) as u8,
                            (140.0 + 100.0 * heat) as u8,
                        );
                        painter.rect_filled(
                            Rect::from_center_size(at, Vec2::splat(half_tile * 1.8)),
                            0.0,
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Live wildfire: flickering flame marks wherever the blaze stands.
pub fn draw_fires(world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
    let t = world.tick().0 as f32;
    for (tile, &fire) in world.blaze.fire.iter().enumerate() {
        if fire == 0 {
            continue;
        }
        let at = center(world, cam, rect, u32::try_from(tile).expect("tile fits"));
        if !rect.contains(at) {
            continue;
        }
        let flick =
            0.7 + 0.3 * (1.0 - (2.0 * ((t * 0.21 + (tile % 7) as f32 * 0.13).fract()) - 1.0).abs());
        let r = (cam.zoom * 0.4).clamp(2.0, 8.0) * flick;
        painter.circle_filled(at, r, Color32::from_rgba_unmultiplied(235, 120, 30, 170));
        painter.circle_filled(
            at,
            r * 0.5,
            Color32::from_rgba_unmultiplied(255, 225, 130, 220),
        );
    }
}

/// Every vent holds a standing ember, pulsing with the clock.
pub fn draw_vents(world: &World, cam: &Camera, painter: &egui::Painter, rect: Rect) {
    if cam.zoom < 1.6 {
        return;
    }
    let t = world.tick().0 as f32;
    for (tile, &strength) in world.genesis.geology.vents.iter().enumerate() {
        if strength == 0 {
            continue;
        }
        let at = center(world, cam, rect, u32::try_from(tile).expect("tile fits"));
        if rect.contains(at) {
            let pulse = 0.6 + 0.4 * (1.0 - (2.0 * ((t * 0.03).fract()) - 1.0).abs());
            let r = (cam.zoom * 0.28).clamp(2.0, 6.0) * pulse;
            painter.circle_filled(at, r, Color32::from_rgba_unmultiplied(240, 110, 30, 200));
            painter.circle_filled(
                at,
                r * 0.45,
                Color32::from_rgba_unmultiplied(255, 220, 120, 230),
            );
        }
    }
}

fn center(world: &World, cam: &Camera, rect: Rect, tile: u32) -> egui::Pos2 {
    let (x, y) = world.genesis.fields.grid().xy(tile as usize);
    cam.to_screen(rect, Vec2::new(x as f32 + 0.5, y as f32 + 0.5))
}
