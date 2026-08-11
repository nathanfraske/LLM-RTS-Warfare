//! World tiles as the strategic unit (docs/15-multiscale-maps.md): a tile is
//! the province — it carries its fields directly and owns a local map below.

use crate::WorldFields;
use crate::hydrology::Water;
use crate::terrain::{self, Terrain};
use world_schema::TileId;

/// Land a nation can hold (lakes are water, not territory).
#[must_use]
pub fn is_land(fields: &WorldFields, tile: usize) -> bool {
    fields.elevation[tile] >= 0 && fields.water[tile] != Water::Lake
}

#[must_use]
pub fn label(fields: &WorldFields, tile: usize) -> Terrain {
    terrain::label(
        fields.elevation[tile],
        fields.water[tile],
        fields.temperature[tile],
        fields.moisture[tile],
    )
}

/// Any 8-neighbor is ocean.
#[must_use]
pub fn coastal(fields: &WorldFields, tile: usize) -> bool {
    let (neighbors, n) = fields.grid().neighbors8(tile);
    neighbors[..n]
        .iter()
        .any(|&nb| fields.water[nb] == Water::Ocean)
}

/// Carries or borders fresh water (river through the tile, or a lake beside it).
#[must_use]
pub fn riverine(fields: &WorldFields, tile: usize) -> bool {
    if fields.water[tile] == Water::River {
        return true;
    }
    let (neighbors, n) = fields.grid().neighbors8(tile);
    neighbors[..n]
        .iter()
        .any(|&nb| fields.water[nb] == Water::Lake)
}

/// Fertile enough, with water access, to host settlement.
#[must_use]
pub fn habitable(fields: &WorldFields, tile: usize) -> bool {
    is_land(fields, tile)
        && fields.cell_fertility[tile] > 55
        && (coastal(fields, tile) || riverine(fields, tile) || fields.moisture[tile] > 150)
}

/// Sorted land neighbors of a tile, as ids.
#[must_use]
pub fn land_neighbors(fields: &WorldFields, tile: usize) -> Vec<TileId> {
    let (neighbors, n) = fields.grid().neighbors8(tile);
    neighbors[..n]
        .iter()
        .copied()
        .filter(|&nb| is_land(fields, nb))
        .map(|nb| TileId(nb as u32))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::WorldSeed;

    #[test]
    fn habitable_tiles_exist_and_are_land() {
        let fields = WorldFields::generate(WorldSeed(42), 96);
        let habitable: Vec<usize> = (0..fields.grid().cells())
            .filter(|&i| habitable(&fields, i))
            .collect();
        assert!(
            !habitable.is_empty(),
            "a 96² world must have habitable tiles"
        );
        for &tile in &habitable {
            assert!(is_land(&fields, tile));
        }
    }
}
