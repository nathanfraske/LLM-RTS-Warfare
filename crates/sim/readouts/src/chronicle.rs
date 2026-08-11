//! Turning the event log into the human lines of a nation's chronicle —
//! only events this nation witnessed (docs/14-bands-and-councils.md fog rules).

use crate::year_month;
use nations::WorldNations;
use sim_events::{Event, EventLog};
use world_schema::NationId;

/// Lines for the events that concern one nation, oldest first, capped to `limit`.
#[must_use]
pub fn chronicle(id: NationId, world: &WorldNations, log: &EventLog, limit: usize) -> Vec<String> {
    let name_of = |n: NationId| {
        world
            .nations
            .iter()
            .find(|x| x.id == n)
            .map_or_else(|| format!("nation {}", n.0), |x| x.name.clone())
    };
    let mut lines: Vec<String> = log
        .iter()
        .filter_map(|e| match e {
            Event::NationSpawned { nation, seat, .. } if *nation == id => {
                Some(format!("Y1 M1 — our people settled province {}", seat.0))
            }
            Event::NationNamed { tick, nation, name } if *nation == id => {
                let (y, m) = year_month(*tick);
                Some(format!("Y{y} M{m} — we took the name \"{name}\""))
            }
            Event::StanceChanged {
                tick,
                nation,
                stance,
            } if *nation == id => {
                let (y, m) = year_month(*tick);
                Some(format!("Y{y} M{m} — the council set a {stance:?} posture"))
            }
            Event::SettlementDecreed {
                tick,
                nation,
                province,
            } if *nation == id => {
                let (y, m) = year_month(*tick);
                Some(format!(
                    "Y{y} M{m} — the council decreed settling province {}",
                    province.0
                ))
            }
            Event::DirectiveRejected {
                tick,
                nation,
                reason,
            } if *nation == id => {
                let (y, m) = year_month(*tick);
                Some(format!("Y{y} M{m} — a decree failed: {reason}"))
            }
            Event::ProvinceSettled {
                tick,
                nation,
                from,
                province,
                settlers,
            } if *nation == id => {
                let (y, m) = year_month(*tick);
                Some(format!(
                    "Y{y} M{m} — {settlers:.0} settlers left province {} and founded province {}",
                    from.0, province.0
                ))
            }
            Event::NationsMet { tick, a, b } if *a == id || *b == id => {
                let (y, m) = year_month(*tick);
                let other = if *a == id { *b } else { *a };
                Some(format!("Y{y} M{m} — we met {}", name_of(other)))
            }
            Event::Famine {
                tick,
                province,
                species,
            } if world.owner[province.0 as usize] == Some(id)
                && world
                    .nations
                    .iter()
                    .any(|n| n.id == id && n.species == *species) =>
            {
                let (y, m) = year_month(*tick);
                Some(format!("Y{y} M{m} — hunger in province {}", province.0))
            }
            _ => None,
        })
        .collect();
    if lines.len() > limit {
        lines.drain(..lines.len() - limit);
    }
    lines
}
