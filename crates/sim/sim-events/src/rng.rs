//! Counter-based deterministic RNG (docs/01-architecture.md).
//!
//! No shared mutable streams: every draw is a pure function of
//! `(world seed, tick, system, key)`, so it is parallel-safe, replay-safe,
//! and order-independent within a tick. SplitMix64-style mixing —
//! statistical quality only, deliberately not cryptographic.

use world_schema::{Quantity, Tick};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldSeed(pub u64);

/// Stable identifier for the drawing system. Each sim crate owns its constant;
/// uniqueness is by convention until an xtask gate registers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemId(pub u16);

/// A uniform `u64` for this exact `(seed, tick, system, key)` coordinate.
#[must_use]
pub fn draw(seed: WorldSeed, tick: Tick, system: SystemId, key: u64) -> u64 {
    let mut x = seed
        .0
        .wrapping_add(tick.0.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(u64::from(system.0) << 48)
        .wrapping_add(key.wrapping_mul(0xD1B5_4A32_D192_ED03));
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A fixed-point draw in `[0, 1)` from the same coordinate space.
#[must_use]
pub fn unit(seed: WorldSeed, tick: Tick, system: SystemId, key: u64) -> Quantity {
    let fractional = draw(seed, tick, system, key) & 0xFFFF;
    Quantity::from_bits(i64::try_from(fractional).expect("16 bits always fit"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_pure_and_coordinate_sensitive() {
        let s = WorldSeed(42);
        let t = Tick(100);
        let sys = SystemId(1);
        assert_eq!(draw(s, t, sys, 7), draw(s, t, sys, 7));
        assert_ne!(draw(s, t, sys, 7), draw(s, t, sys, 8));
        assert_ne!(draw(s, t, sys, 7), draw(s, Tick(101), sys, 7));
        assert_ne!(draw(s, t, sys, 7), draw(WorldSeed(43), t, sys, 7));
        assert_ne!(draw(s, t, sys, 7), draw(s, t, SystemId(2), 7));
    }

    #[test]
    fn unit_stays_in_range() {
        for key in 0..1000 {
            let q = unit(WorldSeed(1), Tick(1), SystemId(1), key);
            assert!(q >= Quantity::ZERO && q < Quantity::ONE);
        }
    }
}
