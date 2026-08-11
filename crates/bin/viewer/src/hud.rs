//! Top bar (date, population, timestep controls) and the tile inspector.

use cohorts::CohortKey;
use eframe::egui::{self, RichText};
use readouts::year_month;
use sim_server::World;
use world_map::tiles;
use world_schema::TileId;

/// Speed presets in ticks per real second.
pub const SPEEDS: [(&str, f64); 4] = [
    ("1 day/s", 24.0),
    ("1 week/s", 168.0),
    ("1 month/s", 720.0),
    ("1 year/s", 8_640.0),
];

pub fn top_bar(
    ui: &mut egui::Ui,
    world: &World,
    paused: &mut bool,
    ticks_per_sec: &mut f64,
    local_tile: Option<TileId>,
) {
    ui.horizontal(|ui| {
        let tick = world.tick();
        let (year, month) = year_month(tick);
        let day = (tick.0 % 720) / 24 + 1;
        ui.label(
            RichText::new(format!("Year {year} · Month {month} · Day {day}"))
                .strong()
                .size(16.0),
        );
        ui.separator();
        ui.label(format!(
            "population {:.0}",
            world.cohorts.total_population()
        ));
        ui.separator();
        let pause_label = if *paused { "▶ resume" } else { "⏸ pause" };
        if ui.button(pause_label).clicked() {
            *paused = !*paused;
        }
        for (label, tps) in SPEEDS {
            let selected = !*paused && (*ticks_per_sec - tps).abs() < f64::EPSILON;
            if ui.selectable_label(selected, label).clicked() {
                *ticks_per_sec = tps;
                *paused = false;
            }
        }
        ui.separator();
        match local_tile {
            Some(t) => {
                ui.label(
                    RichText::new(format!(
                        "LOCAL · tile {} · Esc/Backspace returns to world",
                        t.0
                    ))
                    .strong(),
                );
            }
            None => {
                ui.label("double-click a tile to walk it · drag/WASD pans · wheel zooms");
            }
        }
    });
}

pub fn inspector(ui: &mut egui::Ui, world: &World, selected: Option<TileId>) {
    let Some(t) = selected else {
        ui.weak("Click a tile to inspect it; double-click to descend into it.");
        return;
    };
    let fields = &world.genesis.fields;
    let tile = t.0 as usize;
    ui.heading(format!("Tile {}", t.0));
    ui.label(format!(
        "{:?} · elevation {} m · fertility {}",
        tiles::label(fields, tile),
        fields.elevation[tile],
        fields.cell_fertility[tile],
    ));
    match world.nations.owner[tile] {
        Some(owner) => {
            let nation = world
                .nations
                .nations
                .iter()
                .find(|n| n.id == owner)
                .expect("owner exists");
            let s = &world.table[nation.species.0 as usize];
            let pop = world.cohorts.population_of(CohortKey {
                tile: t,
                species: nation.species,
            });
            let cap = nations::capacity(fields, tile, s);
            ui.label(format!("Held by {} ({})", nation.name, s.name));
            ui.label(format!("Population {pop:.0} / capacity {cap:.0}"));
        }
        None => {
            ui.label(if tiles::habitable(fields, tile) {
                "Unclaimed, habitable"
            } else {
                "Unclaimed wilds"
            });
        }
    }
    ui.label(format!(
        "Climate: {:.1}°C · moisture {} · {}",
        f32::from(fields.temperature[tile]) / 10.0,
        fields.moisture[tile],
        match (tiles::coastal(fields, tile), tiles::riverine(fields, tile)) {
            (true, true) => "coast+river",
            (true, false) => "coast",
            (false, true) => "river",
            (false, false) => "inland",
        }
    ));
}
