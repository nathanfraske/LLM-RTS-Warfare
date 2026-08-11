//! Applying overseer directives: registry validation, mandate pricing,
//! in-world outcomes, everything logged (docs/20-open-directives.md,
//! docs/16-mandate-and-works.md — directives are exactly the replay input).
//!
//! `Set` writes a policy leaf and pins it; `Enact` dispatches to the owning
//! system. The dispatch match below is the single wiring point: a new
//! action registers its def (`registry`) and adds one delegating arm here.

use std::collections::BTreeMap;

use crate::{WorldNations, mandate, registry};
use directive_schema::{Directive, DirectiveEntry};
use knowledge::WorldKnowledge;
use policy::{ActionDef, PolicyValue, Registry, TargetKind};
use sim_events::{Event, EventLog};
use tuning::{Exploration, Society};
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, Tick, TileId};

/// Validate, price, and apply one logged directive at its scheduled tick.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    entry: &DirectiveEntry,
    world: &mut WorldNations,
    fields: &WorldFields,
    reg: &Registry,
    known: &mut WorldKnowledge,
    log: &mut EventLog,
    soc: &Society,
    exp: &Exploration,
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

    // Validate against the registry and the world before charging — a
    // rejected order costs nothing.
    let base = match validate(&entry.directive, ni, world, fields, reg, known, exp) {
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
        } => enact(action, *target, params, ni, world, known, tick, log, soc),
    }
}

/// Registry + in-world legality; returns the base mandate cost on success.
fn validate(
    directive: &Directive,
    ni: usize,
    world: &WorldNations,
    fields: &WorldFields,
    reg: &Registry,
    known: &WorldKnowledge,
    exp: &Exploration,
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
                let work = params["work"].as_text().expect("checked as choice");
                let t = tile.expect("owned-tile target checked");
                if world.works.has_or_building(t, work) {
                    return Err(format!("a {work} already stands or is being built there"));
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

/// Every declared param present and in bounds; nothing undeclared.
fn check_params(def: &ActionDef, params: &BTreeMap<String, PolicyValue>) -> Result<(), String> {
    for p in &def.params {
        let value = params
            .get(&p.name)
            .ok_or(format!("missing param \"{}\"", p.name))?;
        p.kind
            .check(value)
            .map_err(|e| format!("param \"{}\": {e}", p.name))?;
    }
    if let Some(unknown) = params
        .keys()
        .find(|k| !def.params.iter().any(|p| &p.name == *k))
    {
        return Err(format!("unknown param \"{unknown}\""));
    }
    Ok(())
}

/// The action's declared target kind, checked against the world.
fn check_target(
    kind: TargetKind,
    target: Option<u32>,
    ni: usize,
    world: &WorldNations,
    fields: &WorldFields,
) -> Result<Option<u32>, String> {
    let nation_id = world.nations[ni].id;
    match kind {
        TargetKind::Nation => {
            if target.is_some() {
                return Err("this action takes no target tile".into());
            }
            Ok(None)
        }
        TargetKind::OwnedTile => {
            let t = target.ok_or("this action needs a target tile")?;
            if (t as usize) >= fields.grid().cells() {
                return Err("no such tile".into());
            }
            if world.owner[t as usize] != Some(nation_id) {
                return Err("the target must be one of your own tiles".into());
            }
            Ok(Some(t))
        }
        TargetKind::FrontierTile => {
            let t = target.ok_or("this action needs a target tile")?;
            if (t as usize) >= fields.grid().cells() || !tiles::is_land(fields, t as usize) {
                return Err("no such land tile".into());
            }
            if world.owner[t as usize].is_some() {
                return Err("tile is already claimed".into());
            }
            if !world.borders_territory(nation_id, fields, t as usize) {
                return Err("tile does not border your territory".into());
            }
            Ok(Some(t))
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
    tick: Tick,
    log: &mut EventLog,
    soc: &Society,
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
            let work = params["work"].as_text().expect("checked as choice");
            world.works.commission(tile, work, soc);
            log.push(Event::WorkCommissioned {
                tick,
                nation: nation_id,
                tile: TileId(tile),
                work: work.to_string(),
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
