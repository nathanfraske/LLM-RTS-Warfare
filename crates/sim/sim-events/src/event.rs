//! The authoritative world events. Every fact anyone can quote is one of these.

use serde::{Deserialize, Serialize};
use world_schema::{NationId, Quantity, SpeciesId, Tick, TileId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    WorldGenerated {
        land_tiles: u32,
        habitable_tiles: u32,
        flora_species: u16,
        fauna_species: u16,
        cohorts: u32,
        population: Quantity,
    },
    NationSpawned {
        nation: NationId,
        species: SpeciesId,
        seat: TileId,
    },
    NationNamed {
        tick: Tick,
        nation: NationId,
        name: String,
    },
    /// A council decree set a policy leaf (docs/20-open-directives.md).
    PolicySet {
        tick: Tick,
        nation: NationId,
        key: String,
        value: policy::PolicyValue,
    },
    SettlementDecreed {
        tick: Tick,
        nation: NationId,
        tile: TileId,
    },
    DirectiveRejected {
        tick: Tick,
        nation: NationId,
        reason: String,
    },
    BandMoved {
        tick: Tick,
        nation: NationId,
        from: TileId,
        to: TileId,
        /// True when the band gambled into land it had never seen (docs/22).
        blind: bool,
    },
    /// A scout party left to walk a bearing (docs/22).
    ScoutDispatched {
        tick: Tick,
        nation: NationId,
        bearing: String,
    },
    /// A party came home; only now does what it saw enter the map.
    ScoutReturned {
        tick: Tick,
        nation: NationId,
        tiles_learned: u32,
    },
    /// A party never came back; what it knew is gone.
    ScoutLost { tick: Tick, nation: NationId },
    WorkCommissioned {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        /// Registry key of the work, e.g. `farmstead`.
        work: String,
    },
    WorkCompleted {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        work: String,
    },
    TileSettled {
        tick: Tick,
        nation: NationId,
        from: TileId,
        tile: TileId,
        settlers: Quantity,
    },
    NationsMet {
        tick: Tick,
        a: NationId,
        b: NationId,
    },
    Famine {
        tick: Tick,
        tile: TileId,
        species: SpeciesId,
    },
    /// The fire below broke out (docs/29): lava ran `reach` tiles
    /// downtree and ash fell on `ash_tiles` more, downwind.
    VolcanoErupted {
        tick: Tick,
        tile: TileId,
        reach: u32,
        ash_tiles: u32,
    },
    /// Wildfire reached a settled tile (docs/26).
    Wildfire { tick: Tick, tile: TileId },
    /// The fault slipped (docs/29): the ground shook `reach` tiles around.
    Earthquake {
        tick: Tick,
        tile: TileId,
        reach: u32,
    },
    /// The people raised a building on their own need (docs/30) — no
    /// decree, no mandate: room or shelter wanted, and built.
    PeopleRaised {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        work: String,
    },
    /// A shake brought a finished work down.
    WorkToppled {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        work: String,
    },
    MonthClosed {
        tick: Tick,
        births: Quantity,
        deaths: Quantity,
        population: Quantity,
    },
}
