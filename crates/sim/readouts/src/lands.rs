//! The report's remembered world: known lands with the age of each memory,
//! and the bearings still dark (docs/22-knowledge-and-discovery.md).

use std::fmt::Write as _;

use knowledge::WorldKnowledge;
use nations::WorldNations;
use tuning::Tuning;
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, Tick};

use crate::FRONTIER_ROWS;
use crate::sections::water_note;

/// The world as this nation remembers it: every tile someone has walked
/// and reported free, with the age of the memory (docs/22). Unwalked land
/// is absent — and the report says which bearings hold it.
pub(crate) fn known_lands(
    out: &mut String,
    nation_id: NationId,
    world: &WorldNations,
    fields: &WorldFields,
    known: &WorldKnowledge,
    now: Tick,
    tun: &Tuning,
) {
    let _ = writeln!(out, "\n## Known lands (unclaimed, as remembered)\n");
    let memory = known.of(nation_id);
    let cells = fields.grid().cells();
    let mut rows: Vec<(u32, Quantity, u64, bool)> = (0..cells)
        .filter_map(|t| {
            let m = memory.known(t)?;
            if m.owner.is_some() || world.owner[t].is_some() {
                return None;
            }
            if !tiles::is_land(fields, t) {
                return None;
            }
            let age = memory.age_months(t, now).unwrap_or(0);
            let borders = world.borders_territory(nation_id, fields, t);
            Some((t as u32, m.potential, age, borders))
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let shown = rows.len().min(FRONTIER_ROWS);
    let _ = writeln!(
        out,
        "| Tile | Terrain | Food/worker (remembered) | Seen | Borders us | Water |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|");
    for &(t, potential, age, borders) in &rows[..shown] {
        let _ = writeln!(
            out,
            "| {t} | {:?} | {potential:.2} | {} | {} | {} |",
            tiles::label(fields, t as usize),
            if age == 0 {
                "fresh".to_string()
            } else {
                format!("{age} months ago")
            },
            if borders { "yes" } else { "no" },
            water_note(fields, t as usize),
        );
    }
    if rows.len() > shown {
        let _ = writeln!(
            out,
            "\n({} further remembered tiles omitted)",
            rows.len() - shown
        );
    }
    let _ = writeln!(
        out,
        "\nMemories age: the numbers above are what the land looked like when \
         someone last walked it. Settlement decrees need a walked, bordering tile."
    );
    let seat = world
        .nations
        .iter()
        .find(|n| n.id == nation_id)
        .expect("known lands of a real nation")
        .seat;
    let dark: Vec<&str> = (0..8)
        .filter(|&b| bearing_has_unknown(memory, fields, seat, b, tun.exploration.scout_range))
        .map(|b| knowledge::BEARING_NAMES[b])
        .collect();
    if dark.is_empty() {
        let _ = writeln!(out, "\nNo unwalked land within scouting range of the seat.");
    } else {
        let _ = writeln!(
            out,
            "\nBeyond our maps (from the seat): {} — scouts can be sent (band.scout).",
            dark.join(" \u{b7} ")
        );
    }
}

fn bearing_has_unknown(
    memory: &knowledge::NationKnowledge,
    fields: &WorldFields,
    seat: world_schema::TileId,
    bearing: usize,
    range: u16,
) -> bool {
    let (dx, dy) = knowledge::BEARING_DELTAS[bearing];
    let (sx, sy) = fields.grid().xy(seat.0 as usize);
    (1..=i64::from(range)).any(|step| {
        let x = i64::from(sx) + dx * step;
        let y = i64::from(sy) + dy * step;
        if x < 0 || y < 0 || x >= i64::from(fields.size) || y >= i64::from(fields.size) {
            return false;
        }
        memory
            .known((y as usize) * fields.size as usize + x as usize)
            .is_none()
    })
}
