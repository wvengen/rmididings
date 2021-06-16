use std::fmt;

#[derive(Debug,Copy,Clone,PartialEq)]
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
    pub sysex: &'static [u8] // TODO better lifetime specifier
}

impl Event {
    pub fn new(typ: EventType) -> Event {
        Event { typ: typ, port: 0, channel: 0, data1: 0, data2: 0, note: 0, velocity: 0, ctrl: 0, value: 0, program: 0, sysex: &[] }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.typ {
            EventType::NOTEON => format!("port={} channel={}, note={} velocity={}", self.port, self.channel, self.note, self.velocity),
            EventType::NOTEOFF => format!("port={} channel={} note={}", self.port, self.channel, self.note),
            EventType::CTRL => format!("port={} channel={}, ctrl={} value={}", self.port, self.channel, self.ctrl, self.value),
            EventType::SYSEX => format!("port={} sysex={:?}", self.port, self.sysex),
        };
        write!(f, "Event type={} {}", self.typ.to_string(), s)
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
#[allow(non_snake_case)]
pub enum EventType {
    NOTEON,
    NOTEOFF,
    CTRL,
    SYSEX
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

#[derive(PartialEq)]
#[allow(non_snake_case, dead_code)]
pub enum EventAttribute {
    PORT,
    CHANNEL,
    DATA1,
    DATA2,
    NOTE,
    VELOCITY,
    CTRL,
    VALUE,
    PROGRAM
}

#[allow(non_snake_case)]
pub fn NoteOnEvent(port: usize, channel: u8, note: u8, velocity: u8) -> Event {
    Event { port, channel, note, velocity, ..Event::new(EventType::NOTEON) }
}

#[allow(non_snake_case)]
pub fn NoteOffEvent(port: usize, channel: u8, note: u8) -> Event {
    Event { port, channel, note, ..Event::new(EventType::NOTEOFF) }
}

#[allow(non_snake_case)]
pub fn CtrlEvent(port: usize, channel: u8, ctrl: u32, value: i32) -> Event {
    Event { port, channel, ctrl, value, ..Event::new(EventType::CTRL) }
}

#[allow(non_snake_case)]
pub fn SysExEvent(port: usize, sysex: &'static [u8]) -> Event {
    Event { port, sysex, ..Event::new(EventType::SYSEX) }
}
