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
        let (mut r, mut g, mut b) = palette::terrain(&world.genesis, i);
        if world.genesis.fields.elevation[i] >= 0 {
            // What the ground is made of decides its color (docs/27);
            // living vegetation greens it back over.
            let (gr, gg, gb) = ground_color(world, i);
            let veg = u32::from(world.flora_live[i]).min(255) * 60 / 100;
            r = blend(gr, 52, veg);
            g = blend(gg, 108, veg);
            b = blend(gb, 58, veg);
            // The turning year on the ground (docs/26): snow whitens the
            // heights, and low growth browns the green.
            let snow = u32::from(world.climate.snowpack[i]).min(300);
            if snow > 0 {
                let k = snow * 200 / 300;
                r = blend(r, 240, k);
                g = blend(g, 244, k);
                b = blend(b, 250, k);
            } else {
                let growth = u32::from(world.climate.growth[i]);
                if growth < 500 {
                    let k = (500 - growth) * 90 / 500;
                    r = blend(r, 168, k);
                    g = blend(g, 142, k);
                    b = blend(b, 96, k);
                }
            }
        }
        rgb.extend_from_slice(&[r, g, b]);
    }
    ColorImage::from_rgb([size, size], &rgb)
}

/// One slice of the underground (docs/29, the layer camera): bedrock in
/// its mineral's color, veins glinting where their depth is within the
/// slice, caves as hollows, faults as cracks, vents as fire.
#[must_use]
pub fn underground_image(world: &World, band: u8) -> ColorImage {
    let size = world.genesis.fields.size as usize;
    let geo = &world.genesis.geology;
    let vein_depth_cap = if band == 1 { 160 } else { 255 };
    let mut rgb = Vec::with_capacity(size * size * 3);
    for i in 0..size * size {
        let (mut red, mut green, mut blue) = if world.genesis.fields.elevation[i] < 0 {
            (12, 16, 26)
        } else {
            mineral_color(&geo.minerals[geo.bedrock[i] as usize])
        };
        if world.genesis.fields.elevation[i] >= 0 {
            // Deeper slices run darker: the light is farther away.
            if band == 2 {
                red = red * 3 / 5;
                green = green * 3 / 5;
                blue = blue * 3 / 5;
            }
            if geo.caves[i] > 0 && band == 1 {
                let k = u32::from(geo.caves[i]).min(200);
                red = blend(red, 8, k);
                green = blend(green, 8, k);
                blue = blend(blue, 12, k);
            }
            if geo.faults[i] {
                red = blend(red, 70, 110);
                green = blend(green, 40, 110);
                blue = blend(blue, 34, 110);
            }
            if let Some(vein) = geo.veins[i]
                && vein.depth <= vein_depth_cap
            {
                let m = &geo.minerals[vein.mineral as usize];
                let (vr, vg, vb) = vein_glint(m);
                let k = u32::from(vein.richness).min(230);
                red = blend(red, vr, k);
                green = blend(green, vg, k);
                blue = blend(blue, vb, k);
            }
            if geo.vents[i] > 0 {
                red = 235;
                green = 96;
                blue = 30;
            }
        }
        rgb.extend_from_slice(&[red, green, blue]);
    }
    ColorImage::from_rgb([size, size], &rgb)
}

/// A mineral's stone color, derived from its axes: hardness sets the gray,
/// solubility pales it, energy blackens it toward the burning strata.
fn mineral_color(m: &geology::Mineral) -> (u8, u8, u8) {
    let gray = 84 + (u32::from(m.hardness_milli) * 70 / 1000) as u8;
    let mut red = gray;
    let mut green = gray;
    let mut blue = gray + 6;
    if m.solubility_milli > 400 {
        let pale = u32::from(m.solubility_milli - 400) / 3;
        red = blend(red, 224, pale);
        green = blend(green, 214, pale);
        blue = blend(blue, 182, pale);
    }
    if m.energy_milli > 400 {
        let coal = u32::from(m.energy_milli - 400) / 2;
        red = blend(red, 30, coal);
        green = blend(green, 26, coal);
        blue = blend(blue, 24, coal);
    }
    (red, green, blue)
}

/// The glint a vein shows: warmer and brighter the more metal it carries.
fn vein_glint(m: &geology::Mineral) -> (u8, u8, u8) {
    let metal = u32::from(m.metal_milli);
    (
        u8::try_from((150 + metal * 100 / 1000).min(255)).expect("bounded"),
        u8::try_from((120 + metal * 90 / 1000).min(255)).expect("bounded"),
        u8::try_from((60 + metal * 40 / 1000).min(255)).expect("bounded"),
    )
}

/// The regolith's own color: a weighted blend of what it is made of.
fn ground_color(world: &World, i: usize) -> (u8, u8, u8) {
    let reg = &world.regolith;
    let parts = [
        (u32::from(reg.rock[i]), (126u32, 124u32, 128u32)),
        (u32::from(reg.coarse[i]), (152, 140, 120)),
        (u32::from(reg.sand[i]), (216, 194, 140)),
        (u32::from(reg.fines[i]), (150, 118, 92)),
        (u32::from(reg.organic[i]), (92, 78, 56)),
    ];
    let total: u32 = parts.iter().map(|(w, _)| w).sum::<u32>().max(1);
    let mix = |pick: fn(&(u32, u32, u32)) -> u32| {
        u8::try_from(parts.iter().map(|(w, c)| w * pick(c)).sum::<u32>() / total)
            .expect("weighted average of u8 channels")
    };
    (mix(|c| c.0), mix(|c| c.1), mix(|c| c.2))
}

/// Move `from` toward `to` by `k`/255.
fn blend(from: u8, to: u8, k: u32) -> u8 {
    let f = u32::from(from);
    let t = u32::from(to);
    u8::try_from((f * (255 - k) + t * k) / 255).expect("bounded")
}

/// One nation's fog: what it has never seen is near-black, stale memories
/// dim with age, fresh ground is clear (docs/22-knowledge-and-discovery.md).
#[must_use]
pub fn fog_image(world: &World, nation: world_schema::NationId) -> ColorImage {
    let size = world.genesis.fields.size as usize;
    let memory = world.knowledge.of(nation);
    let now = world.tick();
    let mut rgba = vec![0u8; size * size * 4];
    for tile in 0..size * size {
        let alpha = match memory.age_months(tile, now) {
            None => 235,
            Some(age) => u8::try_from((age * 6).min(120)).expect("capped"),
        };
        rgba[tile * 4 + 3] = alpha;
    }
    ColorImage::from_rgba_unmultiplied([size, size], &rgba)
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
    if map.works.iter().any(|w| w.contains("field-works")) {
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
    if map.works.iter().any(|w| w.contains("store-house")) {
        for dy in -12i64..=-8 {
            for dx in -12i64..=-8 {
                paint(cx + dx, cy + dy, (96, 70, 44));
            }
        }
    }
    if map.works.iter().any(|w| w.contains("hearth-hall")) {
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
