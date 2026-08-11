//! Append-only event log with an incremental content hash.
//!
//! The hash over the serialized event stream is the determinism fingerprint:
//! two runs agree iff their logs hash identically (docs/01a-foundation.md).

use crate::event::Event;

#[derive(Debug, Default)]
pub struct EventLog {
    events: Vec<Event>,
    hasher: blake3::Hasher,
}

impl EventLog {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: Event) {
        let bytes = postcard::to_allocvec(&event).expect("event serialization is infallible");
        self.hasher.update(&bytes);
        self.events.push(event);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    /// Hex fingerprint of everything pushed so far.
    #[must_use]
    pub fn hash(&self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_schema::{Quantity, Tick};

    #[test]
    fn hash_is_order_and_content_sensitive() {
        let event = |n: i64| Event::MonthClosed {
            tick: Tick(n.unsigned_abs() * 720),
            births: Quantity::from_num(n),
            deaths: Quantity::ZERO,
            population: Quantity::from_num(1000 + n),
        };
        let mut a = EventLog::new();
        let mut b = EventLog::new();
        a.push(event(1));
        a.push(event(2));
        b.push(event(2));
        b.push(event(1));
        assert_ne!(a.hash(), b.hash());

        let mut c = EventLog::new();
        c.push(event(1));
        c.push(event(2));
        assert_eq!(a.hash(), c.hash());
    }
}
