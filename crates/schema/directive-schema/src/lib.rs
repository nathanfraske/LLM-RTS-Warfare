//! Typed directives — the only way an overseer steers a nation
//! (docs/04-institutions-directives.md; applied from the logged input stream).

use serde::{Deserialize, Serialize};

/// Expansion posture, interpreted by the band autopilot (docs/14-bands-and-councils.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stance {
    /// Stay put; split only under real pressure.
    Consolidate,
    /// Default: split when a settlement grows crowded.
    Steady,
    /// Push the frontier early and often.
    Expansive,
}

/// A commissionable project (docs/16-mandate-and-works.md). Effects live in
/// the sim; this is the policy vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkKind {
    Farmstead,
    Granary,
    Dwellings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Directive {
    /// Christen the nation — names flow into reports and the world log. Free.
    Name { name: String },
    /// Set the expansion posture. Costs mandate.
    SetStance { stance: Stance },
    /// Decree settlement of a specific frontier tile (must border territory).
    /// Costs mandate.
    Settle { tile: u32 },
    /// Commission a work on an owned tile; institutions build it over months.
    /// Costs mandate.
    Commission { tile: u32, work: WorkKind },
    /// Direct the nation's labor across the five subsistence channels
    /// (gather, hunt, fish, cultivate, herd), in parts-per-thousand.
    /// Normalized server-side; overrides the return-following autopilot.
    /// Costs mandate.
    SetLabor {
        gather: u16,
        hunt: u16,
        fish: u16,
        cultivate: u16,
        herd: u16,
    },
}

/// One logged council decision: apply `directive` to `nation` at `tick`.
/// A JSON array of these is the overseer input stream (replay input).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectiveEntry {
    pub tick: u64,
    pub nation: u32,
    pub directive: Directive,
}
