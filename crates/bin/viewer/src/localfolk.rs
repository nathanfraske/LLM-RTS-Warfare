//! Person-scale individuals on a local map: villagers bustle around the camp,
//! one body ≈ one cell (docs/15-multiscale-maps.md). Presentation-layer
//! hydration — the cohort aggregate stays the authoritative truth.

use eframe::egui::{self, Rect, Vec2};
use local_map::LocalMap;
use map_export::palette;
use world_map::Water;

use crate::camera::Camera;

const WALK_SPEED: f32 = 9.0; // local cells per real second
const WANDER_RADIUS: f32 = 34.0;

struct Person {
    pos: Vec2,
    target: Vec2,
}

pub struct LocalFolk {
    people: Vec<Person>,
    nation: u32,
    rng: u64,
}

impl LocalFolk {
    #[must_use]
    pub fn new(map: &LocalMap, count: usize, nation: u32) -> Self {
        let mut folk = Self {
            people: Vec::new(),
            nation,
            rng: 0x5EED_F01C,
        };
        if let Some((cx, cy)) = map.camp {
            let centre = Vec2::new(cx as f32, cy as f32);
            for _ in 0..count {
                let jitter = Vec2::new(folk.next_unit() - 0.5, folk.next_unit() - 0.5) * 10.0;
                folk.people.push(Person {
                    pos: centre + jitter,
                    target: centre + jitter,
                });
            }
        }
        folk
    }

    pub fn update(&mut self, map: &LocalMap, dt: f32) {
        let Some((cx, cy)) = map.camp else { return };
        let centre = Vec2::new(cx as f32, cy as f32);
        let size = map.size as f32;
        for i in 0..self.people.len() {
            let delta = self.people[i].target - self.people[i].pos;
            let dist = delta.length();
            if dist < 0.4 {
                // Pick a new dry spot near camp.
                let mut target = centre;
                for _ in 0..6 {
                    let candidate = centre
                        + Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5)
                            * WANDER_RADIUS
                            * 2.0;
                    if candidate.x < 1.0
                        || candidate.y < 1.0
                        || candidate.x >= size - 1.0
                        || candidate.y >= size - 1.0
                    {
                        continue;
                    }
                    let idx = (candidate.y as usize) * map.size as usize + candidate.x as usize;
                    if map.water[idx] == Water::Dry {
                        target = candidate;
                        break;
                    }
                }
                self.people[i].target = target;
            } else {
                let step = (WALK_SPEED * dt / dist).min(1.0);
                self.people[i].pos += delta * step;
            }
        }
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect) {
        // One person ≈ one local cell: radius scales with the cell size.
        let radius = (cam.zoom * 0.45).clamp(1.0, 6.0);
        let (r, g, b) = palette::id_color(self.nation);
        let body = egui::Color32::from_rgb(
            r.saturating_add(40),
            g.saturating_add(40),
            b.saturating_add(40),
        );
        for p in &self.people {
            let at = cam.to_screen(rect, p.pos);
            if rect.contains(at) {
                painter.circle_filled(at, radius, body);
            }
        }
    }

    /// Visual-only LCG in [0, 1).
    fn next_unit(&mut self) -> f32 {
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.rng >> 40) as f32) / ((1u64 << 24) as f32)
    }
}
