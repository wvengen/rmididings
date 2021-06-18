use std::error::Error;

use super::super::proc::{Event, EventStream};

pub type PortNum = usize;

/// MIDI Backend implementation.
pub trait Backend<'a> {
    fn set_client_name(&mut self, name: &str) -> Result<(), Box<dyn Error>>;

    fn create_in_port(&mut self, port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>>;

    fn create_out_port(&mut self, port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>>;

    fn connect_in_port(&mut self, port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>>;

    fn connect_out_port(&mut self, port: PortNum, name: &'a str) -> Result<bool, Box<dyn Error>>;

    fn get_pollfds(&mut self) -> Result<Vec<libc::pollfd>, Box<dyn Error>>;

    fn run<'evs: 'run, 'run>(&'run mut self) -> Result<EventStream<'evs>, Box<dyn Error>>;

    fn output_event(&mut self, ev: &Event) -> Result<u32, Box<dyn Error>>;
}