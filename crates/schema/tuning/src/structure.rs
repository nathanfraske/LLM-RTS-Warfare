//! Structures as composition (docs/30): build-time scaling, effect
//! scaling, and what calamities cost the built world.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Structures {
    /// Months every raising takes before materials weigh in.
    pub base_months: u8,
    /// Extra month per this much wall mass.
    pub mass_months_divisor: u16,
    /// Extra month per this much wall hardness.
    pub hardness_months_divisor: u16,
    /// Cultivation multiplier at full field-works effect, per mille bonus.
    pub field_mult_permille: u16,
    /// Store capacity at full store-house effect, food units.
    pub store_capacity: u16,
    /// Birth multiplier at full hearth-hall effect, per mille bonus.
    pub shelter_permille: u16,
    /// Integrity a shake spends at full quake strength.
    pub quake_damage: u16,
    /// Integrity a burning month spends on structures that can burn.
    pub fire_scorch: u16,
    /// Integrity heavy ash spends on light roofs.
    pub ash_load: u16,
    /// Roof mass below which ash and fire find easy purchase.
    pub light_roof_mass: u16,
}

impl Default for Structures {
    fn default() -> Self {
        Self {
            base_months: 3,
            mass_months_divisor: 260,
            hardness_months_divisor: 320,
            field_mult_permille: 450,
            store_capacity: 1200,
            shelter_permille: 160,
            quake_damage: 420,
            fire_scorch: 350,
            ash_load: 260,
            light_roof_mass: 300,
        }
    }
}
