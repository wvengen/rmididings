use std::error::Error;

use crate::proc::{Event, EventStream};
use crate::backend::{Backend, PortNum};

/// Null MIDI backend.
///
/// This backend receives no input and generates no output, it accepts all
/// in/out ports. It isn't expected to be useful in practice, just for testing.
pub struct NullBackend {}

impl NullBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {})
    }
}

impl Backend<'_> for NullBackend {
    fn set_client_name(&mut self, _name: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_in_port(&mut self, _port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, _port_name)) = name.split_once(':') {
            if backend_name != "null" { return Ok(false); }
        }
        return Ok(true);
    }

    fn create_out_port(&mut self, _port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, _port_name)) = name.split_once(':') {
            if backend_name != "null" { return Ok(false); }
        }
        return Ok(true);
    }

    fn connect_in_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn connect_out_port(&mut self, _port: PortNum, _name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn get_pollfds(&mut self) -> Result<Vec<libc::pollfd>, Box<dyn Error>> {
        Ok(vec![])
    }

    fn run<'evs: 'run, 'run>(&'run mut self) -> Result<(EventStream<'evs>, bool), Box<dyn Error>> {
        Ok((EventStream::empty(), false))
    }

    fn output_event(&mut self, _ev: &Event) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }
}