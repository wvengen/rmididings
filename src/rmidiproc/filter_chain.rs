use event::*;
use event_stream::*;
use filter_trait::*;

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

#[derive(Debug,PartialEq)]
pub enum ConnectionType {
    Chain,
    Fork,
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

#[macro_export]
macro_rules! define_filter {
    ($name:ident ( $($args:ty),* ) $item:item) => {
        pub struct $name($(pub $args),*);

        impl FilterTrait for $name {
            $item
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
            fn modify_single(&self, ev: &mut Event) {
                *ev = self.generate_single();
            }
        }
    }
}