//! Every sim-behavior number, in one navigable place (docs/01-architecture.md
//! §6 schema-first; demanded by docs/19 depth work). Systems receive their
//! domain struct by reference; nothing re-declares a tunable locally.
//!
//! Values are plain numbers (converted to fixed-point at use sites, which is
//! deterministic), so a world can later load a RON/JSON tuning file with one
//! `serde` call — tuning is world configuration, part of replay input.

use serde::{Deserialize, Serialize};

mod bodies;
mod ecology;
mod exploration;
mod society;
mod subsistence;
mod weather;

pub use bodies::Bodies;
pub use ecology::Ecology;
pub use exploration::Exploration;
pub use society::Society;
pub use subsistence::Subsistence;
pub use weather::{Seasons, Sky, Weather};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Tuning {
    pub ecology: Ecology,
    pub subsistence: Subsistence,
    pub society: Society,
    pub exploration: Exploration,
    pub bodies: Bodies,
    pub seasons: Seasons,
    pub weather: Weather,
    pub sky: Sky,
}
