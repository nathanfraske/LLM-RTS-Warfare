//! Person-scale people (docs/17-presence.md P1): villagers with real jobs
//! from the tile's labor allocation. Hunters chase live animals and carry
//! kills home; gatherers fell trees that visibly drop; fishers work the
//! shore; cultivators tend the rows. Water blocks, trees slow. Presentation
//! only — the ledgers stay authoritative. Wildlife lives in `critters`.

use eframe::egui::{self, Color32, Rect, Vec2};
use local_map::LocalMap;
use map_export::palette;
use world_map::Water;

use crate::camera::Camera;
use crate::critters::{LocalCritters, WildSpec, cell, next_unit, shoreline};

const WALK_SPEED: f32 = 9.0; // local cells per real second
const TREE_SLOW: f32 = 0.55;
const CATCH_RADIUS: f32 = 1.4;

#[derive(Clone, Copy)]
enum Task {
    ToWork,
    Working(f32),
    ToCamp,
    Resting(f32),
}

struct Person {
    pos: Vec2,
    target: Vec2,
    /// Channel index: gather, hunt, fish, cultivate, herd.
    job: usize,
    task: Task,
}

pub struct LocalFolk {
    people: Vec<Person>,
    wild: LocalCritters,
    nation: u32,
    camp: Vec2,
    shoreline: Vec<Vec2>,
    has_farm: bool,
    rng: u64,
}

impl LocalFolk {
    #[must_use]
    pub fn new(
        map: &LocalMap,
        labor_milli: [u16; 5],
        count: usize,
        nation: u32,
        wildlife: &[WildSpec],
    ) -> Self {
        let camp = map
            .camp
            .map_or(Vec2::splat(map.size as f32 / 2.0), |(x, y)| {
                Vec2::new(x as f32, y as f32)
            });
        let mut folk = Self {
            people: Vec::new(),
            wild: LocalCritters::new(map, camp, wildlife),
            nation,
            camp,
            shoreline: shoreline(map),
            has_farm: map.works.contains(&directive_schema::WorkKind::Farmstead),
            rng: 0x5EED_F01C,
        };
        if map.camp.is_some() {
            // Deal jobs proportionally to the labor weights.
            let total: u32 = labor_milli
                .iter()
                .map(|&w| u32::from(w))
                .sum::<u32>()
                .max(1);
            let mut dealt = 0usize;
            for (job, &w) in labor_milli.iter().enumerate() {
                let n = (count * u32::from(w) as usize) / total as usize;
                for _ in 0..n {
                    folk.spawn_person(job);
                    dealt += 1;
                }
            }
            while dealt < count {
                folk.spawn_person(0);
                dealt += 1;
            }
        }
        folk
    }

    fn spawn_person(&mut self, job: usize) {
        let jitter = Vec2::new(
            next_unit(&mut self.rng) - 0.5,
            next_unit(&mut self.rng) - 0.5,
        ) * 8.0;
        let rest = next_unit(&mut self.rng) * 2.0;
        let pos = self.camp + jitter;
        self.people.push(Person {
            pos,
            target: pos,
            job,
            task: Task::Resting(rest),
        });
    }

    /// Advance everyone one frame; returns cells whose tree was felled.
    pub fn update(&mut self, map: &LocalMap, dt: f32) -> Vec<usize> {
        let positions: Vec<Vec2> = self.people.iter().map(|p| p.pos).collect();
        self.wild.update(&positions, map, dt);
        let mut felled = Vec::new();
        for i in 0..self.people.len() {
            match self.people[i].task {
                Task::Resting(t) => {
                    if t <= 0.0 {
                        let job = self.people[i].job;
                        self.people[i].target = self.work_target(map, job);
                        self.people[i].task = Task::ToWork;
                    } else {
                        self.people[i].task = Task::Resting(t - dt);
                    }
                }
                Task::ToWork => {
                    // Hunters chase live quarry; everyone else walks to a spot.
                    if self.people[i].job == 1
                        && let Some(ci) = self.wild.nearest(self.people[i].pos)
                    {
                        let quarry = self.wild.position(ci);
                        self.people[i].target = quarry;
                        if (quarry - self.people[i].pos).length() < CATCH_RADIUS {
                            self.wild.take(ci);
                            self.head_home(i);
                            continue;
                        }
                    }
                    if self.walk(map, i, dt) {
                        self.people[i].task = Task::Working(2.5 + next_unit(&mut self.rng) * 3.5);
                    }
                }
                Task::Working(t) => {
                    let wiggle = Vec2::new(
                        next_unit(&mut self.rng) - 0.5,
                        next_unit(&mut self.rng) - 0.5,
                    ) * 0.35;
                    let anchor = self.people[i].target;
                    let p = &mut self.people[i];
                    p.pos = (p.pos + wiggle)
                        .clamp(anchor - Vec2::splat(1.2), anchor + Vec2::splat(1.2));
                    if t <= 0.0 {
                        // A gatherer finishing at a tree brings it down.
                        if self.people[i].job == 0
                            && let Some(c) = cell(map, self.people[i].target)
                            && map.tree[c]
                        {
                            felled.push(c);
                        }
                        self.head_home(i);
                    } else {
                        self.people[i].task = Task::Working(t - dt);
                    }
                }
                Task::ToCamp => {
                    if self.walk(map, i, dt) {
                        self.people[i].task = Task::Resting(0.8 + next_unit(&mut self.rng) * 1.6);
                    }
                }
            }
        }
        felled
    }

    fn head_home(&mut self, i: usize) {
        let jitter = Vec2::new(
            next_unit(&mut self.rng) - 0.5,
            next_unit(&mut self.rng) - 0.5,
        ) * 6.0;
        self.people[i].target = self.camp + jitter;
        self.people[i].task = Task::ToCamp;
    }

    /// One terrain-aware step; true when arrived. Water blocks, trees slow.
    fn walk(&mut self, map: &LocalMap, i: usize, dt: f32) -> bool {
        let p = self.people[i].pos;
        let delta = self.people[i].target - p;
        let dist = delta.length();
        if dist < 0.5 {
            return true;
        }
        let slow = if cell(map, p).is_some_and(|c| map.tree[c]) {
            TREE_SLOW
        } else {
            1.0
        };
        let step = (WALK_SPEED * slow * dt).min(dist);
        let dir = delta / dist;
        for angle in [0.0f32, 0.7, -0.7, 1.4, -1.4] {
            let (sin, cos) = angle.sin_cos();
            let d = Vec2::new(dir.x * cos - dir.y * sin, dir.x * sin + dir.y * cos);
            let next = p + d * step;
            if cell(map, next).is_some_and(|c| map.water[c] == Water::Dry) {
                self.people[i].pos = next;
                return false;
            }
        }
        self.people[i].target = p;
        true
    }

    /// Where this job gets done, on this actual terrain.
    fn work_target(&mut self, map: &LocalMap, job: usize) -> Vec2 {
        let size = map.size as f32;
        match job {
            2 if !self.shoreline.is_empty() => {
                let i = (next_unit(&mut self.rng) * self.shoreline.len() as f32) as usize;
                self.shoreline[i.min(self.shoreline.len() - 1)]
            }
            3 if self.has_farm => {
                self.camp
                    + Vec2::new(
                        9.0 + next_unit(&mut self.rng) * 21.0,
                        (next_unit(&mut self.rng) - 0.5) * 20.0,
                    )
            }
            4 => {
                let r = 12.0 + next_unit(&mut self.rng) * 9.0;
                let a = next_unit(&mut self.rng) * std::f32::consts::TAU;
                self.camp + Vec2::new(a.cos(), a.sin()) * r
            }
            _ => {
                let range = if job == 1 {
                    40.0 + next_unit(&mut self.rng) * 70.0
                } else {
                    12.0 + next_unit(&mut self.rng) * 45.0
                };
                for attempt in 0..12 {
                    let a = next_unit(&mut self.rng) * std::f32::consts::TAU;
                    let cand = self.camp + Vec2::new(a.cos(), a.sin()) * range;
                    if cand.x < 1.0 || cand.y < 1.0 || cand.x >= size - 1.0 || cand.y >= size - 1.0
                    {
                        continue;
                    }
                    if let Some(c) = cell(map, cand)
                        && map.water[c] == Water::Dry
                    {
                        // Gatherers prefer a standing tree, then any green.
                        let good = if job == 0 {
                            map.tree[c] || (attempt > 5 && map.veg[c] > 90)
                        } else {
                            true
                        };
                        if good {
                            return cand;
                        }
                    }
                }
                self.camp
                    + Vec2::new(
                        next_unit(&mut self.rng) - 0.5,
                        next_unit(&mut self.rng) - 0.5,
                    ) * 10.0
            }
        }
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect) {
        let radius = (cam.zoom * 0.45).clamp(1.0, 6.0);
        self.wild.draw(painter, cam, rect, radius);
        let (r, g, b) = palette::id_color(self.nation);
        let body = Color32::from_rgb(
            r.saturating_add(40),
            g.saturating_add(40),
            b.saturating_add(40),
        );
        let load = Color32::from_rgb(235, 200, 90);
        for p in &self.people {
            let at = cam.to_screen(rect, p.pos);
            if rect.contains(at) {
                painter.circle_filled(at, radius, body);
                if matches!(p.task, Task::ToCamp) {
                    painter.circle_filled(
                        at + egui::vec2(radius * 0.7, -radius * 0.7),
                        radius * 0.45,
                        load,
                    );
                }
            }
        }
    }
}
