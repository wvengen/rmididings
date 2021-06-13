pub mod event;
pub mod event_stream;
pub mod filter_trait;
#[macro_use]
pub mod filter_chain;
pub use event::*;
pub use event_stream::*;
pub use filter_trait::*;
pub use filter_chain::*;


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

// Scene switching

define_filter!(
    SceneSwitch(u8)
    fn run(&self, evs: &mut EventStream) {
        if evs.any() { evs.scene = self.0; }
    }
);

define_filter!(
    SceneSwitchOffset(i16)
    fn run(&self, evs: &mut EventStream) {
        if evs.any() { evs.scene = (evs.scene as i16 + self.0) as u8; }
    }
);

pub struct Init<'a>(pub &'a dyn FilterTrait);
impl FilterTrait for Init<'_> {
    fn run_init(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}

pub struct Exit<'a>(pub &'a dyn FilterTrait);
impl FilterTrait for Exit<'_> {
    fn run_exit(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}

// Misc

define_filter!(
    Print()
    fn run(&self, evs:  &mut EventStream) -> () {
        if evs.any() { println!("{}", evs.to_string()); }
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