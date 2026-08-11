//! Map textures: world terrain, the nation-territory overlay, and the
//! person-scale local map render.

use eframe::egui::ColorImage;
use local_map::LocalMap;
use map_export::palette;
use sim_server::World;
use world_map::Water;

#[must_use]
pub fn terrain_image(world: &World) -> ColorImage {
    let size = world.genesis.fields.size as usize;
    let mut rgb = Vec::with_capacity(size * size * 3);
    for i in 0..size * size {
        let (r, g, b) = palette::terrain(&world.genesis, i);
        rgb.extend_from_slice(&[r, g, b]);
    }
    ColorImage::from_rgb([size, size], &rgb)
}

/// Territory tint: owned tiles carry their nation's color at low alpha.
#[must_use]
pub fn territory_image(world: &World) -> ColorImage {
    let size = world.genesis.fields.size as usize;
    let mut rgba = vec![0u8; size * size * 4];
    for (tile, owner) in world.nations.owner.iter().enumerate() {
        if let Some(owner) = owner {
            let (r, g, b) = palette::id_color(owner.0);
            let at = tile * 4;
            rgba[at] = r;
            rgba[at + 1] = g;
            rgba[at + 2] = b;
            rgba[at + 3] = 96;
        }
    }
    ColorImage::from_rgba_unmultiplied([size, size], &rgba)
}

/// Person-scale render of one local map: water, ground, trees, camp huts.
#[must_use]
pub fn local_image(map: &LocalMap) -> ColorImage {
    let size = map.size as usize;
    let mut rgb = Vec::with_capacity(size * size * 3);
    for i in 0..size * size {
        let color = match map.water[i] {
            Water::Ocean => {
                let depth = (-map.elevation[i]).max(0) as f32 / 2_000.0;
                lerp((52, 118, 178), (10, 42, 92), depth)
            }
            Water::Lake => (68, 136, 188),
            Water::River => (52, 116, 180),
            Water::Dry => {
                let veg = f32::from(map.veg[i]) / 255.0;
                let soil = (172, 158, 122);
                let grass = (98, 138, 74);
                if map.tree[i] {
                    (34, 82, 44)
                } else {
                    lerp(soil, grass, veg)
                }
            }
        };
        rgb.extend_from_slice(&[color.0, color.1, color.2]);
    }
    // Camp huts: a small ring of brown blocks around the camp center.
    if let Some((cx, cy)) = map.camp {
        const HUTS: [(i64, i64); 5] = [(0, 0), (4, 1), (-4, 2), (1, -4), (-2, 4)];
        let bound = i64::from(map.size);
        for (hx, hy) in HUTS {
            for dy in 0..2i64 {
                for dx in 0..2i64 {
                    let x = i64::from(cx) + hx + dx;
                    let y = i64::from(cy) + hy + dy;
                    if x >= 0 && y >= 0 && x < bound && y < bound {
                        let at = ((y as usize) * size + x as usize) * 3;
                        rgb[at] = 122;
                        rgb[at + 1] = 86;
                        rgb[at + 2] = 54;
                    }
                }
            }
        }
    }
    ColorImage::from_rgb([size, size], &rgb)
}

fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    (ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2))
}
