//! Event-sourced log and the deterministic counter RNG (docs/01-architecture.md).

pub mod event;
pub mod log;
pub mod rng;

pub use event::Event;
pub use log::EventLog;
pub use rng::{SystemId, WorldSeed};
