//! Council report IO: rendering every nation's fogged report and the
//! omniscient world summary to disk at the end of a run.

use crate::World;

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
