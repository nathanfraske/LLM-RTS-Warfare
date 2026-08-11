//! Materials as bid from the tile (docs/30): earth from the fines, stone
//! from the scree and the bedrock's own hardness, timber from the standing
//! green, thatch and turf from the light growth. Sources are the authored
//! floor; what any tile actually offers is read, never written.

use regolith::Regolith;

/// A material as bid from the tile: what it is called and what it can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Material {
    pub word: &'static str,
    pub hardness: u16,
    pub binding: u16,
    pub mass: u16,
    /// How much of it the tile actually offers, 0..=255.
    pub supply: u8,
}

/// The materials this tile offers, in deterministic source order:
/// earth, stone, timber, thatch.
#[must_use]
pub fn local_materials(
    ground: &Regolith,
    rocks: &geology::Geology,
    flora_live: &[u8],
    tile: usize,
) -> Vec<Material> {
    let bedrock = &rocks.minerals[rocks.bedrock[tile] as usize];
    let mut out = Vec::new();
    let fines = ground.fines[tile];
    if fines > 30 {
        out.push(Material {
            word: if fines > 100 {
                "clay-walled"
            } else {
                "mud-walled"
            },
            hardness: 180 + u16::from(fines),
            binding: 640,
            mass: 420,
            supply: fines,
        });
    }
    let stone = ground.coarse[tile].saturating_add(ground.rock[tile] / 2);
    if stone > 50 {
        out.push(Material {
            word: "stone-walled",
            hardness: 400 + bedrock.hardness_milli / 3,
            binding: 300,
            mass: 900,
            supply: stone,
        });
    }
    if flora_live[tile] > 110 {
        out.push(Material {
            word: "timber-walled",
            hardness: 340,
            binding: 560,
            mass: 320,
            supply: flora_live[tile],
        });
    }
    if flora_live[tile] > 40 || ground.organic[tile] > 40 {
        out.push(Material {
            word: if ground.organic[tile] > 70 {
                "turf-roofed"
            } else {
                "thatch-roofed"
            },
            hardness: 80,
            binding: 320,
            mass: 110,
            supply: flora_live[tile].max(ground.organic[tile]),
        });
    }
    out
}
