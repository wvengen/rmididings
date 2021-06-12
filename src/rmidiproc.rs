use std::fmt;

// Event

#[derive(Debug,Copy,Clone)]
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
           EventType::SYSEX => format!("port={} sysex={:?}", self.port, self.sysex),
        };
        write!(f, "Event type={} {}", self.typ.to_string(), s)
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
#[allow(non_snake_case)]
pub enum EventType {
    NONE,
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
            EventType::NONE => "NONE",
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

// EventStream

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

// Filter and generator setup

// All filters implement this trait.
pub trait FilterTrait {
    // When adding or removing events, implement run.
    fn run(&self, evs: &mut EventStream) {
        evs.events.retain(|ev| self.filter_single(&ev));
        for ev in evs.events.iter_mut() {
            self.modify_single(ev);
        }
    }

    // When modifying single events, implement modify_single.
    fn modify_single(&self, _evs: &mut Event) {}

    // When selecting which events to keep and which not, implement filter_single.
    fn filter_single(&self, _evs: &Event) -> bool { true }
}

#[derive(Debug,PartialEq)]
pub enum ConnectionType {
    Chain,
    Fork,
}

pub struct FilterChain {
    // need to make both public because of macro use
    pub filters: Vec<Box<dyn FilterTrait>>,
    pub connection: ConnectionType,
}

impl FilterTrait for FilterChain {
    fn run(&self, evs: &mut EventStream) {
        match self.connection {
            ConnectionType::Chain => {
                // Run each filter consequetively. Since they mutate evs, this
                // means each filter is run on top of the changes of the previous.
                for f in self.filters.iter() {
                    f.run(evs);
                }
            },
            ConnectionType::Fork => {
                // Run each filter over the original evs and gather all events
                // into a single EventStream.
                // TODO allocate full size of events_out
                // TODO don't clone for first/last filter (can do when running last) ...
                // TODO ... or repeat evs filters.size times, and run on each slice.
                // TODO remove immediate duplicates
                let mut events_out = Vec::<Event>::new();
                for f in self.filters.iter() {
                    let mut evs_this = evs.clone();
                    f.run(&mut evs_this);
                    events_out.extend(evs_this.events);
                }
                evs.events.clear();
                evs.events.extend(events_out);
            },
        }
    }
}

// Connecting filters

#[macro_export]
macro_rules! Chain {
    ( $($f:expr),+ ) => (
        FilterChain {
            connection: ConnectionType::Chain,
            filters: vec!( $(Box::new($f)),+ ),
        }
    )
}

#[macro_export]
macro_rules! Fork {
    ( $($f:expr),+ ) => (
        FilterChain {
            connection: ConnectionType::Fork,
            filters: vec!( $(Box::new($f)),+ ),
        }
    )
}

// Creating filters

// For now we keep using a macro, even if it doesn't do much.
// This allows us to more easily add common things when desired.

macro_rules! define_filter {
    ($name:ident ( $($args:ty),* ) $item:item) => {
        pub struct $name($(pub $args),*);

        impl FilterTrait for $name {
            $item
        }
    }
}

macro_rules! define_generator {
    ($name:ident ( $($args:ty),* ) $item:item) => {
        pub struct $name($(pub $args),*);

        impl $name {
            $item
        }

        impl FilterTrait for $name {
            fn modify_single(&self, ev: &mut Event) {
                *ev = self.generate_single();
            }
        }
    }
}

// Filters

define_filter!(
    Filter(EventType)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.typ == self.0
    }
);

define_filter!(
    PortFilter(usize)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.port == self.0
    }
);

define_filter!(
    PortsFilter(&'static [usize])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.port)
    }
);

define_filter!(
    ChannelFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.channel == self.0
    }
);

define_filter!(
    ChannelsFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.channel)
    }
);

define_filter!(
    KeyFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.note == self.0
    }
);

define_filter!(
    KeysFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.note)
    }
);

define_filter!(
    KeyRangeFilter(u8, u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.note >= self.0 && ev.note <= self.1
    }
);

define_filter!(
    CtrlFilter(u32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.ctrl == self.0
    }
);

define_filter!(
    CtrlsFilter(&'static [u32])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.ctrl)
    }
);

define_filter!(
    CtrlValueFilter(i32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.value == self.0
    }
);

define_filter!(
    CtrlValuesFilter(&'static[i32])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.value)
    }
);

define_filter!(
    CtrlValueRangeFilter(i32, i32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.value >= self.0 && ev.value <= self.1
    }
);

// Generators

define_generator!(
    NoteOn(u8, u8)
    fn generate_single(&self) -> Event {
        NoteOnEvent(0, 0, self.0, self.1)
    }
);

define_generator!(
    NoteOff(u8)
    fn generate_single(&self) -> Event {
        NoteOffEvent(0, 0, self.0)
    }
);

define_generator!(
    Ctrl(u32, i32)
    fn generate_single(&self) -> Event {
        CtrlEvent(0, 0, self.0, self.1)
    }
);

define_generator!(
    SysEx(&'static [u8])
    fn generate_single(&self) -> Event {
        SysExEvent(0, self.0)
    }
);

// Modifiers

define_filter!(
    Port(usize)
    fn modify_single(&self, ev:&mut  Event) {
        ev.port = self.0;
    }
);

define_filter!(
    Channel(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.channel = self.0;
    }
);

define_filter!(
    Transpose(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note += self.0;
    }
);

define_filter!(
    TransposeOctave(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note += self.0 * 12;
    }
);

define_filter!(
    Key(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note = self.0;
    }
);

define_filter!(
    Velocity(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity += self.0;
    }
);

define_filter!(
    VelocityMultiply(f32)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = ((ev.velocity as f32) * self.0) as u8;
    }
);

define_filter!(
    VelocityFixed(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = self.0;
    }
);

define_filter!(
    CtrlMap(u32, u32)
    fn modify_single(&self, ev: &mut Event) {
        if ev.ctrl == self.0 { ev.ctrl = self.1 };
    }
);

// Misc

define_filter!(
    Print()
    fn run(&self, evs:  &mut EventStream) -> () {
        println!("{}", evs.to_string());
    }
);

define_filter!(
    Pass()
    fn run(&self, _evs: &mut EventStream) -> () {
        // pass, which means: keep event stream as it iss
    }
);

define_filter!(
    Discard()
    fn run(&self, evs: &mut EventStream) -> () {
        evs.events.clear();
    }
);
