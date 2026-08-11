//! The map is a memory (docs/22-knowledge-and-discovery.md): the world
//! starts dark, scouts must walk out and come home before anything joins
//! the map, and the report renders only what the nation has actually seen.

use directive_schema::{Directive, DirectiveEntry};
use policy::PolicyValue;
use sim_server::{RunConfig, World};

fn config_with(directives: Vec<DirectiveEntry>) -> RunConfig {
    RunConfig {
        seed: 42,
        ticks: 0,
        map_size: 96,
        nations: 4,
        directives,
        ..RunConfig::default()
    }
}

fn scout(nation: u32, bearing: &str, tick: u64) -> DirectiveEntry {
    DirectiveEntry {
        tick,
        nation,
        directive: Directive::Enact {
            action: "band.scout".into(),
            target: None,
            params: [("bearing".to_string(), PolicyValue::Text(bearing.into()))]
                .into_iter()
                .collect(),
        },
    }
}

#[test]
fn the_world_starts_dark() {
    let world = World::new(&config_with(Vec::new()));
    let nation = &world.nations.nations[0];
    let known = world.knowledge.of(nation.id).known_count();
    assert!(
        (1..=9).contains(&known),
        "at genesis a nation knows only its seat and surroundings, got {known}"
    );
    let report = readouts::nation_report(
        nation.id,
        &world.nations,
        &world.genesis.fields,
        &world.fauna,
        &world.flora_live,
        &world.climate,
        &world.regolith,
        &world.economy,
        world.table,
        &world.cohorts,
        &world.log,
        world.tick(),
        &world.registry,
        &world.knowledge,
        &world.tuning,
    );
    assert!(report.contains("Known lands"), "the frontier is now memory");
    assert!(
        report.contains("Beyond our maps"),
        "unwalked bearings are named, not enumerated"
    );
    assert!(report.contains("band.scout"), "the charter offers scouting");
}

#[test]
fn scouts_walk_out_and_carry_the_map_home() {
    let probe = World::new(&config_with(Vec::new()));
    let nation = probe.nations.nations[0].id;
    let before = probe.knowledge.of(nation).known_count();

    let mut world = World::new(&config_with(vec![scout(nation.0, "e", 0)]));
    for _ in 0..(720 * 3) {
        world.step();
    }
    let dispatched = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::ScoutDispatched { .. }));
    let returned = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::ScoutReturned { .. }));
    let lost = world
        .log
        .iter()
        .any(|e| matches!(e, sim_events::Event::ScoutLost { .. }));
    assert!(dispatched, "the order put a party on the road");
    assert!(returned || lost, "a party either comes home or it doesn't");
    let after = world.knowledge.of(nation).known_count();
    if returned {
        assert!(
            after > before,
            "a returning party must grow the map ({before} -> {after})"
        );
    }
    // Replay determinism holds with parties afield.
    let mut again = World::new(&config_with(vec![scout(nation.0, "e", 0)]));
    for _ in 0..(720 * 3) {
        again.step();
    }
    assert_eq!(world.log.hash(), again.log.hash());
}

#[test]
fn one_party_at_a_time() {
    let probe = World::new(&config_with(Vec::new()));
    let nation = probe.nations.nations[0].id;
    let world = World::new(&config_with(vec![
        scout(nation.0, "e", 0),
        scout(nation.0, "w", 0),
    ]));
    let rejected = world.log.iter().any(|e| {
        matches!(e, sim_events::Event::DirectiveRejected { reason, .. }
            if reason.contains("afield"))
    });
    assert!(rejected, "the second party has nobody left to send");
}
