//! Procedural flora: per-world plant genomes generated across climate space
//! (docs/13-worldgen.md — the diversity engine; settling lives in `settle`).

pub mod settle;

pub use settle::{FloraMap, NO_FLORA};

use serde::{Deserialize, Serialize};
use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use world_schema::{FloraId, Tick};

const FLORAGEN: SystemId = SystemId(5);

pub const DEFAULT_SPECIES: u16 = 24;

/// A plant genome: tolerance curves over the climate fields plus competition
/// traits. No authored growth-form taxonomy: `woodiness` is a spectrum from
/// grass (0) to old forest (1000); "shrub" is a region of it, not a type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloraSpecies {
    pub id: FloraId,
    /// 0 = grass … 1000 = towering forest.
    pub woodiness_milli: u16,
    /// Temperature optimum / tolerance width, deci-°C.
    pub t_opt: i16,
    pub t_width: i16,
    /// Moisture optimum / tolerance width, 0–255.
    pub m_opt: u8,
    pub m_width: u8,
    /// Hard altitude ceiling, meters.
    pub alt_max: i32,
    /// Competitive vigor ×1000.
    pub vigor_milli: u16,
    /// Seed for the viewer's procedural glyph/palette (docs/10-visualization.md).
    pub glyph_seed: u32,
}

/// Generate `count` species stratified across temperature × moisture space so
/// every climate band gets contenders — deserts and tundra included.
#[must_use]
pub fn generate_species(seed: WorldSeed, count: u16) -> Vec<FloraSpecies> {
    const T_BINS: [i16; 4] = [-80, 60, 170, 270]; // deci-°C bin centers
    const M_BINS: [u8; 3] = [55, 130, 210];
    (0..count)
        .map(|k| {
            let d = |salt: u64| rng::draw(seed, Tick::ZERO, FLORAGEN, u64::from(k) << 8 | salt);
            let t_center = T_BINS[(k as usize / M_BINS.len()) % T_BINS.len()];
            let m_center = M_BINS[k as usize % M_BINS.len()];
            let t_opt = t_center + (d(1) % 90) as i16 - 45;
            let m_opt = (i32::from(m_center) + (d(2) % 70) as i32 - 35).clamp(10, 250) as u8;
            // Woodiness needs water: dry-adapted genomes cap out as scrub.
            let woodiness_milli = if m_opt >= 95 {
                (d(3) % 1000) as u16
            } else {
                (d(3) % 450) as u16
            };
            FloraSpecies {
                id: FloraId(k),
                woodiness_milli,
                t_opt,
                t_width: 70 + (d(4) % 90) as i16,
                m_opt,
                m_width: (55 + (d(5) % 80)) as u8,
                alt_max: 900 + (d(6) % 2_900) as i32,
                vigor_milli: (850 + (d(7) % 350)) as u16,
                glyph_seed: d(8) as u32,
            }
        })
        .collect()
}

impl FloraSpecies {
    /// Habitat fitness in `[0, ~1.2]`: tolerance bumps × vigor, gated by altitude.
    #[must_use]
    pub fn fitness(&self, elevation: i32, temperature_dc: i16, moisture: u8) -> f32 {
        if elevation < 0 || elevation > self.alt_max {
            return 0.0;
        }
        let t = world_map::fertility::bump(
            f32::from(temperature_dc),
            f32::from(self.t_opt),
            f32::from(self.t_width),
        );
        let m = world_map::fertility::bump(
            f32::from(moisture),
            f32::from(self.m_opt),
            f32::from(self.m_width),
        );
        t * m * (f32::from(self.vigor_milli) / 1000.0)
    }
}

/// Monthly regrowth of living vegetation toward its settled baseline
/// (docs/19-ecology-and-subsistence.md — flora density is dynamic now).
pub fn regrow_month(live: &mut [u8], baseline: &[u8], divisor: u8, growth: &[u16]) {
    let divisor = divisor.max(1);
    for (tile, (l, &b)) in live.iter_mut().zip(baseline).enumerate() {
        if *l < b {
            let gap = b - *l;
            let step = u16::from((gap / divisor).max(1));
            // The seasonal gate (docs/26): winter halts the green.
            *l += u8::try_from(u32::from(step) * u32::from(growth[tile]) / 1000)
                .expect("bounded by gap");
        }
    }
}
