//! The underground (docs/29-the-underground.md): a generated mineral
//! palette, a generated geologic history, and the per-tile columns they
//! compile to — bedrock, veins, faults, caves, vents. No ore list exists:
//! minerals are points on property axes, and deposits fall out of events.
//! `minerals` generates the palette; `history` runs the events; `fire`
//! keeps the volcano schedules.

pub mod fire;
mod history;
mod minerals;

pub use minerals::Mineral;

use std::fmt::Write as _;

use sim_events::{SystemId, WorldSeed};
use tuning::Deep;
use world_map::WorldFields;
use world_schema::Tick;

pub(crate) const GEOGEN: SystemId = SystemId(13);

/// A notable deposit in a tile's column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vein {
    pub mineral: u16,
    /// 0 = breaking the surface … 255 = deep under cover.
    pub depth: u8,
    pub richness: u8,
}

/// The world's underground, compiled per tile.
#[derive(Debug)]
pub struct Geology {
    pub minerals: Vec<Mineral>,
    /// Country-rock species per tile.
    pub bedrock: Vec<u16>,
    pub veins: Vec<Option<Vein>>,
    pub faults: Vec<bool>,
    /// Cave size 0 = none.
    pub caves: Vec<u8>,
    /// Vent strength 0 = none.
    pub vents: Vec<u8>,
    /// Eruption schedule per vent tile: (tile, period months, phase).
    pub schedules: Vec<(u32, u16, u16)>,
    /// Quake schedule per epicenter: (fault tile, period months, phase).
    pub quake_clocks: Vec<(u32, u16, u16)>,
}

impl Geology {
    /// Generate the underground from the world as formed: the peaks are
    /// the record of the uplifts, the lowlands of the basins.
    #[must_use]
    pub fn genesis(seed: WorldSeed, fields: &WorldFields, deep: &Deep) -> Self {
        let minerals = minerals::generate(seed, deep.minerals);
        history::compile(seed, fields, &minerals, deep)
    }

    /// Genesis warmth the vents lend the surface, deci-degrees per tile —
    /// added into the temperature field before life is seeded.
    #[must_use]
    pub fn geothermal(&self, fields: &WorldFields, deep: &Deep) -> Vec<i16> {
        let grid = fields.grid();
        let mut warmth = vec![0i16; fields.grid().cells()];
        for (tile, &strength) in self.vents.iter().enumerate() {
            if strength == 0 {
                continue;
            }
            let (vx, vy) = grid.xy(tile);
            let r = i64::from(deep.geothermal_radius);
            for dy in -r..=r {
                for dx in -r..=r {
                    let x = i64::from(vx) + dx;
                    let y = i64::from(vy) + dy;
                    if x < 0 || y < 0 || x >= i64::from(fields.size) || y >= i64::from(fields.size)
                    {
                        continue;
                    }
                    let dist = dx.abs().max(dy.abs());
                    let fade = (r + 1 - dist).max(0);
                    let add = i64::from(deep.geothermal_deci) * fade * i64::from(strength)
                        / ((r + 1) * 255);
                    let at = (y as usize) * fields.size as usize + x as usize;
                    warmth[at] = warmth[at].saturating_add(i16::try_from(add).unwrap_or(i16::MAX));
                }
            }
        }
        warmth
    }

    /// One legible line for what lies beneath a tile (docs/21 duty).
    #[must_use]
    pub fn describe(&self, tile: usize) -> String {
        let bed = &self.minerals[self.bedrock[tile] as usize];
        let mut line = format!("bedrock of {}", bed.describe());
        if let Some(vein) = self.veins[tile] {
            let m = &self.minerals[vein.mineral as usize];
            let depth = match vein.depth {
                0..=60 => "breaking the surface",
                61..=160 => "under light cover",
                _ => "deep under cover",
            };
            let _ = write!(line, "; a vein of {} {depth}", m.describe());
        }
        if self.caves[tile] > 0 {
            line.push_str("; hollow with caves");
        }
        if self.vents[tile] > 0 {
            line.push_str("; the ground here runs hot");
        }
        if self.faults[tile] {
            line.push_str("; faulted");
        }
        line
    }
}

/// Deterministic draw helper shared by the geology passes.
pub(crate) fn draw(seed: WorldSeed, salt: u64) -> u64 {
    sim_events::rng::draw(seed, Tick::ZERO, GEOGEN, salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_world_gets_its_own_underground() {
        let deep = Deep::default();
        let fields = WorldFields::generate(WorldSeed(11), 64);
        let a = Geology::genesis(WorldSeed(11), &fields, &deep);
        let b = Geology::genesis(WorldSeed(11), &fields, &deep);
        assert_eq!(a.bedrock, b.bedrock, "same seed, same ground");
        assert_eq!(a.schedules, b.schedules);

        let other = Geology::genesis(
            WorldSeed(12),
            &WorldFields::generate(WorldSeed(12), 64),
            &deep,
        );
        assert_ne!(
            a.minerals
                .iter()
                .map(|m| (m.hardness_milli, m.metal_milli))
                .collect::<Vec<_>>(),
            other
                .minerals
                .iter()
                .map(|m| (m.hardness_milli, m.metal_milli))
                .collect::<Vec<_>>(),
            "each world rolls its own periodic table"
        );

        let land: Vec<usize> = (0..fields.grid().cells())
            .filter(|&t| fields.elevation[t] >= 0)
            .collect();
        assert!(
            land.iter().any(|&t| a.veins[t].is_some()),
            "the events must leave veins somewhere"
        );
        assert!(a.schedules.len() as u8 <= deep.plumes && !a.schedules.is_empty());
        for &t in land.iter().take(200) {
            assert!(!a.describe(t).is_empty());
        }
    }
}
