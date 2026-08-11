//! The live spectator viewer: watch a world run, steer time, pan/zoom the
//! map, inspect provinces, and follow the overseer feed
//! (docs/10-visualization.md — viewer v0 on the egui shell).

mod app;
mod built;
mod calamity;
mod camera;
mod critters;
mod feed;
mod fogview;
mod folk;
mod frame;
mod hud;
mod layers;
mod lines;
mod localfolk;
mod localview;
mod sky;
mod waters;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let mut config = sim_server::RunConfig {
        report_dir: None,
        ..Default::default()
    };
    let mut directives_path = String::from("directives.json");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let value = |i: usize| args.get(i + 1).cloned();
        let number = |i: usize| -> Option<u64> { value(i)?.parse().ok() };
        match args[i].as_str() {
            "--seed" => config.seed = number(i).unwrap_or(config.seed),
            "--map-size" => {
                config.map_size = number(i).and_then(|v| u32::try_from(v).ok()).unwrap_or(192);
            }
            "--nations" => {
                config.nations = number(i).and_then(|v| u32::try_from(v).ok()).unwrap_or(4);
            }
            "--directives" => directives_path = value(i).unwrap_or(directives_path),
            _ => {
                eprintln!(
                    "usage: viewer [--seed N] [--map-size N] [--nations N] \
                     [--directives file.json]"
                );
                std::process::exit(2);
            }
        }
        i += 2;
    }
    if let Ok(text) = std::fs::read_to_string(&directives_path) {
        match serde_json::from_str(&text) {
            Ok(entries) => config.directives = entries,
            Err(e) => {
                eprintln!("{directives_path}: invalid directive JSON: {e}");
                std::process::exit(2);
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Agent RTS Warfare — Spectator")
            .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Agent RTS Warfare",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, &config)))),
    )
}
