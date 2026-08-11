//! The viewer application: drives the sim clock, owns the textures and
//! camera, and lays out map + feed + inspector.

use eframe::egui::{self, Rect, TextureHandle, TextureOptions, Vec2};
use sim_events::Event;
use sim_server::{RunConfig, World};
use world_map::NO_PROVINCE;
use world_schema::ProvinceId;

use crate::camera::Camera;
use crate::feed::{self, Line};
use crate::folk::Folk;
use crate::{hud, layers};

const MAX_TICKS_PER_FRAME: u64 = 30_000;
const FEED_CAP: usize = 400;

pub struct App {
    world: World,
    cam: Camera,
    paused: bool,
    ticks_per_sec: f64,
    tick_debt: f64,
    terrain: TextureHandle,
    territory: TextureHandle,
    territory_dirty: bool,
    seen_events: usize,
    feed: Vec<Line>,
    folk: Folk,
    selected: Option<ProvinceId>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, config: &RunConfig) -> Self {
        let world = World::new(config);
        let terrain = cc.egui_ctx.load_texture(
            "terrain",
            layers::terrain_image(&world),
            TextureOptions::NEAREST,
        );
        let territory = cc.egui_ctx.load_texture(
            "territory",
            layers::territory_image(&world),
            TextureOptions::NEAREST,
        );
        let mut app = Self {
            cam: Camera::fit(world.genesis.fields.size, Vec2::new(1200.0, 800.0)),
            world,
            paused: false,
            ticks_per_sec: 720.0,
            tick_debt: 0.0,
            terrain,
            territory,
            territory_dirty: false,
            seen_events: 0,
            feed: Vec::new(),
            folk: Folk::default(),
            selected: None,
        };
        app.drain_events();
        app
    }

    fn advance(&mut self, dt: f64) {
        if self.paused {
            return;
        }
        self.tick_debt += self.ticks_per_sec * dt;
        let mut steps = self.tick_debt.floor() as u64;
        self.tick_debt -= steps as f64;
        steps = steps.min(MAX_TICKS_PER_FRAME);
        for _ in 0..steps {
            self.world.step();
        }
        if steps > 0 {
            self.drain_events();
        }
    }

    /// Pull new events into the feed, caravans, and overlay refresh.
    fn drain_events(&mut self) {
        let fresh: Vec<Event> = self
            .world
            .log
            .iter()
            .skip(self.seen_events)
            .cloned()
            .collect();
        self.seen_events = self.world.log.len();
        for event in fresh {
            match &event {
                Event::ProvinceSettled {
                    nation,
                    from,
                    province,
                    ..
                } => {
                    self.folk
                        .spawn_caravan(&self.world, from.0, province.0, nation.0);
                    self.territory_dirty = true;
                }
                Event::NationSpawned { .. } => self.territory_dirty = true,
                _ => {}
            }
            if let Some(line) = feed::describe(&event, &self.world) {
                self.feed.push(line);
            }
        }
        if self.feed.len() > FEED_CAP {
            self.feed.drain(..self.feed.len() - FEED_CAP);
        }
    }

    fn map_canvas(&mut self, ui: &mut egui::Ui, dt: f32) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        self.cam.handle(ui, rect, &response, dt);

        let size = self.world.genesis.fields.size as f32;
        let map_rect = Rect::from_min_max(
            self.cam.to_screen(rect, Vec2::ZERO),
            self.cam.to_screen(rect, Vec2::splat(size)),
        );
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(self.terrain.id(), map_rect, uv, egui::Color32::WHITE);
        painter.image(self.territory.id(), map_rect, uv, egui::Color32::WHITE);
        self.folk.draw(&painter, &self.cam, rect);

        if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let cell = self.cam.to_cell(rect, pointer);
            let (x, y) = (cell.x as i64, cell.y as i64);
            let size = i64::from(self.world.genesis.fields.size);
            self.selected = (x >= 0 && y >= 0 && x < size && y < size)
                .then(|| {
                    let idx = (y as usize) * size as usize + x as usize;
                    let p = self.world.genesis.province_of_cell[idx];
                    (p != NO_PROVINCE).then_some(ProvinceId(p))
                })
                .flatten();
        }
        if let Some(p) = self.selected {
            let centre = self.world.genesis.provinces[p.0 as usize].center;
            let at = self
                .cam
                .to_screen(rect, Vec2::new(centre.0 as f32, centre.1 as f32));
            painter.circle_stroke(
                at,
                (self.cam.zoom * 1.6).clamp(8.0, 28.0),
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = ui.input(|i| f64::from(i.stable_dt)).min(0.1);
        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.paused = !self.paused;
        }
        self.advance(dt);
        self.folk.update(&self.world, dt as f32);
        if self.territory_dirty {
            self.territory.set(
                layers::territory_image(&self.world),
                TextureOptions::NEAREST,
            );
            self.territory_dirty = false;
        }

        egui::Panel::top("bar").show(ui, |ui| {
            hud::top_bar(ui, &self.world, &mut self.paused, &mut self.ticks_per_sec);
        });
        egui::Panel::right("feed")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| {
                ui.heading("Chronicle");
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() * 0.62)
                    .show(ui, |ui| {
                        for line in &self.feed {
                            ui.colored_label(line.kind.color(), &line.text);
                        }
                    });
                ui.separator();
                hud::inspector(ui, &self.world, self.selected);
            });
        egui::CentralPanel::default().show(ui, |ui| {
            self.map_canvas(ui, dt as f32);
        });

        ui.ctx().request_repaint();
    }
}
