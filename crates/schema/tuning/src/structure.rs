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
    /// Buildings a tile can carry, standing or rising.
    pub max_per_tile: u8,
    /// Stores at this share of capacity move the people to raise room,
    /// per mille.
    pub initiative_store_permille: u16,
    /// People enough to raise a roof on their own.
    pub initiative_pop: u16,
    /// Establishment share that asks for worked ground, per mille.
    pub initiative_establish_permille: u16,
    /// Path cells per destination that count as a fair walk — layouts
    /// shorter than this reward, longer punish.
    pub path_norm_cells: u16,
    /// Labor won or lost at the extremes of layout quality, per mille.
    pub planning_gain_permille: u16,
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
            max_per_tile: 3,
            initiative_store_permille: 800,
            initiative_pop: 150,
            initiative_establish_permille: 400,
            path_norm_cells: 44,
            planning_gain_permille: 200,
        }
    }
}
