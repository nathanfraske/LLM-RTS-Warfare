//! Raised structures (docs/30-structures.md): what a nation has built and
//! is building, tile by tile. Every building is a derived `Design` — no
//! catalog exists — and carries live **integrity** that quakes, fire, and
//! ash spend. Effects flow from composition: the walls set the numbers.

use std::collections::BTreeMap;

use sim_events::{Event, EventLog};
use structures::Design;
use world_schema::{NationId, Quantity, Tick, TileId};

/// A standing building: its design and what it has left to withstand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Building {
    pub design: Design,
    pub integrity: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkState {
    pub design: Design,
    pub months_left: u8,
}

/// All structures in the world, keyed by tile.
#[derive(Debug, Default)]
pub struct Works {
    building: BTreeMap<u32, Vec<WorkState>>,
    done: BTreeMap<u32, Vec<Building>>,
}

impl Works {
    /// Everything standing or rising here — the tile's building load.
    #[must_use]
    pub fn load(&self, tile: u32) -> usize {
        self.done.get(&tile).map_or(0, Vec::len) + self.building.get(&tile).map_or(0, Vec::len)
    }

    pub fn commission(&mut self, tile: u32, design: Design) {
        let months_left = design.months;
        self.building.entry(tile).or_default().push(WorkState {
            design,
            months_left,
        });
    }

    #[must_use]
    pub fn completed(&self, tile: u32) -> &[Building] {
        self.done.get(&tile).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn in_progress(&self, tile: u32) -> &[WorkState] {
        self.building.get(&tile).map_or(&[], Vec::as_slice)
    }

    /// Names of everything standing here, for maps and reports.
    #[must_use]
    pub fn names(&self, tile: u32) -> Vec<String> {
        self.completed(tile)
            .iter()
            .map(|b| b.design.name.clone())
            .collect()
    }

    /// The strongest reading of one aspect across everything standing.
    fn best_aspect(&self, tile: u32, read: impl Fn(&Building) -> u16) -> u16 {
        self.completed(tile).iter().map(read).max().unwrap_or(0)
    }

    /// Worked ground multiplies cultivation — whatever building carries it.
    #[must_use]
    pub fn cultivation_mult(&self, tile: u32, st: &tuning::Structures) -> Quantity {
        Quantity::ONE
            + Quantity::from_num(self.best_aspect(tile, |b| b.design.aspects.worked_ground_milli))
                * Quantity::from_num(st.field_mult_permille)
                / Quantity::from_num(1_000_000)
    }

    /// Capacity holds stores over the base — whatever building has room.
    #[must_use]
    pub fn store_bonus(&self, tile: u32, st: &tuning::Structures) -> Quantity {
        Quantity::from_num(self.best_aspect(tile, |b| b.design.aspects.capacity_milli))
            * Quantity::from_num(st.store_capacity)
            / Quantity::from_num(1000)
    }

    /// Cover and hearth shelter families — whatever building offers them.
    #[must_use]
    pub fn birth_mult(&self, tile: u32, st: &tuning::Structures) -> Quantity {
        let shelter = self.best_aspect(tile, |b| {
            b.design.aspects.cover_milli / 2 + b.design.aspects.hearth_milli / 2
        });
        Quantity::ONE
            + Quantity::from_num(shelter) * Quantity::from_num(st.shelter_permille)
                / Quantity::from_num(1_000_000)
    }

    /// Spend integrity on everything here that the filter admits; what
    /// runs out falls, and the fallen names are returned for the record.
    pub fn damage(
        &mut self,
        tile: u32,
        amount: u16,
        hits: impl Fn(&Building) -> bool,
    ) -> Vec<String> {
        let Some(list) = self.done.get_mut(&tile) else {
            return Vec::new();
        };
        let mut fallen = Vec::new();
        for b in list.iter_mut() {
            if !hits(b) {
                continue;
            }
            b.integrity = b.integrity.saturating_sub(amount);
            if b.integrity == 0 {
                fallen.push(b.design.name.clone());
            }
        }
        list.retain(|b| b.integrity > 0);
        if list.is_empty() {
            self.done.remove(&tile);
        }
        fallen
    }

    /// A quake shakes everything standing; poor footings feel it double.
    pub fn shake(&mut self, tile: u32, st: &tuning::Structures) -> Vec<String> {
        let mut fallen = self.damage(tile, st.quake_damage, |b| b.design.footing_milli < 400);
        fallen.extend(self.damage(tile, st.quake_damage / 2, |b| b.design.footing_milli >= 400));
        fallen
    }

    /// Fire finds what can burn: light roofs and soft walls.
    pub fn scorch(&mut self, tile: u32, st: &tuning::Structures) -> Vec<String> {
        self.damage(tile, st.fire_scorch, |b| {
            b.design.roof.mass < st.light_roof_mass || b.design.wall.mass < 500
        })
    }

    /// Heavy ash loads crush light roofs.
    pub fn ash_load(&mut self, tile: u32, st: &tuning::Structures) -> Vec<String> {
        self.damage(tile, st.ash_load, |b| {
            b.design.roof.mass < st.light_roof_mass
        })
    }

    /// Advance construction one month; completions become world events.
    pub fn tick_month(&mut self, owner: &[Option<NationId>], tick: Tick, log: &mut EventLog) {
        let mut finished: Vec<(u32, Design)> = Vec::new();
        for (&tile, states) in &mut self.building {
            for state in states.iter_mut() {
                state.months_left -= 1;
                if state.months_left == 0 {
                    finished.push((tile, state.design.clone()));
                }
            }
            states.retain(|w| w.months_left > 0);
        }
        self.building.retain(|_, v| !v.is_empty());
        for (tile, design) in finished {
            let name = design.name.clone();
            let integrity = design.integrity_milli;
            self.done
                .entry(tile)
                .or_default()
                .push(Building { design, integrity });
            if let Some(nation) = owner[tile as usize] {
                log.push(Event::WorkCompleted {
                    tick,
                    nation,
                    tile: TileId(tile),
                    work: name,
                });
            }
        }
    }
}
