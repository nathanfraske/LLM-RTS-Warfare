//! Commissioned works: direct projects that institutions build over months,
//! with effects on the tile that hosts them (docs/16-mandate-and-works.md).

use std::collections::BTreeMap;

use directive_schema::WorkKind;
use sim_events::{Event, EventLog};
use world_schema::{NationId, Quantity, Tick, TileId};

#[must_use]
pub fn build_months(kind: WorkKind) -> u8 {
    match kind {
        WorkKind::Farmstead => 6,
        WorkKind::Granary => 8,
        WorkKind::Dwellings => 5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkState {
    pub kind: WorkKind,
    pub months_left: u8,
}

/// All works in the world, keyed by tile.
#[derive(Debug, Default)]
pub struct Works {
    building: BTreeMap<u32, Vec<WorkState>>,
    done: BTreeMap<u32, Vec<WorkKind>>,
}

impl Works {
    #[must_use]
    pub fn has_or_building(&self, tile: u32, kind: WorkKind) -> bool {
        self.done.get(&tile).is_some_and(|v| v.contains(&kind))
            || self
                .building
                .get(&tile)
                .is_some_and(|v| v.iter().any(|w| w.kind == kind))
    }

    pub fn commission(&mut self, tile: u32, kind: WorkKind) {
        self.building.entry(tile).or_default().push(WorkState {
            kind,
            months_left: build_months(kind),
        });
    }

    #[must_use]
    pub fn completed(&self, tile: u32) -> &[WorkKind] {
        self.done.get(&tile).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn in_progress(&self, tile: u32) -> &[WorkState] {
        self.building.get(&tile).map_or(&[], Vec::as_slice)
    }

    /// A farmstead grows the land's carrying capacity.
    #[must_use]
    pub fn capacity_mult(&self, tile: u32) -> Quantity {
        if self.completed(tile).contains(&WorkKind::Farmstead) {
            Quantity::from_num(1.35)
        } else {
            Quantity::ONE
        }
    }

    /// Dwellings shelter families.
    #[must_use]
    pub fn birth_mult(&self, tile: u32) -> Quantity {
        if self.completed(tile).contains(&WorkKind::Dwellings) {
            Quantity::from_num(1.12)
        } else {
            Quantity::ONE
        }
    }

    /// A granary holds hunger at bay longer.
    #[must_use]
    pub fn famine_threshold(&self, tile: u32) -> Quantity {
        if self.completed(tile).contains(&WorkKind::Granary) {
            Quantity::from_num(1.40)
        } else {
            Quantity::from_num(1.15)
        }
    }

    /// Advance construction one month; completions become world events.
    pub fn tick_month(&mut self, owner: &[Option<NationId>], tick: Tick, log: &mut EventLog) {
        let mut finished: Vec<(u32, WorkKind)> = Vec::new();
        for (&tile, states) in &mut self.building {
            for state in states.iter_mut() {
                state.months_left -= 1;
                if state.months_left == 0 {
                    finished.push((tile, state.kind));
                }
            }
            states.retain(|w| w.months_left > 0);
        }
        self.building.retain(|_, v| !v.is_empty());
        for (tile, kind) in finished {
            self.done.entry(tile).or_default().push(kind);
            if let Some(nation) = owner[tile as usize] {
                log.push(Event::WorkCompleted {
                    tick,
                    nation,
                    tile: TileId(tile),
                    work: kind,
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
        let mut works = Works::default();
        let owner = vec![Some(NationId(0)); 4];
        let mut log = EventLog::new();
        works.commission(2, WorkKind::Farmstead);
        assert!(works.has_or_building(2, WorkKind::Farmstead));
        assert_eq!(works.capacity_mult(2), Quantity::ONE, "not built yet");
        for month in 1..=6u64 {
            works.tick_month(&owner, Tick(month * 720), &mut log);
        }
        assert_eq!(works.capacity_mult(2), Quantity::from_num(1.35));
        assert_eq!(works.in_progress(2).len(), 0);
        assert_eq!(log.len(), 1, "completion is a world event");
        assert_eq!(works.famine_threshold(2), Quantity::from_num(1.15));
        works.commission(2, WorkKind::Granary);
        for month in 7..=14u64 {
            works.tick_month(&owner, Tick(month * 720), &mut log);
        }
        assert_eq!(works.famine_threshold(2), Quantity::from_num(1.40));
    }
}
