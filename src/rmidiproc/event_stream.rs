use std::fmt;
use event::*;

#[derive(Debug,Clone)]
pub struct EventStream {
    pub events: Vec<Event>,
    pub scene: u8,
}

impl EventStream {
    pub fn none() -> Self {
        Self { events: Vec::<Event>::new(), scene: 0 }
    }

    pub fn any(&self) -> bool {
        self.events.len() > 0
    }
}

impl From<Event> for EventStream {
    fn from(ev: Event) -> Self {
        Self { events: vec!(ev), scene: 0 }
    }
}

impl From<Option<Event>> for EventStream {
    fn from(oev: Option<Event>) -> Self {
        if let Some(ev) = oev {
            Self { events: vec!(ev), scene: 0 }
        } else {
            Self::none()
        }
    }
}

impl fmt::Display for EventStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} | scene={}", self.events.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(", "), self.scene)
    }
}

