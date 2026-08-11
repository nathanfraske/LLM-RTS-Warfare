//! Mandate-priced structures (docs/30): commissioning a *function* raises
//! a building derived from the tile's own ground; it builds over months,
//! its effects flow from its walls, and duplicates are rejected in-world.

use directive_schema::{Directive, DirectiveEntry};
use policy::PolicyValue;
use sim_server::{RunConfig, World};
use world_schema::Quantity;

#[test]
fn commissioned_structures_derive_from_their_ground_and_work() {
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
        directive: Directive::Enact {
            action: "works.commission".into(),
            target: Some(seat.0),
            params: [("work".to_string(), PolicyValue::Text("field-works".into()))]
                .into_iter()
                .collect(),
        },
    };
    // The second, duplicate commission must be rejected, not doubled.
    config.directives = vec![commission(720), commission(1440)];

    let mut world = World::new(&config);
    assert_eq!(
        world
            .nations
            .works
            .cultivation_mult(seat.0, &world.tuning.structures),
        Quantity::ONE,
        "nothing built yet"
    );

    for _ in 0..(720 * 12) {
        world.step();
    }

    assert!(
        world
            .nations
            .works
            .cultivation_mult(seat.0, &world.tuning.structures)
            > Quantity::ONE,
        "the field-works multiply cultivation"
    );
    let standing = world.nations.works.completed(seat.0);
    assert_eq!(standing.len(), 1, "one commission, one building");
    assert!(
        standing[0].design.name.contains("field-works"),
        "the building is named for what it is: {}",
        standing[0].design.name
    );
    assert!(
        standing[0].integrity > 0,
        "a standing building has integrity left"
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
    assert!(
        world.nations.nations[0].autonomy > Quantity::ZERO,
        "direct rule leaves friction behind"
    );
}
