//! Wild critters on a local map: instanced from the tile's real fauna,
//! wandering their terrain, fleeing people, caught by hunters. The shared
//! terrain-sampling helpers for the presence layer live here too.

use eframe::egui::{self, Color32, Rect, Vec2};
use local_map::LocalMap;
use world_map::Water;

use crate::camera::Camera;

const CRITTER_SPEED: f32 = 6.5;
const FLEE_SPEED: f32 = 13.0;
const FLEE_RADIUS: f32 = 7.0;

/// A species' visual spawn spec on this tile: how many, what color.
pub struct WildSpec {
    pub count: usize,
    pub color: (u8, u8, u8),
}

struct Critter {
    pos: Vec2,
    target: Vec2,
    color: Color32,
    flee: f32,
}

/// The tile's visible wildlife.
#[derive(Default)]
pub struct LocalCritters {
    critters: Vec<Critter>,
    rng: u64,
}

impl LocalCritters {
    #[must_use]
    pub fn new(map: &LocalMap, camp: Vec2, wildlife: &[WildSpec]) -> Self {
        let mut wild = Self {
            critters: Vec::new(),
            rng: 0xC217_7E55,
        };
        for spec in wildlife {
            for _ in 0..spec.count {
                wild.spawn(map, camp, spec.color);
            }
        }
        wild
    }

    fn spawn(&mut self, map: &LocalMap, camp: Vec2, color: (u8, u8, u8)) {
        let size = map.size as f32;
        for _ in 0..14 {
            let pos = Vec2::new(
                1.0 + next_unit(&mut self.rng) * (size - 2.0),
                1.0 + next_unit(&mut self.rng) * (size - 2.0),
            );
            if let Some(c) = cell(map, pos)
                && map.water[c] == Water::Dry
                && (pos - camp).length() > 20.0
            {
                self.critters.push(Critter {
                    pos,
                    target: pos,
                    color: Color32::from_rgb(color.0, color.1, color.2),
                    flee: 0.0,
                });
                return;
            }
        }
    }

    /// Wander, and run from anyone who comes close.
    pub fn update(&mut self, people: &[Vec2], map: &LocalMap, dt: f32) {
        let size = map.size as f32;
        for i in 0..self.critters.len() {
            let pos = self.critters[i].pos;
            let threat = people
                .iter()
                .map(|p| (*p - pos).length())
                .fold(f32::MAX, f32::min);
            if threat < FLEE_RADIUS {
                let away: Vec2 = people
                    .iter()
                    .filter(|p| (**p - pos).length() < FLEE_RADIUS + 4.0)
                    .map(|p| pos - *p)
                    .fold(Vec2::ZERO, |a, b| a + b);
                if away.length() > 0.1 {
                    self.critters[i].target = (pos + away.normalized() * 14.0)
                        .clamp(Vec2::splat(1.0), Vec2::splat(size - 2.0));
                }
                self.critters[i].flee = 1.2;
            }
            let fleeing = self.critters[i].flee > 0.0;
            self.critters[i].flee = (self.critters[i].flee - dt).max(0.0);
            let speed = if fleeing { FLEE_SPEED } else { CRITTER_SPEED };
            let delta = self.critters[i].target - self.critters[i].pos;
            let dist = delta.length();
            if dist < 0.6 {
                let wander = Vec2::new(
                    next_unit(&mut self.rng) - 0.5,
                    next_unit(&mut self.rng) - 0.5,
                ) * 26.0;
                let cand = (self.critters[i].pos + wander)
                    .clamp(Vec2::splat(1.0), Vec2::splat(size - 2.0));
                if cell(map, cand).is_some_and(|c| map.water[c] == Water::Dry) {
                    self.critters[i].target = cand;
                }
            } else {
                let step = (speed * dt).min(dist);
                let next = self.critters[i].pos + delta / dist * step;
                if cell(map, next).is_some_and(|c| map.water[c] == Water::Dry) {
                    self.critters[i].pos = next;
                } else {
                    self.critters[i].target = self.critters[i].pos;
                }
            }
        }
    }

    /// The closest animal within hunting sight of `from`.
    #[must_use]
    pub fn nearest(&self, from: Vec2) -> Option<usize> {
        (0..self.critters.len())
            .min_by(|&a, &b| {
                let da = (self.critters[a].pos - from).length();
                let db = (self.critters[b].pos - from).length();
                da.total_cmp(&db)
            })
            .filter(|&i| (self.critters[i].pos - from).length() < 90.0)
    }

    #[must_use]
    pub fn position(&self, i: usize) -> Vec2 {
        self.critters[i].pos
    }

    /// A hunter's catch: the animal is taken.
    pub fn take(&mut self, i: usize) {
        self.critters.swap_remove(i);
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect, radius: f32) {
        for critter in &self.critters {
            let at = cam.to_screen(rect, critter.pos);
            if rect.contains(at) {
                painter.circle_filled(at, radius * 0.9, critter.color);
            }
        }
    }
}

/// Visual-only LCG in [0, 1) — sim determinism rules don't apply up here.
pub fn next_unit(rng: &mut u64) -> f32 {
    *rng = rng
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*rng >> 40) as f32) / ((1u64 << 24) as f32)
}

/// The cell index under a presence-space position, if it's on the map.
#[must_use]
pub fn cell(map: &LocalMap, pos: Vec2) -> Option<usize> {
    let (x, y) = (pos.x as i64, pos.y as i64);
    let size = i64::from(map.size);
    (x >= 0 && y >= 0 && x < size && y < size)
        .then(|| (y as usize) * map.size as usize + x as usize)
}

/// Dry cells that touch water — where fishers stand. Sampled sparsely.
#[must_use]
pub fn shoreline(map: &LocalMap) -> Vec<Vec2> {
    let size = map.size as usize;
    let mut out = Vec::new();
    for y in (1..size - 1).step_by(3) {
        for x in (1..size - 1).step_by(3) {
            let i = y * size + x;
            if map.water[i] != Water::Dry {
                continue;
            }
            let wet = [i - 1, i + 1, i - size, i + size]
                .iter()
                .any(|&n| map.water[n] != Water::Dry);
            if wet {
                out.push(Vec2::new(x as f32, y as f32));
                if out.len() >= 240 {
                    return out;
                }
            }
        }
    }
    out
}
