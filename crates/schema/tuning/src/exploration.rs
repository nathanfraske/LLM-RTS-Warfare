//! Knowledge and discovery: scout parties and reasons to move (docs/22).

use serde::{Deserialize, Serialize};

/// Knowledge and discovery (docs/22): scout parties and the reasons to move.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Exploration {
    /// How many tiles out a scout party walks before turning home.
    pub scout_range: u16,
    /// Ticks (hours) a party needs to cross one world tile.
    pub scout_ticks_per_tile: u16,
    /// Per-mille chance, per hostile tile entered, that a party is lost.
    pub scout_loss_permille: u16,
    /// Species climate fit below which a tile counts as hostile to cross.
    pub hostile_fit: f64,
    /// Active parties a nation can field at once.
    pub max_parties: u8,
    /// Crowding fraction of the split threshold that justifies scouting.
    pub crowd_scout_frac: f64,
    /// Nutrition below this (fed ratio) justifies scouting for a way out.
    pub hungry_scout_nutrition: f64,
    /// Months between need-driven scout dispatches (impatience brake).
    pub scout_cooldown_months: u8,
    /// Extra walking cost per 100 units of climb to a tile, per mille.
    pub travel_slope_permille: u16,
    /// Extra walking cost in high country, per mille.
    pub travel_high_permille: u16,
    /// Elevation where the high-country cost begins.
    pub travel_high_elevation: i32,
    /// Extra walking cost under full snowpack, per mille.
    pub travel_snow_permille: u16,
}

impl Default for Exploration {
    fn default() -> Self {
        Self {
            scout_range: 6,
            scout_ticks_per_tile: 10,
            scout_loss_permille: 30,
            hostile_fit: 0.05,
            max_parties: 1,
            crowd_scout_frac: 0.8,
            hungry_scout_nutrition: 0.9,
            scout_cooldown_months: 4,
            travel_slope_permille: 90,
            travel_high_permille: 350,
            travel_high_elevation: 1_500,
            travel_snow_permille: 800,
        }
    }
}
