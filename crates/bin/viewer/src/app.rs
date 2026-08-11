//! The viewer application: drives the sim clock, owns the textures and
//! cameras, and lays out map + feed + inspector across the world and local
//! view modes (docs/15-multiscale-maps.md).

use cohorts::CohortKey;
use eframe::egui::{self, Rect, TextureHandle, TextureOptions, Vec2};

use sim_server::{RunConfig, World};
use world_map::tiles;
use world_schema::TileId;

use crate::camera::Camera;
use crate::feed::Line;
use crate::folk::Folk;
use crate::localview::LocalView;
use crate::{hud, layers};

pub struct App {
    pub(crate) world: World,
    pub(crate) cam: Camera,
    pub(crate) local: Option<LocalView>,
    pub(crate) paused: bool,
    pub(crate) ticks_per_sec: f64,
    pub(crate) tick_debt: f64,
    pub(crate) terrain: TextureHandle,
    pub(crate) territory: TextureHandle,
    pub(crate) territory_dirty: bool,
    /// The month the terrain texture was last painted for (seasonal tint).
    pub(crate) terrain_month: u64,
    /// Relief against the live sun (docs/28), rebuilt as the hour turns.
    pub(crate) shade: TextureHandle,
    pub(crate) shade_hour: u64,
    pub(crate) fog: crate::fogview::FogView,
    pub(crate) waters: crate::waters::Waters,
    /// The layer camera (docs/29): 0 = surface, 1 = under light cover,
    /// 2 = the deep. Textures cut lazily; geology never changes.
    pub(crate) depth_view: u8,
    pub(crate) underground: [Option<TextureHandle>; 2],
    pub(crate) seen_events: usize,
    pub(crate) feed: Vec<Line>,
    pub(crate) folk: Folk,
    pub(crate) selected: Option<TileId>,
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
        let shade = cc.egui_ctx.load_texture(
            "shade",
            crate::sky::shade_image(&world),
            TextureOptions::LINEAR,
        );
        let waters = crate::waters::Waters::new(&world);
        let mut app = Self {
            cam: Camera::fit(world.genesis.fields.size, Vec2::new(1200.0, 800.0)),
            world,
            local: None,
            paused: false,
            ticks_per_sec: 720.0,
            tick_debt: 0.0,
            terrain,
            territory,
            territory_dirty: false,
            terrain_month: 0,
            shade,
            shade_hour: 0,
            fog: crate::fogview::FogView::default(),
            waters,
            depth_view: 0,
            underground: [None, None],
            seen_events: 0,
            feed: Vec::new(),
            folk: Folk::default(),
            selected: None,
        };
        app.drain_events();
        app
    }

    /// Open the person-scale view of a tile.
    fn descend(&mut self, ctx: &egui::Context, tile: TileId) {
        let (people, nation_color) = self.local_population(tile);
        self.local = Some(LocalView::open(
            ctx,
            &self.world,
            tile,
            people,
            nation_color,
        ));
    }

    /// How many wanderers to show on a tile's local map, and whose color.
    fn local_population(&self, tile: TileId) -> (usize, u32) {
        match self.world.nations.owner[tile.0 as usize] {
            Some(owner) => {
                let nation = self
                    .world
                    .nations
                    .nations
                    .iter()
                    .find(|n| n.id == owner)
                    .expect("owner exists");
                let pop = self
                    .world
                    .cohorts
                    .population_of(CohortKey {
                        tile,
                        species: nation.species,
                    })
                    .to_num::<i64>();
                (
                    usize::try_from((pop / 6).clamp(4, 40)).expect("clamped"),
                    owner.0,
                )
            }
            None => (0, 0),
        }
    }

    /// The stacked map layers: the surface world, or the layer camera's
    /// x-ray slice when the depth view is on.
    fn draw_map_layers(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        map_rect: Rect,
        uv: Rect,
        rect: Rect,
    ) {
        if self.depth_view > 0 {
            // The layer camera: an x-ray of the world, nothing living drawn.
            let band = self.depth_view;
            let world = &self.world;
            let handle = self.underground[usize::from(band - 1)].get_or_insert_with(|| {
                ui.ctx().load_texture(
                    format!("underground-{band}"),
                    layers::underground_image(world, band),
                    TextureOptions::NEAREST,
                )
            });
            painter.image(handle.id(), map_rect, uv, egui::Color32::WHITE);
        } else {
            painter.image(self.terrain.id(), map_rect, uv, egui::Color32::WHITE);
            painter.image(self.shade.id(), map_rect, uv, egui::Color32::WHITE);
            painter.image(self.territory.id(), map_rect, uv, egui::Color32::WHITE);
            self.waters.draw(&self.world, &self.cam, painter, rect);
            self.fog.draw(painter, map_rect, uv);
            self.folk.draw(painter, &self.cam, rect);
            crate::fogview::draw_scouts(&self.world, &self.cam, painter, rect);
            crate::sky::draw_night(&self.world, painter, rect, map_rect);
        }
    }

    fn world_canvas(&mut self, ui: &mut egui::Ui, dt: f32) {
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
        self.draw_map_layers(ui, &painter, map_rect, uv, rect);

        let world_size = i64::from(self.world.genesis.fields.size);
        let cam = &self.cam;
        let clicked_tile = |pos: egui::Pos2| -> Option<TileId> {
            let cell = cam.to_cell(rect, pos);
            let (x, y) = (cell.x as i64, cell.y as i64);
            (x >= 0 && y >= 0 && x < world_size && y < world_size)
                .then(|| TileId((y * world_size + x) as u32))
        };
        if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            self.selected = clicked_tile(pointer)
                .filter(|t| tiles::is_land(&self.world.genesis.fields, t.0 as usize));
        }
        if response.double_clicked()
            && let Some(pointer) = response.interact_pointer_pos()
            && let Some(t) = clicked_tile(pointer)
            && tiles::is_land(&self.world.genesis.fields, t.0 as usize)
        {
            self.selected = Some(t);
            self.descend(ui.ctx(), t);
        }
        if let Some(t) = self.selected {
            let (x, y) = self.world.genesis.fields.grid().xy(t.0 as usize);
            let at = self
                .cam
                .to_screen(rect, Vec2::new(x as f32 + 0.5, y as f32 + 0.5));
            painter.circle_stroke(
                at,
                (self.cam.zoom * 0.8).clamp(6.0, 22.0),
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
        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            self.fog.cycle(self.world.nations.nations.len());
        }
        if ui.input(|i| i.key_pressed(egui::Key::G)) {
            self.depth_view = (self.depth_view + 1) % 3;
        }
        if self.local.is_some()
            && ui.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Backspace))
        {
            self.local = None;
        }
        self.advance(dt);
        if !self.paused {
            self.folk.update(dt as f32);
        }
        self.refresh_textures();

        self.fog.refresh(ui.ctx(), &self.world);

        let local_tile = self.local.as_ref().map(|l| l.tile);
        let fog_label = self.fog.label(&self.world);
        let depth_label = match self.depth_view {
            1 => Some("under light cover"),
            2 => Some("the deep"),
            _ => None,
        };
        egui::Panel::top("bar").show(ui, |ui| {
            hud::top_bar(
                ui,
                &self.world,
                &mut self.paused,
                &mut self.ticks_per_sec,
                local_tile,
                fog_label.as_deref(),
                depth_label,
            );
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
            if let Some(local) = self.local.as_mut() {
                let night = crate::sky::local_night(&self.world, local.tile.0);
                let shadow = crate::sky::cast(&self.world);
                local.canvas(ui, dt as f32, self.paused, night, shadow);
            } else {
                self.world_canvas(ui, dt as f32);
            }
        });

        ui.ctx().request_repaint();
    }
}
