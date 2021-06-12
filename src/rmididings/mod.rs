use std::{thread, time};
use std::error::Error;

use super::rmidiproc::*;

mod alsa;

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
    backend: alsa::Backend,
    patch: &'a dyn FilterTrait,
    //pub scene: 
    control: &'a dyn FilterTrait,
    pre: &'a dyn FilterTrait,
    post: &'a dyn FilterTrait,
    initial_scene: u8,
    current_scene: u8,
    current_patch: &'a dyn FilterTrait,
}

impl<'a> RMididings<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            backend: alsa::Backend::new()?,
            patch: &Discard(),
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
            initial_scene: 0,
            current_scene: 0,
            current_patch: &Discard(),
        })
    }

    pub fn config(&mut self, args: ConfigArguments) -> Result<(), Box<dyn Error>> {
        self.backend.set_client_name(args.client_name)?;

        for port in args.in_ports {
            let alsaport = self.backend.create_in_port(&*port[0])?;
            if &*port[1] != "" {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.backend.connect_in_port(client_name, port_name, alsaport)?;
                }
            }
        }
        for port in args.out_ports {
            let alsaport = self.backend.create_out_port(&*port[0])?;
            if &*port[1] != "" {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.backend.connect_out_port(alsaport, client_name, port_name)?;
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

        self.current_patch = self.patch;

        // TODO error when both patch and scenes are given

        // TODO implement scenes

        // TODO use switch_scene, and make it work with single patch

        self.run_patch(self.pre, Event::new())?;

        loop {
            if let Some(ev) = self.backend.run()? {
                println!("event {}", ev.to_string());
                self.run_patch(self.control, ev.clone())?;
                self.run_patch(self.current_patch, ev.clone())?;
            }
            self.backend.wait()?;
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
        self.backend.output_event(&ev)
    }

    fn run_patch(&self, patch: &'a dyn FilterTrait, ev: Event) -> Result<(), Box<dyn Error>> {
        let mut evs = EventStream::from(ev);
        // run the patch
        patch.run(&mut evs);
        // and output any result from the chain
        for ev in evs.events.iter() { self.output_event(ev)?; };
        Ok(())
    }
}