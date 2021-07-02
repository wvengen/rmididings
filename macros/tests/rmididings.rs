//! A minimal selection of RMididings functionality so the macros can be tested.

pub mod proc {
    // Dummy FilterTrait with Debug
    pub trait FilterTrait: std::fmt::Debug {}

    // Dummy FilterChain
    #[derive(Debug)]
    pub struct FilterChain(ConnectionType, Vec<Box<dyn FilterTrait>>);
    impl FilterChain {
        // This is the only method used by the macro.
        pub fn new(connection: ConnectionType, filters: Vec<Box<dyn FilterTrait>>) -> Self {
            FilterChain(connection, filters)
        }
    }
    impl FilterTrait for FilterChain {}
    #[derive(Debug)]
    pub enum ConnectionType {
        Chain,
        Fork
    }
}

// Two example filters to work with in the tests.
#[derive(Debug)]
pub struct Pass();
impl proc::FilterTrait for Pass {}

#[derive(Debug)]
pub struct Discard();
impl proc::FilterTrait for Discard {}

// Dummy Not implementation
#[derive(Debug)]
pub struct _Not(pub Box<dyn proc::FilterTrait>);
#[macro_export]
macro_rules! Not {
    ( $f:expr ) => {
        _Not(Box::new($f))
    };
}

// Test helper
#[macro_export]
macro_rules! assert_debug_eq {
    ( $a:expr, $b:expr ) => {
        assert_eq!( format!("{:?}", $a), $b )
    }
}