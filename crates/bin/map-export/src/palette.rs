//! Layer colorings over the genesis output (docs/13-worldgen.md — "Seeing it").

use flora::NO_FLORA;
use sim_server::Genesis;
use world_map::{NO_PROVINCE, Water};

pub type Rgb = (u8, u8, u8);

fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t) as u8;
    (ch(a.0, b.0), ch(a.1, b.1), ch(a.2, b.2))
}

/// Deterministic distinct-ish color from an id (provinces, species, nations).
#[must_use]
pub fn id_color(id: u32) -> Rgb {
    let h = (u64::from(id).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as u32;
    (
        96 + (h & 0x7F) as u8,
        96 + ((h >> 7) & 0x7F) as u8,
        96 + ((h >> 14) & 0x7F) as u8,
    )
}

fn water_color(genesis: &Genesis, i: usize) -> Option<Rgb> {
    match genesis.fields.water[i] {
        Water::Ocean => {
            let depth = (-genesis.fields.elevation[i]) as f32 / 3_500.0;
            Some(lerp((46, 111, 174), (8, 38, 84), depth))
        }
        Water::Lake => Some((63, 131, 184)),
        Water::River => Some((47, 111, 176)),
        Water::Dry => None,
    }
}

#[must_use]
pub fn terrain(genesis: &Genesis, i: usize) -> Rgb {
    if let Some(c) = water_color(genesis, i) {
        return c;
    }
    let elev = genesis.fields.elevation[i];
    let temp = genesis.fields.temperature[i];
    let moist = genesis.fields.moisture[i];
    if temp < -60 && elev > 0 {
        return lerp((214, 222, 228), (238, 243, 247), f32::from(moist) / 255.0);
    }
    if elev > 2_400 {
        return lerp(
            (128, 122, 116),
            (196, 196, 200),
            (elev - 2_400) as f32 / 2_100.0,
        );
    }
    let soil = if moist < 70 {
        (198, 173, 122) // arid tan
    } else {
        (168, 156, 118) // temperate soil
    };
    let veg = lerp((116, 152, 80), (34, 96, 44), f32::from(moist) / 255.0);
    let density = f32::from(genesis.flora.density[i]) / 255.0;
    // Climate potential greens fertile-but-unclaimed ground; settled flora deepens it.
    let potential = f32::from(genesis.fields.cell_fertility[i]) / 255.0;
    let ground = lerp(soil, veg, density.max(potential * 0.55));
    // Gentle relief shading by altitude.
    lerp(ground, (255, 255, 255), (elev as f32 / 4_500.0) * 0.25)
}

#[must_use]
pub fn height(genesis: &Genesis, i: usize) -> Rgb {
    let elev = genesis.fields.elevation[i];
    if elev < 0 {
        lerp((30, 60, 110), (5, 15, 40), (-elev) as f32 / 3_500.0)
    } else {
        let t = elev as f32 / 4_500.0;
        lerp((40, 70, 40), (245, 245, 245), t)
    }
}

#[must_use]
pub fn flora_layer(genesis: &Genesis, i: usize) -> Rgb {
    if let Some(c) = water_color(genesis, i) {
        return lerp(c, (0, 0, 0), 0.45);
    }
    let occupant = genesis.flora.occupant[i];
    if occupant == NO_FLORA {
        return (52, 48, 44);
    }
    let base = id_color(u32::from(occupant));
    let density = f32::from(genesis.flora.density[i]) / 255.0;
    lerp((30, 30, 30), base, 0.35 + 0.65 * density)
}

#[must_use]
pub fn provinces_layer(genesis: &Genesis, i: usize) -> Rgb {
    if let Some(c) = water_color(genesis, i) {
        return lerp(c, (0, 0, 0), 0.35);
    }
    let p = genesis.province_of_cell[i];
    if p == NO_PROVINCE {
        return (40, 40, 40);
    }
    let base = id_color(p);
    let province = &genesis.provinces[p as usize];
    if province.habitable {
        base
    } else {
        lerp(base, (20, 20, 20), 0.55)
    }
}
