//! CLI entry:
//! `sim-server --seed 42 --ticks 8640 [--map-size N] [--provinces N] [--nations N]
//!             [--directives file.json] [--report-dir dir] [--hash-only]`

use sim_server::{RunConfig, run_world};

fn main() {
    let mut config = RunConfig::default();
    let mut hash_only = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let value = |i: usize| args.get(i + 1).cloned();
        let number = |i: usize| -> Option<u64> { value(i)?.parse().ok() };
        match args[i].as_str() {
            "--seed" => match number(i) {
                Some(v) => config.seed = v,
                None => return usage(),
            },
            "--ticks" => match number(i) {
                Some(v) => config.ticks = v,
                None => return usage(),
            },
            "--map-size" => match number(i).and_then(|v| u32::try_from(v).ok()) {
                Some(v) => config.map_size = v,
                None => return usage(),
            },
            "--provinces" => match number(i).and_then(|v| u32::try_from(v).ok()) {
                Some(v) => config.provinces = v,
                None => return usage(),
            },
            "--nations" => match number(i).and_then(|v| u32::try_from(v).ok()) {
                Some(v) => config.nations = v,
                None => return usage(),
            },
            "--directives" => match value(i) {
                Some(path) => config.directives = load_directives(&path),
                None => return usage(),
            },
            "--report-dir" => match value(i) {
                Some(dir) => config.report_dir = Some(dir.into()),
                None => return usage(),
            },
            "--hash-only" => {
                hash_only = true;
                i += 1;
                continue;
            }
            _ => return usage(),
        }
        i += 2;
    }

    let report = run_world(&config);
    if hash_only {
        println!("{}", report.hash);
    } else {
        println!(
            "seed {} · {} ticks · {} events · population {:.0}",
            config.seed, config.ticks, report.events, report.population
        );
        println!("event-log hash: {}", report.hash);
        if let Some(dir) = &config.report_dir {
            println!("council reports → {}", dir.display());
        }
    }
}

/// Missing file = empty council log (a fresh world); malformed JSON is fatal.
fn load_directives(path: &str) -> Vec<directive_schema::DirectiveEntry> {
    let Ok(text) = std::fs::read_to_string(path) else {
        eprintln!("note: {path} not found — running with an empty council log");
        return Vec::new();
    };
    match serde_json::from_str(&text) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("{path}: invalid directive JSON: {e}");
            std::process::exit(2);
        }
    }
}

fn usage() {
    eprintln!(
        "usage: sim-server [--seed N] [--ticks N] [--map-size N] [--provinces N] \
         [--nations N] [--directives file.json] [--report-dir dir] [--hash-only]"
    );
    std::process::exit(2);
}
