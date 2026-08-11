//! The band autopilot: monthly splitting, frontier settlement, relocation,
//! and — new with docs/22 — the reasons that put people on the road. All of
//! it runs under the nation's own fog: bands judge only tiles they remember
//! (which may be stale), gamble blind into the unknown when desperate, and
//! send scouts when need outruns knowledge. Nobody moves without a reason.
//!
//! Destinations are judged by *remembered* food potential; the `potential`
//! closure samples the world as it stands and is used only where people
//! actually are (their own tile, and observation on arrival).

use crate::relocation::{relocate_starving, remembered_free};
use crate::settlement::split_crowded;
use crate::{WorldNations, registry};
use cohorts::Cohorts;
use knowledge::WorldKnowledge;
use policy::PolicyTree;
use sim_events::{Event, EventLog, WorldSeed};
use species::Species;
use tuning::{Exploration, Society};
use world_map::{WorldFields, tiles};
use world_schema::{NationId, Quantity, Tick, TileId};

/// Posture interpretation: split-population threshold multiplier. Unknown
/// text can't get in (the registry bounds the leaf) but reads as steady.
pub(crate) fn posture_threshold(tree: &PolicyTree, soc: &Society) -> Quantity {
    match tree.text(registry::POSTURE) {
        registry::POSTURE_CONSOLIDATE => Quantity::from_num(soc.stance_consolidate_mult),
        registry::POSTURE_EXPANSIVE => Quantity::from_num(soc.stance_expansive_mult),
        _ => Quantity::from_num(soc.stance_steady_mult),
    }
}

/// One nation-tick per closed month: starving bands move (informed or
/// blind), crowded bands split into known land, and blocked need sends
/// scouts into the dark.
#[allow(clippy::too_many_arguments)]
pub fn tick_month(
    tick: Tick,
    seed: WorldSeed,
    world: &mut WorldNations,
    fields: &WorldFields,
    table: &[Species],
    cohorts: &mut Cohorts,
    known: &mut WorldKnowledge,
    log: &mut EventLog,
    potential: &dyn Fn(usize) -> Quantity,
    starving: &[TileId],
    hungry: &[TileId],
    soc: &Society,
    exp: &Exploration,
) {
    relocate_starving(
        tick, seed, world, fields, cohorts, known, log, potential, starving, soc,
    );
    split_crowded(
        tick, world, fields, table, cohorts, known, log, potential, soc, exp,
    );
    scout_for_relief(tick, world, fields, known, hungry, log, exp);
}

/// Hunger short of catastrophe also sends scouts: a band eating badly with
/// no known way out looks for one before it must gamble.
fn scout_for_relief(
    tick: Tick,
    world: &WorldNations,
    fields: &WorldFields,
    known: &mut WorldKnowledge,
    hungry: &[TileId],
    log: &mut EventLog,
    exp: &Exploration,
) {
    for &t in hungry {
        let Some(nation_id) = world.owner[t.0 as usize] else {
            continue;
        };
        let memory = known.of(nation_id);
        let any_known_exit = tiles::land_neighbors(fields, t.0 as usize)
            .into_iter()
            .any(|n| remembered_free(world, memory, n).is_some());
        if !any_known_exit {
            let seat = world
                .nations
                .iter()
                .find(|n| n.id == nation_id)
                .map(|n| n.seat)
                .expect("owner exists");
            known.need_scout(nation_id, seat, fields, tick, exp, log);
        }
    }
}

/// First contact fires when territories actually touch — fires visible
/// from home. Scout encounters handle meetings at distance (docs/22).
pub(crate) fn contact_check(
    tick: Tick,
    world: &mut WorldNations,
    fields: &WorldFields,
    tile: TileId,
    nation_id: NationId,
    log: &mut EventLog,
) {
    let (neighbors, n) = fields.grid().neighbors8(tile.0 as usize);
    for &nb in &neighbors[..n] {
        if let Some(other) = world.owner[nb]
            && other != nation_id
        {
            let (lo, hi) = if nation_id.0 <= other.0 {
                (nation_id, other)
            } else {
                (other, nation_id)
            };
            if world.met.insert((lo.0, hi.0)) {
                log.push(Event::NationsMet { tick, a: lo, b: hi });
            }
        }
    }
}
