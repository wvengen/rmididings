#![allow(non_snake_case)]
use std::fmt;

/// MIDI Event
///
/// Please use one of [`Event::new()`], [`NoteOnEvent()`], [`NoteOffEvent()`], [`CtrlEvent()`] or [`SysExEvent()`].
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct Event {
    pub typ: EventType,
    pub port: usize,
    pub channel: u8,
    pub data1: u8,
    pub data2: u8,
    pub note: u8,
    pub velocity: u8,
    pub ctrl: u32,
    pub value: i32,
    pub program: u8,
    pub sysex: &'static [u8], // TODO better lifetime specifier
}

impl Event {
    /// Returns an empty event with a specific type.
    ///
    /// If possible, please use one of: [`NoteOnEvent()`], [`NoteOffEvent()`], [`CtrlEvent()`] or [`SysExEvent()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::event::*;
    /// let ev = Event::new(EventType::NOTEON);
    /// assert_eq!(ev.typ, EventType::NOTEON);
    /// ```
    pub fn new(typ: EventType) -> Event {
        Event { typ, port: 0, channel: 0, data1: 0, data2: 0, note: 0, velocity: 0, ctrl: 0, value: 0, program: 0, sysex: &[] }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.typ {
            EventType::NOTEON => format!(
                "port={} channel={}, note={} velocity={}",
                self.port, self.channel, self.note, self.velocity
            ),
            EventType::NOTEOFF => format!(
                "port={} channel={} note={}",
                self.port, self.channel, self.note
            ),
            EventType::CTRL => format!(
                "port={} channel={}, ctrl={} value={}",
                self.port, self.channel, self.ctrl, self.value
            ),
            EventType::SYSEX => format!(
                "port={} sysex={:?}",
                self.port, self.sysex
            ),
        };
        write!(f, "Event type={} {}", self.typ.to_string(), s)
    }
}

// MIDI Event type
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum EventType {
    NOTEON,
    NOTEOFF,
    CTRL,
    SYSEX,
    // TODO finish - see http://dsacre.github.io/mididings/doc/misc.html
    // TODO handle event type filters (combination of several types)
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            EventType::NOTEON => "NOTEON",
            EventType::NOTEOFF => "NOTEOFF",
            EventType::CTRL => "CTRL",
            EventType::SYSEX => "SYSEX",
        };
        write!(f, "{}", s)
    }
}

/// MIDI note on event
///
/// # Examples
///
/// ```
/// # use rmididings::proc::event::*;
/// let ev = NoteOnEvent(1, 2, 60, 40);
///
/// assert_eq!(ev.typ, EventType::NOTEON);
/// assert_eq!(ev.port, 1);
/// assert_eq!(ev.channel, 2);
/// assert_eq!(ev.note, 60);
/// assert_eq!(ev.velocity, 40);
/// ```
pub fn NoteOnEvent(port: usize, channel: u8, note: u8, velocity: u8) -> Event {
    Event {
        port,
        channel,
        note,
        velocity,
        ..Event::new(EventType::NOTEON)
    }
}

/// MIDI note off event
///
/// # Examples
///
/// ```
/// # use rmididings::proc::event::*;
/// let ev = NoteOffEvent(1, 2, 60);
///
/// assert_eq!(ev.typ, EventType::NOTEOFF);
/// assert_eq!(ev.port, 1);
/// assert_eq!(ev.channel, 2);
/// assert_eq!(ev.note, 60);
/// ```
pub fn NoteOffEvent(port: usize, channel: u8, note: u8) -> Event {
    Event {
        port,
        channel,
        note,
        ..Event::new(EventType::NOTEOFF)
    }
}

/// MIDI controller event
///
/// # Examples
///
/// ```
/// # use rmididings::proc::event::*;
/// let ev = CtrlEvent(1, 2, 7, 80);
///
/// assert_eq!(ev.typ, EventType::CTRL);
/// assert_eq!(ev.port, 1);
/// assert_eq!(ev.channel, 2);
/// assert_eq!(ev.ctrl, 7);
/// assert_eq!(ev.value, 80);
/// ```
pub fn CtrlEvent(port: usize, channel: u8, ctrl: u32, value: i32) -> Event {
    Event {
        port,
        channel,
        ctrl,
        value,
        ..Event::new(EventType::CTRL)
    }
}

/// MIDI system exclusive event
///
/// # Examples
///
/// ```
/// # use rmididings::proc::event::*;
/// let ev = SysExEvent(1, &[0xf7, 0xf0]);
///
/// assert_eq!(ev.typ, EventType::SYSEX);
/// assert_eq!(ev.port, 1);
/// assert_eq!(ev.sysex.to_vec(), vec![0xf7, 0xf0]);
/// ```
pub fn SysExEvent(port: usize, sysex: &'static [u8]) -> Event {
    Event {
        port,
        sysex,
        ..Event::new(EventType::SYSEX)
    }
}
