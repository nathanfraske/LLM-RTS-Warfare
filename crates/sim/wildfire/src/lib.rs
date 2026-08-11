//! Fire in the world (docs/26): where fuel, dryness, and an ignition meet,
//! the land burns — and spreads, downwind fastest, until rain, snow, or
//! bare ground starves it. Lava lights it; dry lightning rarely does; a
//! monthly field pass carries it. Burnt ground loses its green and gains
//! char the ordinary weathering folds back into soil.

use sim_events::rng;
use sim_events::{SystemId, WorldSeed};
use tuning::{Seasons, Wildfire};
use world_map::{Water, WorldFields, tiles};
use world_schema::{Tick, TileId};

const WILDFIRE: SystemId = SystemId(14);

/// The burning world: fire intensity per tile, 0 = cold.
#[derive(Debug)]
pub struct Blaze {
    pub fire: Vec<u8>,
}

/// What a month of burning tells the rest of the sim.
#[derive(Debug, Default)]
pub struct MonthBurn {
    /// Tiles that caught this month (for events and eyes).
    pub ignited: Vec<TileId>,
    /// Burning tiles — anything settled here is in trouble.
    pub burning: Vec<TileId>,
}

impl Blaze {
    #[must_use]
    pub fn new(cells: usize) -> Self {
        Self {
            fire: vec![0; cells],
        }
    }

    /// Something hot touched this tile: lava, a spilled burning humor,
    /// later a torch. It catches if there is anything to burn.
    pub fn ignite(&mut self, tile: usize, flora_live: &[u8], fw: &Wildfire) {
        if flora_live[tile] >= fw.fuel_min {
            self.fire[tile] = self.fire[tile].max(fw.ignite_intensity);
        }
    }

    /// One month of burning: consume fuel, spread by wind and drought,
    /// die where rain, snow, or bare ground starves the flame.
    // Fire, fuel, and sky arrays share the tile index throughout.
    #[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
    pub fn tick_month(
        &mut self,
        seed: WorldSeed,
        tick: Tick,
        fields: &WorldFields,
        sky: &climate::Climate,
        flora_live: &mut [u8],
        se: &Seasons,
        fw: &Wildfire,
    ) -> MonthBurn {
        let mut out = MonthBurn::default();
        let size = fields.size as usize;

        // Dry lightning: rare, deterministic, only where the land is
        // parched tinder.
        for tile in 0..self.fire.len() {
            if self.fire[tile] == 0
                && fields.water[tile] == Water::Dry
                && tiles::is_land(fields, tile)
                && flora_live[tile] >= fw.tinder_fuel
                && sky.delivered[tile] < fw.dry_delivered
                && climate::t_eff(fields, tile, tick.0 / 720, se) > 0
            {
                let roll = rng::draw(seed, tick, WILDFIRE, tile as u64) % 100_000;
                if roll < u64::from(fw.lightning_permyriad) {
                    self.fire[tile] = fw.ignite_intensity;
                    out.ignited.push(TileId(tile as u32));
                }
            }
        }

        // Spread from everything burning, downwind twice as eagerly.
        let burning_now: Vec<usize> = (0..self.fire.len()).filter(|&t| self.fire[t] > 0).collect();
        for &tile in &burning_now {
            let (x, y) = (tile % size, tile / size);
            let wind = climate::wind_dx(y, size);
            let (neighbors, n) = fields.grid().neighbors8(tile);
            for &nb in &neighbors[..n] {
                if self.fire[nb] > 0
                    || !tiles::is_land(fields, nb)
                    || fields.water[nb] != Water::Dry
                    || flora_live[nb] < fw.fuel_min
                    || sky.snowpack[nb] > 0
                {
                    continue;
                }
                let dry = sky.delivered[nb] < fw.dry_delivered;
                let downwind = i64::try_from(nb % size).expect("map-bounded")
                    - i64::try_from(x).expect("map-bounded")
                    == wind as i64;
                let chance = u64::from(fw.spread_permille)
                    * if dry { 2 } else { 1 }
                    * if downwind { 2 } else { 1 };
                let roll = rng::draw(seed, tick, WILDFIRE, (tile as u64) << 20 | nb as u64) % 1000;
                if roll < chance {
                    self.fire[nb] = fw.ignite_intensity;
                    out.ignited.push(TileId(nb as u32));
                }
            }
        }

        // Burn down: fire eats its fuel and gutters out without it or
        // under the rain.
        for tile in 0..self.fire.len() {
            if self.fire[tile] == 0 {
                continue;
            }
            flora_live[tile] = flora_live[tile].saturating_sub(fw.burn_rate);
            let quenched = sky.delivered[tile] >= fw.quench_delivered || sky.snowpack[tile] > 0;
            if quenched || flora_live[tile] < fw.fuel_min {
                self.fire[tile] = 0;
            } else {
                out.burning.push(TileId(tile as u32));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_spreads_through_dry_fuel_and_dies_at_the_water() {
        let fields = WorldFields {
            size: 4,
            elevation: vec![10; 16],
            water: {
                let mut w = vec![Water::Dry; 16];
                w[3] = Water::River;
                w[7] = Water::River;
                w[11] = Water::River;
                w[15] = Water::River;
                w
            },
            flow_acc: vec![1; 16],
            drains_to: vec![u32::MAX; 16],
            temperature: vec![250; 16],
            moisture: vec![40; 16],
            cell_fertility: vec![100; 16],
        };
        let sky = climate::Climate {
            wet: vec![40; 16],
            snowpack: vec![0; 16],
            growth: vec![300; 16],
            delivered: vec![0; 16],
        };
        let fw = Wildfire::default();
        let mut flora = vec![200u8; 16];
        let mut blaze = Blaze::new(16);
        blaze.ignite(0, &flora, &fw);
        assert!(blaze.fire[0] > 0, "tinder catches");

        let seed = WorldSeed(9);
        let mut months_burning = 0;
        for month in 1..=24u64 {
            let burn = blaze.tick_month(
                seed,
                Tick(month * 720),
                &fields,
                &sky,
                &mut flora,
                &Seasons::default(),
                &fw,
            );
            if !burn.burning.is_empty() {
                months_burning += 1;
            }
        }
        assert!(months_burning >= 2, "a dry fire lives more than a month");
        assert!(
            flora.iter().take(3).any(|&f| f < 60),
            "the burn eats the green"
        );
        assert!(
            (0..16).all(|t| fields.water[t] == Water::Dry || blaze.fire[t] == 0),
            "fire never stands on water"
        );

        // Determinism.
        let mut again = Blaze::new(16);
        let mut flora2 = vec![200u8; 16];
        again.ignite(0, &flora2, &fw);
        for month in 1..=24u64 {
            let _ = again.tick_month(
                seed,
                Tick(month * 720),
                &fields,
                &sky,
                &mut flora2,
                &Seasons::default(),
                &fw,
            );
        }
        assert_eq!(flora, flora2, "same seed, same burn scar");
    }
}
