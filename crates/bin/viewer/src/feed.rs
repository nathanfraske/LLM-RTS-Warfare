//! The spectator event feed: omniscient one-liners, with overseer actions
//! (directives) visually distinguished from world events.

use eframe::egui::Color32;
use readouts::year_month;
use sim_events::Event;
use sim_server::World;
use world_schema::NationId;

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

/// Describe an event for the feed; `None` for events too noisy to show.
#[must_use]
pub fn describe(event: &Event, world: &World) -> Option<Line> {
    let name = |n: NationId| {
        world
            .nations
            .nations
            .iter()
            .find(|x| x.id == n)
            .map_or_else(|| format!("nation {}", n.0), |x| x.name.clone())
    };
    match event {
        Event::NationSpawned { nation, seat, .. } => Some(Line {
            text: format!("Y1 M1 · {} settle tile {}", name(*nation), seat.0),
            kind: Kind::Worldly,
        }),
        Event::NationNamed {
            tick,
            nation,
            name: new_name,
        } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!(
                    "Y{y} M{m} · overseer names nation {} \"{new_name}\"",
                    nation.0
                ),
                kind: Kind::Overseer,
            })
        }
        Event::StanceChanged {
            tick,
            nation,
            stance,
        } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!("Y{y} M{m} · {} sets a {stance:?} posture", name(*nation)),
                kind: Kind::Overseer,
            })
        }
        Event::SettlementDecreed { tick, nation, tile } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!(
                    "Y{y} M{m} · {} decrees the settling of tile {}",
                    name(*nation),
                    tile.0
                ),
                kind: Kind::Overseer,
            })
        }
        Event::DirectiveRejected {
            tick,
            nation,
            reason,
        } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!("Y{y} M{m} · {} decree rejected: {reason}", name(*nation)),
                kind: Kind::Alarm,
            })
        }
        Event::TileSettled {
            tick,
            nation,
            from,
            tile,
            settlers,
        } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!(
                    "Y{y} M{m} · {settlers:.0} settlers of {} leave tile {} and found tile {}",
                    name(*nation),
                    from.0,
                    tile.0
                ),
                kind: Kind::Worldly,
            })
        }
        Event::NationsMet { tick, a, b } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!("Y{y} M{m} · first contact: {} meets {}", name(*a), name(*b)),
                kind: Kind::Contact,
            })
        }
        Event::Famine { tick, tile, .. } => {
            let (y, m) = year_month(*tick);
            Some(Line {
                text: format!("Y{y} M{m} · hunger in tile {}", tile.0),
                kind: Kind::Alarm,
            })
        }
        Event::WorldGenerated { .. } | Event::MonthClosed { .. } => None,
    }
}
