//! Applying overseer directives: registry validation, mandate pricing,
//! in-world outcomes, everything logged (docs/20-open-directives.md,
//! docs/16-mandate-and-works.md — directives are exactly the replay input).
//!
//! `Set` writes a policy leaf and pins it; `Enact` dispatches to the owning
//! system. The dispatch match below is the single wiring point: a new
//! action registers its def (`registry`) and adds one delegating arm here.

use std::collections::BTreeMap;

use crate::aims::{check_params, check_target};
use crate::{WorldNations, mandate, registry};
use directive_schema::{Directive, DirectiveEntry};
use geology::Geology;
use knowledge::WorldKnowledge;
use policy::{PolicyValue, Registry};
use regolith::Regolith;
use sim_events::{Event, EventLog};
use tuning::Tuning;
use world_map::WorldFields;
use world_schema::{NationId, Quantity, Tick, TileId};

/// Validate, price, and apply one logged directive at its scheduled tick.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    entry: &DirectiveEntry,
    world: &mut WorldNations,
    fields: &WorldFields,
    reg: &Registry,
    known: &mut WorldKnowledge,
    ground: &Regolith,
    rocks: &Geology,
    flora_live: &[u8],
    log: &mut EventLog,
    tun: &Tuning,
) {
    let soc = &tun.society;
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

    // Validate against the registry and the world before charging — a
    // rejected order costs nothing.
    let base = match validate(
        &entry.directive,
        ni,
        world,
        fields,
        reg,
        known,
        &tun.exploration,
        tun.structures.max_per_tile,
    ) {
        Ok(base) => base,
        Err(reason) => {
            reject(log, reason);
            return;
        }
    };
    let cost = mandate::effective_cost(Quantity::from_num(base), world.nations[ni].autonomy, soc);
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
        mandate::spend(&mut m, &mut a, cost, soc);
        nation.mandate = m;
        nation.autonomy = a;
    }

    match &entry.directive {
        Directive::Set { key, value } => {
            world.nations[ni].policy.set_directed(key, value.clone());
            log.push(Event::PolicySet {
                tick,
                nation: nation_id,
                key: key.clone(),
                value: value.clone(),
            });
        }
        Directive::Enact {
            action,
            target,
            params,
        } => enact(
            action, *target, params, ni, world, known, ground, rocks, flora_live, fields, tick,
            log, tun,
        ),
    }
}

/// Registry + in-world legality; returns the base mandate cost on success.
#[allow(clippy::too_many_arguments)]
fn validate(
    directive: &Directive,
    ni: usize,
    world: &WorldNations,
    fields: &WorldFields,
    reg: &Registry,
    known: &WorldKnowledge,
    exp: &tuning::Exploration,
    max_per_tile: u8,
) -> Result<f64, String> {
    match directive {
        Directive::Set { key, value } => {
            let def = reg.policy(key).ok_or(format!(
                "no lever named \"{key}\" — the report's charter lists what can be set"
            ))?;
            def.kind.check(value)?;
            Ok(def.cost)
        }
        Directive::Enact {
            action,
            target,
            params,
        } => {
            let def = reg.action(action).ok_or(format!(
                "no action named \"{action}\" — the report's charter lists what can be enacted"
            ))?;
            check_params(def, params)?;
            let tile = check_target(def.target, *target, ni, world, fields)?;
            if action == registry::COMMISSION {
                let t = tile.expect("owned-tile target checked");
                if world.works.load(t) >= usize::from(max_per_tile) {
                    return Err("the tile already carries all it can".into());
                }
            }
            if action == registry::SETTLE {
                let t = tile.expect("frontier target checked");
                if known.of(world.nations[ni].id).known(t as usize).is_none() {
                    return Err("none among us has walked that land".into());
                }
            }
            if action == registry::SCOUT
                && known.parties_of(world.nations[ni].id) >= usize::from(exp.max_parties)
            {
                return Err("every party we can field is already afield".into());
            }
            Ok(def.cost)
        }
    }
}

/// Dispatch a validated, paid action to its owning system.
#[allow(clippy::too_many_arguments)]
fn enact(
    action: &str,
    target: Option<u32>,
    params: &BTreeMap<String, PolicyValue>,
    ni: usize,
    world: &mut WorldNations,
    known: &mut WorldKnowledge,
    ground: &Regolith,
    rocks: &Geology,
    flora_live: &[u8],
    fields: &WorldFields,
    tick: Tick,
    log: &mut EventLog,
    tun: &Tuning,
) {
    let nation_id = world.nations[ni].id;
    match action {
        registry::NAME => {
            let name = params["name"]
                .as_text()
                .expect("checked as text")
                .trim()
                .to_string();
            world.nations[ni].name.clone_from(&name);
            log.push(Event::NationNamed {
                tick,
                nation: nation_id,
                name,
            });
        }
        registry::SETTLE => {
            let tile = TileId(target.expect("frontier target checked"));
            world.nations[ni].decreed_target = Some(tile);
            log.push(Event::SettlementDecreed {
                tick,
                nation: nation_id,
                tile,
            });
        }
        registry::COMMISSION => {
            let tile = target.expect("owned-tile target checked");
            let emphasis_word = params["emphasis"].as_text().expect("checked as choice");
            let emphasis = structures::EMPHASES
                .iter()
                .position(|e| *e == emphasis_word)
                .expect("checked against options");
            // The building derives from the ground it will stand on.
            let design = structures::design(
                emphasis,
                ground,
                rocks,
                flora_live,
                fields,
                tile as usize,
                &tun.structures,
            );
            let name = design.name.clone();
            world.works.commission(tile, design);
            log.push(Event::WorkCommissioned {
                tick,
                nation: nation_id,
                tile: TileId(tile),
                work: name,
            });
        }
        registry::SCOUT => {
            let bearing = params["bearing"].as_text().expect("checked as choice");
            let b = knowledge::BEARING_NAMES
                .iter()
                .position(|n| *n == bearing)
                .expect("checked against options");
            known.dispatch(nation_id, world.nations[ni].seat, b, tick, log);
        }
        _ => unreachable!("every registered action has a dispatch arm"),
    }
}
