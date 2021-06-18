use std::error::Error;
use std::vec::Vec;

extern crate alsa;
use alsa::seq;
use alsa::PollDescriptors;
use std::ffi::CString;

use super::super::proc::event::*;
use super::backend::{Backend, PortNum};

/// ALSA sequencer MIDI backend.
pub struct AlsaBackend {
    alsaseq: alsa::Seq,
    alsaseq_pollfds: Vec<alsa::poll::pollfd>,
    in_ports: Vec<i32>,
    out_ports: Vec<i32>,
}

impl AlsaBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut slf = Self {
            alsaseq: alsa::Seq::open(None, None, true)?,
            alsaseq_pollfds: vec!(),
            in_ports: Vec::<i32>::new(),
            out_ports: Vec::<i32>::new(),
        };
        slf.alsaseq_pollfds = (&slf.alsaseq, Some(alsa::Direction::Capture)).get()?;
        Ok(slf)
    }
}

impl Backend for AlsaBackend {
    fn set_client_name(&self, name: &str) -> Result<(), Box<dyn Error>> {
        Ok(self.alsaseq.set_client_name(&CString::new(name).unwrap())?)
    }

    fn create_in_port(&mut self, name: &str) -> Result<PortNum, Box<dyn Error>> {
        let port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.in_ports.push(port);
        Ok((self.in_ports.len() - 1) as PortNum)
    }

    fn create_out_port(&mut self, name: &str) -> Result<PortNum, Box<dyn Error>> {
        let port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::READ | seq::PortCap::SUBS_READ,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.out_ports.push(port);
        Ok((self.out_ports.len() - 1) as PortNum)
    }

    fn connect_in_port(&self, client_name: &str, port_name: &str, in_port: PortNum) -> Result<bool, Box<dyn Error>> {
        if let Some(port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::READ | seq::PortCap::SUBS_READ)? {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(seq::Addr { client: port.get_client(), port: port.get_port() });
            subs.set_dest(seq::Addr { client: self.alsaseq.client_id()?, port: self.in_ports[in_port as usize] });
            self.alsaseq.subscribe_port(&subs)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn connect_out_port(&self, out_port: PortNum, client_name: &str, port_name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE)? {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(seq::Addr { client: self.alsaseq.client_id()?, port: self.out_ports[out_port as usize] });
            subs.set_dest(seq::Addr { client: port.get_client(), port: port.get_port() });
            self.alsaseq.subscribe_port(&subs)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn run(&self) -> Result<Option<Event>, Box<dyn Error>> {
        let mut alsaseq_input = self.alsaseq.input();
        match alsaseq_input.event_input_pending(true) {
            Ok(count) if count > 0 => {
                Ok(self.alsaseq_event_to_event(&alsaseq_input.event_input()?)?)
            },
            Ok(_) => Ok(None),
            // Occasionally, this function may return -ENOSPC error. This means that the input FIFO of
            // sequencer overran, and some events are lost. Once this error is returned, the input FIFO
            // is cleared automatically.
            // TODO emit a warning?
            Err(e) if e.nix_error() == alsa::nix::Error::Sys(alsa::nix::errno::Errno::ENOSPC) => {
                println!("Buffer overrun");
                Ok(None)
            },
            Err(e) => Err(Box::new(e)),
        }
    }

    fn wait(&mut self) -> Result<usize, Box<dyn Error>> {
        Ok(alsa::poll::poll(&mut self.alsaseq_pollfds, 100)?)
    }

    fn output_event(&self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        // TODO self.out_ports bounds checking (!)
        match ev.typ {
            EventType::NOTEON => {
                let mut alsaev = seq::Event::new(seq::EventType::Noteon, &seq::EvNote {
                    // TODO figure out what to do with duration and off_velocity
                    channel: ev.channel, note: ev.note, velocity: ev.velocity, duration: 0, off_velocity: 0
                });
                Ok(self.output_alsaseq_event(self.out_ports[ev.port], &mut alsaev)?)
            },
            EventType::NOTEOFF => {
                let mut alsaev = seq::Event::new(seq::EventType::Noteoff, &seq::EvNote {
                    // TODO figure out what to do with duration and off_velocity
                    channel: ev.channel, note: ev.note, velocity: 0, duration: 0, off_velocity: 0
                });
                Ok(self.output_alsaseq_event(self.out_ports[ev.port], &mut alsaev)?)
            },
            EventType::CTRL => {
                let mut alsaev = seq::Event::new(seq::EventType::Controller, &seq::EvCtrl {
                    channel: ev.channel, param: ev.ctrl, value: ev.value
                });
                Ok(self.output_alsaseq_event(self.out_ports[ev.port], &mut alsaev)?)
            },
            EventType::SYSEX => {
                let mut me = seq::MidiEvent::new(ev.sysex.len() as u32)?;
                let (_, me_enc) = me.encode(ev.sysex)?;
                let mut alsaev = me_enc.unwrap();
                Ok(self.output_alsaseq_event(self.out_ports[ev.port], &mut alsaev)?)
            }
        }
    }
}

impl AlsaBackend {
    fn alsaseq_event_to_event(&self, alsaev: &seq::Event) -> Result<Option<Event>, Box<dyn Error>> {
        // map alsa port to our own port (index in self.in_ports), fallback to port 0
        let alsa_port = alsaev.get_dest().port;
        let port = match self.in_ports.iter().position(|p| p == &alsa_port) {
            Some(port) => port,
            _ => 0
        };

        // convert alsaseq event to our own kind of event
        if let Some(e) = alsaev.get_data::<seq::EvNote>() {
            if alsaev.get_type() == seq::EventType::Noteon {
                Ok(Some(NoteOnEvent(port, e.channel, e.note, e.velocity)))
            } else {
                Ok(Some(NoteOffEvent(port, e.channel, e.note)))
            }
        } else if let Some(e) = alsaev.get_data::<seq::EvCtrl>() {
            Ok(Some(CtrlEvent(port, e.channel, e.param, e.value)))
        } else {
            Ok(None)
        }
    }

    fn find_alsaseq_port(&self, client_name: &str, port_name: &str, caps: seq::PortCap) -> Result<Option<alsa::seq::PortInfo>, Box<dyn Error>> {
        for client in seq::ClientIter::new(&self.alsaseq) {
            if client.get_name()? != client_name { continue; }
            for port in seq::PortIter::new(&self.alsaseq, client.get_client()) {
                let port_caps = port.get_capability();
                if !port.get_type().contains(seq::PortType::MIDI_GENERIC) { continue; }
                if !port_caps.contains(caps) { continue; }
                if port.get_name()? != port_name { continue; }
                return Ok(Some(port));
            }
        }
        Ok(None)
    }

    fn output_alsaseq_event(&self, port: i32, ev: &mut alsa::seq::Event) -> Result<u32, Box<dyn Error>> {
        ev.set_source(port);
        ev.set_subs();
        ev.set_direct();
        Ok(self.alsaseq.event_output_direct(ev)?)
    }
}
