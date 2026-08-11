//! Applying overseer directives: server-side validation, in-world outcomes,
//! everything logged (docs/04-institutions-directives.md — directives are
//! exactly the replay input).

use crate::WorldNations;
use directive_schema::{Directive, DirectiveEntry};
use sim_events::{Event, EventLog};
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Tick, TileId};

/// Validate and apply one logged directive at its scheduled tick.
pub fn apply(
    entry: &DirectiveEntry,
    world: &mut WorldNations,
    fields: &WorldFields,
    log: &mut EventLog,
) {
    let tick = Tick(entry.tick);
    let nation_id = NationId(entry.nation);
    let reject = |log: &mut EventLog, reason: &str| {
        log.push(Event::DirectiveRejected {
            tick,
            nation: nation_id,
            reason: reason.to_string(),
        });
    };

    let Some(ni) = world.nations.iter().position(|n| n.id == nation_id) else {
        reject(log, "no such nation");
        return;
    };

    match &entry.directive {
        Directive::Name { name } => {
            let trimmed = name.trim();
            if trimmed.is_empty() || trimmed.len() > 64 {
                reject(log, "a name must be 1..=64 characters");
                return;
            }
            world.nations[ni].name = trimmed.to_string();
            log.push(Event::NationNamed {
                tick,
                nation: nation_id,
                name: trimmed.to_string(),
            });
        }
        Directive::SetStance { stance } => {
            world.nations[ni].stance = *stance;
            log.push(Event::StanceChanged {
                tick,
                nation: nation_id,
                stance: *stance,
            });
        }
        Directive::Settle { tile } => {
            let t = *tile as usize;
            if t >= fields.grid().cells() || !tiles::is_land(fields, t) {
                reject(log, "no such land tile");
                return;
            }
            if world.owner[t].is_some() {
                reject(log, "tile is already claimed");
                return;
            }
            if !world.borders_territory(nation_id, fields, t) {
                reject(log, "tile does not border your territory");
                return;
            }
            world.nations[ni].decreed_target = Some(TileId(*tile));
            log.push(Event::SettlementDecreed {
                tick,
                nation: nation_id,
                tile: TileId(*tile),
            });
        }
    }
}
