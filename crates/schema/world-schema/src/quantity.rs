//! The fixed-point quantity type used for all authoritative sim math.
//!
//! Authoritative discrete state never uses floats (docs/01-architecture.md,
//! "Determinism and event sourcing"); `Quantity` is a Q48.16 fixed-point number.

/// Fixed-point scalar for populations, goods, wealth, and rates.
pub type Quantity = fixed::types::I48F16;

/// Convenience constructor from an integer count.
#[must_use]
pub fn qty(n: i64) -> Quantity {
    Quantity::from_num(n)
}
