//! The open governance surface (docs/20-open-directives.md): directives are
//! validated against the live registry, rejections are in-world events that
//! cost nothing, decreed leaves are pinned against the autopilot, and the
//! report's charter renders the registry — never a hand-written list.

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

/// A species can fail to spawn on a small world — aim at the first nation
/// that actually exists, never a hardcoded id.
fn first_nation() -> u32 {
    World::new(&config_with(Vec::new())).nations.nations[0].id.0
}

fn set(nation: u32, key: &str, value: PolicyValue) -> DirectiveEntry {
    DirectiveEntry {
        tick: 0,
        nation,
        directive: Directive::Set {
            key: key.into(),
            value,
        },
    }
}

#[test]
fn unknown_and_out_of_bounds_orders_are_rejected_free_of_charge() {
    let silent = World::new(&config_with(Vec::new()));
    let target = first_nation();
    let governed = World::new(&config_with(vec![
        set(target, "taxation.tithe", PolicyValue::Int(100)),
        set(target, "labor.hunt", PolicyValue::Int(4000)),
        set(
            target,
            "expansion.posture",
            PolicyValue::Text("bold".into()),
        ),
    ]));
    let rejections = governed
        .log
        .iter()
        .filter(|e| matches!(e, sim_events::Event::DirectiveRejected { .. }))
        .count();
    assert_eq!(rejections, 3, "each bad order is rejected in-world");
    assert_eq!(
        governed.nations.nations[0].mandate, silent.nations.nations[0].mandate,
        "rejected orders cost nothing"
    );
    assert_eq!(
        economy::labor_milli(&governed.nations.nations[0].policy),
        economy::labor_milli(&silent.nations.nations[0].policy),
        "rejected values never land"
    );
}

#[test]
fn decreed_leaves_hold_against_the_autopilot() {
    let mut world = World::new(&config_with(vec![set(
        first_nation(),
        "labor.hunt",
        PolicyValue::Int(700),
    )]));
    let nation = &world.nations.nations[0];
    assert_eq!(nation.policy.int("labor.hunt"), 700);
    assert!(nation.policy.directed("labor.hunt"));

    for _ in 0..(720 * 12) {
        world.step();
    }
    assert_eq!(
        world.nations.nations[0].policy.int("labor.hunt"),
        700,
        "a year of return-following must not move a decreed leaf"
    );
}

#[test]
fn the_charter_renders_the_live_registry() {
    let world = World::new(&config_with(vec![set(
        first_nation(),
        "expansion.posture",
        PolicyValue::Text("expansive".into()),
    )]));
    let nation = &world.nations.nations[0];
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
    for key in [
        "expansion.posture",
        "labor.gather",
        "labor.hunt",
        "labor.fish",
        "labor.cultivate",
        "labor.herd",
        "nation.name",
        "band.settle",
        "works.commission",
        "farmstead / granary / dwellings",
    ] {
        assert!(report.contains(key), "charter must list {key}");
    }
    assert!(
        report.contains("expansive (decreed)"),
        "decreed values are marked"
    );
    let posture_line = report
        .lines()
        .find(|l| l.starts_with("People:"))
        .expect("header line");
    assert!(posture_line.contains("Posture: expansive"));
}
