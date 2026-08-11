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
            params: [(
                "emphasis".to_string(),
                PolicyValue::Text("ground-working".into()),
            )]
            .into_iter()
            .collect(),
        },
    };
    // The fourth commission must overflow the tile and be rejected.
    config.directives = vec![
        commission(720),
        commission(1440),
        commission(2160),
        commission(2880),
    ];

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
    assert!(standing.len() <= 3, "the tile carries at most its load");
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
    assert!(rejected, "the overflowing commission must be rejected");
    assert!(
        world.nations.nations[0].autonomy > Quantity::ZERO,
        "direct rule leaves friction behind"
    );
}

#[test]
fn the_people_build_unbidden_unless_the_council_forbids_it() {
    let free = RunConfig {
        seed: 42,
        ticks: 0,
        map_size: 96,
        nations: 4,
        ..RunConfig::default()
    };
    let mut world = World::new(&free);
    for _ in 0..(720 * 144) {
        world.step();
    }
    let raised = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::PeopleRaised { .. }));
    assert!(raised, "twelve fat years must move someone to build");

    // The council claims the sole right to build: nobody may.
    let probe = World::new(&free);
    let first = probe.nations.nations[0].id.0;
    let mut forbidden = free.clone();
    forbidden.directives = probe
        .nations
        .nations
        .iter()
        .map(|n| DirectiveEntry {
            tick: 0,
            nation: n.id.0,
            directive: Directive::Set {
                key: "building.initiative".into(),
                value: PolicyValue::Text("council-only".into()),
            },
        })
        .collect();
    let _ = first;
    let mut world = World::new(&forbidden);
    for _ in 0..(720 * 144) {
        world.step();
    }
    let raised = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::PeopleRaised { .. }));
    assert!(!raised, "forbidden people build nothing unbidden");
}
