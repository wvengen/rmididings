extern crate libc;

mod backend;
pub use self::backend::{Backend, PortNum};

mod null;
pub use self::null::NullBackend;

#[cfg(feature = "alsa")]
mod alsa;
#[cfg(feature = "alsa")]
pub use self::alsa::AlsaBackend;

#[cfg(feature = "osc")]
mod osc;
#[cfg(feature = "osc")]
pub use self::osc::OscBackend;