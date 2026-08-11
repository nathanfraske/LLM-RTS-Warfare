//! Top bar (date, population, timestep controls) and the province inspector.

use cohorts::CohortKey;
use eframe::egui::{self, RichText};
use readouts::year_month;
use sim_server::World;
use world_schema::ProvinceId;

/// Speed presets in ticks per real second.
pub const SPEEDS: [(&str, f64); 4] = [
    ("1 day/s", 24.0),
    ("1 week/s", 168.0),
    ("1 month/s", 720.0),
    ("1 year/s", 8_640.0),
];

pub fn top_bar(ui: &mut egui::Ui, world: &World, paused: &mut bool, ticks_per_sec: &mut f64) {
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
        ui.label("space pauses · drag/WASD pans · wheel zooms · click inspects");
    });
}

pub fn inspector(ui: &mut egui::Ui, world: &World, selected: Option<ProvinceId>) {
    let Some(p) = selected else {
        ui.weak("Click a province to inspect it.");
        return;
    };
    let province = &world.genesis.provinces[p.0 as usize];
    ui.heading(format!("Province {}", p.0));
    ui.label(format!(
        "{:?} · {} cells · fertility {:.2}",
        province.terrain, province.cells, province.fertility
    ));
    match world.nations.owner[p.0 as usize] {
        Some(owner) => {
            let nation = world
                .nations
                .nations
                .iter()
                .find(|n| n.id == owner)
                .expect("owner exists");
            let s = &world.table[nation.species.0 as usize];
            let pop = world.cohorts.population_of(CohortKey {
                province: p,
                species: nation.species,
            });
            let cap = nations::capacity(province, s);
            ui.label(format!("Held by {} ({})", nation.name, s.name));
            ui.label(format!("Population {pop:.0} / capacity {cap:.0}"));
        }
        None => {
            ui.label("Unclaimed wilds");
        }
    }
    ui.label(format!(
        "Climate: {:.1}°C · moisture {}",
        f32::from(province.mean_temperature) / 10.0,
        province.mean_moisture
    ));
}
