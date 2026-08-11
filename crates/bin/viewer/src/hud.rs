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
    fog_nation: Option<&str>,
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
                ui.label("double-click a tile to walk it · drag/WASD pans · wheel zooms · F fog");
            }
        }
        if let Some(name) = fog_nation {
            ui.separator();
            ui.label(
                RichText::new(format!("FOG · the world as {name} knows it"))
                    .strong()
                    .color(egui::Color32::from_rgb(240, 205, 90)),
            );
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
        world.regolith.fertility(tile),
    ));
    if fields.elevation[tile] >= 0 {
        ui.label(format!("Ground: {}", world.regolith.describe(tile)));
    }
    if let Some(owner) = world.nations.owner[tile] {
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
        ui.label(format!("Held by {} ({})", nation.name, s.name));
        if let Some(te) = world.economy.tile(t.0) {
            ui.label(format!(
                "Population {pop:.0} · fed {:.0}% · stores {:.0}",
                te.last_nutrition * world_schema::Quantity::from_num(100),
                te.stock
            ));
            ui.label(format!(
                "Fields {:.0}% · herd {:.0}",
                te.establishment * world_schema::Quantity::from_num(100),
                te.herd
            ));
        } else {
            ui.label(format!("Population {pop:.0}"));
        }
    } else {
        ui.label(if tiles::habitable(fields, tile) {
            "Unclaimed, habitable"
        } else {
            "Unclaimed wilds"
        });
        ui.label(format!(
            "Food per worker (est.): {:.2} · wild game {:.0} · fishable {:.0}",
            economy::potential(
                fields,
                &world.fauna,
                &world.flora_live,
                &world.climate,
                &world.regolith,
                tile,
                &world.tuning.subsistence,
                &world.tuning.weather
            ),
            world.fauna.huntable(tile),
            world.fauna.fishable(fields, tile)
        ));
        if let Some(s) = world.fauna.top_species_at(tile) {
            ui.label(format!(
                "Most common beast: {}",
                s.describe(&world.fauna.substances)
            ));
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
