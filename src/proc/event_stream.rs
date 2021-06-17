use super::event::*;
use std::fmt;
use std::collections::HashSet;

pub type SceneNum = u8;
pub type SceneNumOffset = i16;

#[derive(Debug, Clone)]
pub struct EventStream {
    pub events: Vec<Event>,
    pub current_scene: Option<SceneNum>,
    pub new_scene: Option<SceneNum>,
    pub current_subscene: Option<SceneNum>,
    pub new_subscene: Option<SceneNum>,
}

impl EventStream {
    pub fn none() -> Self {
        Self {
            events: Vec::<Event>::new(),
            current_scene: None,
            new_scene: None,
            current_subscene: None,
            new_subscene: None,
        }
    }

    pub fn any(&self) -> bool {
        !self.events.is_empty()
    }

    /// Dedups events.
    pub fn dedup(&mut self) {
        // https://stackoverflow.com/a/47648303
        let mut uniques = HashSet::new();
        self.events.retain(|e| uniques.insert(*e));
    }
}

impl From<Event> for EventStream {
    fn from(ev: Event) -> Self {
        Self {
            events: vec![ev],
            ..Self::none()
        }
    }
}

impl From<Option<Event>> for EventStream {
    fn from(oev: Option<Event>) -> Self {
        if let Some(ev) = oev {
            Self {
                events: vec![ev],
                ..Self::none()
            }
        } else {
            Self::none()
        }
    }
}

impl From<&Vec<Event>> for EventStream {
    fn from(vec: &Vec<Event>) -> Self {
        Self {
            events: vec.clone(),
                ..Self::none()
        }
    }
}

impl fmt::Display for EventStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.events.iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(", "))?;
        if let Some(new_scene) = self.new_scene {
            write!(f, " | scene={}", new_scene)?;
        }
        if let Some(new_subscene) = self.new_subscene {
            write!(f, " | subscene={}", new_subscene)?;
        }
        Ok(())
    }
}
