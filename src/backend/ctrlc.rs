use std::error::Error;
use std::os::unix::io::{RawFd};

extern crate nix;
use nix::sys::signal;

use crate::proc::{Event, EventStream, QuitEvent};
use crate::backend::{Backend, PortNum};

/// A special backend to handle Ctrl-C.
///
/// The only thing it does is emitting the {Quit} event when Ctrl-C is pressed,
/// so that e.g. exit patches can be run.
///
/// Heavily inspired by https://github.com/Detegr/rust-ctrlc/issues/30#issuecomment-326346519
pub struct CtrlcBackend { }

impl CtrlcBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Create a new non-blocking pipe that we write to from our signal handler.
        unsafe {
            PIPE = nix::unistd::pipe2(nix::fcntl::OFlag::O_CLOEXEC | nix::fcntl::OFlag::O_NONBLOCK)?;
        };

        // Set-up our signal handler.
        let handler = signal::SigHandler::Handler(signal_handler);
        let action = signal::SigAction::new(handler,
            signal::SaFlags::SA_RESTART,
            signal::SigSet::empty()
        );
        unsafe {
            signal::sigaction(signal::Signal::SIGINT, &action)?;
        }

        Ok(Self { })
    }
}

impl Backend<'_> for CtrlcBackend {
    fn set_client_name(&mut self, _name: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_in_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn create_out_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn connect_in_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn connect_out_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn get_pollfds(&mut self) -> Result<Vec<libc::pollfd>, Box<dyn Error>> {
        Ok(vec![libc::pollfd { fd: unsafe { PIPE.0 }, events: 1, revents: 0 }])
    }

    fn run<'evs: 'run, 'run>(&'run mut self) -> Result<EventStream<'evs>, Box<dyn Error>> {
        // We are only called when our fd has events, so we can directly return the event.
        Ok(EventStream::from(QuitEvent()))
    }

    fn output_event(&mut self, _ev: &Event) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }
}

static mut PIPE: (RawFd, RawFd) = (-1, -1);

extern fn signal_handler(_: nix::libc::c_int) {
    // Signal handlers are special functions, only [async-signal-safe]
    // (http://man7.org/linux/man-pages/man7/signal-safety.7.html) functions
    // can be called in this context.
    nix::unistd::write(unsafe { PIPE.1 }, &[0u8]).unwrap();
}