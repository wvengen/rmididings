#![macro_use]
pub mod event;
pub mod event_stream;
pub mod filter_trait;
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

define_modifier!(
    Port(usize)
    fn modify_single(&self, ev:&mut  Event) {
        ev.port = self.0;
    }
);

define_modifier!(
    Channel(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.channel = self.0;
    }
);

define_modifier!(
    Transpose(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note += self.0;
    }
);

define_modifier!(
    TransposeOctave(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note += self.0 * 12;
    }
);

define_modifier!(
    Key(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note = self.0;
    }
);

define_modifier!(
    Velocity(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity += self.0;
    }
);

define_modifier!(
    VelocityMultiply(f32)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = ((ev.velocity as f32) * self.0) as u8;
    }
);

define_modifier!(
    VelocityFixed(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = self.0;
    }
);

define_modifier!(
    CtrlMap(u32, u32)
    fn modify_single(&self, ev: &mut Event) {
        if ev.ctrl == self.0 { ev.ctrl = self.1 };
    }
);

// Scene switching

pub struct SceneSwitch(pub SceneNum);
impl FilterTrait for SceneSwitch {
    fn run(&self, evs: &mut EventStream) {
        if evs.any() {
            if evs.scene.is_some() {
                evs.scene = Some(self.0);
            } else {
                // TODO warn about no scenes present
            }
        }
    }
}

pub struct SceneSwitchOffset(pub SceneNumOffset);
impl FilterTrait for SceneSwitchOffset {
    fn run(&self, evs: &mut EventStream) {
        if evs.any() {
            if let Some(scene) = evs.scene {
                evs.scene = Some((scene as SceneNumOffset).saturating_add(self.0) as SceneNum);
            } else {
                // TODO warn about no scenes present
            }
        }
    }
}

pub struct SubSceneSwitch(pub SceneNum);
impl FilterTrait for SubSceneSwitch {
    fn run(&self, evs: &mut EventStream) {
        if evs.any() {
            if evs.subscene.is_some() {
                evs.subscene = Some(self.0);
            } else {
                // TODO warn no subscenes present for the current scene
            }
        }
    }
}

pub struct SubSceneSwitchOffset(pub SceneNumOffset);
impl FilterTrait for SubSceneSwitchOffset {
    fn run(&self, evs: &mut EventStream) {
        if evs.any() {
            if let Some(subscene) = evs.subscene {
                evs.subscene = Some((subscene as SceneNumOffset).saturating_add(self.0) as SceneNum);
            } else {
                // TODO warn no subscenes present for the current scene
            }
        }
    }
}

pub struct Init<'a>(pub &'a dyn FilterTrait);
impl FilterTrait for Init<'_> {
    fn run(&self, _evs: &mut EventStream) {}
    fn run_init(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}

pub struct Exit<'a>(pub &'a dyn FilterTrait);
impl FilterTrait for Exit<'_> {
    fn run(&self, _evs: &mut EventStream) {}
    fn run_exit(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}

// Misc

pub struct Print();
impl FilterTrait for Print {
    fn run(&self, evs:  &mut EventStream) -> () {
        if evs.any() { println!("{}", evs.to_string()); }
    }
}

pub struct Pass();
impl FilterTrait for Pass {
    fn run(&self, _evs: &mut EventStream) -> () {
        // pass, which means: keep event stream as it iss
    }
}

pub struct Discard();
impl FilterTrait for Discard {
    fn run(&self, evs: &mut EventStream) -> () {
        evs.events.clear();
    }
}

pub struct NotBox<'a>(pub Box<dyn FilterTrait + 'a>);
impl FilterTrait for NotBox<'_> {
    fn run(&self, evs: &mut EventStream) {
        self.0.run_inverse(evs);
    }
    fn run_inverse(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}
#[macro_export]
macro_rules! Not {
    ( $f:expr ) => (
        NotBox(Box::new($f))
    )
}