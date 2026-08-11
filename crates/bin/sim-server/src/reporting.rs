//! Council report IO: rendering every nation's fogged report and the
//! omniscient world summary to disk at the end of a run.

use fauna::Fauna;
use sim_events::{Event, EventLog};
use world_map::tiles;

use crate::{Genesis, World};

/// The genesis ledger line: what the world was born holding.
pub(crate) fn log_genesis(
    genesis: &Genesis,
    wild: &Fauna,
    all_cohorts: &cohorts::Cohorts,
    log: &mut EventLog,
) {
    let cells = genesis.fields.grid().cells();
    log.push(Event::WorldGenerated {
        land_tiles: genesis.fields.land_cells(),
        habitable_tiles: u32::try_from(
            (0..cells)
                .filter(|&t| tiles::habitable(&genesis.fields, t))
                .count(),
        )
        .expect("count fits u32"),
        flora_species: u16::try_from(genesis.flora.species.len()).expect("species count fits u16"),
        fauna_species: u16::try_from(wild.species.len()).expect("species count fits u16"),
        cohorts: u32::try_from(all_cohorts.len()).expect("cohort count fits u32"),
        population: all_cohorts.total_population(),
    });
}

impl World {
    /// Write the council + world reports for the current tick. The
    /// directory is cleared first so a vanished nation can't leave a stale
    /// report behind to mislead its overseer.
    pub fn write_reports(&self, dir: &std::path::Path) {
        let now = self.tick();
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).expect("create report directory");
        for nation in &self.nations.nations {
            let report = readouts::nation_report(
                nation.id,
                &self.nations,
                &self.genesis.fields,
                &self.fauna,
                &self.flora_live,
                &self.climate,
                &self.regolith,
                &self.economy,
                self.table,
                &self.cohorts,
                &self.log,
                now,
                &self.registry,
                &self.knowledge,
                &self.tuning,
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
