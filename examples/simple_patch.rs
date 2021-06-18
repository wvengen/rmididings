#[macro_use]
extern crate rmididings;
use rmididings::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {

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

    println!("Started");

    let patch = Pass();

    md.run(RunArguments {
        patch: &patch,
        ..RunArguments::default()
    })?;

    Ok(())
}
