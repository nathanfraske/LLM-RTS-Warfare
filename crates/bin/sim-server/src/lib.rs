//! Composes the headless deterministic simulation loop (docs/01-architecture.md):
//! genesis (fields → flora → nations on tiles), then the tick loop with the
//! directive log as the only external input (docs/14-bands-and-councils.md).

use std::path::PathBuf;

use cohorts::CohortDrive;
use directive_schema::DirectiveEntry;
use flora::FloraMap;
use nations::WorldNations;
use sim_clock::{SimClock, is_month_boundary};
use sim_events::{Event, EventLog, WorldSeed};
use world_map::{WorldFields, tiles};
use world_schema::{Quantity, Tick};

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
pub fn genesis(seed: WorldSeed, map_size: u32) -> Genesis {
    let fields = WorldFields::generate(seed, map_size);
    let flora = flora::settle::settle(seed, &fields, flora::DEFAULT_SPECIES);
    Genesis { fields, flora }
}

#[derive(Debug)]
pub struct RunReport {
    pub hash: String,
    pub events: usize,
    pub population: Quantity,
}

const BASE_BIRTH: f64 = 0.006;
const BASE_DEATH: f64 = 0.0045;

/// A live, steppable world: the same composition `run_world` uses, exposed
/// tick-by-tick so the viewer (and later the server loop) can drive it.
#[derive(Debug)]
pub struct World {
    pub seed: WorldSeed,
    pub genesis: Genesis,
    pub nations: WorldNations,
    pub cohorts: cohorts::Cohorts,
    pub log: EventLog,
    pub table: &'static [species::Species],
    clock: SimClock,
    entries: Vec<DirectiveEntry>,
    next_entry: usize,
}

impl World {
    /// Genesis + nation spawn + founders + any tick-0 directives.
    #[must_use]
    pub fn new(config: &RunConfig) -> Self {
        let seed = WorldSeed(config.seed);
        let genesis = genesis(seed, config.map_size);
        let table = species::archetypes();
        let mut log = EventLog::new();
        let nations = nations::spawn(&genesis.fields, table, config.nations, &mut log);
        let all_cohorts = nations::found_cohorts(seed, &nations);
        let cells = genesis.fields.grid().cells();
        log.push(Event::WorldGenerated {
            land_tiles: genesis.fields.land_cells(),
            habitable_tiles: u32::try_from(
                (0..cells)
                    .filter(|&t| tiles::habitable(&genesis.fields, t))
                    .count(),
            )
            .expect("count fits u32"),
            flora_species: u16::try_from(genesis.flora.species.len())
                .expect("species count fits u16"),
            cohorts: u32::try_from(all_cohorts.len()).expect("cohort count fits u32"),
            population: all_cohorts.total_population(),
        });
        let mut entries = config.directives.clone();
        entries.sort_by_key(|e| e.tick); // stable: same-tick entries keep input order
        let mut world = Self {
            seed,
            genesis,
            nations,
            cohorts: all_cohorts,
            log,
            table,
            clock: SimClock::new(),
            entries,
            next_entry: 0,
        };
        world.apply_due(Tick::ZERO);
        world
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Advance exactly one tick: due directives, then monthly systems.
    pub fn step(&mut self) {
        let tick = self.clock.advance();
        self.apply_due(tick);
        if is_month_boundary(tick) {
            for nation in &mut self.nations.nations {
                nations::mandate::tick_month(&mut nation.mandate, &mut nation.autonomy);
            }
            {
                let WorldNations { works, owner, .. } = &mut self.nations;
                works.tick_month(owner, tick, &mut self.log);
            }
            let drives: Vec<CohortDrive> = self
                .cohorts
                .entries()
                .map(|(key, _)| {
                    let s = &self.table[key.species.0 as usize];
                    let works = &self.nations.works;
                    CohortDrive {
                        birth_rate: Quantity::from_num(BASE_BIRTH)
                            * species::milli(s.birth_mod_milli)
                            * works.birth_mult(key.tile.0),
                        death_rate: Quantity::from_num(BASE_DEATH)
                            * species::milli(s.death_mod_milli),
                        capacity: nations::capacity(
                            &self.genesis.fields,
                            key.tile.0 as usize,
                            s,
                            works,
                        ),
                        famine_threshold: works.famine_threshold(key.tile.0),
                    }
                })
                .collect();
            let delta = self.cohorts.tick_month(self.seed, tick, &drives);
            for key in &delta.famines {
                self.log.push(Event::Famine {
                    tick,
                    tile: key.tile,
                    species: key.species,
                });
            }
            nations::autopilot::tick_month(
                self.seed,
                tick,
                &mut self.nations,
                &self.genesis.fields,
                self.table,
                &mut self.cohorts,
                &mut self.log,
            );
            self.log.push(Event::MonthClosed {
                tick,
                births: delta.births,
                deaths: delta.deaths,
                population: self.cohorts.total_population(),
            });
        }
    }

    fn apply_due(&mut self, tick: Tick) {
        while self.next_entry < self.entries.len() && self.entries[self.next_entry].tick <= tick.0 {
            nations::directives::apply(
                &self.entries[self.next_entry],
                &mut self.nations,
                &self.genesis.fields,
                &mut self.log,
            );
            self.next_entry += 1;
        }
    }

    /// Write the council + world reports for the current tick.
    pub fn write_reports(&self, dir: &std::path::Path) {
        let now = self.tick();
        std::fs::create_dir_all(dir).expect("create report directory");
        for nation in &self.nations.nations {
            let report = readouts::nation_report(
                nation.id,
                &self.nations,
                &self.genesis.fields,
                self.table,
                &self.cohorts,
                &self.log,
                now,
            );
            std::fs::write(dir.join(format!("nation-{}.md", nation.id.0)), report)
                .expect("write nation report");
        }
        let world_summary = readouts::world_report(
            &self.nations,
            &self.genesis.fields,
            self.table,
            &self.cohorts,
            now,
        );
        std::fs::write(dir.join("world.md"), world_summary).expect("write world report");
    }
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
