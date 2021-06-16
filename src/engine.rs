use std::error::Error;
use std::{thread, time};

use super::proc::*;
use super::scene::*;

mod alsa;

pub struct ConfigArguments<'a> {
    pub backend: &'a str,
    pub client_name: &'a str,
    pub in_ports: &'a [[&'a str; 2]],
    pub out_ports: &'a [[&'a str; 2]],
    pub data_offset: u8,
    pub scene_offset: SceneNum,
    //pub octave_offset: u8,
    pub initial_scene: SceneNum,
    pub start_delay: f32,
}

impl ConfigArguments<'_> {
    pub fn default() -> ConfigArguments<'static> {
        ConfigArguments {
            backend: "alsa",
            client_name: "RMididings",
            in_ports: &[],
            out_ports: &[],
            data_offset: 1,
            scene_offset: 1,
            //octave_offset: 2,
            initial_scene: 0,
            start_delay: 0.0,
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
    port_offset: u8,
    channel_offset: u8,
    scene_offset: u8,
    patch: &'a dyn FilterTrait,
    scenes: &'a [&'a Scene<'a>],
    control: &'a dyn FilterTrait,
    pre: &'a dyn FilterTrait,
    post: &'a dyn FilterTrait,
    initial_scene_num: SceneNum,
    current_scene_num: Option<SceneNum>,
    current_subscene_num: Option<SceneNum>,
    stored_subscene_nums: Vec<Option<SceneNum>>,
}

impl<'a> RMididings<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            backend: alsa::Backend::new()?,
            port_offset: 1,
            channel_offset: 1,
            scene_offset: 1,
            patch: &Discard(),
            scenes: &[],
            control: &Discard(),
            pre: &Discard(),
            post: &Discard(),
            initial_scene_num: 0,
            current_scene_num: None,
            current_subscene_num: None,
            stored_subscene_nums: Vec::<Option<SceneNum>>::new(),
        })
    }

    pub fn config(&mut self, args: ConfigArguments) -> Result<(), Box<dyn Error>> {
        self.backend.set_client_name(args.client_name)?;

        for port in args.in_ports {
            let alsaport = self.backend.create_in_port(&*port[0])?;
            if !port[1].is_empty() {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.backend.connect_in_port(client_name, port_name, alsaport)?;
                }
            }
        }
        for port in args.out_ports {
            let alsaport = self.backend.create_out_port(&*port[0])?;
            if !port[1].is_empty() {
                if let Some((client_name, port_name)) = (&*port[1]).split_once(':') {
                    self.backend.connect_out_port(alsaport, client_name, port_name)?;
                }
            }
        }

        if args.start_delay > 0.0 {
            thread::sleep(time::Duration::from_secs_f32(args.start_delay));
        }

        self.initial_scene_num = args.initial_scene;
        self.port_offset = args.data_offset;
        self.channel_offset = args.data_offset;
        self.scene_offset = args.scene_offset;

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

        if !args.scenes.is_empty() {
            self.current_scene_num = Some(self.initial_scene_num);

            self.stored_subscene_nums = args.scenes.iter()
                .map(|scene| { if scene.subscenes.is_empty() { None } else { Some(0) } })
                .collect();
            self.current_subscene_num = *self.get_stored_subscene_num();
        }

        self.run_current_scene_init()?;
        self.run_current_subscene_init()?;

        loop {
            if let Some(mut ev) = self.backend.run()? {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
                ev.channel = ev.channel.saturating_add(self.channel_offset);

                self.run_current_patches(&ev)?;
            }
            self.backend.wait()?;
        }
    }

    pub fn switch_scene(&mut self, scene: SceneNum) -> Result<(), Box<dyn Error>> {
        self.switch_scene_internal(scene.saturating_sub(self.scene_offset), None)
    }

    pub fn switch_subscene(&mut self, subscene: SceneNum) -> Result<(), Box<dyn Error>> {
        self.switch_subscene_internal(subscene.saturating_sub(self.scene_offset))
    }

    // note: don't use this to switch to a new subscene in the current scene
    fn switch_scene_internal(&mut self, new_scene_num: SceneNum, new_subscene_num_opt: Option<SceneNum>) -> Result<(), Box<dyn Error>> {
        // skip if we're already in the scene
        if let Some(current_scene_num) = self.current_scene_num {
            if current_scene_num == new_scene_num { return Ok(()); }
        }

        self.run_current_subscene_exit()?;
        self.run_current_scene_exit()?;

        self.current_scene_num = Some(new_scene_num);
        self.current_subscene_num = new_subscene_num_opt.map_or(
            *self.get_stored_subscene_num(),
            |_| new_subscene_num_opt
        );
        self.print_current_scene();

        self.run_current_scene_init()?;
        self.run_current_subscene_init()?;

        Ok(())
    }

    fn switch_subscene_internal(&mut self, new_subscene_num: SceneNum) -> Result<(), Box<dyn Error>> {
        if let Some(current_scene_num) = self.current_scene_num {
            // skip if we're already in the subscene
            if let Some(current_subscene_num) = self.current_subscene_num {
                if current_subscene_num == new_subscene_num { return Ok(()); }
            }

            self.run_current_subscene_exit()?;

            self.current_subscene_num = Some(new_subscene_num);
            self.stored_subscene_nums[current_scene_num as usize] = Some(new_subscene_num);
            self.print_current_scene();

            self.run_current_subscene_init()?;
        }
        Ok(())
    }

    fn run_current_scene_init(&mut self) -> Result<(), Box<dyn Error>> {
        self.run_patch(self.pre, SceneRunType::Patch, None)?;
        self.run_patch(self.patch, SceneRunType::Init, None)?;
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            self.run_patch(current_scene.init, SceneRunType::Patch, None)?;
            self.run_patch(current_scene.patch, SceneRunType::Init, None)?;
        }
        Ok(())
    }

    fn run_current_subscene_init(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            if let Some(current_subscene) = current_scene.get_subscene_opt(self.current_subscene_num) {
                self.run_patch(current_subscene.init, SceneRunType::Patch, None)?;
                self.run_patch(current_subscene.patch, SceneRunType::Init, None)?;
            }
        }
        Ok(())
    }

    fn run_current_patches(&mut self, ev: &Event) -> Result<(), Box<dyn Error>> {
        self.run_patch(self.control, SceneRunType::Patch, Some(*ev))?;
        // TODO don't run patch when scene was just switched in control
        //      maybe do scene switching at the end of the full patch?
        //      in that case we'll need current_scene and new_scene in EventStream
        self.run_patch(self.patch, SceneRunType::Patch, Some(*ev))?;
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            self.run_patch(current_scene.patch, SceneRunType::Patch, Some(*ev))?;
            if let Some(current_subscene) = current_scene.get_subscene_opt(self.current_subscene_num) {
                self.run_patch(current_subscene.patch, SceneRunType::Patch, Some(*ev))?;
            }
        }
        Ok(())
    }

    fn run_current_subscene_exit(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            if let Some(current_subscene) = current_scene.get_subscene_opt(self.current_subscene_num) {
                self.run_patch(current_subscene.patch, SceneRunType::Exit, None)?;
                self.run_patch(current_subscene.exit, SceneRunType::Patch, None)?;
            }
        }
        Ok(())
    }

    fn run_current_scene_exit(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            self.run_patch(current_scene.patch, SceneRunType::Exit, None)?;
            self.run_patch(current_scene.exit, SceneRunType::Patch, None)?;
        }
        self.run_patch(self.patch, SceneRunType::Exit, None)?;
        self.run_patch(self.post, SceneRunType::Patch, None)?;
        Ok(())
    }

    pub fn output_event(&mut self, ev: &Event) -> Result<u32, Box<dyn Error>> {
        if self.channel_offset == 0 && self.port_offset == 0 {
            self.backend.output_event(&ev)
        } else {
            let mut ev = *ev;
            ev.port = ev.port.saturating_sub(self.port_offset as usize);
            ev.channel = ev.channel.saturating_sub(self.channel_offset);
            self.backend.output_event(&ev)
        }
    }

    fn run_patch(&mut self, filter: &dyn FilterTrait, run_type: SceneRunType, ev: Option<Event>) -> Result<(), Box<dyn Error>> {
        let mut evs = EventStream::from(ev);

        // set scene and subscene on patch
        if let Some(current_scene_num) = self.current_scene_num {
            evs.scene = Some(current_scene_num.saturating_add(self.scene_offset));
        }
        if let Some(subscene_num) = self.current_subscene_num {
            evs.subscene = Some(subscene_num.saturating_add(self.scene_offset));
        }

        // run patch
        match run_type {
            SceneRunType::Patch => filter.run(&mut evs),
            SceneRunType::Init => filter.run_init(&mut evs),
            SceneRunType::Exit => filter.run_exit(&mut evs),
        }

        // handle resulting event stream
        for ev in evs.events.iter() {
            self.output_event(ev)?;
        }

        if let Some(scene) = evs.scene {
            if let Some(subscene) = evs.subscene {
                self.switch_scene_internal(
                    scene.saturating_sub(self.scene_offset),
                    Some(subscene.saturating_sub(self.scene_offset)),
                )?;
            } else {
                self.switch_scene_internal(scene.saturating_sub(self.scene_offset), None)?;
            }
        } else if let Some(subscene) = evs.subscene {
            self.switch_subscene_internal(subscene.saturating_sub(self.scene_offset))?;
        }
        Ok(())
    }

    fn print_current_scene(&self) {
        if let Some(current_scene_num) = self.current_scene_num {
            if let Some(current_scene) = get_scene(self.scenes, self.current_scene_num) {
                if let Some(current_subscene_num) = self.current_subscene_num {
                    if let Some(current_subscene) = current_scene.get_subscene(current_subscene_num)
                    {
                        println!(
                            "Scene {}.{}: {} - {}",
                            current_scene_num.saturating_add(self.scene_offset),
                            current_subscene_num.saturating_add(self.scene_offset),
                            current_scene.name,
                            current_subscene.name
                        );
                        return;
                    }
                }

                println!(
                    "Scene {}: {}",
                    current_scene_num.saturating_add(self.scene_offset),
                    current_scene.name
                );
            }
        }
    }

    fn get_stored_subscene_num(&self) -> &Option<SceneNum> {
        if let Some(current_scene_num) = self.current_scene_num {
            if let Some(stored_subscene_num) = self.stored_subscene_nums.get(current_scene_num as usize) {
                return stored_subscene_num;
            }
        }
        &None
    }
}

enum SceneRunType {
    Patch,
    Init,
    Exit,
}

fn get_scene<'a>(scenes: &'a [&Scene<'a>], scene_num_opt: Option<SceneNum>) -> Option<&'a Scene<'a>> {
    if let Some(scene_num) = scene_num_opt {
        if scenes.len() > scene_num as usize {
            return Some(&scenes[scene_num as usize]);
        }
    }
    None
}
