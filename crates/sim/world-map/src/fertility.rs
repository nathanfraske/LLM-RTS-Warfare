//! Per-cell fertility derived from the climate fields (docs/13-worldgen.md —
//! fields, not biomes). Flora density enriches it at the province level.

use crate::hydrology::Water;

/// Fertility 0–255: warm-temperate, moist, low-altitude land scores highest.
#[must_use]
pub fn cell_fertility(
    elevation: &[i32],
    water: &[Water],
    temperature: &[i16],
    moisture: &[u8],
) -> Vec<u8> {
    elevation
        .iter()
        .enumerate()
        .map(|(i, &elev)| {
            if elev < 0 || water[i] == Water::Lake {
                return 0;
            }
            let t = bump(f32::from(temperature[i]) / 10.0, 21.0, 26.0);
            let m = (f32::from(moisture[i]) / 255.0).powi(2) * 1.4;
            let a = 1.0 - (elev as f32 / 2_600.0).clamp(0.0, 1.0);
            (255.0 * t * m.min(1.0) * a) as u8
        })
        .collect()
}

/// Quadratic tolerance bump: 1 at `opt`, 0 beyond `width`.
#[must_use]
pub fn bump(value: f32, opt: f32, width: f32) -> f32 {
    let d = (value - opt) / width;
    (1.0 - d * d).max(0.0)
}
