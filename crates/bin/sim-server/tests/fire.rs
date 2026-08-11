//! The fire below is live (docs/29 §3): vents erupt on their clocks, lava
//! runs the drainage tree, and the world both records and survives it.

use sim_server::{RunConfig, World};

#[test]
fn volcanoes_erupt_and_the_world_carries_on() {
    let mut config = RunConfig {
        seed: 42,
        ticks: 0,
        map_size: 96,
        nations: 4,
        ..RunConfig::default()
    };
    // Impatient mountains for the test: every vent fires within two years.
    config.tuning.deep.eruption_min_months = 6;
    config.tuning.deep.eruption_max_months = 24;

    let mut world = World::new(&config);
    for _ in 0..(720 * 36) {
        world.step();
    }
    let eruptions: Vec<(u32, u32)> = world
        .log
        .iter()
        .filter_map(|e| match e {
            sim_events::Event::VolcanoErupted {
                tile,
                reach,
                ash_tiles,
                ..
            } => Some((tile.0, *reach + *ash_tiles)),
            _ => None,
        })
        .collect();
    assert!(
        !eruptions.is_empty(),
        "impatient vents must fire — schedules: {:?}, vents: {}",
        world.genesis.geology.schedules,
        world
            .genesis
            .geology
            .vents
            .iter()
            .filter(|&&v| v > 0)
            .count()
    );
    let (vent, touched) = eruptions[0];
    assert!(
        touched >= 4,
        "lava and ash together must touch real country, got {touched}"
    );
    assert!(
        world.regolith.rock[vent as usize] > 200,
        "fresh rock buries the vent's ground"
    );
    assert!(
        world.cohorts.total_population() > world_schema::Quantity::ZERO,
        "the world survives its mountains"
    );

    // Same seed, same fire: replay determinism holds through eruptions.
    let mut again = World::new(&config);
    for _ in 0..(720 * 36) {
        again.step();
    }
    assert_eq!(world.log.hash(), again.log.hash());
}
