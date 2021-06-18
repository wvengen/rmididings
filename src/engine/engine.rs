use std::error::Error;
use std::{thread, time};

use crate::proc::SceneNum;

use crate::backend::*;
use super::{RunArguments, Runner};

pub enum BackendType {
    Null,
    #[cfg(feature = "alsa")]
    Alsa,
}

pub struct ConfigArguments<'a> {
    pub backend: BackendType,
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
            #[cfg(feature = "alsa")]
            backend: BackendType::Alsa,
            #[cfg(not(feature = "alsa"))]
            backend: BackendType::Null,
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

pub struct RMididings<'a> {
    backends: Vec<Box::<dyn Backend<'a> + 'a>>,
    port_offset: u8,
    channel_offset: u8,
    scene_offset: u8,
    initial_scene_num: SceneNum,
}

impl<'a, 'cfgargs: 'a> RMididings<'a> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            backends: vec![],
            port_offset: 1,
            channel_offset: 1,
            scene_offset: 1,
            initial_scene_num: 0,
        })
    }

    pub fn config(&mut self, args: ConfigArguments<'cfgargs>) -> Result<(), Box<dyn Error>> {
        self.backends = vec![match args.backend {
                BackendType::Null => Box::new(NullBackend::new()?),
                #[cfg(feature = "alsa")]
                BackendType::Alsa => Box::new(AlsaBackend::new()?),
            },
            // TODO include Osc backend only when osc ports are defined
            #[cfg(feature = "osc")]
            Box::new(OscBackend::new()?),
            // #[cfg(feature = "dbus")]
            // Box::new(DbusBackend::new()?),
        ];

        for b in self.backends.iter_mut() { b.set_client_name(args.client_name)?; }

        for (port_id, [name, connect]) in args.in_ports.iter().enumerate() {
            for backend in self.backends.iter_mut() {
                if backend.create_in_port(port_id, name)? {
                    backend.connect_in_port(port_id, connect)?;
                    break;
                }
            }
        }

        for (port_id, [name, connect]) in args.out_ports.iter().enumerate() {
            for backend in self.backends.iter_mut() {
                if backend.create_out_port(port_id, name)? {
                    backend.connect_out_port(port_id, connect)?;
                    break;
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

    pub fn run(&mut self, args: RunArguments<'_>) -> Result<(), Box<dyn Error>> {
        Runner::new(
            args,
            &mut self.backends,
            self.port_offset,
            self.channel_offset,
            self.scene_offset,
            self.initial_scene_num,
        ).run()
    }
}