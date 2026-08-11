//! The opened-tile view: one world tile's local map, camera, and folk
//! (docs/15-multiscale-maps.md — the person-scale layer).

use eframe::egui::{self, Rect, TextureHandle, TextureOptions, Vec2};
use sim_server::World;
use world_schema::TileId;

use crate::camera::Camera;
use crate::layers;
use crate::localfolk::LocalFolk;

pub struct LocalView {
    pub tile: TileId,
    map: local_map::LocalMap,
    texture: TextureHandle,
    folk: LocalFolk,
    cam: Camera,
}

impl LocalView {
    /// Generate and open the person-scale view of `tile`.
    #[must_use]
    pub fn open(
        ctx: &egui::Context,
        world: &World,
        tile: TileId,
        people: usize,
        nation: u32,
    ) -> Self {
        let populated = world.nations.owner[tile.0 as usize].is_some();
        let map = local_map::generate(
            world.seed,
            &world.genesis.fields,
            &world.genesis.flora,
            tile,
            populated,
        );
        let texture = ctx.load_texture("local", layers::local_image(&map), TextureOptions::NEAREST);
        Self {
            tile,
            folk: LocalFolk::new(&map, people, nation),
            cam: Camera::fit(map.size, Vec2::new(1200.0, 800.0)),
            map,
            texture,
        }
    }

    pub fn canvas(&mut self, ui: &mut egui::Ui, dt: f32) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        self.cam.handle(ui, rect, &response, dt);

        let size = self.map.size as f32;
        let map_rect = Rect::from_min_max(
            self.cam.to_screen(rect, Vec2::ZERO),
            self.cam.to_screen(rect, Vec2::splat(size)),
        );
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(self.texture.id(), map_rect, uv, egui::Color32::WHITE);
        self.folk.update(&self.map, dt);
        self.folk.draw(&painter, &self.cam, rect);
    }
}
