//! The golden replay gate: identical config ⇒ identical event-log hash
//! (docs/01-architecture.md, "Testing and CI gates").

use directive_schema::{Directive, DirectiveEntry, Stance};
use sim_server::{RunConfig, run_world};

fn small() -> RunConfig {
    RunConfig {
        seed: 42,
        ticks: sim_clock::TICKS_PER_YEAR,
        map_size: 96,
        nations: 4,
        ..RunConfig::default()
    }
}

#[test]
fn same_seed_same_world() {
    let config = small();
    let a = run_world(&config);
    let b = run_world(&config);
    assert_eq!(a.hash, b.hash);
    assert_eq!(a.population, b.population);
    assert!(
        a.events > 12,
        "a year must close twelve months plus genesis"
    );
}

#[test]
fn different_seed_different_world() {
    let a = run_world(&RunConfig { seed: 1, ..small() });
    let b = run_world(&RunConfig { seed: 2, ..small() });
    assert_ne!(a.hash, b.hash);
}

#[test]
fn directives_are_replay_input() {
    let silent = small();
    let mut governed = small();
    governed.directives = vec![
        DirectiveEntry {
            tick: 0,
            nation: 0,
            directive: Directive::Name {
                name: "The Ember Compact".into(),
            },
        },
        DirectiveEntry {
            tick: 720,
            nation: 0,
            directive: Directive::SetStance {
                stance: Stance::Expansive,
            },
        },
        DirectiveEntry {
            tick: 720,
            nation: 99,
            directive: Directive::SetStance {
                stance: Stance::Expansive,
            },
        },
    ];
    let a = run_world(&silent);
    let b = run_world(&governed);
    let b2 = run_world(&governed);
    // Directives change history; the invalid nation is rejected, not ignored.
    assert_ne!(a.hash, b.hash);
    assert_eq!(b.hash, b2.hash);
    assert_eq!(b.events, b2.events);
}
