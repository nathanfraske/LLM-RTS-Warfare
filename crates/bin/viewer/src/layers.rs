//! Map textures: static terrain, and the nation-territory overlay that
//! refreshes when ownership changes.

use eframe::egui::ColorImage;
use map_export::palette;
use sim_server::World;

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

/// Territory tint: owned cells carry their nation's color at low alpha.
#[must_use]
pub fn territory_image(world: &World) -> ColorImage {
    let size = world.genesis.fields.size as usize;
    let mut rgba = vec![0u8; size * size * 4];
    for (cell, &province) in world.genesis.province_of_cell.iter().enumerate() {
        if province == world_map::NO_PROVINCE {
            continue;
        }
        if let Some(owner) = world.nations.owner[province as usize] {
            let (r, g, b) = palette::id_color(owner.0);
            let at = cell * 4;
            rgba[at] = r;
            rgba[at + 1] = g;
            rgba[at + 2] = b;
            rgba[at + 3] = 88;
        }
    }
    ColorImage::from_rgba_unmultiplied([size, size], &rgba)
}
