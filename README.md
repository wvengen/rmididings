# RMididings

[![](https://img.shields.io/crates/v/rmididings.svg)](https://crates.io/crates/rmididings)

RMididings is a partial clone of [mididings](http://das.nasophon.de/mididings/)
in [Rust](https://www.rust-lang.org/), allowing one to use a syntax not unlike
mididings for MIDI event routing and processing. Mididings is a Python-based
MIDI router and processor.

It is in early development, and many things are not available. What is:
- `NoteOn`, `NoteOff`, `Ctrl` and `SysEx` events.
- Only supports the `alsa` backend, which ties it to Linux.
- A limited set of filters, modifiers and generators.
- A limited set of connections: `Chain!`, `Fork!` and `Not!`.
- Scenes and subscenes, scene switching and running a single patch.
- Pre, post, init, exit and control patches.

Some missing things can be implemented, but there are some limitations using Rust,
e.g. syntax can differ, and not all variations of argument types to filters etc.
are supported. We'll see.

## Running

You'll need [Rust](https://www.rust-lang.org/) 1.52.0+ with [Cargo](https://doc.rust-lang.org/cargo/).
The easiest option to get a recent enough Rust is using [Rustup](https://rustup.rs/).

You also need the ALSA headers. On Debian you would need to run `apt-get install libasound2-dev`,
on Fedora `dnf install alsa-lib-devel`.

Rmididings is a crate, which means that you write your own program that uses it. Let's start with a
simple example. Create a project directory, and a subdirectory `src`, where you put the following file
as `main.rs`:

```rust
// src/main.rs
#[macro_use]
extern crate rmididings;
use rmididings::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut md = RMididings::new()?;

  md.config(ConfigArguments {
    in_ports:  &[["input",  "Virtual Keyboard:Virtual Keyboard"]],
    out_ports: &[["output", "midisnoop:MIDI Input"]],
    ..ConfigArguments::default()
  })?;

  md.run(RunArguments {
    patch: &Pass(),
    ..RunArguments::default()
  })?;

  Ok(())
}
```

To get this running, you'll also need a `Cargo.toml` in the project directory:

```
[package]
name = "myproject"
version = "0.0.1"

[dependencies]
rmididings = "^0.1.0"
```

Then, from within the project directory, run `cargo run`, and you're set. This sample
program passes all events from the input to the output port. When before running you
have [vkeybd](https://github.com/tiwai/vkeybd) and [midisnoop](https://github.com/surfacepatterns/midisnoop)
running (same package names in Debian/Ubuntu), they will be connected automatically.
Terminate the program with <kbd>Ctrl-C</kbd>

## Building a patch

The patch above merely passes all events with `Pass()`. It becomes more interesting when
we start building a chain of filters. Instead of `&Pass()`, we could use the following:

```
&Chain!(
  ChannelFilter(1),
  Fork!(
    Pass(),
    Chain!(Transpose(4), VelocityMultiply(0.8)),
    Chain!(Transpose(7), VelocityMultiply(0.5))
  ),
  Channel(2)
)
```

This chain ignores any other channel than 1 and returns a major chord for the note, with
higher notes slightly attenuated. Finally all notes are sent out at channel 2.

## Scenes

Instead of a patch, one can pass `scenes` to the `run` function:

```rust
md.run(RunArguments {
    scenes: &[
        // Scene 1
        &Scene {
            name: "Run",
            patch: &Pass(),
            ..Scene::default()
        },
        // Scene 2
        &Scene {
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
```

Here there are two scenes, one that passes all events and one that discards them.
The `control` patch is always run, here the note central C and the following D are used
to switch between the scenes.

## Plans

- OSC support, with filters etc. like MIDI (this was hard to do in mididings without threading issues).
- Improving syntax with macros.
- Adding port connected / disconnected event types.
- Make `SysEx` work without needing to borrow (allows returning them from a function).

## License

[GPL-3.0 or later](LICENSE.md) or later.
