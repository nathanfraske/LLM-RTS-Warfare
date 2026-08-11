//! Commissioned works: direct projects that institutions build over months,
//! with effects on the tile that hosts them (docs/16-mandate-and-works.md).
//! The catalog is registry data — what `works.commission` offers is exactly
//! what `catalog` returns; effects key off the work's name here, its owner.

use std::collections::BTreeMap;

use sim_events::{Event, EventLog};
use tuning::Society;
use world_schema::{NationId, Quantity, Tick, TileId};

pub const FARMSTEAD: &str = "farmstead";
pub const GRANARY: &str = "granary";
pub const DWELLINGS: &str = "dwellings";

/// Every commissionable work and its build time. Adding a work here (plus
/// its effect below) is the whole job — the registry and reports follow.
#[must_use]
pub fn catalog(soc: &Society) -> Vec<(&'static str, u8)> {
    vec![
        (FARMSTEAD, soc.farmstead_months),
        (GRANARY, soc.granary_months),
        (DWELLINGS, soc.dwellings_months),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkState {
    pub work: String,
    pub months_left: u8,
}

/// All works in the world, keyed by tile.
#[derive(Debug, Default)]
pub struct Works {
    building: BTreeMap<u32, Vec<WorkState>>,
    done: BTreeMap<u32, Vec<String>>,
}

impl Works {
    #[must_use]
    pub fn has_or_building(&self, tile: u32, work: &str) -> bool {
        self.done
            .get(&tile)
            .is_some_and(|v| v.iter().any(|w| w == work))
            || self
                .building
                .get(&tile)
                .is_some_and(|v| v.iter().any(|w| w.work == work))
    }

    pub fn commission(&mut self, tile: u32, work: &str, soc: &Society) {
        let months = catalog(soc)
            .iter()
            .find(|(key, _)| *key == work)
            .map(|(_, months)| *months)
            .expect("commission is validated against the catalog");
        self.building.entry(tile).or_default().push(WorkState {
            work: work.to_string(),
            months_left: months,
        });
    }

    #[must_use]
    pub fn completed(&self, tile: u32) -> &[String] {
        self.done.get(&tile).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn in_progress(&self, tile: u32) -> &[WorkState] {
        self.building.get(&tile).map_or(&[], Vec::as_slice)
    }

    /// A farmstead multiplies cultivation yield (docs/19).
    #[must_use]
    pub fn cultivation_mult(&self, tile: u32, soc: &Society) -> Quantity {
        if self.completed(tile).iter().any(|w| w == FARMSTEAD) {
            Quantity::from_num(soc.farmstead_cultivation_mult)
        } else {
            Quantity::ONE
        }
    }

    /// Dwellings shelter families.
    #[must_use]
    pub fn birth_mult(&self, tile: u32, soc: &Society) -> Quantity {
        if self.completed(tile).iter().any(|w| w == DWELLINGS) {
            Quantity::from_num(soc.dwellings_birth_mult)
        } else {
            Quantity::ONE
        }
    }

    /// A granary is a real container: it raises the food storage cap.
    #[must_use]
    pub fn has_granary(&self, tile: u32) -> bool {
        self.completed(tile).iter().any(|w| w == GRANARY)
    }

    /// Advance construction one month; completions become world events.
    pub fn tick_month(&mut self, owner: &[Option<NationId>], tick: Tick, log: &mut EventLog) {
        let mut finished: Vec<(u32, String)> = Vec::new();
        for (&tile, states) in &mut self.building {
            for state in states.iter_mut() {
                state.months_left -= 1;
                if state.months_left == 0 {
                    finished.push((tile, state.work.clone()));
                }
            }
            states.retain(|w| w.months_left > 0);
        }
        self.building.retain(|_, v| !v.is_empty());
        for (tile, work) in finished {
            self.done.entry(tile).or_default().push(work.clone());
            if let Some(nation) = owner[tile as usize] {
                log.push(Event::WorkCompleted {
                    tick,
                    nation,
                    tile: TileId(tile),
                    work,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_build_over_months_and_apply_their_effects() {
        let soc = Society::default();
        let mut works = Works::default();
        let owner = vec![Some(NationId(0)); 4];
        let mut log = EventLog::new();
        works.commission(2, FARMSTEAD, &soc);
        assert!(works.has_or_building(2, FARMSTEAD));
        assert_eq!(
            works.cultivation_mult(2, &soc),
            Quantity::ONE,
            "not built yet"
        );
        for month in 1..=6u64 {
            works.tick_month(&owner, Tick(month * 720), &mut log);
        }
        assert_eq!(
            works.cultivation_mult(2, &soc),
            Quantity::from_num(soc.farmstead_cultivation_mult)
        );
        assert_eq!(works.in_progress(2).len(), 0);
        assert_eq!(log.len(), 1, "completion is a world event");
        assert!(!works.has_granary(2));
        works.commission(2, GRANARY, &soc);
        for month in 7..=14u64 {
            works.tick_month(&owner, Tick(month * 720), &mut log);
        }
        assert!(works.has_granary(2));
    }
}
