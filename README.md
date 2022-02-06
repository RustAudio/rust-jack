# Rust JACK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/RustAudio/rust-jack/workflows/Rust/badge.svg)](https://github.com/RustAudio/rust-jack/actions)

[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)

Rust bindings for [JACK Audio Connection Kit](https://jackaudio.org).

[:heart: Sponsor](https://github.com/sponsors/wmedrano)

Check out the `examples` directory for usage.

## Crates

```toml
[dependencies]
jack = "0.9"
```

### Windows

Install `JACK` from the [official website](http://jackaudio.org/downloads/).
libjack64.dll (or libjack.dll for 32bit) is required for Windows to work.

## Running

- `libjack` is required. Consult your package manager or the [official](http://jackaudio.org/downloads/) website.

- The general workflow for a JACK application is to start up a JACK daemon and connect the client to it. [qjackctl](http://qjackctl.sourceforge.net/) is a convinient way to configure and bring up a JACK server through a GUI.

- [JACK FAQ](http://jackaudio.org/faq/)

## Testing

Testing requires setting up a dummy server and running the tests using a single
thread.

```bash
# Set up a dummy server for tests.
./dummy_jack_server.sh &
# Run tests with limited concurrency.
RUST_TEST_THREADS=1 cargo test
```

**Note:** We use a single thread for tests since too multiple client
instantiations in short periods of time cause the JACK server to become flaky.

### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Increase the value used by `sleep_on_test` in `client/common.rs`.

Another case is that libjack may be broken on your setup. Try switching between
libjack and libjack2 (they have the same API and libjack2 isn't necessarily
newer than libjack), or using a different version.
