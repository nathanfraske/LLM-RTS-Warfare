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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Directive {
    /// Christen the nation — names flow into reports and the world log.
    Name { name: String },
    /// Set the expansion posture.
    SetStance { stance: Stance },
    /// Decree settlement of a specific frontier province (must border territory).
    Settle { province: u32 },
}

/// One logged council decision: apply `directive` to `nation` at `tick`.
/// A JSON array of these is the overseer input stream (replay input).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectiveEntry {
    pub tick: u64,
    pub nation: u32,
    pub directive: Directive,
}
