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
            &world.nations.works.names(tile.0),
            &world.flora_live,
        );
        let texture = ctx.load_texture("local", layers::local_image(&map), TextureOptions::NEAREST);
        let labor = world.nations.owner[tile.0 as usize]
            .and_then(|o| world.nations.nations.iter().find(|n| n.id == o))
            .map_or(world.tuning.society.spawn_labor, |n| {
                economy::labor_milli(&n.policy)
            });
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

    /// Every standing tree in view throws a shadow away from the light,
    /// long at dawn and dusk, pooled at noon (docs/28 §3).
    fn tree_shadows(&self, painter: &egui::Painter, rect: Rect, dir: Vec2, len: f32, alpha: u8) {
        if self.cam.zoom < 3.0 {
            return;
        }
        let size = self.map.size as usize;
        let a = self.cam.to_cell(rect, rect.min);
        let b = self.cam.to_cell(rect, rect.max);
        let (x0, x1) = (
            (a.x.floor().max(0.0)) as usize,
            (b.x.ceil() as usize).min(size),
        );
        let (y0, y1) = (
            (a.y.floor().max(0.0)) as usize,
            (b.y.ceil() as usize).min(size),
        );
        let color = egui::Color32::from_rgba_unmultiplied(12, 14, 10, alpha);
        for y in y0..y1 {
            for x in x0..x1 {
                if !self.map.tree[y * size + x] {
                    continue;
                }
                let foot = Vec2::new(x as f32 + 0.5, y as f32 + 0.8);
                let tip = foot + dir * len;
                painter.line_segment(
                    [
                        self.cam.to_screen(rect, foot),
                        self.cam.to_screen(rect, tip),
                    ],
                    egui::Stroke::new((self.cam.zoom * 0.35).clamp(1.0, 5.0), color),
                );
            }
        }
    }

    pub fn canvas(
        &mut self,
        ui: &mut egui::Ui,
        dt: f32,
        paused: bool,
        night: Option<egui::Color32>,
        shadow: Option<(Vec2, f32, u8)>,
    ) {
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
        // Pause stops people too — the presence layer renders sim time,
        // not wall time.
        let felled = if paused {
            Vec::new()
        } else {
            self.folk.update(&self.map, dt)
        };
        if !felled.is_empty() {
            for c in felled {
                self.map.tree[c] = false;
            }
            self.texture
                .set(layers::local_image(&self.map), TextureOptions::NEAREST);
        }
        if let Some((dir, len, alpha)) = shadow {
            self.tree_shadows(&painter, rect, dir, len, alpha);
        }
        self.folk.draw(&painter, &self.cam, rect);
        if let Some(tint) = night {
            painter.rect_filled(rect, 0.0, tint);
        }
    }
}
