//! The five subsistence channels: yield math against the living world.
//! Honest comparative economics is the whole design (docs/19 §4): each
//! channel's returns follow its real logic, none is gated by doctrine.

use crate::TileEconomy;
use fauna::Fauna;
use nations::works::Works;
use tuning::{Ecology, Society, Subsistence};
use world_map::WorldFields;
use world_schema::Quantity;

pub const CHANNELS: usize = 5;
pub const CHANNEL_NAMES: [&str; CHANNELS] = ["gather", "hunt", "fish", "cultivate", "herd"];

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ChannelYields {
    pub by: [Quantity; CHANNELS],
}

impl ChannelYields {
    #[must_use]
    pub fn total(&self) -> Quantity {
        self.by.iter().fold(Quantity::ZERO, |a, b| a + *b)
    }
}

fn shares(labor_milli: &[u16; CHANNELS]) -> [Quantity; CHANNELS] {
    let sum: i64 = labor_milli.iter().map(|&w| i64::from(w)).sum();
    let sum = Quantity::from_num(sum.max(1));
    labor_milli.map(|w| Quantity::from_num(w) / sum)
}

/// Run one month of extraction on a settled tile. Mutates the wild world and
/// the tile economy; returns food gained per channel.
#[allow(clippy::too_many_arguments)]
pub fn extract(
    labor_milli: &[u16; CHANNELS],
    workers: Quantity,
    fields: &WorldFields,
    wild: &mut Fauna,
    flora_live: &mut [u8],
    econ: &mut TileEconomy,
    works: &Works,
    tile: usize,
    sub: &Subsistence,
    eco: &Ecology,
    society: &Society,
) -> ChannelYields {
    let share = shares(labor_milli);
    let mut out = ChannelYields::default();

    // Gather: the edible fraction of living vegetation, worn down by the taking.
    let crew = workers * share[0];
    let flora_q = Quantity::from_num(flora_live[tile]) / Quantity::from_num(255);
    out.by[0] = crew * Quantity::from_num(sub.gather_eff) * flora_q;
    let wear = (out.by[0] / Quantity::from_num(sub.gather_wear_divisor))
        .to_num::<i64>()
        .clamp(0, 6);
    flora_live[tile] = flora_live[tile]
        .saturating_sub(u8::try_from(wear).expect("clamped"))
        .max(eco.flora_floor);

    // Hunt: biomass off the land species, refuge-limited inside `Fauna`.
    let crew = workers * share[1];
    out.by[1] = wild.hunt(tile, crew * Quantity::from_num(sub.hunt_eff), eco);

    // Fish: the waters here and next door.
    let crew = workers * share[2];
    out.by[2] = wild.fish(fields, tile, crew * Quantity::from_num(sub.fish_eff), eco);

    // Cultivate: worthless until establishment is built; then the best
    // yield-per-land anywhere fertile. Farmstead works multiply it.
    let crew = workers * share[3];
    let fert = Quantity::from_num(fields.cell_fertility[tile]) / Quantity::from_num(255);
    out.by[3] = crew
        * Quantity::from_num(sub.cultivate_eff)
        * fert
        * econ.establishment
        * works.cultivation_mult(tile as u32, society);
    econ.establishment = (econ.establishment + share[3] * Quantity::from_num(sub.establish_rate)
        - Quantity::from_num(sub.establish_decay))
    .clamp(Quantity::ZERO, Quantity::ONE);

    // Herd: seeded by capturing wild grazers, grown on pasture, eaten steadily.
    let crew = workers * share[4];
    if share[4] > Quantity::ZERO && econ.herd < Quantity::ONE {
        econ.herd += wild.capture_grazers(tile, crew * Quantity::from_num(sub.capture_eff), eco);
    }
    let pasture = Quantity::from_num(flora_live[tile]) / Quantity::from_num(255);
    if econ.herd > Quantity::ZERO {
        out.by[4] = econ.herd * Quantity::from_num(sub.herd_yield_per_head);
        let herd_cap = Quantity::from_num(sub.herd_cap_per_pasture) * pasture;
        econ.herd =
            (econ.herd + econ.herd * Quantity::from_num(sub.herd_growth) * pasture).min(herd_cap);
        let wear = (econ.herd / Quantity::from_num(sub.herd_wear_divisor))
            .to_num::<i64>()
            .clamp(0, 4);
        flora_live[tile] = flora_live[tile]
            .saturating_sub(u8::try_from(wear).expect("clamped"))
            .max(eco.flora_floor);
    }

    out
}

/// Per-worker marginal estimates — what the report prints and the autopilot
/// follows. Cultivation shows a bootstrap floor so sunk-cost channels are
/// visible without being teleologically favored.
#[must_use]
pub fn marginal(
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    econ: &TileEconomy,
    tile: usize,
    sub: &Subsistence,
) -> [Quantity; CHANNELS] {
    let flora_q = Quantity::from_num(flora_live[tile]) / Quantity::from_num(255);
    let fert = Quantity::from_num(fields.cell_fertility[tile]) / Quantity::from_num(255);
    let hunt_stock =
        (wild.huntable(tile) / Quantity::from_num(sub.hunt_stock_norm)).min(Quantity::ONE);
    let fish_stock =
        (wild.fishable(fields, tile) / Quantity::from_num(sub.fish_stock_norm)).min(Quantity::ONE);
    let establish = econ
        .establishment
        .max(Quantity::from_num(sub.establish_floor));
    let pasture_prospect = ((wild.huntable(tile) + econ.herd)
        / Quantity::from_num(sub.pasture_norm))
    .min(Quantity::ONE)
        * flora_q;
    [
        Quantity::from_num(sub.gather_eff) * flora_q,
        Quantity::from_num(sub.hunt_eff) * hunt_stock,
        Quantity::from_num(sub.fish_eff) * fish_stock,
        Quantity::from_num(sub.cultivate_eff) * fert * establish,
        Quantity::from_num(sub.herd_prospect_eff) * pasture_prospect,
    ]
}

/// One comparable per-worker number for a tile nobody lives on yet —
/// steers splits, moves, and the frontier table.
#[must_use]
pub fn potential(
    fields: &WorldFields,
    wild: &Fauna,
    flora_live: &[u8],
    tile: usize,
    sub: &Subsistence,
) -> Quantity {
    let bare = TileEconomy::default();
    let m = marginal(fields, wild, flora_live, &bare, tile, sub);
    let mut sorted = m;
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    // The two best channels a newcomer band could actually run.
    sorted[0] + sorted[1] / Quantity::from_num(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_map::hydrology::Water as W;

    /// A hand-built 2x2 world: [river-plain, fertile-plain, dry-steppe, ocean].
    fn tiny_world() -> world_map::WorldFields {
        world_map::WorldFields {
            size: 2,
            elevation: vec![10, 40, 30, -500],
            water: vec![W::River, W::Dry, W::Dry, W::Ocean],
            flow_acc: vec![90, 2, 1, 0],
            temperature: vec![190, 200, 240, 180],
            moisture: vec![210, 170, 60, 220],
            cell_fertility: vec![140, 220, 40, 0],
        }
    }

    /// The anti-teleology core (docs/19 §4): different land, different best
    /// channel — nothing is favored by doctrine.
    #[test]
    fn the_landscape_decides_the_best_channel() {
        let fields = tiny_world();
        // Sparse scrub on the riverbank: the waters, not the bushes, feed you.
        let flora = vec![40u8, 160, 70, 0];
        let eco = tuning::Ecology::default();
        let sub = Subsistence::default();
        let wild = fauna::Fauna::genesis(sim_events::WorldSeed(7), &fields, &flora, &eco);

        // River tile with stocked waters: fishing must top the table.
        let bare = TileEconomy::default();
        let river = marginal(&fields, &wild, &flora, &bare, 0, &sub);
        let best_river = (0..CHANNELS).max_by_key(|&i| river[i].to_bits()).unwrap();
        assert_eq!(CHANNEL_NAMES[best_river], "fish");

        // Fertile tile with established fields: cultivation must win there.
        let farmed = TileEconomy {
            establishment: Quantity::ONE,
            ..TileEconomy::default()
        };
        let fertile = marginal(&fields, &wild, &flora, &farmed, 1, &sub);
        let best_fertile = (0..CHANNELS).max_by_key(|&i| fertile[i].to_bits()).unwrap();
        assert_eq!(CHANNEL_NAMES[best_fertile], "cultivate");

        // The same fertile tile unestablished must NOT rank cultivation first —
        // agrarianism is an investment, not a default.
        let unfarmed = marginal(&fields, &wild, &flora, &bare, 1, &sub);
        let best_unfarmed = (0..CHANNELS)
            .max_by_key(|&i| unfarmed[i].to_bits())
            .unwrap();
        assert_ne!(CHANNEL_NAMES[best_unfarmed], "cultivate");
    }
}
