use std::fmt;
use event::*;

#[derive(Debug,Clone)]
pub struct EventStream {
    pub events: Vec<Event>,
}

impl EventStream {
    pub fn none() -> Self {
        Self { events: Vec::<Event>::new() }
    }

    pub fn one() -> Self {
        Self { events: vec!(Event::new()) }
    }
}

impl From<Event> for EventStream {
    fn from(ev: Event) -> Self {
        Self { events: vec!(ev) }
    }
}

impl fmt::Display for EventStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.events.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(", "))
    }
}

