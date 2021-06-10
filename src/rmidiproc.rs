use std::fmt;
use std::ops;

// Event

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
    pub sysex: &'static [u8]
}

impl Event {
    pub fn new() -> Event {
        Event { typ: EventType::NONE, port: 0, channel: 0, data1: 0, data2: 0, note: 0, velocity: 0, ctrl: 0, value: 0, program: 0, sysex: &[] }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.typ {
            EventType::NONE => "".to_string(),
            EventType::NOTEON => format!("port={} channel={}, note={} velocity={}", self.port, self.channel, self.note, self.velocity),
            EventType::NOTEOFF => format!("port={} channel={}, note={}", self.port, self.channel, self.note),
            EventType::CTRL => format!("port={} channel={}, ctrl={} value={}", self.port, self.channel, self.ctrl, self.value),
            //EventType::PROGRAM => format!("port={} channel={}, program={}", self.port, self.channel, self.program),
            //EventType::PITCHBEND => format!("port={} channel={}, value={}", self.port, self.channel, self.value),
            //EventType::AFTERTOUCH => format!("port={} channel={}, value={}", self.port, self.channel, self.value),
            //EventType::POLY_AFTERTOUCH => format!("port={} channel={}, value={}", self.port, self.channel, self.value),
            EventType::SYSEX => format!("port={} sysex={:?}", self.port, self.sysex),
        };
        write!(f, "Event type={} {}", self.typ.to_string(), s)
    }
}

#[derive(PartialEq)]
#[allow(non_snake_case)]
pub enum EventType {
    NONE,
    NOTEON,
    NOTEOFF,
    CTRL,
    //PROGRAM,
    //PITCHBEND,
    //AFTERTOUCH,
    //POLY_AFTERTOUCH,
    SYSEX
    // TODO finish - see http://dsacre.github.io/mididings/doc/misc.html
    // TODO handle event type filters (combination of several types)
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            EventType::NONE => "NONE",
            EventType::NOTEON => "NOTEON",
            EventType::NOTEOFF => "NOTEOFF",
            EventType::CTRL => "CTRL",
            //EventType::PROGRAM => "PROGRAM",
            //EventType::PITCHBEND => "PITCHBEND",
            //EventType::AFTERTOUCH => "AFTERTOUCH",
            //EventType::POLY_AFTERTOUCH => "POLY_AFTERTOUCH",
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
    Event { typ: EventType::NOTEON, port, channel, note, velocity, ..Event::new() }
}

#[allow(non_snake_case)]
pub fn NoteOffEvent(port: usize, channel: u8, note: u8) -> Event {
    Event { typ: EventType::NOTEOFF, port, channel, note, ..Event::new() }
}

#[allow(non_snake_case)]
pub fn CtrlEvent(port: usize, channel: u8, ctrl: u32, value: i32) -> Event {
    Event { typ: EventType::CTRL, port, channel, ctrl, value, ..Event::new() }
}

#[allow(non_snake_case)]
pub fn SysExEvent(port: usize, sysex: &'static [u8]) -> Event {
    Event { typ: EventType::SYSEX, port, sysex, ..Event::new() }
}

// Filter and generator setup

pub trait FilterTrait {
    fn run(&self, ev: Event) -> Event;
}

impl<T> ops::Shr<T> for Event where T: FilterTrait {
    type Output = Event;

    fn shr(self, rhs: T) -> Self::Output {
        rhs.run(self)
    }
}

pub struct ChainedFilter {
    f1: Box<dyn FilterTrait>,
    f2: Box<dyn FilterTrait>
}

impl ChainedFilter {
    pub fn new(f1: impl FilterTrait + 'static, f2: impl FilterTrait + 'static) -> ChainedFilter {
        ChainedFilter {
            f1: Box::new(f1),
            f2: Box::new(f2)
        }
    }
}

impl FilterTrait for ChainedFilter {
    fn run(&self, ev: Event) -> Event {
        self.f2.run(self.f1.run(ev))
    }
}

impl<T> ops::Shr<T> for ChainedFilter where T: FilterTrait + 'static {
    type Output = ChainedFilter;

    fn shr(self, rhs: T) -> Self::Output {
        ChainedFilter::new(self, rhs)
    }
}

macro_rules! define_filter {
    ($name:ident ( $($args:ty),* ) $item:item) => {
        pub struct $name($(pub $args),*);

        impl FilterTrait for $name {
            $item
        }

        impl<T> ops::Shr<T> for $name where T: FilterTrait + 'static {
            type Output = ChainedFilter;

            fn shr(self, rhs: T) -> Self::Output {
                ChainedFilter::new(self, rhs)
            }
        }
    }
}

// Filters

define_filter!(
    Filter(EventType)
    fn run(&self, ev: Event) -> Event { 
        if ev.typ == self.0 { ev } else { Event::new() }
    }
);

define_filter!(
    PortFilter(usize)
    fn run(&self, ev: Event) -> Event { 
        if ev.port == self.0 { ev } else { Event::new() }
    }
);

define_filter!(
    PortsFilter(&'static [usize])
    fn run(&self, ev: Event) -> Event { 
        if self.0.contains(&ev.port) { ev } else { Event::new() }
    }
);

define_filter!(
    ChannelFilter(u8)
    fn run(&self, ev: Event) -> Event { 
        if ev.channel == self.0 { ev } else { Event::new() }
    }
);

define_filter!(
    ChannelsFilter(&'static [u8])
    fn run(&self, ev: Event) -> Event { 
        if self.0.contains(&ev.channel) { ev } else { Event::new() }
    }
);

define_filter!(
    CtrlFilter(u32)
    fn run(&self, ev: Event) -> Event { 
        if ev.ctrl == self.0 { ev } else { Event::new() }
    }
);

define_filter!(
    CtrlsFilter(&'static [u32])
    fn run(&self, ev: Event) -> Event { 
        if self.0.contains(&ev.ctrl) { ev } else { Event::new() }
    }
);

define_filter!(
    CtrlValueFilter(i32)
    fn run(&self, ev: Event) -> Event { 
        if ev.value == self.0 { ev } else { Event::new() }
    }
);

define_filter!(
    CtrlValuesFilter(&'static[i32])
    fn run(&self, ev: Event) -> Event { 
        if self.0.contains(&ev.value) { ev } else { Event::new() }
    }
);

define_filter!(
    CtrlValueRangeFilter(i32, i32)
    fn run(&self, ev: Event) -> Event { 
        if ev.value >= self.0 && ev.value <= self.1 { ev } else { Event::new() }
    }
);

// Generators

define_filter!(
    NoteOn(u8, u8)
    fn run(&self, _: Event) -> Event {
        NoteOnEvent(0, 0, self.0, self.1)
    }
);

define_filter!(
    NoteOff(u8)
    fn run(&self, _: Event) -> Event {
        NoteOffEvent(0, 0, self.0)
    }
);

define_filter!(
    Ctrl(u32, i32)
    fn run(&self, _: Event) -> Event {
        CtrlEvent(0, 0, self.0, self.1)
    }
);

define_filter!(
    SysEx(&'static [u8])
    fn run(&self, _: Event) -> Event {
        SysExEvent(0, self.0)
    }
);

// Modifiers

define_filter!(
    Port(usize)
    fn run(&self, ev: Event) -> Event {
        Event { port: self.0, ..ev }
    }
);

define_filter!(
    Channel(u8)
    fn run(&self, ev: Event) -> Event {
        Event { channel: self.0, ..ev }
    }
);

define_filter!(
    Transpose(u8)
    fn run(&self, ev: Event) -> Event {
        Event { note: ev.note + self.0, ..ev }
    }
);

define_filter!(
    TransposeOctave(u8)
    fn run(&self, ev: Event) -> Event {
        Event { note: ev.note + self.0 * 12, ..ev }
    }
);

define_filter!(
    Key(u8)
    fn run(&self, ev: Event) -> Event {
        Event { note: self.0, ..ev }
    }
);

define_filter!(
    CtrlMap(u32, u32)
    fn run(&self, ev: Event) -> Event {
        if ev.ctrl == self.0 { Event { ctrl: self.1, ..ev } } else { ev }
    }
);

// Misc

define_filter!(
    Print()
    fn run(&self, ev: Event) -> Event {
        println!("{}", ev.to_string());
        ev
    }
);

define_filter!(
    Pass()
    fn run(&self, ev: Event) -> Event {
        ev
    }
);

define_filter!(
    Discard()
    fn run(&self, _: Event) -> Event {
        Event::new()
    }
);
