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
- Running a single patch.

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
let patch = Chain!(Pass());
```

Even with the limited set we have now, it is possible to make slightly more complex chains:

```rust
let patch = Chain!(
    ChannelFilter(0),
    Fork!(
        Pass(),
        Chain!(Transpose(4), VelocityMultiply(0.8)),
        Chain!(Transpose(7), VelocityMultiply(0.5))
    ),
    Channel(1)
);
```

This example ignores any other channel than 0 (note that mididings' `data_offset` is not
implemented yet, so we start counting at 0) and returns a major chord for the note, with
higher notes slightly attenuated. Finally this is sent out at channel 1.


## Plans

- Adding more connections (and overhauling the filter chains).
- Adding scenes.
- Adding subscenes.
- Improving syntax with macros.
- OSC support, with filters etc. like MIDI (this was hard to do in mididings without threading issues).
- Adding port connected / disconnected event types.

## License

TODO (something open source)
