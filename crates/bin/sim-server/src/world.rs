//! The live, steppable world: the composition `run_world` drives to
//! completion and the viewer drives frame by frame. Owns every system's
//! state and the strict monthly order: upkeep, wild world, harvest,
//! demography, movement, ledger close.

use climate::Climate;
use cohorts::CohortDrive;
use directive_schema::DirectiveEntry;
use economy::Economy;
use fauna::Fauna;
use knowledge::WorldKnowledge;
use nations::WorldNations;
use policy::Registry;
use regolith::Regolith;
use sim_clock::{SimClock, is_month_boundary};
use sim_events::{Event, EventLog, WorldSeed};
use tuning::Tuning;
use world_map::tiles;
use world_schema::{Quantity, Tick};

use crate::{Genesis, RunConfig, genesis};

/// A live, steppable world: the same composition `run_world` uses, exposed
/// tick-by-tick so the viewer (and later the server loop) can drive it.
#[derive(Debug)]
pub struct World {
    pub seed: WorldSeed,
    pub genesis: Genesis,
    pub nations: WorldNations,
    pub cohorts: cohorts::Cohorts,
    /// The wild kingdoms (docs/19-ecology-and-subsistence.md).
    pub fauna: Fauna,
    /// Living vegetation — genesis density is the regrowth baseline.
    pub flora_live: Vec<u8>,
    /// The sky and the snow: seasonal forcing and the water cycle (docs/26).
    pub climate: Climate,
    /// The ground itself: composition, weathering, wash (docs/27).
    pub regolith: Regolith,
    pub economy: Economy,
    pub log: EventLog,
    pub table: &'static [species::Species],
    pub tuning: Tuning,
    /// Every lever and action alive in this world (docs/20-open-directives.md).
    pub registry: Registry,
    /// What each nation has actually seen, and the parties afield (docs/22).
    pub knowledge: WorldKnowledge,
    clock: SimClock,
    entries: Vec<DirectiveEntry>,
    next_entry: usize,
}

impl World {
    /// Genesis + nation spawn + founders + any tick-0 directives.
    #[must_use]
    pub fn new(config: &RunConfig) -> Self {
        let seed = WorldSeed(config.seed);
        let genesis = genesis(
            seed,
            config.map_size,
            config.tuning.ecology.flora_species,
            &config.tuning.deep,
        );
        let table = species::archetypes();
        let mut log = EventLog::new();
        let tuning = config.tuning.clone();
        let registry = crate::registry::assemble(&tuning.society);
        let nations = nations::spawn(
            &genesis.fields,
            table,
            config.nations,
            &mut log,
            &registry,
            &tuning.society,
        );
        let all_cohorts = nations::found_cohorts(seed, &nations, &tuning.society);
        let flora_live = genesis.flora.density.clone();
        let sky = Climate::genesis(&genesis.fields, &tuning.weather, &tuning.seasons);
        let ground = Regolith::genesis(&genesis.fields, &genesis.flora.density, &tuning.ground);
        let wild = Fauna::genesis(
            seed,
            &genesis.fields,
            &flora_live,
            &tuning.ecology,
            &tuning.bodies,
        );
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
            fauna_species: u16::try_from(wild.species.len()).expect("species count fits u16"),
            cohorts: u32::try_from(all_cohorts.len()).expect("cohort count fits u32"),
            population: all_cohorts.total_population(),
        });
        let knowledge = WorldKnowledge::new(cells, nations.nations.iter().map(|n| n.id));
        let mut entries = config.directives.clone();
        entries.sort_by_key(|e| e.tick); // stable: same-tick entries keep input order
        let mut world = Self {
            seed,
            genesis,
            nations,
            cohorts: all_cohorts,
            fauna: wild,
            flora_live,
            climate: sky,
            regolith: ground,
            economy: Economy::default(),
            log,
            table,
            tuning,
            registry,
            knowledge,
            clock: SimClock::new(),
            entries,
            next_entry: 0,
        };
        // The world starts dark: each nation knows its seat and surroundings.
        world.refresh_home_knowledge(Tick::ZERO);
        world.apply_due(Tick::ZERO);
        world
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Advance exactly one tick: due directives, parties afield, then
    /// monthly systems.
    pub fn step(&mut self) {
        let tick = self.clock.advance();
        self.apply_due(tick);
        self.step_scouts(tick);
        if is_month_boundary(tick) {
            self.close_month(tick);
        }
    }

    /// The month in order: upkeep, the wild world, the harvest, people,
    /// movement, ledger close.
    fn close_month(&mut self, tick: Tick) {
        self.upkeep(tick);
        let food = self.harvest(tick);
        let delta = self.demography(tick, &food);
        self.movement(tick, &food);
        self.refresh_home_knowledge(tick);
        self.log.push(Event::MonthClosed {
            tick,
            births: delta.births,
            deaths: delta.deaths,
            population: self.cohorts.total_population(),
        });
    }

    /// Mandate regen, construction progress, and the living world breathing.
    fn upkeep(&mut self, tick: Tick) {
        for nation in &mut self.nations.nations {
            nations::mandate::tick_month(
                &mut nation.mandate,
                &mut nation.autonomy,
                &self.tuning.society,
            );
        }
        {
            let WorldNations { works, owner, .. } = &mut self.nations;
            works.tick_month(owner, tick, &mut self.log);
        }
        self.breathe(tick);
    }

    /// People extract, eat, and store; hunger becomes famine events.
    fn harvest(&mut self, tick: Tick) -> economy::MonthFood {
        let food = self.economy.tick_month(
            &mut self.nations,
            &self.genesis.fields,
            &mut self.fauna,
            &mut self.flora_live,
            &self.climate,
            &mut self.regolith,
            &self.cohorts,
            &self.tuning,
        );
        for t in &food.famines {
            if let Some(owner) = self.nations.owner[t.0 as usize] {
                let species = self
                    .nations
                    .nations
                    .iter()
                    .find(|n| n.id == owner)
                    .map(|n| n.species);
                if let Some(species) = species {
                    self.log.push(Event::Famine {
                        tick,
                        tile: *t,
                        species,
                    });
                }
            }
        }
        food
    }

    /// Nutrition drives births and deaths.
    fn demography(&mut self, tick: Tick, food: &economy::MonthFood) -> cohorts::MonthDelta {
        let drives: Vec<CohortDrive> = self
            .cohorts
            .entries()
            .map(|(key, _)| {
                let s = &self.table[key.species.0 as usize];
                let works = &self.nations.works;
                let soc = &self.tuning.society;
                CohortDrive {
                    birth_rate: Quantity::from_num(soc.base_birth)
                        * species::milli(s.birth_mod_milli)
                        * works.birth_mult(key.tile.0, soc),
                    death_rate: Quantity::from_num(soc.base_death)
                        * species::milli(s.death_mod_milli),
                    nutrition: food
                        .nutrition
                        .get(&key.tile.0)
                        .copied()
                        .unwrap_or(Quantity::ONE),
                }
            })
            .collect();
        self.cohorts
            .tick_month(self.seed, tick, &drives, &self.tuning.society)
    }

    /// Splits and starvation relocations; stores and herds travel with movers.
    fn movement(&mut self, tick: Tick, food: &economy::MonthFood) {
        let watermark = self.log.len();
        {
            let fields = &self.genesis.fields;
            let wild = &self.fauna;
            let flora_live = &self.flora_live;
            let sky = &self.climate;
            let ground = &self.regolith;
            let sub = &self.tuning.subsistence;
            let wx = &self.tuning.weather;
            let potential =
                |t: usize| economy::potential(fields, wild, flora_live, sky, ground, t, sub, wx);
            nations::autopilot::tick_month(
                tick,
                self.seed,
                &mut self.nations,
                fields,
                self.table,
                &mut self.cohorts,
                &mut self.knowledge,
                &mut self.log,
                &potential,
                &food.starving_moves,
                &food.hungry,
                &self.tuning.society,
                &self.tuning.exploration,
            );
        }
        let moves: Vec<(world_schema::TileId, world_schema::TileId)> = self
            .log
            .iter()
            .skip(watermark)
            .filter_map(|e| match e {
                Event::BandMoved { from, to, .. } => Some((*from, *to)),
                _ => None,
            })
            .collect();
        for (from, to) in moves {
            self.economy.relocate(from, to);
        }
    }

    fn apply_due(&mut self, tick: Tick) {
        while self.next_entry < self.entries.len() && self.entries[self.next_entry].tick <= tick.0 {
            nations::directives::apply(
                &self.entries[self.next_entry],
                &mut self.nations,
                &self.genesis.fields,
                &self.registry,
                &mut self.knowledge,
                &mut self.log,
                &self.tuning.society,
                &self.tuning.exploration,
            );
            self.next_entry += 1;
        }
    }
}
