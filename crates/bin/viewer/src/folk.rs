//! Presentation-layer hydration (docs/02-simulation-core.md): villagers
//! bustle around settlements and settler caravans travel on real settlement
//! events. Visual only — cohort aggregates remain the authoritative truth.

use std::collections::HashMap;

use cohorts::CohortKey;
use eframe::egui::{self, Rect, Vec2};
use map_export::palette;
use sim_server::World;
use world_map::NO_PROVINCE;

use crate::camera::Camera;

const VILLAGER_SPEED: f32 = 2.2; // cells per real second — bustle is constant
const CARAVAN_SPEED: f32 = 9.0;

struct Villager {
    pos: Vec2,
    target: Vec2,
    nation: u32,
}

struct Caravan {
    from: Vec2,
    to: Vec2,
    progress: f32,
    nation: u32,
}

#[derive(Default)]
pub struct Folk {
    villagers: HashMap<u32, Vec<Villager>>,
    caravans: Vec<Caravan>,
    rng: u64,
}

impl Folk {
    /// Called on `ProvinceSettled` events: a caravan sets out.
    pub fn spawn_caravan(&mut self, world: &World, from: u32, to: u32, nation: u32) {
        let centre = |p: u32| {
            let c = world.genesis.provinces[p as usize].center;
            Vec2::new(c.0 as f32, c.1 as f32)
        };
        self.caravans.push(Caravan {
            from: centre(from),
            to: centre(to),
            progress: 0.0,
            nation,
        });
    }

    pub fn update(&mut self, world: &World, dt: f32) {
        self.sync_villagers(world);
        for flock in self.villagers.values_mut() {
            for v in flock.iter_mut() {
                let delta = v.target - v.pos;
                let dist = delta.length();
                if dist < 0.15 {
                    v.target = v.pos; // re-targeted in sync pass below
                } else {
                    v.pos += delta * (VILLAGER_SPEED * dt / dist).min(1.0);
                }
            }
        }
        self.retarget_idle(world);
        for caravan in &mut self.caravans {
            let journey = (caravan.to - caravan.from).length().max(1.0);
            caravan.progress += CARAVAN_SPEED * dt / journey;
        }
        self.caravans.retain(|c| c.progress < 1.0);
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect) {
        if cam.zoom < 1.2 {
            return; // too far out for individuals; the territory tint carries it
        }
        let radius = (cam.zoom * 0.16).clamp(1.3, 4.5);
        for flock in self.villagers.values() {
            for v in flock {
                let at = cam.to_screen(rect, v.pos);
                if rect.contains(at) {
                    let (r, g, b) = palette::id_color(v.nation);
                    painter.circle_filled(at, radius, egui::Color32::from_rgb(r, g, b));
                }
            }
        }
        for caravan in &self.caravans {
            let head = caravan.from + (caravan.to - caravan.from) * caravan.progress.min(1.0);
            for k in 0..5 {
                let lag = caravan.from
                    + (caravan.to - caravan.from)
                        * (caravan.progress - 0.012 * k as f32).clamp(0.0, 1.0);
                let at = cam.to_screen(rect, lag);
                if rect.contains(at) {
                    painter.circle_filled(at, radius * 1.1, egui::Color32::from_rgb(240, 205, 90));
                }
            }
            let at = cam.to_screen(rect, head);
            if rect.contains(at) {
                let (r, g, b) = palette::id_color(caravan.nation);
                painter.circle_stroke(
                    at,
                    radius * 2.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(r, g, b)),
                );
            }
        }
    }

    /// Keep villager counts proportional to settlement populations.
    fn sync_villagers(&mut self, world: &World) {
        for nation in &world.nations.nations {
            for p in world.nations.owned_provinces(nation.id) {
                let pop = world
                    .cohorts
                    .population_of(CohortKey {
                        province: p,
                        species: nation.species,
                    })
                    .to_num::<i64>();
                let want = usize::try_from((pop / 9).clamp(3, 26)).expect("clamped positive");
                let centre = world.genesis.provinces[p.0 as usize].center;
                let centre = Vec2::new(centre.0 as f32, centre.1 as f32);
                let have = self.villagers.get(&p.0).map_or(0, Vec::len);
                if have < want {
                    let mut newcomers = Vec::with_capacity(want - have);
                    for _ in have..want {
                        let jitter =
                            Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * 3.0;
                        newcomers.push(Villager {
                            pos: centre + jitter,
                            target: centre + jitter,
                            nation: nation.id.0,
                        });
                    }
                    self.villagers.entry(p.0).or_default().extend(newcomers);
                } else if have > want {
                    self.villagers
                        .get_mut(&p.0)
                        .expect("present")
                        .truncate(want);
                }
            }
        }
    }

    /// Idle villagers pick a new spot inside their home province.
    fn retarget_idle(&mut self, world: &World) {
        let size = world.genesis.fields.size;
        let of_cell = &world.genesis.province_of_cell;
        let mut draws: Vec<(u32, usize)> = Vec::new();
        for (&province, flock) in &self.villagers {
            for (vi, v) in flock.iter().enumerate() {
                if (v.target - v.pos).length() < 0.15 {
                    draws.push((province, vi));
                }
            }
        }
        for (province, vi) in draws {
            let cells = world.genesis.provinces[province as usize].cells as f32;
            let radius = (cells.sqrt() * 0.4).max(2.0);
            let centre = world.genesis.provinces[province as usize].center;
            let centre = Vec2::new(centre.0 as f32, centre.1 as f32);
            let mut target = centre;
            for _ in 0..5 {
                let candidate = centre
                    + Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * radius * 2.0;
                let (x, y) = (candidate.x as i64, candidate.y as i64);
                if x >= 0 && y >= 0 && x < i64::from(size) && y < i64::from(size) {
                    let idx = (y as usize) * size as usize + x as usize;
                    if of_cell[idx] == province && of_cell[idx] != NO_PROVINCE {
                        target = candidate;
                        break;
                    }
                }
            }
            if let Some(flock) = self.villagers.get_mut(&province)
                && let Some(v) = flock.get_mut(vi)
            {
                v.target = target;
            }
        }
    }

    /// Visual-only LCG in [0, 1) — the sim's determinism rules don't apply here.
    fn next_unit(&mut self) -> f32 {
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.rng >> 40) as f32) / ((1u64 << 24) as f32)
    }
}
