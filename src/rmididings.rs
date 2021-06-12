use std::{thread, time};
use std::error::Error;
use std::vec::Vec;

use alsa::seq;
use std::ffi::CString;

use super::rmidiproc::*;

pub struct ConfigArguments<'a> {
    pub backend: &'a str,
    pub client_name: &'a str,
    pub in_ports: &'a [[&'a str; 2]],
    pub out_ports: &'a [[&'a str; 2]],
    //pub data_offset: u8,
    //pub octave_offset: u8,
    pub initial_scene: u8,
    pub start_delay: f32
}

impl ConfigArguments<'_> {
    pub fn default() -> ConfigArguments<'static> {
        ConfigArguments {
            backend: "alsa",
            client_name: "RMididings",
            in_ports: &[],
            out_ports: &[],
            //data_offset: 1,
            //octave_offset: 2,
            initial_scene: 0,
            start_delay: 0.0
        }
    }
}

pub struct RunArguments<'a> {
    pub patch: &'a dyn FilterTrait,
    //pub scene: 
    pub control: &'a dyn FilterTrait,
    pub pre: &'a dyn FilterTrait,
    pub post: &'a dyn FilterTrait,
}

impl RunArguments<'_> {
    pub fn default() -> RunArguments<'static> {
        RunArguments {
            patch: &Discard(),
            //scenes: ,
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
        }
    }
}

pub struct RMididings<'a> {
    alsaseq: alsa::Seq,
    in_ports: Vec<i32>,
    out_ports: Vec<i32>,
    patch: &'a dyn FilterTrait,
    //pub scene: 
    control: &'a dyn FilterTrait,
    pre: &'a dyn FilterTrait,
    post: &'a dyn FilterTrait,
    current_scene: u8,
    initial_scene: u8,
}

impl<'a> RMididings<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            alsaseq: alsa::Seq::open(None, None, true)?,
            in_ports: Vec::<i32>::new(),
            out_ports: Vec::<i32>::new(),
            patch: &Discard(),
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
            current_scene: 0,
            initial_scene: 0,
        })
    }

    pub fn config(&mut self, args: ConfigArguments) -> Result<(), Box<dyn Error>> {
        self.alsaseq.set_client_name(&CString::new(args.client_name).unwrap())?;

        for port in args.in_ports {
            let alsaport = self.create_in_port(&*port[0])?;
            if &*port[1] != "" {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.connect_in_port(client_name, port_name, alsaport)?;
                }
            }
        }
        for port in args.out_ports {
            let alsaport = self.create_out_port(&*port[0])?;
            if &*port[1] != "" {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.connect_out_port(alsaport, client_name, port_name)?;
                }
            }
        }

        if args.start_delay > 0.0 {
            thread::sleep(time::Duration::from_secs_f32(args.start_delay));
        }

        self.initial_scene = args.initial_scene;

        Ok(())
    }

    pub fn run(&mut self, args: RunArguments<'a>) -> Result<(), Box<dyn Error>> {
        // Handle arguments
        self.patch = args.patch;
        self.control = args.control;
        self.pre = args.pre;
        self.post = args.post;

        // TODO error when both patch and scenes are given

        // TODO implement scenes

        // TODO use switch_scene, and make it work with single patch

        // Poll-wait loop
        use alsa::PollDescriptors;
        let mut alsaseq_input = self.alsaseq.input();
        let mut fds = &mut (&self.alsaseq, Some(alsa::Direction::Capture)).get()?;

        loop {
            // Handle pending MIDI event.
            if alsaseq_input.event_input_pending(true)? > 0 {
                self.handle_alsaseq_event(&alsaseq_input.event_input()?)?;
                continue;
            }

            // Nothing to do, let's sleep until woken up by the kernel.
            alsa::poll::poll(&mut fds, 100)?;
        }
    }

    pub fn switch_scene(&mut self, scene: u8) -> Result<(), Box<dyn Error>> {
        if self.current_scene == scene { return Ok(()) }

        // let mut evs = EventStream::one();
        // self.post.run(&mut evs);
        // for ev in evs.events.iter() { self.output_event(ev)?; };

        // TODO switch scene
        println!("TODO switch_scene");

        // let mut evs = EventStream::one();
        // self.pre.run(&mut evs);
        // for ev in evs.events.iter() { self.output_event(ev)?; };

        Ok(())
    }

    pub fn output_event(&self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        println!("output_event {}", ev.to_string());
        // TODO self.out_ports bounds checking (!)
        match ev.typ {
            EventType::NONE => Ok(0),
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
                let mut me = seq::MidiEvent::new(16)?; // TODO buffer size
                let (_, me_enc) = me.encode(ev.sysex)?;
                let mut alsaev = me_enc.unwrap();
                Ok(self.output_alsaseq_event(self.out_ports[ev.port], &mut alsaev)?)
            }
        }
    }

    fn handle_alsaseq_event(&self, alsaev: &seq::Event) -> Result<(), Box<dyn Error>> {
        // map alsa port to our own port (index in self.in_ports), fallback to port 0
        let alsa_port = alsaev.get_dest().port;
        let port = match self.in_ports.iter().position(|p| p == &alsa_port) {
            Some(port) => port,
            _ => 0
        };

        // convert alsaseq event to our own kind of event
        let ev_in = if let Some(e) = alsaev.get_data::<seq::EvNote>() {
            if alsaev.get_type() == seq::EventType::Noteon {
                NoteOnEvent(port, e.channel, e.note, e.velocity)
            } else {
                NoteOffEvent(port, e.channel, e.note)
            }
        } else if let Some(e) = alsaev.get_data::<seq::EvCtrl>() {
            CtrlEvent(port, e.channel, e.param, e.value)
        } else {
            Event { port, ..Event::new() }
        };

        println!("handle_midi_event {:?} - {}", alsaev, ev_in.to_string());
        // go through the processing chain
        let mut evs = EventStream::from(ev_in);
        self.patch.run(&mut evs);

        // and output any result from the chain
        for ev in evs.events.iter() { self.output_event(ev)?; };

        return Ok(());
    }

    fn create_in_port(&mut self, name: &str) -> Result<i32, Box<dyn Error>> {
        let port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.in_ports.push(port);
        Ok(port)
    }

    fn create_out_port(&mut self, name: &str) -> Result<i32, Box<dyn Error>> {
        let port = self.alsaseq.create_simple_port(
            &CString::new(name).unwrap(),
            seq::PortCap::READ | seq::PortCap::SUBS_READ,
            seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION
        )?;
        self.out_ports.push(port);
        Ok(port)
    }

    fn connect_in_port(&self, client_name: &str, port_name: &str, in_port: i32) -> Result<bool, Box<dyn Error>> {
        if let Some(port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::READ | seq::PortCap::SUBS_READ)? {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(seq::Addr { client: port.get_client(), port: port.get_port() });
            subs.set_dest(seq::Addr { client: self.alsaseq.client_id()?, port: in_port });
            self.alsaseq.subscribe_port(&subs)?;
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn connect_out_port(&self, out_port: i32, client_name: &str, port_name: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(port) = self.find_alsaseq_port(client_name, port_name, seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE)? {
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(seq::Addr { client: self.alsaseq.client_id()?, port: out_port });
            subs.set_dest(seq::Addr { client: port.get_client(), port: port.get_port() });
            self.alsaseq.subscribe_port(&subs)?;
            return Ok(true);
        } else {
            return Ok(false);
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
        return Ok(None);
    }

    fn output_alsaseq_event(&self, port: i32, ev: &mut alsa::seq::Event) -> Result<u32, Box<dyn Error>> {
        ev.set_source(port);
        ev.set_subs();
        ev.set_direct();
        return Ok(self.alsaseq.event_output_direct(ev)?);
    }
}

