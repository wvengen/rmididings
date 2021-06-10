# RMididings

_This project is in its early stages, take care._

RMididings is a partial clone of [mididings](http://das.nasophon.de/mididings/)
in [Rust](https://www.rust-lang.org/), allowing one to use a syntax not unlike
mididings for MIDI event routing and processing. Mididings is a Python-based
MIDI router and processor.

It is very early in development, and most things don't work. What does work:
- `NoteOn`, `NoteOff`, `Ctrl` and `SysEx` events.
- Only supports the `alsa` backend, which ties it to Linux.
- A limited set of filters and generators.
- A limited set of connections, at the moment only chaining.
- Running a single patch.

Some missing things can be implemented, but there are some limitations using Rust,
e.g. syntax can differ, and not all variations of argument types to filters etc.
are supported. We'll see.

## Plans

- Adding more connections (and overhauling the filter chains).
- Adding scenes.
- Adding subscenes.
- Improving syntax with macros.
- OSC support, with filters etc. like MIDI (this was hard to do in mididings without threading issues).
- Adding port connected / disconnected event types.

## License

TODO (something open source)
