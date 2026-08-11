//! The authored periodic table of the body (docs/23 §3): ten part roles.
//! This enum is the whole authored vocabulary — everything a body *is* gets
//! composed from these, and everything it *can do* is derived in `function`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Seat of coordination. Lose it, lose the creature.
    Core,
    /// Drives the carrier; small thin-blooded bodies can do without.
    Pump,
    /// Carries flow; a breach drains the carrier.
    Conduit,
    /// Turns food into life.
    Processor,
    /// Stores against lean hours.
    Reservoir,
    /// Perceives along a medium axis: mechanical … chemical … radiant.
    Sensor,
    /// The sensor axes mirrored outward: voice, scent, glow.
    Emitter,
    /// Moves the body along a medium axis: substrate … open water.
    Locomotor,
    /// Grasps and works.
    Manipulator,
    /// Integument: hide, plate, bark, stone.
    Shell,
}

/// One part instance in a plan. `medium_milli` means different axes per
/// role (locomotion medium vs. sense medium); `count` carries symmetry
/// (2 = a pair, 4 = two pairs …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub role: Role,
    pub medium_milli: u16,
    pub size_milli: u16,
    pub count: u8,
    pub armor_milli: u16,
}

/// The sense-medium word for describe lines.
#[must_use]
pub fn sense_word(medium_milli: u16) -> &'static str {
    match medium_milli {
        0..=333 => "touch and tremor",
        334..=666 => "scent and taste",
        _ => "light and heat",
    }
}

/// The voice-medium word for describe lines.
#[must_use]
pub fn emit_word(medium_milli: u16) -> &'static str {
    match medium_milli {
        0..=333 => "drums and calls",
        334..=666 => "scent-marks",
        _ => "glow and flare",
    }
}
