//! A MIDI router/processor on Rust
//!
//! RMididings is a partial clone of [mididings] in Rust, allowing one to use
//! a syntax not unlike mididings for MIDI event routing and processing.
//!
//! It is very early in development, take care.
//!
//! [mididings]: http://das.nasophon.de/mididings/

pub mod proc;
pub use proc::*;

mod scene;
pub use scene::*;

mod engine;
pub use engine::*;