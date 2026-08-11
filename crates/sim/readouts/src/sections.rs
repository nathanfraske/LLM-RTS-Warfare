//! The report's body sections: territory, labor, works, frontier, peoples.
//! Pure rendering over sim state; fog rules live with each section.

use std::fmt::Write as _;

use climate::Climate;
use cohorts::{CohortKey, Cohorts};
use economy::Economy;
use economy::channels::{CHANNEL_NAMES, CHANNELS, LABOR_KEYS};
use fauna::Fauna;
use nations::WorldNations;
use species::Species;
use tuning::Tuning;
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity};

pub(crate) fn works(out: &mut String, nation_id: NationId, world: &WorldNations) {
    let _ = writeln!(out, "\n## Works\n");
    let mut any = false;
    for t in world.owned_tiles(nation_id) {
        for work in world.works.completed(t.0) {
            let _ = writeln!(out, "- tile {}: {work} (complete)", t.0);
            any = true;
        }
        for state in world.works.in_progress(t.0) {
            let _ = writeln!(
                out,
                "- tile {}: {} (building, {} months left)",
                t.0, state.work, state.months_left
            );
            any = true;
        }
    }
    if !any {
        let _ = writeln!(out, "None commissioned yet.");
    }
}

pub(crate) fn territory(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    fields: &WorldFields,
    econ: &Economy,
    all_cohorts: &Cohorts,
) {
    let nation = world
        .nations
        .iter()
        .find(|n| n.id == nation_id)
        .expect("territory of a real nation");
    let _ = writeln!(
        out,
        "
## Territory
"
    );
    let _ = writeln!(
        out,
        "| Tile | Terrain | Population | Fed | Stores | Fields | Herd | Water |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|");
    let mut total = Quantity::ZERO;
    for t in world.owned_tiles(nation_id) {
        let pop = all_cohorts.population_of(CohortKey {
            tile: t,
            species: nation.species,
        });
        total += pop;
        let te = econ.tile(t.0).cloned().unwrap_or_default();
        let _ = writeln!(
            out,
            "| {} | {:?} | {pop:.0} | {:.0}% | {:.0} | {:.0}% | {:.0} | {} |",
            t.0,
            tiles::label(fields, t.0 as usize),
            te.last_nutrition * Quantity::from_num(100),
            te.stock,
            te.establishment * Quantity::from_num(100),
            te.herd,
            water_note(fields, t.0 as usize),
        );
    }
    let _ = writeln!(
        out,
        "
Total population: {total:.0}"
    );
}

/// The decision surface for feeding a people: current allocation and what
/// one worker earns in each channel, tile by tile. Numbers, not vocabulary.
#[allow(clippy::too_many_arguments)]
pub(crate) fn labor(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    sky: &Climate,
    econ: &Economy,
    tun: &Tuning,
) {
    let nation = world
        .nations
        .iter()
        .find(|n| n.id == nation_id)
        .expect("labor of a real nation");
    let _ = writeln!(
        out,
        "
## Labor and returns
"
    );
    let labor = economy::labor_milli(&nation.policy);
    let total: u32 = labor.iter().map(|&w| u32::from(w)).sum();
    let mut parts: Vec<String> = Vec::new();
    let mut any_pinned = false;
    for i in 0..CHANNELS {
        let pinned = nation.policy.directed(LABOR_KEYS[i]);
        any_pinned |= pinned;
        parts.push(format!(
            "{} {}%{}",
            CHANNEL_NAMES[i],
            u32::from(labor[i]) * 100 / total.max(1),
            if pinned { "*" } else { "" }
        ));
    }
    let _ = writeln!(out, "Allocation: {}", parts.join(" · "));
    let _ = writeln!(
        out,
        "{}",
        if any_pinned {
            "(* pinned by council decree; unmarked weights follow returns)"
        } else {
            "(all weights follow returns)"
        }
    );
    let _ = writeln!(
        out,
        "
Food per worker per month, by tile:
"
    );
    for t in world.owned_tiles(nation_id) {
        let te = econ.tile(t.0).cloned().unwrap_or_default();
        let m = economy::channels::marginal(
            fields,
            wild,
            flora_live,
            &te,
            sky,
            t.0 as usize,
            &tun.subsistence,
            &tun.weather,
        );
        let mut parts: Vec<String> = Vec::new();
        for (i, name) in CHANNEL_NAMES.iter().enumerate() {
            parts.push(format!("{name} {:.2}", m[i]));
        }
        let _ = writeln!(out, "- tile {}: {}", t.0, parts.join(" · "));
    }
}

pub(crate) fn known_peoples(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    fields: &WorldFields,
    table: &[Species],
) {
    let _ = writeln!(out, "\n## Known peoples\n");
    let mut any = false;
    for other in &world.nations {
        let pair = (nation_id.0.min(other.id.0), nation_id.0.max(other.id.0));
        if other.id != nation_id && world.met.contains(&pair) {
            let borders: Vec<u32> = world
                .owned_tiles(nation_id)
                .filter(|t| {
                    let (neighbors, n) = fields.grid().neighbors8(t.0 as usize);
                    neighbors[..n]
                        .iter()
                        .any(|&nb| world.owner[nb] == Some(other.id))
                })
                .map(|t| t.0)
                .collect();
            let _ = writeln!(
                out,
                "- {} ({} people) — bordering our tiles {borders:?}",
                other.name, table[other.species.0 as usize].name,
            );
            any = true;
        }
    }
    if !any {
        let _ = writeln!(out, "None encountered yet.");
    }
}

pub(crate) fn water_note(fields: &WorldFields, tile: usize) -> &'static str {
    match (tiles::coastal(fields, tile), tiles::riverine(fields, tile)) {
        (true, true) => "coast+river",
        (true, false) => "coast",
        (false, true) => "river",
        (false, false) => "inland",
    }
}
