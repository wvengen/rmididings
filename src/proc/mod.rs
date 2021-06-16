#![macro_use]
pub mod event;
pub mod event_stream;
pub mod filter_chain;
pub mod filter_trait;
pub use self::event::*;
pub use self::event_stream::*;
pub use self::filter_chain::*;
pub use self::filter_trait::*;

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
                evs.subscene =
                    Some((subscene as SceneNumOffset).saturating_add(self.0) as SceneNum);
            } else {
                // TODO warn no subscenes present for the current scene
            }
        }
    }
}

#[doc(hidden)]
pub struct _Init<'a>(pub Box<dyn FilterTrait + 'a>);
#[doc(hidden)]
impl FilterTrait for _Init<'_> {
    fn run(&self, _evs: &mut EventStream) {}
    fn run_init(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}
#[macro_export]
macro_rules! Init {
    ( $f:expr ) => {
        _Init(Box::new($f))
    };
}

#[doc(hidden)]
pub struct _Exit<'a>(pub Box<dyn FilterTrait + 'a>);
#[doc(hidden)]
impl FilterTrait for _Exit<'_> {
    fn run(&self, _evs: &mut EventStream) {}
    fn run_exit(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}
#[macro_export]
macro_rules! Exit {
    ( $f:expr ) => {
        _Exit(Box::new($f))
    };
}

// Misc

pub struct Print();
impl FilterTrait for Print {
    fn run(&self, evs: &mut EventStream) {
        if evs.any() {
            println!("{}", evs.to_string());
        }
    }
}

/// Pass all events, i.e. a no-op.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let f = Pass();
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f.run(&mut evs);
///
/// assert_eq!(evs.events.len(), 1);
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let f = Not!(Pass());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f.run(&mut evs);
///
/// assert!(evs.events.is_empty());
/// # }
/// ```
pub struct Pass();
impl FilterTrait for Pass {
    fn run(&self, _evs: &mut EventStream) {
        // pass, which means: keep event stream as it is
    }

    fn run_inverse(&self, evs: &mut EventStream) {
        evs.events.clear();
    }
}

/// Discards all events.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let f = Discard();
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f.run(&mut evs);
///
/// assert!(evs.events.is_empty());
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let f = Not!(Discard());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f.run(&mut evs);
///
/// assert_eq!(evs.events.len(), 1);
/// # }
/// ```
pub struct Discard();
impl FilterTrait for Discard {
    fn run(&self, evs: &mut EventStream) {
        evs.events.clear();
    }

    fn run_inverse(&self, _evs: &mut EventStream) {
        // pass, which means: keep event stream as it is
    }
}

#[doc(hidden)]
pub struct _Not<'a>(pub Box<dyn FilterTrait + 'a>);
#[doc(hidden)]
impl FilterTrait for _Not<'_> {
    fn run(&self, evs: &mut EventStream) {
        self.0.run_inverse(evs);
    }
    fn run_inverse(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}

/// Inverses the effect of filters.
///
/// The `Not!()` macro accepts a single argument, which is another [FilterTrait].
/// The behavior of modifiers and generators is unchanged.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = Not!(KeyFilter(60));
///
/// let event1 = NoteOnEvent(0,0,60,20);
/// let event2 = NoteOnEvent(0,0,61,20);
///
/// let mut evs = EventStream::from(&vec![event1, event2]);
/// filter.run(&mut evs);
/// assert_eq!(evs.events.len(), 1);
/// assert_eq!(evs.events[0], event2);
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let f1 = KeyFilter(60);
/// let f2 = Not!(KeyFilter(60));
/// let f3 = Not!(Not!(KeyFilter(60)));
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f1.run(&mut evs);
/// assert!(!evs.events.is_empty());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f2.run(&mut evs);
/// assert!(evs.events.is_empty());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f3.run(&mut evs);
/// assert!(!evs.events.is_empty());
/// # }
/// ```
#[macro_export]
macro_rules! Not {
    ( $f:expr ) => {
        _Not(Box::new($f))
    };
}
