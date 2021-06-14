#![macro_use]
use event::*;
use event_stream::*;
use filter_trait::*;

pub struct FilterChain {
    // need to make both public because of macro use
    filters: Vec<Box<dyn FilterTrait>>,
    connection: ConnectionType,
}

impl FilterChain {
    pub fn new(connection: ConnectionType, filters: Vec<Box<dyn FilterTrait>>) -> Self {
        FilterChain { connection, filters }
    }
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
                    evs.scene = evs_this.scene;
                    evs.subscene = evs_this.subscene;
                }
                evs.events.clear();
                evs.events.extend(events_out);
            },
        }
        evs.events.dedup();
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


#[derive(Debug,PartialEq)]
pub enum ConnectionType {
    Chain,
    Fork,
}

// Connecting filters

#[macro_export]
macro_rules! Chain {
    ( $($f:expr),+ ) => (
        FilterChain::new(
            ConnectionType::Chain,
            vec!( $(Box::new($f)),+ )
        )
    )
}

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
    ($name:ident ( $($args:ty),* ) $item:item) => {
        pub struct $name($(pub $args),*);

        impl $name {
            $item
        }

        impl FilterTrait for $name {
            fn run(&self, evs: &mut EventStream) {
                evs.events.retain(|ev| self.filter_single(&ev));
            }
        }
    }
}

#[macro_export]
macro_rules! define_modifier {
    ($name:ident ( $($args:ty),* ) $item:item) => {
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
    ($name:ident ( $($args:ty),* ) $item:item) => {
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