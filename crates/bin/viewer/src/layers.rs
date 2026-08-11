//! Map textures: world terrain, the nation-territory overlay, and the
//! person-scale local map render.

use directive_schema::WorkKind;
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
    if let Some((cx, cy)) = map.camp {
        draw_camp(&mut rgb, map, cx, cy);
    }
    ColorImage::from_rgb([size, size], &rgb)
}

/// The camp and its completed works: huts, tilled fields, granary.
fn draw_camp(rgb: &mut [u8], map: &LocalMap, cx: u32, cy: u32) {
    const HUTS: [(i64, i64); 5] = [(0, 0), (4, 1), (-4, 2), (1, -4), (-2, 4)];
    const EXTRA: [(i64, i64); 4] = [(8, 8), (-8, 8), (8, -8), (-9, -2)];
    let size = i64::from(map.size);
    let mut paint = |x: i64, y: i64, color: (u8, u8, u8)| {
        if x >= 0 && y >= 0 && x < size && y < size {
            let at = ((y as usize) * map.size as usize + x as usize) * 3;
            rgb[at] = color.0;
            rgb[at + 1] = color.1;
            rgb[at + 2] = color.2;
        }
    };
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    for (hx, hy) in HUTS {
        for dy in 0..2i64 {
            for dx in 0..2i64 {
                paint(cx + hx + dx, cy + hy + dy, (122, 86, 54));
            }
        }
    }
    if map.works.contains(&WorkKind::Farmstead) {
        for dy in -10i64..=10 {
            for dx in 9i64..=30 {
                let tilled = dy.rem_euclid(2) == 0;
                let color = if tilled {
                    (128, 96, 58)
                } else {
                    (150, 128, 80)
                };
                paint(cx + dx, cy + dy, color);
            }
        }
    }
    if map.works.contains(&WorkKind::Granary) {
        for dy in -12i64..=-8 {
            for dx in -12i64..=-8 {
                paint(cx + dx, cy + dy, (96, 70, 44));
            }
        }
    }
    if map.works.contains(&WorkKind::Dwellings) {
        for (hx, hy) in EXTRA {
            for dy in 0..2i64 {
                for dx in 0..2i64 {
                    paint(cx + hx + dx, cy + hy + dy, (122, 86, 54));
                }
            }
        }
    }
}

fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    (ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2))
}
