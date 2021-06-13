# RMididings

_This project is in its early stages, take care._

RMididings is a partial clone of [mididings](http://das.nasophon.de/mididings/)
in [Rust](https://www.rust-lang.org/), allowing one to use a syntax not unlike
mididings for MIDI event routing and processing. Mididings is a Python-based
MIDI router and processor.

It is very early in development, and most things don't work. What does work:
- `NoteOn`, `NoteOff`, `Ctrl` and `SysEx` events.
- Only supports the `alsa` backend, which ties it to Linux.
- A limited set of filters, modifiers and generators.
- A limited set of connections: `Chain` and `Fork`.
- Scenes, scene switching and running a single patch.
- Pre, post, init, exit and control patches.

Some missing things can be implemented, but there are some limitations using Rust,
e.g. syntax can differ, and not all variations of argument types to filters etc.
are supported. We'll see.

## Running

You'll need [Rust](https://www.rust-lang.org/) 1.52.0+ with [Cargo](https://doc.rust-lang.org/cargo/).
The easiest option to get a recent enough Rust is using [Rustup](https://rustup.rs/). Then run

```sh
cargo run
```

and you're set. The default program passes all events from the input to the output port.
Terminate the program with <kbd>Ctrl-C</kbd>

## Building a patch

The main patch is defined in [`src/main.rs`](src/main.rs) and is passed as the `patch` argument
to the `run` function. You need to start with a connection type and include filters, modifiers
or generators inside of it. The default is just to pass all events:

```rust
let patch = Pass();
```

Even with the limited set we have now, it is possible to make slightly more complex chains:

```rust
let patch = Chain!(
    ChannelFilter(1),
    Fork!(
        Pass(),
        Chain!(Transpose(4), VelocityMultiply(0.8)),
        Chain!(Transpose(7), VelocityMultiply(0.5))
    ),
    Channel(2)
);
```

This example ignores any other channel than 1 and returns a major chord for the note, with
higher notes slightly attenuated. Finally this is sent out at channel 2.

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

- Adding subscenes.
- OSC support, with filters etc. like MIDI (this was hard to do in mididings without threading issues).
- Improving syntax with macros.
- Adding port connected / disconnected event types.

## License

TODO (something open source)
