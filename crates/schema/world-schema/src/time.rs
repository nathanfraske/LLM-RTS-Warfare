//! The simulation tick — pure data; calendar math lives in `sim-clock`.

use serde::{Deserialize, Serialize};

/// One fixed timestep. Working proposal (docs/01-architecture.md): 1 tick = 1 sim-hour.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Tick(pub u64);

impl Tick {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}
