//! The narrator's sentences: every event the feed shows, phrased — one
//! function per voice (decrees, scouts, raisings, calamities, the world).

use sim_events::Event;
use sim_server::World;

use crate::feed::{Kind, Line, nation_name, stamp};

/// Overseer actions: the directive-driven events, highlighted in gold.
pub(crate) fn overseer_line(event: &Event, world: &World) -> Option<Line> {
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
pub(crate) fn scout_line(event: &Event, world: &World) -> Option<Line> {
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
pub(crate) fn worldly_line(event: &Event, world: &World) -> Option<Line> {
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
        _ => return None,
    };
    Some(Line { text, kind })
}

/// The built world rising: raised by decree or by the people's own need.
pub(crate) fn raising_line(event: &Event, world: &World) -> Option<Line> {
    let (text, kind) = match event {
        Event::PeopleRaised {
            tick,
            nation,
            tile,
            work,
        } => (
            format!(
                "{} · the people of {} raise a {work} at tile {} unbidden",
                stamp(*tick),
                nation_name(world, *nation),
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
        _ => return None,
    };
    Some(Line { text, kind })
}

/// The world's violence: fire from below and the shaking earth.
pub(crate) fn calamity_line(event: &Event, world: &World) -> Option<Line> {
    let (text, kind) = match event {
        Event::VolcanoErupted {
            tick,
            tile,
            reach,
            ash_tiles,
        } => (
            format!(
                "{} · the mountain at tile {} erupts — lava runs {} tiles, ash falls on {}",
                stamp(*tick),
                tile.0,
                reach,
                ash_tiles
            ),
            Kind::Alarm,
        ),
        Event::Wildfire { tick, tile } => (
            format!("{} · fire sweeps tile {}", stamp(*tick), tile.0),
            Kind::Alarm,
        ),
        Event::Earthquake { tick, tile, reach } => (
            format!(
                "{} · the earth shakes at tile {} — {} tiles rattled",
                stamp(*tick),
                tile.0,
                reach
            ),
            Kind::Alarm,
        ),
        Event::WorkToppled {
            tick,
            nation,
            tile,
            work,
        } => (
            format!(
                "{} · the {work} of {} at tile {} falls in the shaking",
                stamp(*tick),
                nation_name(world, *nation),
                tile.0
            ),
            Kind::Alarm,
        ),
        _ => return None,
    };
    Some(Line { text, kind })
}
