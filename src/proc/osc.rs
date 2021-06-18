use crate::proc::event::*;
use crate::proc::filter_trait::*;
use crate::proc::event_stream::*;

define_generator!(
    #[doc(hidden)]
    _Osc(String, Vec<rosc::OscType>)
    fn generate_single(&self) -> Event<'static> {
        OscEvent(0, self.0.clone(), self.1.clone())
    }
);

/// Generates an OSC message.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let filter = Osc!("/foo");
///
/// let mut evs = EventStream::empty();
/// filter.run(&mut evs);
/// assert_eq!(evs, OscEvent(0, "/foo".to_string(), vec![]));
/// # }
/// ```
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// use rosc::OscType as o;
///
/// # fn main() {
/// let filter = Osc!("/bar", o::Int(5), o::String("yes".to_string()));
///
/// let mut evs = EventStream::empty();
/// filter.run(&mut evs);
/// assert_eq!(evs, OscEvent(0, "/bar".to_string(), vec![o::Int(5), o::String("yes".to_string())]));
/// # }
/// ```
#[macro_export]
macro_rules! Osc {
    ( $msg:expr ) => {
        _Osc(String::from($msg), vec![])
    };
    ( $msg:expr, $( $arg:expr ),+ ) => {
        _Osc(String::from($msg), vec![ $($arg),+ ])
    }
}

define_filter!(
    /// Filter on OSC address
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = OscAddrFilter(&"/foo");
    ///
    /// let ev1 = OscEvent(0, "/foo".to_string(), vec![]);
    /// let ev2 = OscEvent(0, "/bar".to_string(), vec![]);
    ///
    /// let mut evs = EventStream::from(vec![&ev1, &ev2]);
    /// filter.run(&mut evs);
    /// assert_eq!(evs, ev1);
    /// ```
    OscAddrFilter(&'static str)
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Osc(ev) => ev.addr == self.0,
            _ => true,
        }
    }
);

/// Filters OSC messages on an address prefix and strips the prefix from the address.
///
/// # Examples
///
/// ```
/// # use rmididings::proc::*;
/// let filter = OscStripPrefix("/coolapp");
///
/// let ev1 = OscEvent(0, "/foo".to_string(), vec![]);
/// let ev2 = OscEvent(0, "/coolapp/bar".to_string(), vec![]);
///
/// let mut evs = EventStream::from(vec![&ev1, &ev2]);
/// filter.run(&mut evs);
/// assert_eq!(evs, OscEvent(0, "/bar".to_string(), vec![]));
/// ```
pub struct OscStripPrefix(pub &'static str);
impl OscStripPrefix {
    fn filter_single(&self, ev: &Event) -> bool {
        match ev {
            Event::Osc(ev) => ev.addr.starts_with(self.0),
            _ => true,
        }
    }

    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::Osc(ev) => ev.addr = ev.addr.strip_prefix(self.0).unwrap().to_string(),
            _ => {},
        }
    }
}
impl FilterTrait for OscStripPrefix {
    fn run(&self, evs: &mut EventStream) {
        evs.retain(|ev| self.filter_single(&ev));
        for ev in evs.iter_mut() {
            self.modify_single(ev);
        }
    }
}

define_modifier!(
    /// Adds an address prefix to OSC messages
    ///
    /// # Examples
    ///
    /// ```
    /// # use rmididings::proc::*;
    /// let filter = OscAddPrefix("/coolapp");
    ///
    /// let ev = OscEvent(0, "/bar".to_string(), vec![]);
    ///
    /// let mut evs = EventStream::from(ev);
    /// filter.run(&mut evs);
    ///
    /// assert_eq!(evs, OscEvent(0, "/coolapp/bar".to_string(), vec![]));
    /// ```
    OscAddPrefix(&'static str)
    fn modify_single(&self, ev: &mut Event) {
        match ev {
            Event::Osc(ev) => ev.addr = self.0.to_string() + &ev.addr,
            _ => {},
        }
    }
);

#[doc(hidden)]
pub struct _ProcessOsc(pub Box<dyn Fn(&Vec<rosc::OscType>) -> Box<dyn FilterTrait>>);
impl FilterTrait for _ProcessOsc {
    fn run(&self, evs: &mut EventStream) {
        let mut filters: Vec<Box<dyn FilterTrait>> = vec![];

        for ev in evs.iter_mut() {
            match ev {
                Event::Osc(OscEventImpl { port: _, addr: _, args }) => {
                    filters.push(self.0(args));
                    // Remove event (by making it a NoneEvent).
                    *ev = NoneEvent();
                },
                _ => {},
            }
        }

        for f in filters {
            f.run(evs);
        }
    }

    // TODO run inverse, what would that mean?
}

/// Process the incoming OSC event using a function, which returns a patch to run.
///
/// Note that matching OSC events are removed from the event stream.
/// A maximum of eight arguments is currently supported (please open an issue if you need more).
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// use rosc::OscType as o;
///
/// # fn main() {
/// let filter = Chain!(OscAddrFilter("/foo"), ProcessOsc!(o::Int, |i| NoteOn(i as u8, 30)));
///
/// let mut evs = EventStream::from(OscEvent(0, "/foo".to_string(), vec![o::Int(60)]));
/// filter.run(&mut evs);
/// assert_eq!(evs, vec![NoneEvent(), NoteOnEvent(0,0,60,30)]);
/// # }
/// ```
#[macro_export]
macro_rules! ProcessOsc {
    ( $argt0:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argt0(arg0)] => { Box::new($f(arg0)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1)] => { Box::new($f(arg0, arg1)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2)] => { Box::new($f(arg0, arg1, arg2)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $arg3:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2), $argt3(arg3)] => { Box::new($f(arg0, arg1, arg2, arg3)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $arg3:path, $arg4:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2), $argt3(arg3), $argt4(arg4)] => { Box::new($f(arg0, arg1, arg2, arg3, arg4)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $arg3:path, $arg4:path, $arg5:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2), $argt3(arg3), $argt4(arg4), $argt5(arg5)] => { Box::new($f(arg0, arg1, arg2, arg3, arg4, arg5)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $arg3:path, $arg4:path, $arg5:path, $arg6:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2), $argt3(arg3), $argt4(arg4), $argt5(arg5), $argt6(arg6)] => { Box::new($f(arg0, arg1, arg2, arg3, arg4, arg5, arg6)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
    ( $argt0:path, $argt1:path, $arg2:path, $arg3:path, $arg4:path, $arg5:path, $arg6:path, $arg7:path, $f:expr ) => {
        _ProcessOsc(
            Box::new(
                |args: &Vec<rosc::OscType>| {
                    match args[..] {
                        [$argtyp(arg0), $argt1(arg1), $argt2(arg2), $argt3(arg3), $argt4(arg4), $argt5(arg5), $argt6(arg6), $argt7(arg7)] => { Box::new($f(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7)) },
                        _ => Box::new(Pass()),
                    }
                }
            )
        )
    };
}