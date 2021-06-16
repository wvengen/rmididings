#![macro_use]
use super::event::*;
use super::event_stream::*;
use super::filter_trait::*;

/// Collections of filters that are run either in sequence or in parallel.
///
/// See the [Chain!] and [Fork!] macros.
pub struct FilterChain<'a> {
    // lifetime: https://www.reddit.com/r/rust/comments/30ehed/why_must_this_reference_have_a_static_lifetime/
    filters: Vec<Box<dyn FilterTrait + 'a>>,
    connection: ConnectionType,
}

impl<'a> FilterChain<'a> {
    pub fn new(connection: ConnectionType, filters: Vec<Box<dyn FilterTrait + 'a>>) -> Self {
        FilterChain { filters, connection, }
    }

    fn run_chain(&self, evs: &mut EventStream, method: &dyn Fn(&Box<dyn FilterTrait + 'a>, &mut EventStream)) {
        // Run each filter consequetively. Since they mutate evs, this
        // means each filter is run on top of the changes of the previous.
        for f in self.filters.iter() {
            method(&f, evs);
        }
        evs.dedup();
    }

    fn run_fork(&self, evs: &mut EventStream, method: &dyn Fn(&Box<dyn FilterTrait + 'a>, &mut EventStream)) {
        // Run each filter over the original evs and gather all events
        // into a single EventStream.
        // TODO allocate full size of events_out
        // TODO don't clone for first/last filter (can do when running last) ...
        // TODO ... or repeat evs filters.size times, and run on each slice.
        let mut events_out = Vec::<Event>::new();
        for f in self.filters.iter() {
            let mut evs_this = evs.clone();
            method(&f, &mut evs_this);
            events_out.extend(evs_this.events);
            evs.scene = evs_this.scene;
            evs.subscene = evs_this.subscene;
        }
        evs.events.clear();
        evs.events.extend(events_out);
        evs.dedup();
    }
}

fn run_single<'a>(f: &Box<dyn FilterTrait + 'a>, evs: &mut EventStream) {
    f.run(evs)
}
fn run_inverse_single<'a>(f: &Box<dyn FilterTrait + 'a>, evs: &mut EventStream) {
    f.run_inverse(evs)
}

impl<'a> FilterTrait for FilterChain<'a> {
    fn run(&self, evs: &mut EventStream) {
        match self.connection {
            ConnectionType::Chain => self.run_chain(evs, &run_single),
            ConnectionType::Fork => self.run_fork(evs, &run_single),
        }
    }

    fn run_inverse(&self, evs: &mut EventStream) {
        match self.connection {
            ConnectionType::Chain => self.run_fork(evs, &run_inverse_single),
            ConnectionType::Fork => self.run_chain(evs, &run_inverse_single),
        }
    }

    fn run_init(&self, evs: &mut EventStream) {
        for f in self.filters.iter() {
            f.run_init(evs);
        }
    }

    fn run_exit(&self, evs: &mut EventStream) {
        for f in self.filters.iter() {
            f.run_exit(evs);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionType {
    Chain,
    Fork,
}

// Connecting filters

/// Adds multiple filters in a chain.
///
/// This means that each filter is run in sequence. When filtering,
/// this means each event needs to be let through by each of the filters.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let chain = Chain!(ChannelFilter(1), KeyFilter(60));
///
/// let ev1 = NoteOnEvent(0,0,60,20);
/// let ev2 = NoteOnEvent(0,0,61,20);
/// let ev3 = NoteOnEvent(0,1,60,20);
/// let ev4 = NoteOnEvent(0,1,61,20);
///
/// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
/// chain.run(&mut evs);
///
/// assert_eq!(evs.events.to_vec(), vec![ev3]);
/// # }
/// ```
///
/// TODO test inverse
#[macro_export]
macro_rules! Chain {
    ( $($f:expr),+ ) => (
        FilterChain::new(
            ConnectionType::Chain,
            vec!( $(Box::new($f)),+ )
        )
    )
}

/// Adds multiple filters in parallel.
///
/// Each event is passed to each of the filters, they are run in parallel.
/// At the end of the filter chain, duplicate events are filtered out.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rmididings;
/// # use rmididings::proc::*;
/// # fn main() {
/// let chain = Fork!(ChannelFilter(1), KeyFilter(60));
///
/// let ev1 = NoteOnEvent(0,0,60,20);
/// let ev2 = NoteOnEvent(0,0,61,20);
/// let ev3 = NoteOnEvent(0,1,60,20);
/// let ev4 = NoteOnEvent(0,1,61,20);
///
/// let mut evs = EventStream::from(&vec![ev1, ev2, ev3, ev4]);
/// chain.run(&mut evs);
///
/// assert_eq!(evs.events.to_vec(), vec![ev3, ev4, ev1]);
/// # }
/// ```
///
/// TODO test inverse
#[macro_export]
macro_rules! Fork {
    ( $($f:expr),+ ) => (
        FilterChain::new(
            ConnectionType::Fork,
            vec!( $(Box::new($f)),+ )
        )
    )
}

#[macro_export]
macro_rules! define_filter {
    ($(#[$meta:meta])* $name:ident ( $($args:ty),* ) $item:item) => {
        $(#[$meta])*
        pub struct $name($(pub $args),*);

        impl $name {
            $item
        }

        impl FilterTrait for $name {
            fn run(&self, evs: &mut EventStream) {
                evs.events.retain(|ev| self.filter_single(&ev));
            }

            fn run_inverse(&self, evs: &mut EventStream) {
                evs.events.retain(|ev| !self.filter_single(&ev));
            }
        }
    }
}

#[macro_export]
macro_rules! define_modifier {
    ($(#[$meta:meta])* $name:ident ( $($args:ty),* ) $item:item) => {
        $(#[$meta])*
        pub struct $name($(pub $args),*);

        impl $name {
            $item
        }

        impl FilterTrait for $name {
            fn run(&self, evs: &mut EventStream) {
                for ev in evs.events.iter_mut() {
                    self.modify_single(ev);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! define_generator {
    ($(#[$meta:meta])* $name:ident ( $($args:ty),* ) $item:item) => {
        $(#[$meta])*
        pub struct $name($(pub $args),*);

        impl $name {
            $item
        }

        impl FilterTrait for $name {
            fn run(&self, evs: &mut EventStream) {
                evs.events.push(self.generate_single());
            }
        }
    }
}
