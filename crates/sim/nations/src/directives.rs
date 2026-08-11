//! Applying overseer directives: server-side validation, mandate pricing,
//! in-world outcomes, everything logged (docs/04-institutions-directives.md,
//! docs/16-mandate-and-works.md — directives are exactly the replay input).

use crate::{WorldNations, mandate};
use directive_schema::{Directive, DirectiveEntry};
use sim_events::{Event, EventLog};
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Tick, TileId};

/// Validate, price, and apply one logged directive at its scheduled tick.
pub fn apply(
    entry: &DirectiveEntry,
    world: &mut WorldNations,
    fields: &WorldFields,
    log: &mut EventLog,
) {
    let tick = Tick(entry.tick);
    let nation_id = NationId(entry.nation);
    let reject = |log: &mut EventLog, reason: String| {
        log.push(Event::DirectiveRejected {
            tick,
            nation: nation_id,
            reason,
        });
    };

    let Some(ni) = world.nations.iter().position(|n| n.id == nation_id) else {
        reject(log, "no such nation".into());
        return;
    };

    // Validate before charging — a rejected order costs nothing.
    if let Err(reason) = validate(&entry.directive, ni, world, fields) {
        reject(log, reason);
        return;
    }
    let cost = mandate::effective_cost(&entry.directive, world.nations[ni].autonomy);
    if world.nations[ni].mandate < cost {
        reject(
            log,
            format!(
                "the council lacks the mandate: need {cost:.1}, have {:.1}",
                world.nations[ni].mandate
            ),
        );
        return;
    }
    {
        let nation = &mut world.nations[ni];
        let (mut m, mut a) = (nation.mandate, nation.autonomy);
        mandate::spend(&mut m, &mut a, cost);
        nation.mandate = m;
        nation.autonomy = a;
    }

    match &entry.directive {
        Directive::Name { name } => {
            let trimmed = name.trim().to_string();
            world.nations[ni].name.clone_from(&trimmed);
            log.push(Event::NationNamed {
                tick,
                nation: nation_id,
                name: trimmed,
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
            world.nations[ni].decreed_target = Some(TileId(*tile));
            log.push(Event::SettlementDecreed {
                tick,
                nation: nation_id,
                tile: TileId(*tile),
            });
        }
        Directive::Commission { tile, work } => {
            world.works.commission(*tile, *work);
            log.push(Event::WorkCommissioned {
                tick,
                nation: nation_id,
                tile: TileId(*tile),
                work: *work,
            });
        }
    }
}

/// In-world legality, checked before any mandate is charged.
fn validate(
    directive: &Directive,
    ni: usize,
    world: &WorldNations,
    fields: &WorldFields,
) -> Result<(), String> {
    let nation_id = world.nations[ni].id;
    match directive {
        Directive::Name { name } => {
            let trimmed = name.trim();
            if trimmed.is_empty() || trimmed.len() > 64 {
                return Err("a name must be 1..=64 characters".into());
            }
        }
        Directive::SetStance { .. } => {}
        Directive::Settle { tile } => {
            let t = *tile as usize;
            if t >= fields.grid().cells() || !tiles::is_land(fields, t) {
                return Err("no such land tile".into());
            }
            if world.owner[t].is_some() {
                return Err("tile is already claimed".into());
            }
            if !world.borders_territory(nation_id, fields, t) {
                return Err("tile does not border your territory".into());
            }
        }
        Directive::Commission { tile, work } => {
            let t = *tile as usize;
            if t >= fields.grid().cells() {
                return Err("no such tile".into());
            }
            if world.owner[t] != Some(nation_id) {
                return Err("you can only commission works on your own tiles".into());
            }
            if world.works.has_or_building(*tile, *work) {
                return Err(format!("{work:?} already stands or is being built there"));
            }
        }
    }
    Ok(())
}
