# JACK (for Rust)

Rust bindings for [JACK Audio Connection Kit](https://jackaudio.org).

[![Crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![Docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)
[![Test](https://github.com/RustAudio/rust-jack/actions/workflows/testing.yml/badge.svg)](https://github.com/RustAudio/rust-jack/actions/workflows/testing.yml)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[:heart: Sponsor](https://github.com/sponsors/wmedrano)

## Overview

JACK is a low-latency audio server that allows multiple applications to share
audio and MIDI devices and route signals between each other. This crate provides
safe Rust bindings to create JACK clients that can process audio and MIDI in
real-time.

## Documentation

- [Guide](https://rustaudio.github.io/rust-jack) - Quickstart, features, and tutorials
- [API Reference](https://docs.rs/jack/) - Complete API documentation

## Quick Example

```rust
use std::io;

fn main() {
    // Create a JACK client
    let (client, _status) =
        jack::Client::new("rust_jack_simple", jack::ClientOptions::default()).unwrap();

    // Register input and output ports
    let in_port = client
        .register_port("input", jack::AudioIn::default())
        .unwrap();
    let mut out_port = client
        .register_port("output", jack::AudioOut::default())
        .unwrap();

    // Create a processing callback that copies input to output
    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            out_port.as_mut_slice(ps).clone_from_slice(in_port.as_slice(ps));
            jack::Control::Continue
        },
    );

    // Activate the client
    let _active_client = client.activate_async((), process).unwrap();

    // Wait for user to quit
    println!("Press enter to quit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
}
```

See the [examples](examples/) directory for more.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
jack = "0.13"
```

### JACK Server Setup

A JACK server must be running for clients to connect. Install one of:

- **Linux/BSD**: JACK2 (lowest latency), Pipewire JACK (easiest), or JACK1 via
  your package manager
- **Windows**: [Official installer](http://jackaudio.org/downloads/) or
  [Chocolatey](https://community.chocolatey.org/packages/jack)
- **macOS**: [Official installer](http://jackaudio.org/downloads/) or
  [Homebrew](https://formulae.brew.sh/formula/jack)

By default, clients request the server to start on demand. Use
`ClientOptions::default()` or the `NO_START_SERVER` flag to disable this.

## Testing

Tests require a dummy JACK server and must run single-threaded:

```sh
./dummy_jack_server.sh &
cargo nextest run
```

If `cargo nextest` is unavailable: `RUST_TEST_THREADS=1 cargo test`

### Troubleshooting

- Use `cargo nextest` instead of `cargo test` for better handling of timing-sensitive tests
- Try libjack2 or pipewire-jack if tests fail with your current JACK implementation

## License

MIT - see [LICENSE](LICENSE) for details.
