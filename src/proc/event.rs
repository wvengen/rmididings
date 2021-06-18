#![allow(non_snake_case)]
use std::hash::{Hash, Hasher};

#[cfg(feature = "osc")]
extern crate rosc;
#[cfg(feature = "dbus")]
extern crate dbus;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Event<'a> {
    None(NoneEventImpl),
    NoteOn(NoteOnEventImpl),
    NoteOff(NoteOffEventImpl),
    Ctrl(CtrlEventImpl),
    SysEx(SysExEventImpl<'a>),
    SceneSwitch(SceneSwitchEventImpl),
    SubSceneSwitch(SubSceneSwitchEventImpl),
    Quit(QuitEventImpl),
    #[cfg(feature = "osc")]
    Osc(OscEventImpl),
    #[cfg(feature = "dbus")]
    Dbus(DbusEventImpl),
}
impl Default for Event<'_> {
    fn default() -> Self {
        Event::None(NoneEventImpl::default())
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct NoneEventImpl {}
pub fn NoneEvent<'a>() -> Event<'a> {
    Event::None(NoneEventImpl { })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct NoteOnEventImpl {
    pub port: usize,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}
pub fn NoteOnEvent<'a>(port: usize, channel: u8, note: u8, velocity: u8) -> Event<'a> {
    Event::NoteOn(NoteOnEventImpl { port, channel, note, velocity })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct NoteOffEventImpl {
    pub port: usize,
    pub channel: u8,
    pub note: u8,
}
pub fn NoteOffEvent<'a>(port: usize, channel: u8, note: u8) -> Event<'a> {
    Event::NoteOff(NoteOffEventImpl { port, channel, note })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct CtrlEventImpl {
    pub port: usize,
    pub channel: u8,
    pub ctrl: u32,
    pub value: i32,
}
pub fn CtrlEvent<'a>(port: usize, channel: u8, ctrl: u32, value: i32) -> Event<'a> {
    Event::Ctrl(CtrlEventImpl { port, channel, ctrl, value })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct SysExEventImpl<'a> {
    pub port: usize,
    pub data: &'a [u8],
}
pub fn SysExEvent<'a>(port: usize, data: &'a [u8]) -> Event<'a> {
    Event::SysEx(SysExEventImpl { port, data })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct QuitEventImpl {}
pub fn QuitEvent<'a>() -> Event<'a> {
    Event::Quit(QuitEventImpl { })
}

pub type SceneNum = u8;
pub type SceneOffset = i16; // large enough to do computation too

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum SceneSwitchValue {
    Fixed(SceneNum),
    Offset(SceneOffset),
}
impl Default for SceneSwitchValue {
    fn default() -> Self {
        SceneSwitchValue::Offset(0)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct SceneSwitchEventImpl {
    pub scene: SceneSwitchValue,
}
pub fn SceneSwitchEvent<'a>(scene: SceneNum) -> Event<'a> {
    Event::SceneSwitch(SceneSwitchEventImpl { scene: SceneSwitchValue::Fixed(scene) })
}
pub fn SceneSwitchOffsetEvent<'a>(offset: SceneOffset) -> Event<'a> {
    Event::SceneSwitch(SceneSwitchEventImpl { scene: SceneSwitchValue::Offset(offset) })
}

#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct SubSceneSwitchEventImpl {
    pub subscene: SceneSwitchValue,
}
pub fn SubSceneSwitchEvent<'a>(subscene: SceneNum) -> Event<'a> {
    Event::SubSceneSwitch(SubSceneSwitchEventImpl { subscene: SceneSwitchValue::Fixed(subscene) })
}
pub fn SubSceneSwitchOffsetEvent<'a>(offset: SceneOffset) -> Event<'a> {
    Event::SubSceneSwitch(SubSceneSwitchEventImpl { subscene: SceneSwitchValue::Offset(offset) })
}

#[cfg(feature = "osc")]
#[derive(Debug, Clone, PartialEq)]
pub struct OscEventImpl {
    pub port: usize,
    pub addr: String,
    pub args: Vec<rosc::OscType>,
}
impl OscEventImpl {
    fn hash_osc_type<H: Hasher>(&self, arg: &rosc::OscType, state: &mut H) {
        match arg {
            rosc::OscType::Int(ref x) => x.hash(state),
            rosc::OscType::Float(x) => ((x * 1e6) as u64).hash(state),
            rosc::OscType::String(ref x) => x.hash(state),
            rosc::OscType::Blob(ref x) => x.hash(state),
            rosc::OscType::Time(ref x) => x.hash(state),
            rosc::OscType::Long(ref x) => x.hash(state),
            rosc::OscType::Double(x) => ((x * 1e6) as u64).hash(state),
            rosc::OscType::Char(ref x) => x.hash(state),
            rosc::OscType::Color(ref x) => [x.red, x.green, x.blue, x.alpha].hash(state),
            rosc::OscType::Midi(ref x) => [x.port, x.status, x.data1, x.data2].hash(state),
            rosc::OscType::Bool(ref x) => x.hash(state),
            rosc::OscType::Array(ref x) => for el in x.content.iter() { self.hash_osc_type(el, state); },
            rosc::OscType::Nil => 0.hash(state),
            rosc::OscType::Inf => 1.hash(state),
        }
    }
}
// TODO get Hash, Eq support in rosc
#[cfg(feature = "osc")]
impl Hash for OscEventImpl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
        self.args.len().hash(state);
        for arg in self.args.iter() {
            self.hash_osc_type(&arg, state);
        }
    }
}
#[cfg(feature = "osc")]
impl Eq for OscEventImpl {}

#[cfg(feature = "osc")]
pub fn OscEvent<'a>(port: usize, addr: String, args: Vec<rosc::OscType>) -> Event<'a> {
    Event::Osc(OscEventImpl { port, addr, args })
}

#[cfg(feature = "osc")]
impl From<rosc::OscMessage> for Event<'_> {
    fn from(message: rosc::OscMessage) -> Self {
        OscEvent(0, message.addr, message.args)
    }
}

#[cfg(feature = "dbus")]
#[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub struct DbusEventImpl {
    pub service: String,
    pub path: String,
    pub interface: String,
    pub method: String,
    pub args: Vec<dbus::arg::ArgType>
}
#[cfg(feature = "dbus")]
pub fn DbusEvent<'a>(service: String, path: String, interface: String, method: String, args: Vec<dbus::arg::ArgType>) -> Event<'a> {
    Event::Dbus(DbusEventImpl { service, path, interface, method, args })
}