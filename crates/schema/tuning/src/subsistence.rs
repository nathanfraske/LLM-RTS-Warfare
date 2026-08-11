//! The five channels: efficiencies, stores, hunger (docs/19).

use serde::{Deserialize, Serialize};

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
