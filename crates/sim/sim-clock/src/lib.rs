//! Fixed-timestep calendar: tick advancement and boundary math.

use world_schema::Tick;

pub const TICKS_PER_DAY: u64 = 24;
pub const DAYS_PER_MONTH: u64 = 30;
pub const MONTHS_PER_YEAR: u64 = 12;
pub const TICKS_PER_MONTH: u64 = TICKS_PER_DAY * DAYS_PER_MONTH;
pub const TICKS_PER_YEAR: u64 = TICKS_PER_MONTH * MONTHS_PER_YEAR;

/// The authoritative simulation clock. Advances one tick at a time, never skips.
#[derive(Debug, Default)]
pub struct SimClock {
    tick: Tick,
}

impl SimClock {
    #[must_use]
    pub fn new() -> Self {
        Self { tick: Tick::ZERO }
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Advance one tick and return the new current tick.
    pub fn advance(&mut self) -> Tick {
        self.tick = self.tick.next();
        self.tick
    }
}

/// True on the tick that closes a month (monthly systems run here).
#[must_use]
pub fn is_month_boundary(tick: Tick) -> bool {
    tick.0 > 0 && tick.0.is_multiple_of(TICKS_PER_MONTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_boundaries_land_every_720_ticks() {
        let boundaries: Vec<u64> = (0..=TICKS_PER_YEAR)
            .filter(|t| is_month_boundary(Tick(*t)))
            .collect();
        assert_eq!(boundaries.len() as u64, MONTHS_PER_YEAR);
        assert_eq!(boundaries[0], TICKS_PER_MONTH);
        assert_eq!(*boundaries.last().unwrap(), TICKS_PER_YEAR);
    }
}
