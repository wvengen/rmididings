#[macro_use]
extern crate rmididings;
use rmididings::*;
use rosc::OscType as o;

/// Example patch that works with livedings.
///
/// Note that in the future there will probably came an implementation that you can
/// plug into your RMididings program, either as a filter for use e.g. in the control
/// patch, or as a hook.
///
/// You can do scene switches from livedings, but scene switches happening elsewhere
/// within a patch are not communicated to livedings.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut md = RMididings::new()?;

    md.config(ConfigArguments {
        client_name: "RMididings Demo",
        in_ports: &[
            ["input", "Virtual Keyboard:Virtual Keyboard"],
            ["osc.udp://localhost:56418", ""],
        ],
        out_ports: &[
            ["output", "midisnoop:MIDI Input"],
            ["osc.udp:", "localhost:56419"],
        ],
        ..ConfigArguments::default()
    })?;

    md.run(RunArguments {
        scenes: &[
            &Scene { // 1
                name: "Run",
                patch: &Not!(TypeFilter!(Osc)),
                ..Scene::default()
            },
            &Scene { // 2
                name: "Pause",
                patch: &Discard(),
                ..Scene::default()
            }
        ],
        control: &Chain!(TypeFilter!(Osc), OscStripPrefix("/mididings"), Fork!(
            Chain!(OscAddrFilter("/query"),
                Fork!(
                    Osc!("/data_offset", o::Int(1)),
                    Osc!("/begin_scenes"),
                    Osc!("/add_scene", o::Int(1), o::String("Run".to_string())),
                    Osc!("/add_scene", o::Int(2), o::String("Pause".to_string())),
                    Osc!("/end_scenes")
                ),
                OscAddPrefix("/mididings")
            ),
            Chain!(OscAddrFilter("/switch_scene"), ProcessOsc!(o::Int, |s| SceneSwitch(s as u8))),
            Chain!(OscAddrFilter("/next_scene"), SceneSwitchOffset(1)),
            Chain!(OscAddrFilter("/prev_scene"), SceneSwitchOffset(-1)),
            Chain!(OscAddrFilter("/prev_subscene"), SubSceneSwitchOffset(-1)),
            Chain!(OscAddrFilter("/next_subscene"), SubSceneSwitchOffset(1)),
            Chain!(OscAddrFilter("/panic"), Panic()),
            Chain!(OscAddrFilter("/quit"), Quit())
        )),
        // We can't notify on scene switch yet.
        // post: &Fork!(
        //     Pass(),
        //     Chain!(TypeFilter(SceneSwitch), ... Process(|ev| Osc!("/mididings/current_scene", scene, subscene)))
        // ),
        ..RunArguments::default()
    })?;

    Ok(())
}