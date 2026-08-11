//! Turning the event log into the human lines of a nation's chronicle —
//! only events this nation witnessed (docs/14-bands-and-councils.md fog rules).

use crate::year_month;
use nations::WorldNations;
use sim_events::{Event, EventLog};
use world_schema::{NationId, Tick};

fn stamp(tick: Tick) -> String {
    let (y, m) = year_month(tick);
    format!("Y{y} M{m}")
}

/// Lines for the events that concern one nation, oldest first, capped to `limit`.
#[must_use]
pub fn chronicle(id: NationId, world: &WorldNations, log: &EventLog, limit: usize) -> Vec<String> {
    let mut lines: Vec<String> = log
        .iter()
        .filter_map(|e| council_line(e, id).or_else(|| world_line(e, id, world)))
        .collect();
    if lines.len() > limit {
        lines.drain(..lines.len() - limit);
    }
    lines
}

/// The council's own acts: directives issued, honored, or refused.
fn council_line(e: &Event, id: NationId) -> Option<String> {
    match e {
        Event::NationNamed { tick, nation, name } if *nation == id => {
            Some(format!("{} — we took the name \"{name}\"", stamp(*tick)))
        }
        Event::StanceChanged {
            tick,
            nation,
            stance,
        } if *nation == id => Some(format!(
            "{} — the council set a {stance:?} posture",
            stamp(*tick)
        )),
        Event::SettlementDecreed { tick, nation, tile } if *nation == id => Some(format!(
            "{} — the council decreed settling tile {}",
            stamp(*tick),
            tile.0
        )),
        Event::WorkCommissioned {
            tick,
            nation,
            tile,
            work,
        } if *nation == id => Some(format!(
            "{} — the council commissioned a {work:?} on tile {}",
            stamp(*tick),
            tile.0
        )),
        Event::DirectiveRejected {
            tick,
            nation,
            reason,
        } if *nation == id => Some(format!("{} — a decree failed: {reason}", stamp(*tick))),
        _ => None,
    }
}

/// What happened to the nation in the world.
fn world_line(e: &Event, id: NationId, world: &WorldNations) -> Option<String> {
    let name_of = |n: NationId| {
        world
            .nations
            .iter()
            .find(|x| x.id == n)
            .map_or_else(|| format!("nation {}", n.0), |x| x.name.clone())
    };
    match e {
        Event::NationSpawned { nation, seat, .. } if *nation == id => {
            Some(format!("Y1 M1 — our people settled tile {}", seat.0))
        }
        Event::TileSettled {
            tick,
            nation,
            from,
            tile,
            settlers,
        } if *nation == id => Some(format!(
            "{} — {settlers:.0} settlers left tile {} and founded tile {}",
            stamp(*tick),
            from.0,
            tile.0
        )),
        Event::WorkCompleted {
            tick,
            nation,
            tile,
            work,
        } if *nation == id => Some(format!(
            "{} — our {work:?} on tile {} stands complete",
            stamp(*tick),
            tile.0
        )),
        Event::NationsMet { tick, a, b } if *a == id || *b == id => {
            let other = if *a == id { *b } else { *a };
            Some(format!("{} — we met {}", stamp(*tick), name_of(other)))
        }
        Event::Famine {
            tick,
            tile,
            species,
        } if world.owner[tile.0 as usize] == Some(id)
            && world
                .nations
                .iter()
                .any(|n| n.id == id && n.species == *species) =>
        {
            Some(format!("{} — hunger in tile {}", stamp(*tick), tile.0))
        }
        _ => None,
    }
}
