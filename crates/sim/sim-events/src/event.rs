//! The authoritative world events. Every fact anyone can quote is one of these.

use serde::{Deserialize, Serialize};
use world_schema::{NationId, Quantity, SpeciesId, Tick, TileId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    WorldGenerated {
        land_tiles: u32,
        habitable_tiles: u32,
        flora_species: u16,
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
    StanceChanged {
        tick: Tick,
        nation: NationId,
        stance: directive_schema::Stance,
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
    WorkCommissioned {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        work: directive_schema::WorkKind,
    },
    WorkCompleted {
        tick: Tick,
        nation: NationId,
        tile: TileId,
        work: directive_schema::WorkKind,
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
    MonthClosed {
        tick: Tick,
        births: Quantity,
        deaths: Quantity,
        population: Quantity,
    },
}
