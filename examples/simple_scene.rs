use std::error::Error;

extern crate rmididings;
use rmididings::*;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {

    let mut md = RMididings::new()?;

    md.config(ConfigArguments {
        client_name: "RMididings Demo",
        in_ports: &[
            ["input", "Virtual Keyboard:Virtual Keyboard"],
        ],
        out_ports: &[
            ["output", "midisnoop:MIDI Input"],
        ],
        ..ConfigArguments::default()
    })?;

    md.run(RunArguments {
        scenes: &[
            &Scene { // 1
                name: "Run",
                patch: &Pass(),
                ..Scene::default()
            },
            &Scene { // 2
                name: "Pause",
                patch: &Discard(),
                ..Scene::default()
            }
        ],
        control: &Fork!(
            Chain!(KeyFilter(62), SceneSwitch(2), Discard()),
            Chain!(KeyFilter(60), SceneSwitch(1), Discard())
        ),
        ..RunArguments::default()
    })?;

    Ok(())
}

