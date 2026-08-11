//! Council reports: the overseer's entire view of the world, fogged to what
//! the nation can know (docs/05-agents-and-mcp.md — the amnesiac-leader test:
//! a cold-started overseer must govern well from this text alone).

pub mod chronicle;

use std::fmt::Write as _;

use chronicle::chronicle;
use cohorts::{CohortKey, Cohorts};
use nations::WorldNations;
use sim_events::EventLog;
use species::Species;
use world_map::Province;
use world_schema::{NationId, Quantity, Tick};

#[must_use]
pub fn year_month(tick: Tick) -> (u64, u64) {
    (tick.0 / 8_640 + 1, (tick.0 % 8_640) / 720 + 1)
}

/// The per-nation council report (markdown). Fog rules: own territory in
/// full; the frontier one province deep; other nations only after contact.
#[must_use]
pub fn nation_report(
    nation_id: NationId,
    world: &WorldNations,
    provinces: &[Province],
    table: &[Species],
    all_cohorts: &Cohorts,
    log: &EventLog,
    now: Tick,
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
        "\nPeople: {} · Stance: {:?} · Seat: province {}",
        s.name, nation.stance, nation.seat.0
    );
    territory_section(&mut out, nation_id, world, provinces, s, all_cohorts);
    frontier_section(&mut out, nation_id, world, provinces, s);
    known_peoples_section(&mut out, nation_id, world, provinces, table);

    let _ = writeln!(out, "\n## Chronicle (our recent history)\n");
    for line in chronicle(nation_id, world, log, 20) {
        let _ = writeln!(out, "- {line}");
    }

    let _ = writeln!(
        out,
        "\n## Council directives\n\nAppend JSON objects to the directive log \
         (applied at their tick; current tick is {}). Examples:\n\n```json\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Name\", \"name\": \"...\" }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"SetStance\", \"stance\": \"Expansive\" }} }}\n\
         {{ \"tick\": {}, \"nation\": {}, \"directive\": {{ \"kind\": \"Settle\", \"province\": <frontier id> }} }}\n```",
        now.0, now.0, nation_id.0, now.0, nation_id.0, now.0, nation_id.0
    );
    out
}

/// Omniscient spectator summary.
#[must_use]
pub fn world_report(
    world: &WorldNations,
    provinces: &[Province],
    table: &[Species],
    all_cohorts: &Cohorts,
    now: Tick,
) -> String {
    let (year, month) = year_month(now);
    let mut out = String::new();
    let _ = writeln!(out, "# World Report — Year {year}, Month {month}\n");
    let _ = writeln!(out, "| Nation | People | Stance | Provinces | Population |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for nation in &world.nations {
        let provinces_held = world.owned_provinces(nation.id).count();
        let pop: Quantity = world
            .owned_provinces(nation.id)
            .map(|p| {
                all_cohorts.population_of(CohortKey {
                    province: p,
                    species: nation.species,
                })
            })
            .fold(Quantity::ZERO, |a, b| a + b);
        let _ = writeln!(
            out,
            "| {} | {} | {:?} | {} | {:.0} |",
            nation.name, table[nation.species.0 as usize].name, nation.stance, provinces_held, pop
        );
    }
    let claimed = world.owner.iter().filter(|o| o.is_some()).count();
    let _ = writeln!(
        out,
        "\nProvinces claimed: {claimed} / {} · Contacts made: {}",
        provinces.len(),
        world.met.len()
    );
    out
}

fn territory_section(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    provinces: &[Province],
    s: &Species,
    all_cohorts: &Cohorts,
) {
    let nation = world
        .nations
        .iter()
        .find(|n| n.id == nation_id)
        .expect("territory of a real nation");
    let _ = writeln!(out, "\n## Territory\n");
    let _ = writeln!(
        out,
        "| Province | Terrain | Population | Capacity | Pressure | Water |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|");
    let mut total = Quantity::ZERO;
    for p in world.owned_provinces(nation_id) {
        let province = &provinces[p.0 as usize];
        let pop = all_cohorts.population_of(CohortKey {
            province: p,
            species: nation.species,
        });
        total += pop;
        let cap = nations::capacity(province, s);
        let pressure = if cap > Quantity::ZERO {
            (pop * Quantity::from_num(100) / cap).to_num::<i64>()
        } else {
            999
        };
        let _ = writeln!(
            out,
            "| {} | {:?} | {pop:.0} | {cap:.0} | {pressure}% | {} |",
            p.0,
            province.terrain,
            water_note(province),
        );
    }
    let _ = writeln!(out, "\nTotal population: {total:.0}");
}

fn frontier_section(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    provinces: &[Province],
    s: &Species,
) {
    let _ = writeln!(out, "\n## Frontier (settleable borders)\n");
    let _ = writeln!(
        out,
        "| Province | Terrain | Est. capacity | Climate fit | Water |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    let mut frontier: Vec<u32> = world
        .owned_provinces(nation_id)
        .flat_map(|p| provinces[p.0 as usize].neighbors.iter().map(|n| n.0))
        .filter(|&n| world.owner[n as usize].is_none())
        .collect();
    frontier.sort_unstable();
    frontier.dedup();
    for n in frontier {
        let province = &provinces[n as usize];
        let fit = species::province_fitness(s, province);
        let _ = writeln!(
            out,
            "| {n} | {:?} | {:.0} | {fit:.2} | {} |",
            province.terrain,
            nations::capacity(province, s),
            water_note(province),
        );
    }
}

fn known_peoples_section(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    provinces: &[Province],
    table: &[Species],
) {
    let _ = writeln!(out, "\n## Known peoples\n");
    let mut any = false;
    for other in &world.nations {
        let pair = (nation_id.0.min(other.id.0), nation_id.0.max(other.id.0));
        if other.id != nation_id && world.met.contains(&pair) {
            let borders: Vec<u32> = world
                .owned_provinces(nation_id)
                .filter(|p| {
                    provinces[p.0 as usize]
                        .neighbors
                        .iter()
                        .any(|nb| world.owner[nb.0 as usize] == Some(other.id))
                })
                .map(|p| p.0)
                .collect();
            let _ = writeln!(
                out,
                "- {} ({} people) — bordering our provinces {borders:?}",
                other.name, table[other.species.0 as usize].name,
            );
            any = true;
        }
    }
    if !any {
        let _ = writeln!(out, "None encountered yet.");
    }
}

fn water_note(province: &Province) -> &'static str {
    match (province.coastal, province.riverine) {
        (true, true) => "coast+river",
        (true, false) => "coast",
        (false, true) => "river",
        (false, false) => "inland",
    }
}
