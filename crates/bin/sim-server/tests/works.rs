//! Mandate-priced commissions: works build over months, apply their effects,
//! and directive spending is deterministic replay input
//! (docs/16-mandate-and-works.md).

use directive_schema::{Directive, DirectiveEntry, WorkKind};
use sim_server::{RunConfig, World};
use world_schema::Quantity;

#[test]
fn commissioned_works_complete_and_raise_capacity() {
    let mut config = RunConfig {
        seed: 42,
        ticks: 0,
        map_size: 96,
        nations: 4,
        ..RunConfig::default()
    };
    // A species can fail to spawn on a small world (no habitable fit) — use
    // the first nation that actually exists, not a hardcoded id.
    let probe = World::new(&config);
    let nation = probe.nations.nations[0].id;
    let seat = probe.nations.nations[0].seat;

    let commission = |tick: u64| DirectiveEntry {
        tick,
        nation: nation.0,
        directive: Directive::Commission {
            tile: seat.0,
            work: WorkKind::Farmstead,
        },
    };
    // The second, duplicate commission must be rejected, not doubled.
    config.directives = vec![commission(720), commission(1440)];

    let mut world = World::new(&config);
    let species = &world.table[world.nations.nations[0].species.0 as usize];
    let before = nations::capacity(
        &world.genesis.fields,
        seat.0 as usize,
        species,
        &world.nations.works,
    );

    for _ in 0..(720 * 8) {
        world.step();
    }

    let after = nations::capacity(
        &world.genesis.fields,
        seat.0 as usize,
        species,
        &world.nations.works,
    );
    assert_eq!(after, before * Quantity::from_num(1.35), "farmstead feeds");
    assert!(
        world.nations.nations[0].autonomy > Quantity::ZERO,
        "direct rule leaves friction behind"
    );
    let completed = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::WorkCompleted { .. }));
    let rejected = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::DirectiveRejected { .. }));
    assert!(completed, "completion must be a world event");
    assert!(rejected, "the duplicate commission must be rejected");
}
