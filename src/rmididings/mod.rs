use std::{thread, time};
use std::error::Error;

mod alsa;
pub mod scene;

use super::rmidiproc::*;
pub use self::scene::*;

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
    pub scenes: &'a [&'a Scene<'a>],
    pub control: &'a dyn FilterTrait,
    pub pre: &'a dyn FilterTrait,
    pub post: &'a dyn FilterTrait,
}

impl RunArguments<'_> {
    pub fn default() -> RunArguments<'static> {
        RunArguments {
            patch: &Discard(),
            scenes: &[],
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
        }
    }
}

pub struct RMididings<'a> {
    backend: alsa::Backend,
    patch: &'a dyn FilterTrait,
    scenes: &'a [&'a Scene<'a>],
    control: &'a dyn FilterTrait,
    pre: &'a dyn FilterTrait,
    post: &'a dyn FilterTrait,
    initial_scene: u8,
    current_scene: u8,
}

impl<'a> RMididings<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            backend: alsa::Backend::new()?,
            patch: &Discard(),
            scenes: &[],
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
            initial_scene: 0,
            current_scene: 0,
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
        self.scenes = args.scenes;
        self.control = args.control;
        self.pre = args.pre;
        self.post = args.post;

        // TODO error when both patch and scenes are given

        self.current_scene = self.initial_scene;

        self.run_patch(self.pre, None)?;
        self.run_init(self.patch, None)?;
        self.run_patch(get_scene(self.scenes, self.current_scene).init, None)?;
        self.run_init(get_scene(self.scenes, self.current_scene).patch, None)?;

        loop {
            if let Some(ev) = self.backend.run()? {
                self.run_patch(self.control, Some(ev.clone()))?;
                // TODO don't run patch when scene was just switched in control
                self.run_patch(self.patch, Some(ev.clone()))?;
                self.run_patch(get_scene(self.scenes, self.current_scene).patch, Some(ev.clone()))?;
            }
            self.backend.wait()?;
        }
    }

    pub fn switch_scene(&mut self, scene: u8) -> Result<(), Box<dyn Error>> {
        // skip if we're already in the scene
        if self.current_scene == scene { return Ok(()); }
        // TODO scene bounds checking (!)

        println!("Scene {}: {}", scene, get_scene(self.scenes, scene).name);

        // TODO make sure we don't run post and exit patch the first time
        //      we don't run this yet on init, but it would be nice to use
        let current_scene = get_scene(self.scenes, self.current_scene);
        self.run_exit(current_scene.patch, None)?;
        self.run_patch(current_scene.exit, None)?;
        self.run_exit(self.patch, None)?;
        self.run_patch(self.post, None)?;

        self.current_scene = scene;

        let current_scene = get_scene(self.scenes, self.current_scene);
        self.run_patch(self.pre, None)?;
        self.run_init(self.patch, None)?;
        self.run_patch(current_scene.init, None)?;
        self.run_init(current_scene.patch, None)?;

        Ok(())
    }

    pub fn output_event(&self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        self.backend.output_event(&ev)
    }

    // TODO put run_patch, run_init and run_exit together
    fn run_patch(&mut self, patch: &dyn FilterTrait, ev: Option<Event>) -> Result<(), Box<dyn Error>> {
        let mut evs = EventStream::from(ev);
        evs.scene = self.current_scene;
        patch.run(&mut evs);
        // handle resulting event stream
        for ev in evs.events.iter() { self.output_event(ev)?; };
        self.switch_scene(evs.scene)?;
        Ok(())
    }

    fn run_init(&mut self, patch: &dyn FilterTrait, ev: Option<Event>) -> Result<(), Box<dyn Error>> {
        let mut evs = EventStream::from(ev);
        evs.scene = self.current_scene;
        patch.run_init(&mut evs);
        // output resulting events
        for ev in evs.events.iter() { self.output_event(ev)?; };
        self.switch_scene(evs.scene)?;
        Ok(())
    }

    fn run_exit(&mut self, patch: &dyn FilterTrait, ev: Option<Event>) -> Result<(), Box<dyn Error>> {
        let mut evs = EventStream::from(ev);
        evs.scene = self.current_scene;
        patch.run_exit(&mut evs);
        // output resulting events
        for ev in evs.events.iter() { self.output_event(ev)?; };
        self.switch_scene(evs.scene)?;
        Ok(())
    }
}

fn get_scene<'a>(scenes: &'a [&Scene<'a>], index: u8) -> &'a Scene<'a> {
    if scenes.len() > index as usize {
        scenes[index as usize]
    } else {
        &Scene::DEFAULT
    }
}