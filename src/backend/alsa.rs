use std::error::Error;
use std::vec::Vec;
use std::collections::HashMap;

extern crate alsa;
use alsa::seq;
use alsa::PollDescriptors;
use std::ffi::CString;

use super::super::proc::event::*;
use super::super::proc::EventStream;
use super::backend::{Backend, PortNum};

/// ALSA sequencer MIDI backend.
pub struct AlsaBackend {
    alsaseq: alsa::Seq,
    in_ports: HashMap<PortNum, i32>,
    out_ports: HashMap<PortNum, i32>,
}

impl AlsaBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            alsaseq: alsa::Seq::open(None, None, true)?,
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
        })
    }

    fn _create_in_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        let alsaseq_port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.in_ports.insert(backend_port, alsaseq_port);
        Ok(true)
    }

    fn _create_out_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        let alsaseq_port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::READ | seq::PortCap::SUBS_READ,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.out_ports.insert(backend_port, alsaseq_port);
        Ok(true)
    }
}

impl Backend<'_> for AlsaBackend {
    fn set_client_name(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        Ok(self.alsaseq.set_client_name(&CString::new(name).unwrap())?)
    }

    fn create_in_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, port_name)) = name.split_once(':') {
            if backend_name != "alsa" { return Ok(false); }
            self._create_in_port(backend_port, port_name)
        } else {
            self._create_in_port(backend_port, name)
        }
    }

    fn create_out_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some((backend_name, port_name)) = name.split_once(':') {
            if backend_name != "alsa" { return Ok(false); }
            self._create_out_port(backend_port, port_name)
        } else {
            self._create_out_port(backend_port, name)
        }
    }

    fn connect_in_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(alsaseq_port) = self.in_ports.get(&backend_port) {
            if let Some((client_name, port_name)) = name.split_once(':') {
                if let Some(connect_port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::READ | seq::PortCap::SUBS_READ)? {
                    let subs = seq::PortSubscribe::empty()?;
                    subs.set_sender(seq::Addr { client: connect_port.get_client(), port: connect_port.get_port() });
                    subs.set_dest(seq::Addr { client: self.alsaseq.client_id()?, port: *alsaseq_port });
                    self.alsaseq.subscribe_port(&subs)?;
                    return Ok(true);
                }
            }
        }
        return Ok(false);
    }

    fn connect_out_port(&mut self, backend_port: PortNum, name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(alsaseq_port) = self.out_ports.get(&backend_port) {
            if let Some((client_name, port_name)) = name.split_once(':') {
                if let Some(connect_port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE)? {
                    let subs = seq::PortSubscribe::empty()?;
                    subs.set_sender(seq::Addr { client: self.alsaseq.client_id()?, port: *alsaseq_port });
                    subs.set_dest(seq::Addr { client: connect_port.get_client(), port: connect_port.get_port() });
                    self.alsaseq.subscribe_port(&subs)?;
                    return Ok(true);
                }
            }
        }
        return Ok(false);
    }

    fn get_pollfds(&mut self) -> Result<Vec<libc::pollfd>, Box<dyn Error>> {
        Ok((&self.alsaseq, Some(alsa::Direction::Capture)).get()?)
    }

    fn run<'evs: 'run, 'run>(&'run mut self) -> Result<(EventStream<'evs>, bool), Box<dyn Error>> {
        let mut alsaseq_input = self.alsaseq.input();
        match alsaseq_input.event_input_pending(true) {
            Ok(count) if count > 0 => {
                Ok((EventStream::from(self.alsaseq_event_to_event(&alsaseq_input.event_input()?)?), false))
            },
            Ok(_) => Ok((EventStream::empty(), false)),
            // Occasionally, this function may return -ENOSPC error. This means that the input FIFO of
            // sequencer overran, and some events are lost. Once this error is returned, the input FIFO
            // is cleared automatically.
            // TODO emit a warning?
            Err(e) if e.nix_error() == alsa::nix::Error::Sys(alsa::nix::errno::Errno::ENOSPC) => {
                println!("Buffer overrun");
                Ok((EventStream::empty(), false))
            },
            Err(e) => Err(Box::new(e)),
        }
    }

    fn output_event(&mut self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        // TODO self.out_ports bounds checking (!)
        match ev {
            Event::NoteOn(ev) => {
                let mut alsaev = seq::Event::new(seq::EventType::Noteon, &seq::EvNote {
                    // TODO figure out what to do with duration and off_velocity
                    channel: ev.channel, note: ev.note, velocity: ev.velocity, duration: 0, off_velocity: 0
                });
                Ok(self.output_alsaseq_event(&ev.port, &mut alsaev)?)
            },
            Event::NoteOff(ev) => {
                let mut alsaev = seq::Event::new(seq::EventType::Noteoff, &seq::EvNote {
                    // TODO figure out what to do with duration and off_velocity
                    channel: ev.channel, note: ev.note, velocity: 0, duration: 0, off_velocity: 0
                });
                Ok(self.output_alsaseq_event(&ev.port, &mut alsaev)?)
            },
            Event::Ctrl(ev) => {
                let mut alsaev = seq::Event::new(seq::EventType::Controller, &seq::EvCtrl {
                    channel: ev.channel, param: ev.ctrl, value: ev.value
                });
                Ok(self.output_alsaseq_event(&ev.port, &mut alsaev)?)
            },
            Event::SysEx(ev) => {
                let mut me = seq::MidiEvent::new(ev.data.len() as u32)?;
                let (_, me_enc) = me.encode(ev.data)?;
                let mut alsaev = me_enc.unwrap();
                Ok(self.output_alsaseq_event(&ev.port, &mut alsaev)?)
            },
            _ => {
                Ok(0)
            },
        }
    }
}

impl AlsaBackend {
    fn alsaseq_event_to_event<'a>(&self, alsaev: &seq::Event) -> Result<Option<Event<'a>>, Box<dyn Error>> {
        // map alsa port to our own port (index in self.in_ports), fallback to port 0
        let alsaseq_port = alsaev.get_dest().port;
        if let Some((port, _)) = self.in_ports.iter().find(|(_, as_p)| **as_p == alsaseq_port) {
            // convert alsaseq event to our own kind of event
            if let Some(e) = alsaev.get_data::<seq::EvNote>() {
                if alsaev.get_type() == seq::EventType::Noteon {
                    return Ok(Some(NoteOnEvent(*port, e.channel, e.note, e.velocity)));
                } else {
                    return Ok(Some(NoteOffEvent(*port, e.channel, e.note)));
                }
            } else if let Some(e) = alsaev.get_data::<seq::EvCtrl>() {
                return Ok(Some(CtrlEvent(*port, e.channel, e.param, e.value)));
            }
        }
        return Ok(None);
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

    fn output_alsaseq_event(&self, backend_port: &PortNum, ev: &mut alsa::seq::Event) -> Result<u32, Box<dyn Error>> {
        if let Some(alsaseq_port) = self.out_ports.get(backend_port) {
            ev.set_source(*alsaseq_port);
            ev.set_subs();
            ev.set_direct();
            Ok(self.alsaseq.event_output_direct(ev)?)
        } else {
            Ok(0)
        }
    }
}
