//! The opened-tile view: one world tile's local map, camera, and folk
//! (docs/15-multiscale-maps.md — the person-scale layer).

use eframe::egui::{self, Rect, TextureHandle, TextureOptions, Vec2};
use sim_server::World;
use world_schema::TileId;

use crate::camera::Camera;
use crate::critters::WildSpec;
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
            world.nations.works.completed(tile.0),
            &world.flora_live,
        );
        let texture = ctx.load_texture("local", layers::local_image(&map), TextureOptions::NEAREST);
        let labor = world.nations.owner[tile.0 as usize]
            .and_then(|o| world.nations.nations.iter().find(|n| n.id == o))
            .map_or(world.tuning.society.spawn_labor, |n| n.labor_milli);
        // Instance visible wildlife from the tile's actual fauna: count from
        // biomass, color from trait space (greener = plant diet, redder =
        // flesh, bluer = water-going).
        let wildlife: Vec<WildSpec> = world
            .fauna
            .species
            .iter()
            .filter_map(|s| {
                let pop = world.fauna.at(s.id as usize, tile.0 as usize);
                let land = pop * s.land_frac();
                let count = (land.to_num::<i64>() / 22).clamp(0, 10) as usize;
                (count > 0).then(|| WildSpec {
                    count,
                    color: (
                        (90 + u32::from(s.diet_milli) * 140 / 1000) as u8,
                        (80 + (1000 - u32::from(s.diet_milli)) * 120 / 1000) as u8,
                        (70 + u32::from(s.water_milli) * 150 / 1000) as u8,
                    ),
                })
            })
            .collect();
        Self {
            tile,
            folk: LocalFolk::new(&map, labor, people, nation, &wildlife),
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
        let felled = self.folk.update(&self.map, dt);
        if !felled.is_empty() {
            for c in felled {
                self.map.tree[c] = false;
            }
            self.texture
                .set(layers::local_image(&self.map), TextureOptions::NEAREST);
        }
        self.folk.draw(&painter, &self.cam, rect);
    }
}
