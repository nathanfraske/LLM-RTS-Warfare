//! Person-scale individuals with real jobs (docs/17-presence.md P1):
//! villagers are assigned to the tile's actual labor channels and run a
//! work → carry-home → rest loop. Terrain is respected — water is
//! impassable, trees slow the walk. Presentation-layer only; the ledgers
//! stay authoritative.

use eframe::egui::{self, Rect, Vec2};
use local_map::LocalMap;
use map_export::palette;
use world_map::Water;

use crate::camera::Camera;

const WALK_SPEED: f32 = 9.0; // local cells per real second
const TREE_SLOW: f32 = 0.55;

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
    nation: u32,
    camp: Vec2,
    shoreline: Vec<Vec2>,
    has_farm: bool,
    rng: u64,
}

impl LocalFolk {
    #[must_use]
    pub fn new(map: &LocalMap, labor_milli: [u16; 5], count: usize, nation: u32) -> Self {
        let camp = map
            .camp
            .map_or(Vec2::splat(map.size as f32 / 2.0), |(x, y)| {
                Vec2::new(x as f32, y as f32)
            });
        let mut folk = Self {
            people: Vec::new(),
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
        let jitter = Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * 8.0;
        let rest = self.next_unit() * 2.0;
        let pos = self.camp + jitter;
        self.people.push(Person {
            pos,
            target: pos,
            job,
            task: Task::Resting(rest),
        });
    }

    pub fn update(&mut self, map: &LocalMap, dt: f32) {
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
                    if self.walk(map, i, dt) {
                        self.people[i].task = Task::Working(2.5 + self.next_unit() * 3.5);
                    }
                }
                Task::Working(t) => {
                    // A busy shuffle around the worksite.
                    let wiggle = Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * 0.35;
                    let anchor = self.people[i].target;
                    let p = &mut self.people[i];
                    p.pos = (p.pos + wiggle)
                        .clamp(anchor - Vec2::splat(1.2), anchor + Vec2::splat(1.2));
                    if t <= 0.0 {
                        let jitter =
                            Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * 6.0;
                        self.people[i].target = self.camp + jitter;
                        self.people[i].task = Task::ToCamp;
                    } else {
                        self.people[i].task = Task::Working(t - dt);
                    }
                }
                Task::ToCamp => {
                    if self.walk(map, i, dt) {
                        self.people[i].task = Task::Resting(0.8 + self.next_unit() * 1.6);
                    }
                }
            }
        }
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
        // Try straight, then deflect around water.
        for angle in [0.0f32, 0.7, -0.7, 1.4, -1.4] {
            let (sin, cos) = angle.sin_cos();
            let d = Vec2::new(dir.x * cos - dir.y * sin, dir.x * sin + dir.y * cos);
            let next = p + d * step;
            if cell(map, next).is_some_and(|c| map.water[c] == Water::Dry) {
                self.people[i].pos = next;
                return false;
            }
        }
        // Boxed in: give up on this destination.
        self.people[i].target = p;
        true
    }

    /// Where this job gets done, on this actual terrain.
    fn work_target(&mut self, map: &LocalMap, job: usize) -> Vec2 {
        let size = map.size as f32;
        match job {
            // Fish: the shore, if there is one.
            2 if !self.shoreline.is_empty() => {
                let i = (self.next_unit() * self.shoreline.len() as f32) as usize;
                self.shoreline[i.min(self.shoreline.len() - 1)]
            }
            // Cultivate: the farm rows east of camp when fields exist.
            3 if self.has_farm => {
                self.camp
                    + Vec2::new(
                        9.0 + self.next_unit() * 21.0,
                        (self.next_unit() - 0.5) * 20.0,
                    )
            }
            // Herd: the meadow ring around camp.
            4 => {
                let r = 12.0 + self.next_unit() * 9.0;
                let a = self.next_unit() * std::f32::consts::TAU;
                self.camp + Vec2::new(a.cos(), a.sin()) * r
            }
            // Hunt ranges far; gather works the green nearby.
            _ => {
                let range = if job == 1 {
                    40.0 + self.next_unit() * 70.0
                } else {
                    12.0 + self.next_unit() * 45.0
                };
                for _ in 0..12 {
                    let a = self.next_unit() * std::f32::consts::TAU;
                    let cand = self.camp + Vec2::new(a.cos(), a.sin()) * range;
                    if cand.x < 1.0 || cand.y < 1.0 || cand.x >= size - 1.0 || cand.y >= size - 1.0
                    {
                        continue;
                    }
                    if let Some(c) = cell(map, cand)
                        && map.water[c] == Water::Dry
                        && (job == 1 || map.veg[c] > 90)
                    {
                        return cand;
                    }
                }
                self.camp + Vec2::new(self.next_unit() - 0.5, self.next_unit() - 0.5) * 10.0
            }
        }
    }

    pub fn draw(&self, painter: &egui::Painter, cam: &Camera, rect: Rect) {
        let radius = (cam.zoom * 0.45).clamp(1.0, 6.0);
        let (r, g, b) = palette::id_color(self.nation);
        let body = egui::Color32::from_rgb(
            r.saturating_add(40),
            g.saturating_add(40),
            b.saturating_add(40),
        );
        let load = egui::Color32::from_rgb(235, 200, 90);
        for p in &self.people {
            let at = cam.to_screen(rect, p.pos);
            if rect.contains(at) {
                painter.circle_filled(at, radius, body);
                // Homeward-bound workers visibly carry their take.
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

    /// Visual-only LCG in [0, 1).
    fn next_unit(&mut self) -> f32 {
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.rng >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

fn cell(map: &LocalMap, pos: Vec2) -> Option<usize> {
    let (x, y) = (pos.x as i64, pos.y as i64);
    let size = i64::from(map.size);
    (x >= 0 && y >= 0 && x < size && y < size)
        .then(|| (y as usize) * map.size as usize + x as usize)
}

/// Dry cells that touch water — where fishers stand. Sampled sparsely.
fn shoreline(map: &LocalMap) -> Vec<Vec2> {
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
