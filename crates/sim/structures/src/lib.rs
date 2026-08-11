//! Structures as composition (docs/30-structures.md): a building is a
//! *function* realized in *materials bid from the local ground*. The
//! authored floor is the function vocabulary and the material sources;
//! every actual building — its name, cost, effect, and strength — is
//! derived from the tile it stands on. No building list exists.

use regolith::Regolith;
use tuning::Structures;
use world_map::WorldFields;

/// The authored function vocabulary — effect channels, not building names.
pub const FUNCTIONS: [&str; 3] = ["field-works", "store-house", "hearth-hall"];
pub const FUNCTION_SUMMARIES: [&str; 3] = [
    "Cleared, worked ground that multiplies cultivation.",
    "Held stores against the lean months; capacity from the walls.",
    "Shelter for families; births rise under a sound roof.",
];

pub const FIELD_WORKS: usize = 0;
pub const STORE_HOUSE: usize = 1;
pub const HEARTH_HALL: usize = 2;

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

/// A derived building: function, materials, and everything they imply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Design {
    pub function: usize,
    pub wall: Material,
    pub roof: Material,
    /// Ground quality under the footing, 0..=1000.
    pub footing_milli: u16,
    pub name: String,
    /// Function-specific effect strength, 0..=1000.
    pub effect_milli: u16,
    /// Design integrity: what the built thing can withstand, 0..=1000.
    pub integrity_milli: u16,
    pub months: u8,
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

/// Derive the building this tile would raise for a function. Pure and
/// deterministic: the same ground always designs the same building.
#[must_use]
pub fn design(
    function: usize,
    ground: &Regolith,
    rocks: &geology::Geology,
    flora_live: &[u8],
    fields: &WorldFields,
    tile: usize,
    st: &Structures,
) -> Design {
    let materials = local_materials(ground, rocks, flora_live, tile);
    let fallback = Material {
        word: "earth-walled",
        hardness: 140,
        binding: 380,
        mass: 350,
        supply: 40,
    };
    // Wall: the strongest thing standing about; roof: the lightest thing
    // that still binds.
    let wall = materials
        .iter()
        .max_by_key(|m| u32::from(m.binding) + u32::from(m.hardness) / 2 + u32::from(m.supply) * 2)
        .copied()
        .unwrap_or(fallback);
    let roof = materials
        .iter()
        .filter(|m| m.mass < 300)
        .max_by_key(|m| u32::from(m.binding) + u32::from(m.supply))
        .copied()
        .unwrap_or(Material {
            word: "brush-roofed",
            hardness: 60,
            binding: 220,
            mass: 100,
            supply: 30,
        });
    // The footing meets the actual ground: rock stands, sand leans.
    let footing_milli = u16::from(ground.rock[tile]) * 2
        + u16::from(ground.coarse[tile]) * 2
        + u16::from(ground.fines[tile])
        - u16::from(ground.sand[tile]).min(
            u16::from(ground.rock[tile]) * 2
                + u16::from(ground.coarse[tile]) * 2
                + u16::from(ground.fines[tile]),
        );
    let footing_milli = footing_milli.min(1000);

    let effect_milli = match function {
        FIELD_WORKS => 400 + u16::from(ground.fines[tile]).min(300) + wall.binding / 4,
        STORE_HOUSE => 250 + wall.hardness / 2 + wall.mass / 4,
        _ => 250 + roof.binding + wall.binding / 3,
    }
    .min(1000);

    let integrity_milli =
        (wall.binding * 4 / 10 + wall.hardness * 3 / 10 + roof.binding / 10 + footing_milli / 5)
            .min(1000);

    let months = u8::try_from(
        u32::from(st.base_months)
            + u32::from(wall.mass) / u32::from(st.mass_months_divisor).max(1)
            + u32::from(wall.hardness) / u32::from(st.hardness_months_divisor).max(1),
    )
    .unwrap_or(u8::MAX);

    let name = if function == FIELD_WORKS {
        // Fields are ground, not rooms: named for the earth they work.
        format!("{} field-works", earth_word(ground, tile))
    } else {
        format!("{} {} {}", wall.word, roof.word, FUNCTIONS[function])
    };

    let _ = fields; // slope-aware footings arrive with local-map footprints
    Design {
        function,
        wall,
        roof,
        footing_milli,
        name,
        effect_milli,
        integrity_milli,
        months,
    }
}

fn earth_word(ground: &Regolith, tile: usize) -> &'static str {
    if ground.organic[tile] > 80 {
        "loam-bedded"
    } else if ground.fines[tile] > 90 {
        "silt-bedded"
    } else if ground.sand[tile] > 110 {
        "sand-scratched"
    } else {
        "stone-picked"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::WorldSeed;

    #[test]
    fn different_ground_designs_different_buildings() {
        let st = Structures::default();
        let deep = tuning::Deep::default();
        let g = tuning::Ground::default();
        let fields = world_map::WorldFields::generate(WorldSeed(21), 64);
        let rocks = geology::Geology::genesis(WorldSeed(21), &fields, &deep);
        // The green follows the wet: dry country builds without timber.
        let flora: Vec<u8> = (0..fields.grid().cells())
            .map(|t| fields.moisture[t])
            .collect();
        let ground = regolith::Regolith::genesis(&fields, &flora, &g);

        let land: Vec<usize> = (0..fields.grid().cells())
            .filter(|&t| fields.elevation[t] >= 0)
            .collect();
        let designs: Vec<Design> = land
            .iter()
            .step_by(7)
            .take(400)
            .map(|&t| design(STORE_HOUSE, &ground, &rocks, &flora, &fields, t, &st))
            .collect();
        let names: std::collections::BTreeSet<&str> =
            designs.iter().map(|d| d.name.as_str()).collect();
        assert!(
            names.len() > 1,
            "one world must raise more than one architecture: {names:?}"
        );
        for d in &designs {
            assert!(d.name.contains("store-house"));
            assert!(d.integrity_milli > 0 && d.months > 0);
            let again = design(STORE_HOUSE, &ground, &rocks, &flora, &fields, land[0], &st);
            let _ = again;
        }
        let a = design(STORE_HOUSE, &ground, &rocks, &flora, &fields, land[0], &st);
        let b = design(STORE_HOUSE, &ground, &rocks, &flora, &fields, land[0], &st);
        assert_eq!(a, b, "same ground, same building");
    }
}
