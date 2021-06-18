use std::error::Error;

use super::super::proc::event::Event;

pub type PortNum = u8;

/// MIDI Backend implementation.
pub trait Backend {
    fn set_client_name(&self, name: &str) -> Result<(), Box<dyn Error>>;

    fn create_in_port(&mut self, name: &str) -> Result<PortNum, Box<dyn Error>>;

    fn create_out_port(&mut self, name: &str) -> Result<PortNum, Box<dyn Error>>;

    fn connect_in_port(&self, client_name: &str, port_name: &str, in_port: PortNum) -> Result<bool, Box<dyn Error>>;

    fn connect_out_port(&self, out_port: PortNum, client_name: &str, port_name: &str) -> Result<bool, Box<dyn Error>>;

    fn run(&self) -> Result<Option<Event>, Box<dyn Error>>;

    fn wait(&mut self) -> Result<usize, Box<dyn Error>>;

    fn output_event(&self, ev: &Event) -> Result<u32, Box<dyn Error>>;
}