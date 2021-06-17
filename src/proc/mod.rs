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
    /// Filter on event type
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = Filter(EventType::NOTEON);
    ///
    /// let ev1 = NoteOnEvent(0,0,60,20);
    /// let ev2 = NoteOffEvent(0,0,61);
    ///
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0], ev1);
    /// ```
    Filter(EventType)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.typ == self.0
    }
);

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
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0], ev2);
    /// ```
    PortFilter(usize)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.port == self.0
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev2, ev3]);
    /// ```

    PortsFilter(&'static [usize])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.port)
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0], ev2);
    /// ```
    ChannelFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.channel == self.0
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev3, ev4]);
    /// ```
    ChannelsFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.channel)
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0], ev1);
    /// ```
    KeyFilter(u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.note == self.0
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1, ev4]);
    /// ```
    KeysFilter(&'static [u8])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.note)
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1, ev2, ev3]);
    /// ```
    KeyRangeFilter(u8, u8)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.note >= self.0 && ev.note <= self.1
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1]);
    /// ```
    CtrlFilter(u32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.ctrl == self.0
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1, ev2]);
    /// ```
    CtrlsFilter(&'static [u32])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.ctrl)
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1]);
    /// ```
    CtrlValueFilter(i32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.value == self.0
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1, ev2]);
    /// ```
    CtrlValuesFilter(&'static[i32])
    fn filter_single(&self, ev: &Event) -> bool {
        self.0.contains(&ev.value)
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
    /// let mut evs = EventStream::from(&vec![ev1, ev2, ev3]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs.events.to_vec(), vec![ev1, ev2]);
    /// ```
    CtrlValueRangeFilter(i32, i32)
    fn filter_single(&self, ev: &Event) -> bool {
        ev.value >= self.0 && ev.value <= self.1
    }
);

// Generators

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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].typ, EventType::NOTEON);
    /// assert_eq!(evs.events[0].note, 60);
    /// assert_eq!(evs.events[0].velocity, 20);
    /// ```
    NoteOn(u8, u8)
    fn generate_single(&self) -> Event {
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].typ, EventType::NOTEOFF);
    /// assert_eq!(evs.events[0].note, 65);
    /// ```
    NoteOff(u8)
    fn generate_single(&self) -> Event {
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].typ, EventType::CTRL);
    /// assert_eq!(evs.events[0].ctrl, 7);
    /// assert_eq!(evs.events[0].value, 40);
    /// ```
    Ctrl(u32, i32)
    fn generate_single(&self) -> Event {
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].typ, EventType::SYSEX);
    /// assert_eq!(evs.events[0].sysex.to_vec(), vec![0xf7, 0xf0]);
    /// ```
    SysEx(&'static [u8])
    fn generate_single(&self) -> Event {
        SysExEvent(0, self.0)
    }
);

// Modifiers

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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].port, 1);
    /// ```
    Port(usize)
    fn modify_single(&self, ev:&mut  Event) {
        ev.port = self.0;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].channel, 1);
    /// ```
    Channel(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.channel = self.0;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].note, 64);
    /// ```
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Transpose(-4);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].note, 56);
    /// ```
    Transpose(i16)
    fn modify_single(&self, ev: &mut Event) {
        ev.note = (ev.note as i16).saturating_add(self.0) as u8;
    }
);

define_modifier!(
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].note, 72);
    /// ```
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = TransposeOctave(-1);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,20));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].note, 48);
    /// ```
    TransposeOctave(i16)
    fn modify_single(&self, ev: &mut Event) {
        ev.note = (ev.note as i16).saturating_add(self.0 * 12) as u8;
    }
);

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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].note, 68);
    /// ```
    Key(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.note = self.0;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].velocity, 50);
    /// ```
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let modifier = Velocity(-10);
    ///
    /// let mut evs = EventStream::from(NoteOnEvent(0,0,60,40));
    /// modifier.run(&mut evs);
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].velocity, 30);
    /// ```
    Velocity(i16)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = (ev.velocity as i16).saturating_add(self.0) as u8;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].velocity, 20);
    /// ```
    VelocityMultiply(f32)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = ((ev.velocity as f32) * self.0) as u8;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].velocity, 50);
    /// ```
    VelocityFixed(u8)
    fn modify_single(&self, ev: &mut Event) {
        ev.velocity = self.0;
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
    /// assert_eq!(evs.events.len(), 1);
    /// assert_eq!(evs.events[0].ctrl, 8);
    /// ```
    CtrlMap(u32, u32)
    fn modify_single(&self, ev: &mut Event) {
        if ev.ctrl == self.0 { ev.ctrl = self.1 };
    }
);

// Scene switching

/// Switches to a specific scene.
///
/// The argument is: _scene_number_.
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
/// let modifier = SceneSwitch(5);
///
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.scene, Some(5));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SceneSwitch(5);
///
/// let mut evs = EventStream::none();
/// evs.scene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.scene, Some(2));
/// ```
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

/// Change the current scene by the specified amount.
///
/// The argument is: _scene_delta.
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
/// let modifier = SceneSwitchOffset(1);
///
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.scene, Some(3));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SceneSwitchOffset(-1);
///
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.scene, Some(1));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SceneSwitchOffset(1);
///
/// let mut evs = EventStream::none();
/// evs.scene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.scene, Some(2));
/// ```
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
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(0);
/// evs.subscene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.subscene, Some(5));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SubSceneSwitch(5);
///
/// let mut evs = EventStream::none();
/// evs.scene = Some(0);
/// evs.subscene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.subscene, Some(2));
/// ```
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

/// Change the current subscene by the specified amount.
///
/// The argument is: _subscene_delta.
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
/// let modifier = SubSceneSwitchOffset(1);
///
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(0);
/// evs.subscene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.subscene, Some(3));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SubSceneSwitchOffset(-1);
///
/// let mut evs = EventStream::from(Event::new(EventType::NOTEON));
/// evs.scene = Some(0);
/// evs.subscene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.subscene, Some(1));
/// ```
///
/// ```
/// # use rmididings::proc::*;
/// let modifier = SubSceneSwitchOffset(1);
///
/// let mut evs = EventStream::none();
/// evs.scene = Some(0);
/// evs.subscene = Some(2);
/// modifier.run(&mut evs);
/// assert_eq!(evs.subscene, Some(2));
/// ```
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

// Misc

/// Prints the current events.
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
/// let ev1 = NoteOnEvent(0,0,60,20);
/// let ev2 = NoteOnEvent(0,0,61,20);
///
/// let mut evs = EventStream::from(&vec![ev1, ev2]);
/// filter.run(&mut evs);
/// assert_eq!(evs.events.len(), 1);
/// assert_eq!(evs.events[0], ev2);
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