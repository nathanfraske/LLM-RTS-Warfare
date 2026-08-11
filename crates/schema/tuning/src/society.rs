//! Governance and demographics: mandate, movement, births, deaths.

use serde::{Deserialize, Serialize};

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
    pub cost_scout: f64,
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
            cost_scout: 1.0,
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
