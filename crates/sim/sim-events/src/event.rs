//! The authoritative world events. Every fact anyone can quote is one of these.

use serde::{Deserialize, Serialize};
use world_schema::{NationId, ProvinceId, Quantity, SpeciesId, Tick};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    WorldGenerated {
        land_cells: u32,
        provinces: u32,
        habitable_provinces: u32,
        flora_species: u16,
        cohorts: u32,
        population: Quantity,
    },
    NationSpawned {
        nation: NationId,
        species: SpeciesId,
        seat: ProvinceId,
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
        province: ProvinceId,
    },
    DirectiveRejected {
        tick: Tick,
        nation: NationId,
        reason: String,
    },
    ProvinceSettled {
        tick: Tick,
        nation: NationId,
        from: ProvinceId,
        province: ProvinceId,
        settlers: Quantity,
    },
    NationsMet {
        tick: Tick,
        a: NationId,
        b: NationId,
    },
    Famine {
        tick: Tick,
        province: ProvinceId,
        species: SpeciesId,
    },
    MonthClosed {
        tick: Tick,
        births: Quantity,
        deaths: Quantity,
        population: Quantity,
    },
}
