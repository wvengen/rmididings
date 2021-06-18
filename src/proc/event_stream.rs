use std::iter::FromIterator;
use std::collections::HashSet;

use super::event::*;

#[derive(Debug, Clone, Eq, Default, Hash, PartialEq)]
pub struct EventStream<'a> {
    events: Vec<Event<'a>>,
}

impl<'a> EventStream<'a> {
    pub fn append(&mut self, other: &mut Vec<Event<'a>>) {
        self.events.append(other);
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Event<'a>> {
        self.events.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Event<'a>> {
        self.events.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn push(&mut self, value: Event<'a>) {
        self.events.push(value);
    }

    pub fn pop(&mut self) -> Option<Event<'_>> {
        self.events.pop()
    }

    pub fn remove(&mut self, index: usize) -> Event<'_> {
        self.events.remove(index)
    }

    pub fn retain<F>(&mut self, f: F) where F: FnMut(&Event) -> bool {
        self.events.retain(f)
    }

    pub fn replace(&mut self, other: EventStream<'a>) {
        self.events = other.events;
    }

    /// EventStream with a single None event.
    ///
    /// This is used mainly for init and exit patches, so that e.g. a {SceneSwitch}
    /// will work there, as it only works when there is at least one event.
    pub fn none() -> Self {
        Self { events: vec![Event::default()] }
    }

    /// EventStream without any events.
    /// 
    /// This is an alias for {default()}, this name is more explicit.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Dedups events.
    pub fn dedup(&mut self) {
        // https://stackoverflow.com/a/47648303
        let mut uniques = HashSet::new();
        self.events.retain(|e| uniques.insert(e.clone()));
    }
}

impl<'a> PartialEq<Vec<Event<'a>>> for EventStream<'a> {
    fn eq(&self, other: &Vec<Event<'a>>) -> bool {
        self.events == *other
    }
}

impl<'a> PartialEq<Event<'a>> for EventStream<'a> {
    fn eq(&self, other: &Event) -> bool {
        self.events.len() == 1 && self.events[0] == *other
    }
}

impl<'a> Extend<Event<'a>> for EventStream<'a> {
    fn extend<I: IntoIterator<Item = Event<'a>>>(&mut self, iter: I) {
        self.events.extend(iter.into_iter());
    }
}

impl<'a> From<Event<'a>> for EventStream<'a> {
    fn from(ev: Event<'a>) -> Self {
        Self { events: vec![ev] }
    }
}

impl<'a> From<&Event<'a>> for EventStream<'a> {
    fn from(ev: &Event<'a>) -> Self {
        Self { events: vec![ev.clone()] }
    }
}

impl<'a> From<Option<Event<'a>>> for EventStream<'a> {
    fn from(oev: Option<Event<'a>>) -> Self {
        if let Some(ev) = oev {
            Self::from(ev)
        } else {
            Self::default()
        }
    }
}

impl<'a> From<Option<&Event<'a>>> for EventStream<'a> {
    fn from(oev: Option<&Event<'a>>) -> Self {
        if let Some(ev) = oev {
            Self::from(ev.clone())
        } else {
            Self::default()
        }
    }
}

impl<'a> From<Vec<Event<'a>>> for EventStream<'a> {
    fn from(events: Vec<Event<'a>>) -> Self {
        Self { events }
    }
}

impl<'a> From<&Vec<Event<'a>>> for EventStream<'a> {
    fn from(events: &Vec<Event<'a>>) -> Self {
        Self { events: events.clone() }
    }
}

impl<'a> From<Vec<&Event<'a>>> for EventStream<'a> {
    fn from(events: Vec<&Event<'a>>) -> Self {
        Self { events: events.into_iter().map(|e| e.clone()).collect() }
    }
}

impl<'a> FromIterator<Event<'a>> for EventStream<'a> {
    fn from_iter<I: IntoIterator<Item=Event<'a>>>(iter: I) -> Self {
        let mut s = Self::default();
        for ev in iter.into_iter() { s.push(ev); }
        s
    }
}

impl<'a> FromIterator<EventStream<'a>> for EventStream<'a> {
    fn from_iter<I: IntoIterator<Item=EventStream<'a>>>(iter: I) -> Self {
        let mut s = Self::default();
        for evs in iter.into_iter() { s.extend(evs); }
        s
    }
}

impl<'a> IntoIterator for EventStream<'a> {
    type Item = Event<'a>;
    type IntoIter = std::vec::IntoIter<Event<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl<'a> IntoIterator for &'a EventStream<'a> {
    type Item = &'a Event<'a>;
    type IntoIter = std::slice::Iter<'a, Event<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

impl<'a> IntoIterator for &'a mut EventStream<'a> {
    type Item = &'a mut Event<'a>;
    type IntoIter = std::slice::IterMut<'a, Event<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter_mut()
    }
}