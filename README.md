# Rust JACK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/RustAudio/rust-jack/workflows/Rust/badge.svg)](https://github.com/RustAudio/rust-jack/actions)

[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)

Rust bindings for the [JACK Audio Connection Kit](https://jackaudio.org). These bindings work on every
operating system that JACK does.

The JACK server is usually started by the user or system. Clients can request that the JACK server is
started on demand when they connect, but this can be disabled by the user and is the recommended
configuration.

  * Linux and BSD users may install JACK1, JACK2, or Pipewire JACK from their system package
manager.
  * Windows users may install JACK from the
  [official website](http://jackaudio.org/downloads/) or
  [Chocolatey](https://community.chocolatey.org/packages/jack).
  * macOS users may install JACK from the [official website](http://jackaudio.org/downloads/) or
  [Homebrew](https://formulae.brew.sh/formula/jack).

[:heart: Sponsor](https://github.com/sponsors/wmedrano)

Refer to the [documentation](https://docs.rs/jack/) for details about the API, building, and packaging.
Also take a look at the `examples` directory for usage.

## Testing

Testing requires setting up a dummy server and running the tests using a single
thread.

```bash
# Set up a dummy server for tests. The script is included in this repository.
./dummy_jack_server.sh &
# Run tests with limited concurrency.
RUST_TEST_THREADS=1 cargo test
```

**Note:** We use a single thread for tests since too many client
instantiations in short periods of time cause the JACK server to become flaky.

### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Increase the value used by `sleep_on_test` in `client/common.rs`.

Another case is that libjack may be broken on your setup. Try switching between
libjack and libjack2 (they have the same API and libjack2 isn't necessarily
newer than libjack), or using a different version.
