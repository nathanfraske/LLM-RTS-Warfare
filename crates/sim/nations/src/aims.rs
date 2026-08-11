//! The legality of aims and arguments (docs/20): every action's declared
//! params checked against their schemas, every target checked against the
//! world — before any mandate is charged.

use std::collections::BTreeMap;

use crate::WorldNations;
use policy::{ActionDef, PolicyValue, TargetKind};
use world_map::{WorldFields, tiles};

/// Every declared param present and in bounds; nothing undeclared.
pub(crate) fn check_params(
    def: &ActionDef,
    params: &BTreeMap<String, PolicyValue>,
) -> Result<(), String> {
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
pub(crate) fn check_target(
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
