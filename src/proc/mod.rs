#![allow(non_snake_case)]
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
    #[doc(hidden)]
    _TypeMidiFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(_) => true,
            Event::NoteOff(_) => true,
            Event::Ctrl(_) => true,
            Event::SysEx(_) => true,
            _ => false,
        }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeNoteFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(_) => true,
            Event::NoteOff(_) => true,
            _ => false,
        }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeNoteOnFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::NoteOn(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeNoteOffFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::NoteOff(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeCtrlFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::Ctrl(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeSysExFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::SysEx(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeNoneFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::None(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeQuitFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::Quit(_) = ev { true } else { false }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeSceneSwitchFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::SceneSwitch(_) => true,
            _ => false,
        }
    }
);
define_filter!(
    #[doc(hidden)]
    _TypeSubSceneSwitchFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::SubSceneSwitch(_) => true,
            _ => false,
        }
    }
);
#[cfg(feature = "osc")]
define_filter!(
    #[doc(hidden)]
    _TypeOscFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::Osc(_) = ev { true } else { false }
    }
);
#[cfg(feature = "dbus")]
define_filter!(
    #[doc(hidden)]
    _TypeDbusFilter()
    fn filter_single(&self, ev: &Event) -> bool {
        if let Event::Dbus(_) = ev { true } else { false }
    }
);

/// Filter on event type
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypeFilter!(Midi);
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// filter.run(&mut evs);
/// assert_eq!(evs, NoteOnEvent(0,0,60,20));
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypeFilter!(Note);
///
/// let mut evs = EventStream::from(vec![NoteOnEvent(0,0,60,20), CtrlEvent(0,0,7,20)]);
/// filter.run(&mut evs);
/// assert_eq!(evs, NoteOnEvent(0,0,60,20));
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypeFilter!(NoteOn);
///
/// let mut evs = EventStream::from(vec![NoteOnEvent(0,0,60,20), NoteOffEvent(0,0,60)]);
/// filter.run(&mut evs);
/// assert_eq!(evs, NoteOnEvent(0,0,60,20));
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypeFilter!(NoteOff);
///
/// let mut evs = EventStream::from(vec![NoteOnEvent(0,0,60,20), NoteOffEvent(0,0,60)]);
/// filter.run(&mut evs);
/// assert_eq!(evs, NoteOffEvent(0,0,60));
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypeFilter!(Ctrl);
///
/// let mut evs = EventStream::from(vec![NoteOnEvent(0,0,60,20), CtrlEvent(0,0,7,20)]);
/// filter.run(&mut evs);
/// assert_eq!(evs, CtrlEvent(0,0,7,20));
/// # }
/// ```
#[macro_export]
macro_rules! TypeFilter {
    (Midi) => { _TypeMidiFilter() };
    (Note) => { _TypeNoteFilter() };
    (NoteOn) => { _TypeNoteOnFilter() };
    (NoteOff) => { _TypeNoteOffFilter() };
    (Ctrl) => { _TypeCtrlFilter() };
    (SysEx) => { _TypeSysExFilter() };
    (Quit) => { _TypeQuitFilter() };
    (SceneSwitch) => { _TypeSceneSwitchFilter() };
    (Osc) => { _TypeOscFilter() };
    (Dbus) => { _TypeDbusFilter() };
}

/// Filter on multiple event types
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = TypesFilter!(Note, Ctrl);
///
/// let ev1 = NoteOnEvent(0,0,60,20);
/// let ev2 = CtrlEvent(0,0,7,60);
/// let ev3 = SceneSwitchEvent(2);
///
/// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3]);
/// filter.run(&mut evs);
/// assert_eq!(evs, vec![ev1, ev2]);
/// # }
/// ```
#[macro_export]
macro_rules! TypesFilter {
    ( $( $ty:ident ),+ ) => {
        Fork!( $( TypeFilter!($ty) ),+ )
    }
}

define_filter!(
    /// Filter on port number
    ///
    /// When calling [`RMididings.config()`] the `in_ports` and `out_ports`
    /// are arrays that indicate which MIDI ports to create. The index in
    /// these arrays are the port number (starting with index 0).
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = PortFilter(1);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(1,0,60,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev2)
    /// ```
    PortFilter(usize)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => ev.port == self.0,
            Event::NoteOff(ev) => ev.port == self.0,
            Event::Ctrl(ev) => ev.port == self.0,
            Event::SysEx(ev) => ev.port == self.0,
            #[cfg(feature = "osc")]
            Event::Osc(ev) => ev.port == self.0,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on multiple port numbers
    ///
    /// When calling [`RMididings.config()`] the `in_ports` and `out_ports`
    /// are arrays that indicate which MIDI ports to create. The index in
    /// these arrays are the port number (starting with index 0).
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = PortsFilter(&[1,2]);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(1,0,60,20);
    /// let ev3 = NoteOnEvent(2,0,60,20);
    /// let ev4 = NoteOnEvent(4,0,60,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3, &ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev2, ev3]);
    /// ```
    PortsFilter(&'static [usize])
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => self.0.contains(&ev.port),
            Event::NoteOff(ev) => self.0.contains(&ev.port),
            Event::Ctrl(ev) => self.0.contains(&ev.port),
            Event::SysEx(ev) => self.0.contains(&ev.port),
            #[cfg(feature = "osc")]
            Event::Osc(ev) => self.0.contains(&ev.port),
            _ => true,
        }
    }

);

define_filter!(
    /// Filter on channel
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = ChannelFilter(1);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(0,1,60,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev2);
    /// ```
    ChannelFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => ev.channel == self.0,
            Event::NoteOff(ev) => ev.channel == self.0,
            Event::Ctrl(ev) => ev.channel == self.0,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on multiple channels
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = ChannelsFilter(&[2,3]);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(0,1,60,20);
    /// let ev3 = NoteOnEvent(0,2,60,20);
    /// let ev4 = NoteOnEvent(0,3,60,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3, &ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev3, ev4]);
    /// ```
    ChannelsFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => self.0.contains(&ev.channel),
            Event::NoteOff(ev) => self.0.contains(&ev.channel),
            Event::Ctrl(ev) => self.0.contains(&ev.channel),
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on key (note)
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = KeyFilter(60);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(0,0,61,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev1);
    /// ```
    KeyFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => ev.note == self.0,
            Event::NoteOff(ev) => ev.note == self.0,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on multiple keys (notes)
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = KeysFilter(&[60, 63]);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(0,0,61,20);
    /// let ev3 = NoteOnEvent(0,0,62,20);
    /// let ev4 = NoteOnEvent(0,0,63,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3, &ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev1, ev4]);
    /// ```
    KeysFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => self.0.contains(&ev.note),
            Event::NoteOff(ev) => self.0.contains(&ev.note),
            _ => true,
        }

    }
);

define_filter!(
    /// Filter on a range of keys (notes)
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = KeyRangeFilter(60, 62);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOnEvent(0,0,61,20);
    /// let ev3 = NoteOnEvent(0,0,62,20);
    /// let ev4 = NoteOnEvent(0,0,63,20);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3, &ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev1, ev2, ev3]);
    /// ```
    KeyRangeFilter(u8, u8)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::NoteOn(ev) => ev.note >= self.0 && ev.note <= self.1,
            Event::NoteOff(ev) => ev.note >= self.0 && ev.note <= self.1,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on controller (CC)
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = CtrlFilter(7);
    ///
    /// let ev1 = CtrlEvent(0,0,7,40);
    /// let ev2 = CtrlEvent(0,0,8,40);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev1);
    /// ```
    CtrlFilter(u32)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Ctrl(ev) => ev.ctrl == self.0,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter multiple controllers (CC)
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = CtrlsFilter(&[7, 8]);
    ///
    /// let ev1 = CtrlEvent(0,0,7,40);
    /// let ev2 = CtrlEvent(0,0,8,40);
    /// let ev3 = CtrlEvent(0,0,9,40);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev1, ev2]);
    /// ```
    CtrlsFilter(&'static [u32])
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Ctrl(ev) => self.0.contains(&ev.ctrl),
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on a controller (CC) value
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = CtrlValueFilter(0);
    ///
    /// let ev1 = CtrlEvent(0,0,7,0);
    /// let ev2 = CtrlEvent(0,0,7,80);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev1);
    /// ```
    CtrlValueFilter(i32)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Ctrl(ev) => ev.value == self.0,
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on multiple controller (CC) values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = CtrlValuesFilter(&[0,1]);
    ///
    /// let ev1 = CtrlEvent(0,0,7,0);
    /// let ev2 = CtrlEvent(0,0,7,1);
    /// let ev3 = CtrlEvent(0,0,7,2);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev1, ev2]);
    /// ```
    CtrlValuesFilter(&'static[i32])
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Ctrl(ev) => self.0.contains(&ev.value),
            _ => true,
        }
    }
);

define_filter!(
    /// Filter on a range of controller (CC) values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = CtrlValueRangeFilter(0, 20);
    ///
    /// let ev1 = CtrlEvent(0,0,7,0);
    /// let ev2 = CtrlEvent(0,0,7,10);
    /// let ev3 = CtrlEvent(0,0,7,50);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2, &ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, vec![ev1, ev2]);
    /// ```
    CtrlValueRangeFilter(i32, i32)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Ctrl(ev) => ev.value >= self.0 && ev.value <= self.1,
            _ => true,
        }
    }
);

// // Generators

define_generator!(
    /// Generate a NoteOn event.
    ///
    /// The arguments are: _note_, _velocity_.
    ///
    /// Port and channel are set to `0`, you can use the modifiers
    /// [Port] and [Channel] so change them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = NoteOn(60, 20);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0, 0, 60, 20))
    /// ```
    NoteOn(u8, u8)
    fn generate_single(&self) -> Event<'static> {
        NoteOnEvent(0, 0, self.0, self.1)
    }
);

define_generator!(
    /// Generate a NoteOff event.
    ///
    /// The argument is: _note_.
    ///
    /// Port and channel are set to `0`, you can use the modifiers
    /// [Port] and [Channel] so change them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = NoteOff(65);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, NoteOffEvent(0, 0, 65))
    /// ```
    NoteOff(u8)
    fn generate_single(&self) -> Event<'static> {
        NoteOffEvent(0, 0, self.0)
    }
);

define_generator!(
    /// Generate a controller (CC) event.
    ///
    /// The argument is: _ctrl_, _value_.
    ///
    /// Port and channel are set to `0`, you can use the modifiers
    /// [Port] and [Channel] so change them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = Ctrl(7, 40);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, CtrlEvent(0, 0, 7, 40));
    /// ```
    Ctrl(u32, i32)
    fn generate_single(&self) -> Event<'static> {
        CtrlEvent(0, 0, self.0, self.1)
    }
);

define_generator!(
    /// Generate a system exclusive event.
    ///
    /// The argument is: _sysex message_.
    ///
    /// Port and channel are set to `0`, you can use the modifiers
    /// [Port] and [Channel] so change them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = SysEx(&[0xf7, 0xf0]);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, SysExEvent(0, &[0xf7, 0xf0]));
    /// ```
    SysEx(&'static [u8])
    fn generate_single(&self) -> Event<'static> {
        SysExEvent(0, self.0)
    }
);

// // Modifiers

define_modifier!(
    /// Modify the port to a set value.
    ///
    /// The argument is: _port_.
    ///
    /// When calling [`RMididings.config()`] the `in_ports` and `out_ports`
    /// are arrays that indicate which MIDI ports to create. The index in
    /// these arrays are the port number (starting with index 0).
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Port(1);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(1,0,60,20));
    /// ```
    Port(usize)
    fn modify_single(&self, ev:&mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.port = self.0,
            Event::NoteOff(ev) => ev.port = self.0,
            Event::Ctrl(ev) => ev.port = self.0,
            Event::SysEx(ev) => ev.port = self.0,
            #[cfg(feature = "osc")]
            Event::Osc(ev) => ev.port = self.0,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modify the channel to a set value.
    ///
    /// The argument is: _channel_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Channel(1);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,1,60,20));
    /// ```
    Channel(u8)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.channel = self.0,
            Event::NoteOff(ev) => ev.channel = self.0,
            Event::Ctrl(ev) => ev.channel = self.0,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modify the key (note) by a number of semitones.
    ///
    /// The argument is: _semitones_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Transpose(4);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,64,20));
    /// ```
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Transpose(-4);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,56,20));
    /// ```
    Transpose(i16)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.note = (ev.note as i16).saturating_add(self.0) as u8,
            Event::NoteOff(ev) => ev.note = (ev.note as i16).saturating_add(self.0) as u8,
            _ => {},
        }
    }
);

/// Modify the key (note) by an number of octaves.
///
/// The argument is: _octaves_.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = TransposeOctave(1);
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// modifier.run(&mut evs);
/// assert_eq!(evs, NoteOnEvent(0,0,72,20));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = TransposeOctave(-1);
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// modifier.run(&mut evs);
/// assert_eq!(evs, NoteOnEvent(0,0,48,20));
/// ```
pub fn TransposeOctave(octaves: i16) -> Transpose {
    Transpose(octaves * 12)
}

define_modifier!(
    /// Modify the key (note) to a set value.
    ///
    /// The argument is: _key_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Key(68);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,68,20));
    /// ```
    Key(u8)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.note = self.0,
            Event::NoteOff(ev) => ev.note = self.0,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modify the note velocity by an amount.
    ///
    /// The argument is: _offset_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Velocity(10);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,40));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,60,50));
    /// ```
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Velocity(-10);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,40));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,60,30));
    /// ```
    Velocity(i16)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.velocity = (ev.velocity as i16).saturating_add(self.0) as u8,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modify the note velocity by a multiplication factor.
    ///
    /// The argument is: _factor_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = VelocityMultiply(0.5);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,40));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,60,20));
    /// ```
    VelocityMultiply(f32)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.velocity = ((ev.velocity as f32) * self.0) as u8,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modify the note velocity to a set value.
    ///
    /// The argument is: _velocity_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Velocity(10);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,40));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, NoteOnEvent(0,0,60,50));
    /// ```
    VelocityFixed(u8)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => ev.velocity = self.0,
            _ => {},
        }
    }
);

define_modifier!(
    /// Modifies the controller number (CC), changing one for another.
    ///
    /// The arguments are: _from_ctrl_ and _to_ctrl_.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = CtrlMap(7, 8);
    ///
    /// let mut evs = EventStream::from(CtrlEvent(0,0,7,50));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, CtrlEvent(0,0,8,50));
    /// ```
    CtrlMap(u32, u32)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::Ctrl(ev) if ev.ctrl == self.0 => ev.ctrl = self.1,
            _ => {}
        }
    }
);

// // Scene switching

/// Switches to a specific scene.
///
/// The argument is: _scene_number_.
///
/// This event consumes all other events, so after this filter
/// only the curent scene switch remains.
///
/// Note that the scene is only switched when there are events, so
/// that when an event filter discards all events, the scene switch
/// is not done. It also means that you need to generate an event
/// when putting this in a pre, init, exit or post patch.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let generator = SceneSwitch(5);
///
/// let mut evs = EventStream::none();
/// generator.run(&mut evs);
/// assert_eq!(evs, SceneSwitchEvent(5));
/// ```
pub struct SceneSwitch(pub SceneNum);
impl FilterTrait for SceneSwitch {
    fn run(&self, evs: &mut EventStream) {
        if evs.is_empty() { return; }
        TypeFilter!(SceneSwitch).run(evs);
        evs.push(SceneSwitchEvent(self.0));
    }
}

define_generator!(
    /// Change the current scene by the specified amount.
    ///
    /// The argument is: _scene_delta_.
    ///
    /// To go to the next scene, use `SceneSwitchOffset(1)`,
    /// to go to the previous scene, use `SceneSwitchOffset(-1)`.
    ///
    /// Note that the scene is only switched when there are events, so
    /// that when an event filter discards all events, the scene switch
    /// is not done. It also means that you need to generate an event
    /// when putting this in a pre, init, exit or post patch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = SceneSwitchOffset(1);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, SceneSwitchOffsetEvent(1));
    /// ```
    SceneSwitchOffset(SceneOffset)
    fn generate_single(&self) -> Event<'static> {
        SceneSwitchOffsetEvent(self.0)
    }
);

define_generator!(
    /// Switches to a specific subscene.
    ///
    /// The argument is: _subscene_number_.
    ///
    /// Note that the subscene is only switched when there are events, so
    /// that when an event filter discards all events, the subscene switch
    /// is not done. It also means that you need to generate an event
    /// when putting this in a pre, init, exit or post patch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = SubSceneSwitch(5);
    ///
    /// let mut evs = EventStream::none();
    /// modifier.run(&mut evs);
    /// assert_eq!(evs, SubSceneSwitchEvent(5));
    /// ```
    SubSceneSwitch(SceneNum)
    fn generate_single(&self) -> Event<'static> {
        SubSceneSwitchEvent(self.0)
    }
);

define_generator!(
    /// Change the current subscene by the specified amount.
    ///
    /// The argument is: _subscene_delta_.
    ///
    /// To go to the next scene, use `SubSceneSwitchOffset(1)`,
    /// to go to the previous scene, use `SubSceneSwitchOffset(-1)`.
    ///
    /// Note that the subscene is only switched when there are events, so
    /// that when an event filter discards all events, the subscene switch
    /// is not done. It also means that you need to generate an event
    /// when putting this in a pre, init, exit or post patch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let generator = SubSceneSwitchOffset(1);
    ///
    /// let mut evs = EventStream::none();
    /// generator.run(&mut evs);
    /// assert_eq!(evs, SubSceneSwitchOffsetEvent(1));
    /// ```
    SubSceneSwitchOffset(SceneOffset)
    fn generate_single(&self) -> Event<'static> {
        SubSceneSwitchOffsetEvent(self.0)
    }
);

#[doc(hidden)]
pub struct _Init<'a>(pub Box<dyn FilterTrait + 'a>);
#[doc(hidden)]
impl FilterTrait for _Init<'_> {
    fn run(&self, _evs: &mut EventStream) {}
    fn run_init(&self, evs: &mut EventStream) {
        self.0.run(evs);
    }
}
/// Run contained filters on (sub)scene or patch init.
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
/// Run contained filters on (sub)scene or patch exit.
#[macro_export]
macro_rules! Exit {
    ( $f:expr ) => {
        _Exit(Box::new($f))
    };
}

// // Misc

/// Prints the current events.
pub struct Print();
impl FilterTrait for Print {
    fn run(&self, evs: &mut EventStream) {
        if !evs.is_empty() {
            println!("{:?}", evs);
        }
    }
}

/// Quit mididings
///
/// This event consumes all other events, so after this filter
/// only the quit event remains.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let generator = Quit();
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// generator.run(&mut evs);
/// assert_eq!(evs, QuitEvent());
/// ```
pub struct Quit();
impl FilterTrait for Quit {
    fn run(&self, evs: &mut EventStream) {
        if !evs.is_empty() {
            evs.clear();
            evs.push(QuitEvent());
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
/// assert_eq!(evs.len(), 1);
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
/// assert!(evs.is_empty());
/// # }
/// ```
pub struct Pass();
impl FilterTrait for Pass {
    fn run(&self, _evs: &mut EventStream) {
        // pass, which means: keep event stream as it is
    }

    fn run_inverse(&self, evs: &mut EventStream) {
        evs.clear();
    }
}

/// Discard all events.
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
/// assert!(evs.is_empty());
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
/// assert_eq!(evs.len(), 1);
/// # }
/// ```
pub struct Discard();
impl FilterTrait for Discard {
    fn run(&self, evs: &mut EventStream) {
        evs.clear();
    }

    fn run_inverse(&self, _evs: &mut EventStream) {
        // pass, which means: keep event stream as it is
    }
}

/// Send MIDI panic
///
/// Sends all notes off (CC#123) and sustain off (CC#64) on all channels.
///
/// Note that, in contrast to mididings, the events are subject to port
/// selection, so if you have multiple ports, send multiple MIDI panic
/// events (one to each port).
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let generator = Panic();
///
/// let mut evs = EventStream::empty();
/// generator.run(&mut evs);
///
/// assert_eq!(evs.len(), 32);
/// ```
pub struct Panic();
impl FilterTrait for Panic {
    fn run(&self, evs: &mut EventStream) {
        evs.extend((0..16).map(|c| CtrlEvent(0, c, 123, 0)));
        evs.extend((0..16).map(|c| CtrlEvent(0, c,  64, 0)));
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
/// let ev1 = NoteOnEvent(0,0,60,20);
/// let ev2 = NoteOnEvent(0,0,61,20);
///
/// let mut evs = EventStream::from(vec![&ev1, &ev2]);
/// filter.run(&mut evs);
/// assert_eq!(evs, ev2);
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
/// assert!(!evs.is_empty());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f2.run(&mut evs);
/// assert!(evs.is_empty());
///
/// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
/// f3.run(&mut evs);
/// assert!(!evs.is_empty());
/// # }
/// ```
#[macro_export]
macro_rules! Not {
    ( $f:expr ) => {
        _Not(Box::new($f))
    };
}

/// Process the incoming event using a custom function, returning a patch.
///
/// Any other processing will be stalled until function returns, so this should only be used with
/// functions that don’t block.
// pub struct Process<'a>(dyn Fn(&Event) -> FilterChain<'a>);
// impl FilterTrait for Process<'_> {
//     fn run(&self, evs: &mut EventStream) {
//         let filters: Vec<Box<dyn FilterTrait>> = vec![];

//         for ev in evs.iter() {
//             filters.push(Box::new(self.0(&ev)));
//         }

//         for f in filters {
//             f.run(evs);
//         }
//     }
// }

#[cfg(feature = "osc")]
pub mod osc;
#[cfg(feature = "osc")]
pub use osc::*;