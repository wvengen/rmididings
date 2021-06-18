use std::error::Error;
use std::{thread, time};

use super::super::proc::event::Event;
use super::backend::{Backend, PortNum};

/// Null MIDI backend.
///
/// This backend receives no input and generates no output, it accepts all
/// in/out ports. It isn't expected to be useful in practice, just for testing.
pub struct NullBackend {
    in_port_count: u8,
    out_port_count: u8,
}

impl NullBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            in_port_count: 0,
            out_port_count: 0,
        })
    }
}

impl Backend for NullBackend {
    fn set_client_name(&self, _name: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_in_port(&mut self, _name: &str) -> Result<PortNum, Box<dyn Error>> {
        self.in_port_count += 1;
        Ok(self.in_port_count)
    }

    fn create_out_port(&mut self, _name: &str) -> Result<PortNum, Box<dyn Error>> {
        self.out_port_count += 1;
        Ok(self.out_port_count)
    }

    fn connect_in_port(&self, _client_name: &str, _port_name: &str, _in_port: PortNum) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn connect_out_port(&self, _out_port: PortNum, _client_name: &str, _port_name: &str) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    fn run(&self) -> Result<Option<Event>, Box<dyn Error>> {
        Ok(None)
    }

    fn wait(&mut self) -> Result<usize, Box<dyn Error>> {
        thread::sleep(time::Duration::from_secs(1));
        Ok(0)
    }

    fn output_event(&self, _ev: &Event) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }
}