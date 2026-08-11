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

pub(crate) fn nation_name(world: &World, n: NationId) -> String {
    world
        .nations
        .nations
        .iter()
        .find(|x| x.id == n)
        .map_or_else(|| format!("nation {}", n.0), |x| x.name.clone())
}

pub(crate) fn stamp(tick: Tick) -> String {
    let (y, m) = year_month(tick);
    format!("Y{y} M{m}")
}

/// Describe an event for the feed; `None` for events too noisy to show.
#[must_use]
pub fn describe(event: &Event, world: &World) -> Option<Line> {
    crate::lines::overseer_line(event, world)
        .or_else(|| crate::lines::scout_line(event, world))
        .or_else(|| crate::lines::raising_line(event, world))
        .or_else(|| crate::lines::calamity_line(event, world))
        .or_else(|| crate::lines::worldly_line(event, world))
}
