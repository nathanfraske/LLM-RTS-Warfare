//! The known world: what each nation has actually seen, and when
//! (docs/22-knowledge-and-discovery.md). Nothing is given — reports,
//! autopilot, and decree validation all read this memory, never the world.
//! Scout parties, the walking couriers of knowledge, live in `scouts`.

pub mod scouts;

use std::collections::BTreeMap;

use sim_events::{Event, EventLog};
use world_map::WorldFields;
use world_schema::{NationId, Quantity, Tick, TileId};

/// One remembered tile: when we last saw it and what we saw. Terrain is
/// implied (once seen, forever — mountains don't move); numbers stale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileMemory {
    pub last_seen: Tick,
    /// The food promise as it looked that day.
    pub potential: Quantity,
    /// Who held it that day.
    pub owner: Option<NationId>,
}

/// A single nation's map-as-memory.
#[derive(Debug)]
pub struct NationKnowledge {
    tiles: Vec<Option<TileMemory>>,
}

impl NationKnowledge {
    #[must_use]
    pub fn new(cells: usize) -> Self {
        Self {
            tiles: vec![None; cells],
        }
    }

    #[must_use]
    pub fn known(&self, tile: usize) -> Option<&TileMemory> {
        self.tiles.get(tile).and_then(Option::as_ref)
    }

    /// Months since this tile was last seen; `None` if never.
    #[must_use]
    pub fn age_months(&self, tile: usize, now: Tick) -> Option<u64> {
        self.known(tile)
            .map(|m| (now.0.saturating_sub(m.last_seen.0)) / 720)
    }

    pub fn observe(&mut self, tile: usize, seen: TileMemory) {
        if tile < self.tiles.len() {
            self.tiles[tile] = Some(seen);
        }
    }

    #[must_use]
    pub fn known_count(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_some()).count()
    }
}

/// What one step into a tile costs a walker, per mille of the base pace
/// (docs/22, docs/26): climbs are slow, high country is slow, deep snow
/// is slowest. Every ledger-scale mover — scouts today, envoys, caravans,
/// and armies tomorrow — pays the same ground the same way.
#[must_use]
pub fn travel_milli(
    fields: &WorldFields,
    sky: &climate::Climate,
    from: usize,
    into: usize,
    exp: &tuning::Exploration,
) -> u32 {
    let climb = i64::from(fields.elevation[into] - fields.elevation[from]).max(0);
    let mut cost =
        1000 + u32::try_from(climb).unwrap_or(0) * u32::from(exp.travel_slope_permille) / 100;
    if fields.elevation[into] > exp.travel_high_elevation {
        cost += u32::from(exp.travel_high_permille);
    }
    let snow = u32::from(sky.snowpack[into]);
    if snow > 0 {
        cost += u32::from(exp.travel_snow_permille) * snow.min(900) / 900;
    }
    cost
}

/// The eight bearings a scout can be sent out on.
pub const BEARING_NAMES: [&str; 8] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];
pub const BEARING_DELTAS: [(i64, i64); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Every nation's memory plus the parties currently afield.
#[derive(Debug, Default)]
pub struct WorldKnowledge {
    nations: BTreeMap<u32, NationKnowledge>,
    pub parties: Vec<scouts::Party>,
    /// Earliest tick each nation may next dispatch a need-driven scout.
    need_cooldown: BTreeMap<u32, u64>,
}

impl WorldKnowledge {
    #[must_use]
    pub fn new(cells: usize, nation_ids: impl Iterator<Item = NationId>) -> Self {
        Self {
            nations: nation_ids
                .map(|id| (id.0, NationKnowledge::new(cells)))
                .collect(),
            parties: Vec::new(),
            need_cooldown: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn of(&self, nation: NationId) -> &NationKnowledge {
        self.nations.get(&nation.0).expect("nation has a memory")
    }

    pub fn of_mut(&mut self, nation: NationId) -> &mut NationKnowledge {
        self.nations
            .get_mut(&nation.0)
            .expect("nation has a memory")
    }

    /// Home is always fresh: re-observe owned tiles and their surroundings.
    pub fn refresh_home(
        &mut self,
        nation: NationId,
        owned: &[TileId],
        fields: &WorldFields,
        sample: &dyn Fn(usize) -> (Quantity, Option<NationId>),
        now: Tick,
    ) {
        let grid = fields.grid();
        let memory = self.of_mut(nation);
        for &t in owned {
            let (neighbors, n) = grid.neighbors8(t.0 as usize);
            for &tile in std::iter::once(&(t.0 as usize)).chain(&neighbors[..n]) {
                let (potential, holder) = sample(tile);
                memory.observe(
                    tile,
                    TileMemory {
                        last_seen: now,
                        potential,
                        owner: holder,
                    },
                );
            }
        }
    }

    /// How many active parties a nation has afield.
    #[must_use]
    pub fn parties_of(&self, nation: NationId) -> usize {
        self.parties.iter().filter(|p| p.nation == nation).count()
    }

    /// Send a party out. The caller has validated party limits and pricing.
    pub fn dispatch(
        &mut self,
        nation: NationId,
        home: TileId,
        bearing: usize,
        now: Tick,
        log: &mut EventLog,
    ) {
        self.parties
            .push(scouts::Party::new(nation, home, bearing, now));
        log.push(Event::ScoutDispatched {
            tick: now,
            nation,
            bearing: BEARING_NAMES[bearing % 8].to_string(),
        });
    }

    /// A need-driven dispatch (autopilot): rate-limited, capped, aimed at
    /// the darkest bearing. Returns true if a party actually left.
    pub fn need_scout(
        &mut self,
        nation: NationId,
        home: TileId,
        fields: &WorldFields,
        now: Tick,
        exp: &tuning::Exploration,
        log: &mut EventLog,
    ) -> bool {
        if self.parties_of(nation) >= usize::from(exp.max_parties) {
            return false;
        }
        if now.0 < self.need_cooldown.get(&nation.0).copied().unwrap_or(0) {
            return false;
        }
        let bearing = self.darkest_bearing(nation, home, fields, exp.scout_range);
        self.need_cooldown
            .insert(nation.0, now.0 + u64::from(exp.scout_cooldown_months) * 720);
        self.dispatch(nation, home, bearing, now, log);
        true
    }

    /// The bearing with the most unseen tiles within scouting range of home.
    #[must_use]
    pub fn darkest_bearing(
        &self,
        nation: NationId,
        home: TileId,
        fields: &WorldFields,
        range: u16,
    ) -> usize {
        let grid = fields.grid();
        let (hx, hy) = grid.xy(home.0 as usize);
        let memory = self.of(nation);
        let mut best = (0usize, -1i64);
        for (b, (dx, dy)) in BEARING_DELTAS.iter().enumerate() {
            let mut unknown = 0i64;
            for step in 1..=i64::from(range) {
                let x = i64::from(hx) + dx * step;
                let y = i64::from(hy) + dy * step;
                if x < 0 || y < 0 || x >= i64::from(fields.size) || y >= i64::from(fields.size) {
                    break;
                }
                let tile = (y as usize) * fields.size as usize + x as usize;
                if memory.known(tile).is_none() {
                    unknown += 1;
                }
            }
            if unknown > best.1 {
                best = (b, unknown);
            }
        }
        best.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_is_observed_not_given() {
        let mut world = WorldKnowledge::new(16, [NationId(0)].into_iter());
        assert_eq!(
            world.of(NationId(0)).known_count(),
            0,
            "the world starts dark"
        );
        world.of_mut(NationId(0)).observe(
            5,
            TileMemory {
                last_seen: Tick(720),
                potential: Quantity::from_num(2),
                owner: None,
            },
        );
        assert_eq!(world.of(NationId(0)).known_count(), 1);
        assert_eq!(
            world.of(NationId(0)).age_months(5, Tick(2880)),
            Some(3),
            "memories age"
        );
        assert_eq!(world.of(NationId(0)).age_months(6, Tick(2880)), None);
    }

    #[test]
    fn the_ground_prices_the_walk() {
        let exp = tuning::Exploration::default();
        let fields = WorldFields {
            size: 2,
            elevation: vec![10, 900, 2_000, 10],
            water: vec![world_map::Water::Dry; 4],
            flow_acc: vec![1; 4],
            drains_to: vec![u32::MAX; 4],
            temperature: vec![150; 4],
            moisture: vec![100; 4],
            cell_fertility: vec![100; 4],
        };
        let mut sky = climate::Climate {
            wet: vec![100; 4],
            snowpack: vec![0; 4],
            growth: vec![500; 4],
            delivered: vec![10; 4],
        };
        let flat = travel_milli(&fields, &sky, 3, 0, &exp);
        let climb = travel_milli(&fields, &sky, 0, 1, &exp);
        let high = travel_milli(&fields, &sky, 1, 2, &exp);
        assert!(climb > flat, "climbing costs more than the flat");
        assert!(high > climb, "the high country costs more still");
        sky.snowpack[0] = 600;
        let snowed = travel_milli(&fields, &sky, 3, 0, &exp);
        assert!(snowed > flat, "deep snow slows the same ground");
    }
}
