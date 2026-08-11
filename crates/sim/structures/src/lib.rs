//! Structures as composition (docs/30-structures.md): a building is an
//! **allocation of effort over physical aspect space**, realized in
//! materials bid from the local ground. Nothing in the sim dispatches on
//! a building's name or kind: effects read the aspect numbers, and the
//! nouns ("long-store", "field-works") are describe-words over the space,
//! exactly as "omnivore" and "loam" are. The authored floor is the aspect
//! vocabulary — the physical ways a built thing couples to the world's
//! existing state — plus the material sources. Everything else derives.

mod materials;

pub use materials::{Material, local_materials};

use regolith::Regolith;
use tuning::Structures;
use world_map::WorldFields;

/// The physical couplings a built thing can have (docs/30 §1). Closed
/// over current sim state; new couplings (walls against foes, height for
/// seeing) are registrations when their state exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Aspects {
    /// Kept off the sky: roof coverage and soundness.
    pub cover_milli: u16,
    /// Held apart from the world: wall enclosure.
    pub enclosure_milli: u16,
    /// Room to hold things: floor area under sound walls.
    pub capacity_milli: u16,
    /// Ground cleared, drained, and made ready for working.
    pub worked_ground_milli: u16,
    /// Heat held against the cold.
    pub hearth_milli: u16,
}

/// The commissioning vocabulary: effort emphases — pointers into aspect
/// space, not building types. What each one *builds* depends entirely on
/// the ground it is built from.
pub const EMPHASES: [&str; 5] = [
    "roomy",
    "sheltering",
    "ground-working",
    "hearth-warm",
    "balanced",
];
pub const EMPHASIS_SUMMARIES: [&str; 5] = [
    "Effort into floor and walls: room to hold stores against the year.",
    "Effort into roof and walls: cover for people and goods.",
    "Effort into the ground itself: cleared, worked land for cultivation.",
    "Effort into hearth and enclosure: warmth held against the cold.",
    "Effort spread even: a little of everything.",
];

/// Effort weights per emphasis: (area, walls, roof, groundwork, hearth),
/// summing to 100.
const ALLOCATIONS: [[u16; 5]; 5] = [
    [40, 25, 20, 5, 10],
    [15, 30, 35, 5, 15],
    [30, 5, 5, 55, 5],
    [15, 25, 25, 5, 30],
    [20, 20, 20, 20, 20],
];

/// A derived building: materials, aspects, and everything they imply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Design {
    pub wall: Material,
    pub roof: Material,
    /// Ground quality under the footing, 0..=1000.
    pub footing_milli: u16,
    pub aspects: Aspects,
    /// Floor-area effort, milli — the footprint follows from it.
    pub area_milli: u16,
    pub name: String,
    /// Design integrity: what the built thing can withstand, 0..=1000.
    pub integrity_milli: u16,
    pub months: u8,
}

impl Design {
    /// Ground the building takes at person scale, in local-map cells:
    /// bigger effort into floor, bigger footprint.
    #[must_use]
    pub fn footprint(&self) -> (u8, u8) {
        let w = 4 + self.area_milli / 80;
        let d = 3 + self.area_milli / 130;
        (
            u8::try_from(w.min(16)).expect("clamped"),
            u8::try_from(d.min(12)).expect("clamped"),
        )
    }

    /// Coarse material class for rendering: 1 earth, 2 stone, 3 timber.
    #[must_use]
    pub fn wall_class(&self) -> u8 {
        if self.wall.word.contains("stone") {
            2
        } else if self.wall.word.contains("timber") {
            3
        } else {
            1
        }
    }

    /// Ground-working designs are plots, not rooms.
    #[must_use]
    pub fn is_groundwork(&self) -> bool {
        let a = &self.aspects;
        a.worked_ground_milli
            >= a.capacity_milli
                .max(a.cover_milli)
                .max(a.hearth_milli)
                .max(a.enclosure_milli)
    }
}

/// Derive the building this tile would raise under an emphasis. Pure and
/// deterministic: the same ground and intent always design the same thing.
#[must_use]
pub fn design(
    emphasis: usize,
    ground: &Regolith,
    rocks: &geology::Geology,
    flora_live: &[u8],
    fields: &WorldFields,
    tile: usize,
    st: &Structures,
) -> Design {
    let alloc = ALLOCATIONS[emphasis % EMPHASES.len()];
    let (area, walls, roof_effort, groundwork, hearth_effort) =
        (alloc[0], alloc[1], alloc[2], alloc[3], alloc[4]);
    let materials = local_materials(ground, rocks, flora_live, tile);
    let fallback = Material {
        word: "earth-walled",
        hardness: 140,
        binding: 380,
        mass: 350,
        supply: 40,
    };
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
    let solid = u16::from(ground.rock[tile]) * 2
        + u16::from(ground.coarse[tile]) * 2
        + u16::from(ground.fines[tile]);
    let footing_milli = solid.saturating_sub(u16::from(ground.sand[tile])).min(1000);

    // Every aspect is effort × what the materials make of it.
    let enclosure = (walls * (wall.binding + wall.hardness / 2) / 60).min(1000);
    let aspects = Aspects {
        cover_milli: (roof_effort * roof.binding / 12).min(1000),
        enclosure_milli: enclosure,
        capacity_milli: (area * (500 + enclosure / 2) / 45).min(1000),
        worked_ground_milli: (groundwork * (700 + u16::from(ground.fines[tile]).min(300)) / 60)
            .min(1000),
        hearth_milli: (hearth_effort * (400 + enclosure / 2 + wall.mass / 4) / 35).min(1000),
    };

    let integrity_milli =
        (wall.binding * 4 / 10 + wall.hardness * 3 / 10 + roof.binding / 10 + footing_milli / 5)
            .saturating_sub(area / 2)
            .min(1000);

    let months = u8::try_from(
        u32::from(st.base_months)
            + u32::from(wall.mass) * u32::from(walls + area)
                / (u32::from(st.mass_months_divisor).max(1) * 40)
            + u32::from(wall.hardness) / u32::from(st.hardness_months_divisor).max(1),
    )
    .unwrap_or(u8::MAX)
    .max(1);

    let name = name_of(&aspects, wall, roof, ground, tile);
    let _ = fields; // slope-aware footings arrive with local-map footprints
    Design {
        wall,
        roof,
        footing_milli,
        aspects,
        area_milli: area * 10,
        name,
        integrity_milli,
        months,
    }
}

/// The dominant aspect names the thing — describe-words over aspect
/// space, never types the sim dispatches on.
fn name_of(a: &Aspects, wall: Material, roof: Material, ground: &Regolith, tile: usize) -> String {
    let ranked = [
        (a.capacity_milli, "long-store"),
        (a.cover_milli, "roof-hall"),
        (a.worked_ground_milli, "field-works"),
        (a.hearth_milli, "hearth-house"),
        (a.enclosure_milli, "stead"),
    ];
    let (_, noun) = ranked
        .iter()
        .max_by_key(|(v, _)| *v)
        .copied()
        .expect("five aspects");
    if noun == "field-works" {
        let earth = if ground.organic[tile] > 80 {
            "loam-bedded"
        } else if ground.fines[tile] > 90 {
            "silt-bedded"
        } else if ground.sand[tile] > 110 {
            "sand-scratched"
        } else {
            "stone-picked"
        };
        format!("{earth} field-works")
    } else {
        format!("{} {} {noun}", wall.word, roof.word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::WorldSeed;

    #[test]
    fn ground_and_intent_design_the_building_never_a_type() {
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

        // Same intent, different ground: different architecture.
        let names: std::collections::BTreeSet<String> = land
            .iter()
            .step_by(7)
            .take(400)
            .map(|&t| design(0, &ground, &rocks, &flora, &fields, t, &st).name)
            .collect();
        assert!(
            names.len() > 1,
            "one world must raise more than one architecture: {names:?}"
        );

        // Same ground, different intent: different aspects dominate.
        let t = land[0];
        let roomy = design(0, &ground, &rocks, &flora, &fields, t, &st);
        let sheltering = design(1, &ground, &rocks, &flora, &fields, t, &st);
        let worked = design(2, &ground, &rocks, &flora, &fields, t, &st);
        assert!(roomy.aspects.capacity_milli > sheltering.aspects.capacity_milli);
        assert!(sheltering.aspects.cover_milli > roomy.aspects.cover_milli);
        assert!(worked.aspects.worked_ground_milli > roomy.aspects.worked_ground_milli);
        assert!(worked.name.contains("field-works"));

        // Size is real: floor effort sets the footprint.
        let (rw, rd) = roomy.footprint();
        let (hw, hd) = design(3, &ground, &rocks, &flora, &fields, t, &st).footprint();
        assert!(
            u16::from(rw) * u16::from(rd) > u16::from(hw) * u16::from(hd),
            "a roomy raising takes more ground than a hearth-warm one"
        );

        // Determinism.
        let again = design(0, &ground, &rocks, &flora, &fields, t, &st);
        assert_eq!(roomy, again, "same ground, same intent, same building");
        for d in [&roomy, &sheltering, &worked] {
            assert!(d.integrity_milli > 0 && d.months > 0);
        }
    }
}
