//! Every sim-behavior number, in one navigable place (docs/01-architecture.md
//! §6 schema-first; demanded by docs/19 depth work). Systems receive their
//! domain struct by reference; nothing re-declares a tunable locally.
//!
//! Values are plain numbers (converted to fixed-point at use sites, which is
//! deterministic), so a world can later load a RON/JSON tuning file with one
//! `serde` call — tuning is world configuration, part of replay input.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Tuning {
    pub ecology: Ecology,
    pub subsistence: Subsistence,
    pub society: Society,
}

/// The wild world: flora regrowth, fauna growth, trophic pressure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Ecology {
    /// Grazer carrying capacity on a fully vegetated, perfectly fit tile.
    pub grazer_k_full: f64,
    /// Predator habitat ceiling at perfect fit.
    pub predator_habitat_k: f64,
    /// Predator K as a fraction of local prey biomass.
    pub predator_k_prey_frac: f64,
    /// Monthly predator food demand as a fraction of predator biomass.
    pub predator_demand_frac: f64,
    /// At most this share of prey can be eaten in one month.
    pub predation_max_frac: f64,
    /// Aquatic K by water body at perfect fit.
    pub aquatic_k_river: f64,
    pub aquatic_k_lake: f64,
    pub aquatic_k_ocean: f64,
    /// Die-back keep-fraction when habitat has collapsed.
    pub collapse_keep: f64,
    /// Vegetation points a full grazer load wears off per month (cap 8).
    pub grazing_pressure: f64,
    /// Vegetation never wears below this.
    pub flora_floor: u8,
    /// Share of an overcrowded stock that migrates to a better neighbor.
    pub diffusion_frac: f64,
    /// No harvest strips a wild stock below this refuge share in a month.
    pub refuge_frac: f64,
    /// Fraction of K that populations seed at genesis.
    pub genesis_fill: f64,
    /// Monthly regrowth: gap-to-baseline divided by this (min 1 point).
    pub regrow_divisor: u8,
}

impl Default for Ecology {
    fn default() -> Self {
        Self {
            grazer_k_full: 380.0,
            predator_habitat_k: 60.0,
            predator_k_prey_frac: 0.125,
            predator_demand_frac: 0.333,
            predation_max_frac: 0.2,
            aquatic_k_river: 240.0,
            aquatic_k_lake: 320.0,
            aquatic_k_ocean: 180.0,
            collapse_keep: 0.7,
            grazing_pressure: 3.0,
            flora_floor: 10,
            diffusion_frac: 0.0625,
            refuge_frac: 0.25,
            genesis_fill: 0.5,
            regrow_divisor: 10,
        }
    }
}

/// Feeding a people: channel efficiencies, investment rates, stores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Subsistence {
    /// Food per worker-month at full input, per channel.
    pub gather_eff: f64,
    pub hunt_eff: f64,
    pub fish_eff: f64,
    pub cultivate_eff: f64,
    /// Food per head of herd per month.
    pub herd_yield_per_head: f64,
    /// Herd head captured per hunting-worker-month of capture effort.
    pub capture_eff: f64,
    /// Monthly herd growth rate at full pasture, and head per pasture unit.
    pub herd_growth: f64,
    pub herd_cap_per_pasture: f64,
    /// Field establishment: build rate per full labor share, monthly decay.
    pub establish_rate: f64,
    pub establish_decay: f64,
    /// Vegetation wear divisors (higher = gentler).
    pub gather_wear_divisor: f64,
    pub herd_wear_divisor: f64,
    /// Storage caps and monthly keep-fraction (spoilage takes the rest).
    pub store_base: f64,
    pub store_granary: f64,
    pub store_keep: f64,
    /// Food needed per person per month.
    pub food_per_head: f64,
    /// Below this nutrition, a month counts as famine.
    pub famine_nutrition: f64,
    /// Famine months in a row before a band looks for somewhere better.
    pub hunger_streak_to_move: u8,
    /// Parts-per-thousand the labor autopilot shifts per month.
    pub autopilot_step: u16,
    /// Marginal-estimate normalizers (stock levels that read as "plenty").
    pub hunt_stock_norm: f64,
    pub fish_stock_norm: f64,
    pub pasture_norm: f64,
    pub herd_prospect_eff: f64,
    /// Cultivation marginal shows at least this establishment (bootstrap
    /// visibility — docs/19 §4: visible, not favored).
    pub establish_floor: f64,
}

impl Default for Subsistence {
    fn default() -> Self {
        Self {
            gather_eff: 2.2,
            hunt_eff: 2.5,
            fish_eff: 2.0,
            cultivate_eff: 2.6,
            herd_yield_per_head: 0.4,
            capture_eff: 0.15,
            herd_growth: 0.05,
            herd_cap_per_pasture: 300.0,
            establish_rate: 0.05,
            establish_decay: 0.008,
            gather_wear_divisor: 40.0,
            herd_wear_divisor: 60.0,
            store_base: 500.0,
            store_granary: 1500.0,
            store_keep: 0.94,
            food_per_head: 1.0,
            famine_nutrition: 0.75,
            hunger_streak_to_move: 3,
            autopilot_step: 60,
            hunt_stock_norm: 300.0,
            fish_stock_norm: 250.0,
            pasture_norm: 150.0,
            herd_prospect_eff: 1.8,
            establish_floor: 0.25,
        }
    }
}

/// Governance and demographics: mandate, movement, births and deaths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Society {
    /// Mandate pool (docs/16).
    pub mandate_cap: f64,
    pub starting_mandate: f64,
    pub mandate_regen: f64,
    pub autonomy_per_spend: f64,
    pub autonomy_cap: f64,
    /// Cost multiplier = 1 + autonomy / this.
    pub autonomy_cost_divisor: f64,
    /// Regen multiplier = 1 - autonomy / this.
    pub autonomy_regen_divisor: f64,
    pub autonomy_decay_keep: f64,
    /// Directive base costs.
    pub cost_stance: f64,
    pub cost_settle: f64,
    pub cost_commission: f64,
    pub cost_labor: f64,
    /// Works: build months and effects.
    pub farmstead_months: u8,
    pub granary_months: u8,
    pub dwellings_months: u8,
    pub farmstead_cultivation_mult: f64,
    pub dwellings_birth_mult: f64,
    /// Band splitting and movement.
    pub split_base_pop: f64,
    pub stance_consolidate_mult: f64,
    pub stance_steady_mult: f64,
    pub stance_expansive_mult: f64,
    pub split_min_pop: f64,
    pub settlers_frac: f64,
    pub settlers_min: f64,
    pub split_potential_floor: f64,
    /// A starving band moves only for at least this yield ratio.
    pub relocate_gain: f64,
    /// Demographics.
    pub base_birth: f64,
    pub base_death: f64,
    pub birth_factor_min: f64,
    pub birth_factor_max: f64,
    /// Death factor = `clamp(death_offset - nutrition, min, max)`.
    pub death_offset: f64,
    pub death_factor_min: f64,
    pub death_factor_max: f64,
    pub famine_nutrition: f64,
    /// Founder band size: base + roll in [0, spread).
    pub founder_base: i64,
    pub founder_spread: u64,
    /// Default labor weights at spawn (per mille).
    pub spawn_labor: [u16; 5],
}

impl Default for Society {
    fn default() -> Self {
        Self {
            mandate_cap: 10.0,
            starting_mandate: 6.0,
            mandate_regen: 1.2,
            autonomy_per_spend: 6.0,
            autonomy_cap: 100.0,
            autonomy_cost_divisor: 60.0,
            autonomy_regen_divisor: 200.0,
            autonomy_decay_keep: 0.95,
            cost_stance: 1.0,
            cost_settle: 2.0,
            cost_commission: 3.0,
            cost_labor: 1.0,
            farmstead_months: 6,
            granary_months: 8,
            dwellings_months: 5,
            farmstead_cultivation_mult: 1.35,
            dwellings_birth_mult: 1.12,
            split_base_pop: 220.0,
            stance_consolidate_mult: 1.5,
            stance_steady_mult: 1.0,
            stance_expansive_mult: 0.7,
            split_min_pop: 120.0,
            settlers_frac: 0.4,
            settlers_min: 60.0,
            split_potential_floor: 0.4,
            relocate_gain: 1.25,
            base_birth: 0.006,
            base_death: 0.0045,
            birth_factor_min: 0.35,
            birth_factor_max: 1.1,
            death_offset: 1.9,
            death_factor_min: 0.9,
            death_factor_max: 1.9,
            famine_nutrition: 0.75,
            founder_base: 140,
            founder_spread: 160,
            spawn_labor: [350, 350, 200, 50, 50],
        }
    }
}
