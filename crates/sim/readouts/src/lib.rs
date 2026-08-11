//! Council reports: the overseer's entire view of the world, fogged to what
//! the nation can know (docs/05-agents-and-mcp.md — the amnesiac-leader test:
//! a cold-started overseer must govern well from this text alone).

pub mod chronicle;
mod sections;

use std::fmt::Write as _;

use chronicle::chronicle;
use cohorts::{CohortKey, Cohorts};
use economy::Economy;
use fauna::Fauna;
use nations::WorldNations;
use sim_events::EventLog;
use species::Species;
use tuning::Tuning;
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, Tick};

const FRONTIER_ROWS: usize = 24;

#[must_use]
pub fn year_month(tick: Tick) -> (u64, u64) {
    (tick.0 / 8_640 + 1, (tick.0 % 8_640) / 720 + 1)
}

/// The per-nation council report (markdown). Fog rules: own territory in
/// full; the frontier one tile deep; other nations only after contact.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn nation_report(
    nation_id: NationId,
    world: &WorldNations,
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    econ: &Economy,
    table: &[Species],
    all_cohorts: &Cohorts,
    log: &EventLog,
    now: Tick,
    tun: &Tuning,
) -> String {
    let nation = world
        .nations
        .iter()
        .find(|n| n.id == nation_id)
        .expect("report for a real nation");
    let s = &table[nation.species.0 as usize];
    let (year, month) = year_month(now);
    let mut out = String::new();

    let _ = writeln!(
        out,
        "# {} — Council Report, Year {year}, Month {month}",
        nation.name
    );
    let _ = writeln!(
        out,
        "\nPeople: {} · Stance: {:?} · Seat: tile {}",
        s.name, nation.stance, nation.seat.0
    );
    let _ = writeln!(
        out,
        "Mandate: {:.1}/10 · Autonomy: {:.0}% (directive costs ×{:.2})",
        nation.mandate,
        nation.autonomy,
        Quantity::ONE + nation.autonomy / Quantity::from_num(tun.society.autonomy_cost_divisor)
    );
    sections::territory(&mut out, nation_id, world, fields, econ, all_cohorts);
    sections::labor(
        &mut out, nation_id, world, fields, wild, flora_live, econ, tun,
    );
    sections::works(&mut out, nation_id, world);
    sections::frontier(&mut out, nation_id, world, fields, wild, flora_live, tun);
    sections::known_peoples(&mut out, nation_id, world, fields, table);

    let _ = writeln!(out, "\n## Chronicle (our recent history)\n");
    for line in chronicle(nation_id, world, log, 20) {
        let _ = writeln!(out, "- {line}");
    }

    let _ = writeln!(
        out,
        "\n## Council directives\n\nAppend JSON objects to the directive log \
         (applied at their tick; current tick is {}). Paid directives spend mandate \
         (docs/16): Name free · SetStance 1 · SetLabor 1 · Settle 2 · Commission 3, \
         all scaled by autonomy. Works: Farmstead, Granary, Dwellings. SetLabor \
         weights are parts-per-thousand across gather/hunt/fish/cultivate/herd and \
         override the return-following autopilot. Examples:\n\n```json\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Name\", \"name\": \"...\" }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"SetStance\", \"stance\": \"Expansive\" }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Settle\", \"tile\": <frontier id> }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Commission\", \"tile\": <owned id>, \"work\": \"Farmstead\" }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"SetLabor\", \"gather\": 300, \"hunt\": 250, \"fish\": 250, \"cultivate\": 150, \"herd\": 50 }} }}\n```",
        now.0,
        now.0,
        nation_id.0,
        now.0,
        nation_id.0,
        now.0,
        nation_id.0,
        now.0,
        nation_id.0,
        now.0,
        nation_id.0
    );
    out
}

/// Omniscient spectator summary.
#[must_use]
pub fn world_report(
    world: &WorldNations,
    fields: &WorldFields,
    table: &[Species],
    all_cohorts: &Cohorts,
    now: Tick,
) -> String {
    let (year, month) = year_month(now);
    let mut out = String::new();
    let _ = writeln!(out, "# World Report — Year {year}, Month {month}\n");
    let _ = writeln!(out, "| Nation | People | Stance | Tiles | Population |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for nation in &world.nations {
        let tiles_held = world.owned_tiles(nation.id).count();
        let pop: Quantity = world
            .owned_tiles(nation.id)
            .map(|t| {
                all_cohorts.population_of(CohortKey {
                    tile: t,
                    species: nation.species,
                })
            })
            .fold(Quantity::ZERO, |a, b| a + b);
        let _ = writeln!(
            out,
            "| {} | {} | {:?} | {} | {:.0} |",
            nation.name, table[nation.species.0 as usize].name, nation.stance, tiles_held, pop
        );
    }
    let claimed = world.owner.iter().filter(|o| o.is_some()).count();
    let _ = writeln!(
        out,
        "\nTiles claimed: {claimed} / {} land · Contacts made: {}",
        (0..fields.grid().cells())
            .filter(|&t| tiles::is_land(fields, t))
            .count(),
        world.met.len()
    );
    out
}
