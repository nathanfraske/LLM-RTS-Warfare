//! Core identifier, time, and quantity types shared across every layer.

pub mod ids;
pub mod quantity;
pub mod time;

pub use ids::{CohortId, FloraId, NationId, ProvinceId, SpeciesId};
pub use quantity::Quantity;
pub use time::Tick;
