use std::error::Error;

use crate::proc::*;
use crate::scene::*;
use crate::backend::Backend;

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
            pre: &Pass(),
            post: &Pass(),
        }
    }
}

pub struct Runner<'a, 'backend: 'a> {
    backends: &'a mut Vec<Box::<dyn Backend<'backend> + 'backend>>,
    port_offset: u8,
    channel_offset: u8,
    scene_offset: SceneNum,
    patch: &'a dyn FilterTrait,
    scenes: &'a [&'a Scene<'a>],
    control: &'a dyn FilterTrait,
    pre: &'a dyn FilterTrait,
    post: &'a dyn FilterTrait,
    initial_scene_num: SceneNum,
    current_scene_num: Option<SceneNum>,
    current_subscene_num: Option<SceneNum>,
    stored_subscene_nums: Vec<Option<SceneNum>>,
    running: bool,
}

impl<'a, 'backend: 'a> Runner<'a, 'backend> {
    pub fn new(args: RunArguments<'a>, backends: &'a mut Vec<Box::<dyn Backend<'backend> + 'backend>>, port_offset: u8, channel_offset: u8, scene_offset: SceneNum, initial_scene_num: SceneNum) -> Self {
        // TODO error when both patch and scenes are given?

        let stored_subscene_nums = args.scenes
            .iter()
            .map(|scene| { if scene.subscenes.is_empty() { None } else { Some(0) } })
            .collect();

        Self {
            backends,
            port_offset,
            channel_offset,
            scene_offset,
            patch: args.patch,
            scenes: args.scenes,
            control: args.control,
            pre: args.pre,
            post: args.post,
            initial_scene_num,
            current_scene_num: None,
            current_subscene_num: None,
            stored_subscene_nums,
            running: false,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Gather polling file descriptors
        let mut pollfds: Vec<libc::pollfd> = vec![];

        for backend in self.backends.iter_mut() {
            pollfds.extend(backend.get_pollfds()?);
        }

        // Setup scene
        if !self.scenes.is_empty() {
            self.current_scene_num = Some(self.initial_scene_num);

            self.current_subscene_num = *self.get_stored_subscene_num();
            self.print_current_scene();
        }

        self.running = true;

        self.run_current_scene_init()?;
        self.run_current_subscene_init()?;

        // Main runner loop
        while self.running {
            // Backend
            let events: EventStream = self.backends.iter_mut().flat_map(|b| b.run()).collect();
            for mut ev in events.into_iter() {
                self.backend_event_to_user(&mut ev);
                self.run_current_patches(&ev)?;
            }

            // Wait until there is a new event
            poll(&mut pollfds, 1000);
        }

        Ok(())
    }

    fn switch_scene_internal(&mut self, new_scene_num: SceneNum, new_subscene_num_opt: Option<SceneNum>) -> Result<(), Box<dyn Error>> {
        if let Some(current_scene_num) = self.current_scene_num {
            if let Some(new_subscene_num) = new_subscene_num_opt {
                // Only switch subscene if there is just a subscene change.
                return self.switch_subscene_internal(new_subscene_num);
            } else if current_scene_num == new_scene_num {
                // Skip if we're already in the scene.
                return Ok(());
            }
        }

        // TODO scene bounds checking

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
            // Skip if we're already in the subscene.
            if let Some(current_subscene_num) = self.current_subscene_num {
                if current_subscene_num == new_subscene_num { return Ok(()); }
            }

            // TODO subscene bounds checking

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
        self.run_patch(self.control, SceneRunType::Patch, Some(ev))?;
        // TODO don't run patch when scene was just switched in control
        //      maybe do scene switching at the end of the full patch?
        //      in that case we'll need current_scene and new_scene in EventStream
        self.run_patch(self.patch, SceneRunType::Patch, Some(ev))?;
        if let Some(current_scene) = get_scene(&self.scenes, self.current_scene_num) {
            self.run_patch(current_scene.patch, SceneRunType::Patch, Some(ev))?;
            if let Some(current_subscene) = current_scene.get_subscene_opt(self.current_subscene_num) {
                self.run_patch(current_subscene.patch, SceneRunType::Patch, Some(ev))?;
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
        match ev {
            Event::Quit(_) => {
                self.running = false;
            },
            Event::SceneSwitch(SceneSwitchEventImpl { scene: SceneSwitchValue::Fixed(f) }) => {
                self.switch_scene_internal(f.saturating_sub(self.scene_offset), None)?;
            },
            Event::SceneSwitch(SceneSwitchEventImpl { scene: SceneSwitchValue::Offset(o) }) => {
                if let Some(current_scene) = self.current_scene_num {
                    let f = (current_scene as SceneOffset).saturating_add(*o) as SceneNum;
                    self.switch_scene_internal(f, None)?;
                }
            },
            Event::SubSceneSwitch(SubSceneSwitchEventImpl { subscene: SceneSwitchValue::Fixed(f) }) => {
                self.switch_subscene_internal(f.saturating_sub(self.scene_offset))?;
            },
            Event::SubSceneSwitch(SubSceneSwitchEventImpl { subscene: SceneSwitchValue::Offset(o) }) => {
                if let Some(current_subscene) = self.current_subscene_num {
                    let f = (current_subscene as SceneOffset).saturating_add(*o) as SceneNum;
                    self.switch_subscene_internal(f)?;
                }
            },
            _ => {
                // If there is no channel and port offset, we can directly send the event.
                if self.channel_offset == 0 && self.port_offset == 0 {
                    // Try all backends until one handles it (i.e. sends more than 0 bytes).
                    for backend in self.backends.iter_mut() {
                        let r = backend.output_event(&ev)?;
                        if r > 0 { return Ok(r); }
                    }
                // Otherwise we need to modify a copy of the event and send it.
                } else {
                    let mut ev = ev.clone();
                    self.user_event_to_backend(&mut ev);
                    // Try all backends until one handles it (i.e. sends more than 0 bytes).
                    for backend in self.backends.iter_mut() {
                        let r = backend.output_event(&ev)?;
                        if r > 0 { return Ok(r); }
                    }
                }
            }
        }
        Ok(0)
    }

    fn run_patch<'oev>(&mut self, filter: &dyn FilterTrait, run_type: SceneRunType, ev: Option<&Event<'oev>>) -> Result<(), Box<dyn Error>> {
        let mut evs = if let Some(ev) = ev { EventStream::from(ev) } else { EventStream::none() };

        // run patch
        match run_type {
            SceneRunType::Patch => filter.run(&mut evs),
            SceneRunType::Init => filter.run_init(&mut evs),
            SceneRunType::Exit => filter.run_exit(&mut evs),
        }

        // handle resulting event stream
        for ev in evs.iter() {
            self.output_event(ev)?;
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

    fn backend_event_to_user(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
                ev.channel = ev.channel.saturating_add(self.channel_offset);
            },
            Event::NoteOff(ev) => {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
                ev.channel = ev.channel.saturating_add(self.channel_offset);
            },
            Event::Ctrl(ev) => {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
                ev.channel = ev.channel.saturating_add(self.channel_offset);
            },
            Event::SysEx(ev) => {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
            },
            Event::Osc(ev) => {
                ev.port = ev.port.saturating_add(self.port_offset as usize);
            },
            _ => {}
        }
    }

    fn user_event_to_backend(&self, ev: &mut Event) {
        match ev {
            Event::NoteOn(ev) => {
                ev.port = ev.port.saturating_sub(self.port_offset as usize);
                ev.channel = ev.channel.saturating_sub(self.channel_offset);
            },
            Event::NoteOff(ev) => {
                ev.port = ev.port.saturating_sub(self.port_offset as usize);
                ev.channel = ev.channel.saturating_sub(self.channel_offset);
            },
            Event::Ctrl(ev) => {
                ev.port = ev.port.saturating_sub(self.port_offset as usize);
                ev.channel = ev.channel.saturating_sub(self.channel_offset);
            },
            Event::SysEx(ev) => {
                ev.port = ev.port.saturating_sub(self.port_offset as usize);
            },
            Event::Osc(ev) => {
                ev.port = ev.port.saturating_sub(self.port_offset as usize);
            },
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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

// https://www.reddit.com/r/rust/comments/65kflg/does_rust_have_native_epoll_support/dgcnbtd?utm_source=share&utm_medium=web2x&context=3
fn poll(fds: &mut [libc::pollfd], timeout: libc::c_int) -> libc::c_int {
    unsafe {
        libc::poll(&mut fds[0] as *mut libc::pollfd, fds.len() as libc::nfds_t, timeout)
    }
}
