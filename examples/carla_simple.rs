#![allow(non_snake_case)]
#[macro_use]
extern crate rmididings;
use rmididings::*;
use rmididings::osc::OscType as o;

/// Example patch that shows how to interact with the Carla audio-plugin host.
///
/// The first parameter of the first plugin in Carla is synchronized with
/// MIDI controller 1 (modulation wheel). Changes in Carla are sent to the MIDI
/// device, changes on the MIDI device are propagated to Carla.
///
/// Make sure Carla is running (including the engine), and OSC is enabled on
/// the default port 22752.
///
/// Note that this uses a Carla-internal OSC protocol, which could change between
/// Carla versions. It was tested with Carla version 2.3.0.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut md = RMididings::new()?;

    md.config(ConfigArguments {
        client_name: "RMididings Demo",
        in_ports: &[
            ["input", "Virtual Keyboard:Virtual Keyboard"],
            ["osc:127.0.0.1:22852", ""],     // Listen both on UDP and TCP.
        ],
        out_ports: &[
            ["output", "midisnoop:MIDI Input"],
            ["osc.tcp:", "127.0.0.1:22752"], // Send on TCP only, to the default Carla OSC port.
        ],
        ..ConfigArguments::default()
    })?;

    md.run(RunArguments {
        patch: &Fork!(
            Chain!(
                CarlaFilter(),
                OscAddrFilter("/cb"),
                ProcessOsc!(
                    o::Int, o::Int, o::Int, o::Int, o::Int, o::Float, o::String,
                    |action: &i32, plugin_id: &i32, ival: &i32, _, _, fval: &f32, _| {
                        // Only react to value changed callback for the first plugin and the first parameter.
                        if *action == 5 && *plugin_id == 0 && *ival == 0 {
                            Chain!(Ctrl(1, (*fval * 127.0) as i32), Synth())
                        } else {
                            Chain!(Discard())
                        }
                    }
                )
            ),
            Chain!(
                SynthFilter(),
                TypeFilter!(Ctrl),
                CtrlFilter(1),
                Process!(|ev: &Event| {
                    match ev {
                        Event::Ctrl(ev) => Box::new(CarlaSetParamValue(0, 0, ev.value as f32 / 127.0)),
                        _ => Box::new(Discard()),
                    }
                })
            ),
            Init!(Chain!(Osc!("/register", o::String("osc.tcp://127.0.0.1:22852/Carla".to_string())), CarlaPort())),
            Exit!(Chain!(Osc!("/unregister", o::String("127.0.0.1".to_string())), CarlaPort()))
        ),
       ..RunArguments::default()
    })?;

    Ok(())
}


fn SynthFilter() -> PortFilter {
    PortFilter(1)
}

fn Synth() -> Port {
    Port(1)
}

// For accepted messages, see: https://github.com/falkTX/Carla/blob/main/source/backend/engine/CarlaEngineOscHandlers.cpp
// Note that the format can change between Carla releases, so take care!
fn CarlaSetParamValue(plugin_id: i32, param_id: i32, value: f32) -> FilterChain<'static> {
    Chain!(Osc!(format!("/{}/set_parameter_value", plugin_id), o::Int(param_id), o::Float(value)), Carla())
}

fn CarlaFilter() -> FilterChain<'static> {
    Chain!(PortFilter(2), OscStripPrefix("/Carla"), Not!(OscAddrFilter("/runtime")))
}

fn Carla() -> FilterChain<'static> {
    Chain!(OscAddPrefix("/Carla"), CarlaPort())
}

fn CarlaPort() -> Port {
    Port(2)
}