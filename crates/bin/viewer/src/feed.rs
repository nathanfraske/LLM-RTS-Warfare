//! The spectator event feed: omniscient one-liners, with overseer actions
//! (directives) visually distinguished from world events.

use eframe::egui::Color32;
use readouts::year_month;
use sim_events::Event;
use sim_server::World;
use world_schema::{NationId, Tick};

pub enum Kind {
    Overseer,
    Worldly,
    Alarm,
    Contact,
}

impl Kind {
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            Kind::Overseer => Color32::from_rgb(235, 200, 90),
            Kind::Worldly => Color32::from_rgb(190, 195, 205),
            Kind::Alarm => Color32::from_rgb(235, 120, 100),
            Kind::Contact => Color32::from_rgb(110, 200, 235),
        }
    }
}

pub struct Line {
    pub text: String,
    pub kind: Kind,
}

fn nation_name(world: &World, n: NationId) -> String {
    world
        .nations
        .nations
        .iter()
        .find(|x| x.id == n)
        .map_or_else(|| format!("nation {}", n.0), |x| x.name.clone())
}

fn stamp(tick: Tick) -> String {
    let (y, m) = year_month(tick);
    format!("Y{y} M{m}")
}

/// Describe an event for the feed; `None` for events too noisy to show.
#[must_use]
pub fn describe(event: &Event, world: &World) -> Option<Line> {
    overseer_line(event, world)
        .or_else(|| scout_line(event, world))
        .or_else(|| worldly_line(event, world))
}

/// Overseer actions: the directive-driven events, highlighted in gold.
fn overseer_line(event: &Event, world: &World) -> Option<Line> {
    let (text, kind) = match event {
        Event::NationNamed {
            tick,
            nation,
            name: new_name,
        } => (
            format!(
                "{} · overseer names nation {} \"{new_name}\"",
                stamp(*tick),
                nation.0
            ),
            Kind::Overseer,
        ),
        Event::PolicySet {
            tick,
            nation,
            key,
            value,
        } => (
            format!(
                "{} · {} sets {key} to {value}",
                stamp(*tick),
                nation_name(world, *nation)
            ),
            Kind::Overseer,
        ),
        Event::SettlementDecreed { tick, nation, tile } => (
            format!(
                "{} · {} decrees the settling of tile {}",
                stamp(*tick),
                nation_name(world, *nation),
                tile.0
            ),
            Kind::Overseer,
        ),
        Event::WorkCommissioned {
            tick,
            nation,
            tile,
            work,
        } => (
            format!(
                "{} · {} commissions a {work} on tile {}",
                stamp(*tick),
                nation_name(world, *nation),
                tile.0
            ),
            Kind::Overseer,
        ),
        Event::DirectiveRejected {
            tick,
            nation,
            reason,
        } => (
            format!(
                "{} · {} decree rejected: {reason}",
                stamp(*tick),
                nation_name(world, *nation)
            ),
            Kind::Alarm,
        ),
        _ => return None,
    };
    Some(Line { text, kind })
}

/// Parties afield: dispatches, homecomings, and the ones that never return.
fn scout_line(event: &Event, world: &World) -> Option<Line> {
    let (text, kind) = match event {
        Event::ScoutDispatched {
            tick,
            nation,
            bearing,
        } => (
            format!(
                "{} · scouts of {} set out to the {bearing}",
                stamp(*tick),
                nation_name(world, *nation)
            ),
            Kind::Worldly,
        ),
        Event::ScoutReturned {
            tick,
            nation,
            tiles_learned,
        } => (
            format!(
                "{} · scouts of {} come home, {tiles_learned} tiles mapped",
                stamp(*tick),
                nation_name(world, *nation)
            ),
            Kind::Worldly,
        ),
        Event::ScoutLost { tick, nation } => (
            format!(
                "{} · the scouts of {} never come back",
                stamp(*tick),
                nation_name(world, *nation)
            ),
            Kind::Alarm,
        ),
        _ => return None,
    };
    Some(Line { text, kind })
}

/// World events: what the simulation itself did.
fn worldly_line(event: &Event, world: &World) -> Option<Line> {
    let (text, kind) = match event {
        Event::NationSpawned { nation, seat, .. } => (
            format!(
                "Y1 M1 · {} settle tile {}",
                nation_name(world, *nation),
                seat.0
            ),
            Kind::Worldly,
        ),
        Event::TileSettled {
            tick,
            nation,
            from,
            tile,
            settlers,
        } => (
            format!(
                "{} · {settlers:.0} settlers of {} leave tile {} and found tile {}",
                stamp(*tick),
                nation_name(world, *nation),
                from.0,
                tile.0
            ),
            Kind::Worldly,
        ),
        Event::WorkCompleted {
            tick,
            nation,
            tile,
            work,
        } => (
            format!(
                "{} · the {work} of {} on tile {} stands complete",
                stamp(*tick),
                nation_name(world, *nation),
                tile.0
            ),
            Kind::Worldly,
        ),
        Event::BandMoved {
            tick,
            nation,
            from,
            to,
            blind,
        } => (
            format!(
                "{} · hunger drives the band of {} from tile {} {} tile {}",
                stamp(*tick),
                nation_name(world, *nation),
                from.0,
                if *blind {
                    "blindly into unwalked land at"
                } else {
                    "to"
                },
                to.0
            ),
            Kind::Alarm,
        ),
        Event::NationsMet { tick, a, b } => (
            format!(
                "{} · first contact: {} meets {}",
                stamp(*tick),
                nation_name(world, *a),
                nation_name(world, *b)
            ),
            Kind::Contact,
        ),
        Event::Famine { tick, tile, .. } => (
            format!("{} · hunger in tile {}", stamp(*tick), tile.0),
            Kind::Alarm,
        ),
        Event::VolcanoErupted { tick, tile, reach } => (
            format!(
                "{} · the mountain at tile {} erupts — lava runs {} tiles",
                stamp(*tick),
                tile.0,
                reach
            ),
            Kind::Alarm,
        ),
        _ => return None,
    };
    Some(Line { text, kind })
}
