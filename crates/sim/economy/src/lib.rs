//! E0 of the economy program: food from the living world through five
//! subsistence channels, ledgered exactly, with nutrition as the output that
//! drives demographics (docs/19-ecology-and-subsistence.md, docs/18-economy.md).
//!
//! No channel is a "stage of history": allocation across channels is policy
//! (directed) or marginal-return-following (autopilot), and the landscape
//! decides what pays. `channels` computes yields; `lib` owns state and the tick.

pub mod channels;

use std::collections::BTreeMap;

use channels::{CHANNELS, ChannelYields};
use cohorts::{CohortKey, Cohorts};
use fauna::Fauna;
use nations::WorldNations;
use tuning::Tuning;
use world_map::WorldFields;
use world_schema::{Quantity, TileId};

/// Per-settlement economic state.
#[derive(Debug, Default, Clone)]
pub struct TileEconomy {
    pub stock: Quantity,
    /// Cultivation establishment 0..1 — sunk labor that makes fields real.
    pub establishment: Quantity,
    /// Domesticated grazer head kept by the band on this tile.
    pub herd: Quantity,
    /// Last month's channel yields, for reports and the viewer.
    pub last: ChannelYields,
    pub last_nutrition: Quantity,
}

/// All settlement economies plus hunger tracking.
#[derive(Debug, Default)]
pub struct Economy {
    pub tiles: BTreeMap<u32, TileEconomy>,
    hunger_streak: BTreeMap<u32, u8>,
}

/// What the monthly tick tells the rest of the sim.
#[derive(Debug, Default)]
pub struct MonthFood {
    /// Nutrition per settled tile (1 = fully fed), keyed by tile id.
    pub nutrition: BTreeMap<u32, Quantity>,
    /// Tiles that went hungry enough to count as famine.
    pub famines: Vec<TileId>,
    /// Bands hungry long enough that the autopilot should move them.
    pub starving_moves: Vec<TileId>,
}

impl Economy {
    /// One month: allocate labor, extract from the living world, eat, store.
    pub fn tick_month(
        &mut self,
        world: &mut WorldNations,
        fields: &WorldFields,
        wild: &mut Fauna,
        flora_live: &mut [u8],
        all_cohorts: &Cohorts,
        tun: &Tuning,
    ) -> MonthFood {
        let mut out = MonthFood::default();
        let occupied: Vec<(u32, u32)> = world
            .owner
            .iter()
            .enumerate()
            .filter_map(|(t, o)| o.map(|n| (t as u32, n.0)))
            .collect();

        for (t, nation_idx) in occupied {
            let Some(ni) = world.nations.iter().position(|n| n.id.0 == nation_idx) else {
                continue;
            };
            let workers = all_cohorts.population_of(CohortKey {
                tile: TileId(t),
                species: world.nations[ni].species,
            });
            if workers <= Quantity::ZERO {
                continue;
            }
            let entry = self.tiles.entry(t).or_default();
            if !world.nations[ni].labor_directed {
                let mut labor = world.nations[ni].labor_milli;
                autopilot_weights(
                    &mut labor,
                    fields,
                    wild,
                    flora_live,
                    entry,
                    t as usize,
                    &tun.subsistence,
                );
                world.nations[ni].labor_milli = labor;
            }
            let labor = world.nations[ni].labor_milli;
            let yields = channels::extract(
                &labor,
                workers,
                fields,
                wild,
                flora_live,
                entry,
                &world.works,
                t as usize,
                &tun.subsistence,
                &tun.ecology,
                &tun.society,
            );

            entry.stock += yields.total();
            let need = workers * Quantity::from_num(tun.subsistence.food_per_head);
            let eaten = entry.stock.min(need);
            entry.stock -= eaten;
            let cap = if world.works.has_granary(t) {
                Quantity::from_num(tun.subsistence.store_granary)
            } else {
                Quantity::from_num(tun.subsistence.store_base)
            };
            entry.stock = (entry.stock * Quantity::from_num(tun.subsistence.store_keep)).min(cap);

            let nutrition = if need > Quantity::ZERO {
                eaten / need
            } else {
                Quantity::ONE
            };
            entry.last = yields;
            entry.last_nutrition = nutrition;
            out.nutrition.insert(t, nutrition);

            if nutrition < Quantity::from_num(tun.subsistence.famine_nutrition) {
                out.famines.push(TileId(t));
                let streak = self.hunger_streak.entry(t).or_insert(0);
                *streak = streak.saturating_add(1);
                if *streak >= tun.subsistence.hunger_streak_to_move {
                    out.starving_moves.push(TileId(t));
                }
            } else {
                self.hunger_streak.remove(&t);
            }
        }
        out
    }

    /// A band moved: its stores and herd travel with it; fields stay behind.
    pub fn relocate(&mut self, from: TileId, to: TileId) {
        let carried = self.tiles.remove(&from.0).unwrap_or_default();
        let target = self.tiles.entry(to.0).or_default();
        target.stock += carried.stock;
        target.herd += carried.herd;
        // Establishment does NOT move — cleared fields are place, not property.
        self.hunger_streak.remove(&from.0);
    }

    #[must_use]
    pub fn tile(&self, tile: u32) -> Option<&TileEconomy> {
        self.tiles.get(&tile)
    }
}

/// Sustainable per-worker food estimate for a tile — the number that steers
/// splits, moves, and frontier tables. One formula, no vocabulary.
#[must_use]
pub fn potential(
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    tile: usize,
    sub: &tuning::Subsistence,
) -> Quantity {
    if fields.elevation[tile] < 0 {
        return Quantity::ZERO;
    }
    channels::potential(fields, wild, flora_live, tile, sub)
}

/// Undirected bands follow marginal returns: shift a step toward the best
/// channel, away from the worst. Deterministic, slow, unbiased.
fn autopilot_weights(
    labor: &mut [u16; CHANNELS],
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    tile_econ: &TileEconomy,
    tile: usize,
    sub: &tuning::Subsistence,
) {
    let marginal = channels::marginal(fields, wild, flora_live, tile_econ, tile, sub);
    let best = (0..CHANNELS)
        .max_by_key(|&i| (marginal[i].to_bits(), CHANNELS - i))
        .expect("channels");
    let worst = (0..CHANNELS)
        .filter(|&i| labor[i] > 0)
        .min_by_key(|&i| (marginal[i].to_bits(), CHANNELS - i))
        .expect("channels");
    if best == worst {
        return;
    }
    let step = labor[worst].min(sub.autopilot_step);
    labor[worst] -= step;
    labor[best] = labor[best].saturating_add(step);
}
