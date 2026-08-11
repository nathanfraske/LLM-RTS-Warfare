//! Composes the headless deterministic simulation loop (docs/01-architecture.md):
//! genesis (fields → flora → nations on tiles), then the tick loop with the
//! directive log as the only external input (docs/14-bands-and-councils.md).

pub mod world;

use std::path::PathBuf;

use directive_schema::DirectiveEntry;
use flora::FloraMap;
use sim_events::WorldSeed;
use tuning::Tuning;
use world_map::WorldFields;
use world_schema::Quantity;

pub use world::World;

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub seed: u64,
    pub ticks: u64,
    /// World size in world tiles per side (docs/15-multiscale-maps.md).
    pub map_size: u32,
    pub nations: u32,
    /// The overseer input stream, applied at each entry's tick.
    pub directives: Vec<DirectiveEntry>,
    /// When set, council + world reports are written here at the end tick.
    pub report_dir: Option<PathBuf>,
    /// Every sim-behavior number (docs/01a; a tuning file can load into this).
    pub tuning: Tuning,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            ticks: sim_clock::TICKS_PER_YEAR,
            map_size: 192,
            nations: 4,
            directives: Vec::new(),
            report_dir: None,
            tuning: Tuning::default(),
        }
    }
}

/// Everything genesis produces, before the first tick.
#[derive(Debug)]
pub struct Genesis {
    pub fields: WorldFields,
    pub flora: FloraMap,
}

/// Dawn-of-time world creation (docs/13-worldgen.md): physical fields at
/// world-tile resolution, then flora settling. Tiles are the provinces.
#[must_use]
pub fn genesis(seed: WorldSeed, map_size: u32, flora_species: u16) -> Genesis {
    let fields = WorldFields::generate(seed, map_size);
    let flora = flora::settle::settle(seed, &fields, flora_species);
    Genesis { fields, flora }
}

#[derive(Debug)]
pub struct RunReport {
    pub hash: String,
    pub events: usize,
    pub population: Quantity,
}

/// Run a world to completion. Same config ⇒ same report, bit for bit.
#[must_use]
pub fn run_world(config: &RunConfig) -> RunReport {
    let mut world = World::new(config);
    for _ in 0..config.ticks {
        world.step();
    }
    if let Some(dir) = &config.report_dir {
        world.write_reports(dir);
    }
    RunReport {
        hash: world.log.hash(),
        events: world.log.len(),
        population: world.cohorts.total_population(),
    }
}
