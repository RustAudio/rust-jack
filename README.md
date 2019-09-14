# Rust JACK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)

[![Build Status](https://travis-ci.org/RustAudio/rust-jack.svg?branch=master)](https://travis-ci.org/RustAudio/rust-jack)


Nice Rust bindings for
[JACK Audio Connection Kit](https://www.jackaudio.org)

[Documentation for Master](https://RustAudio.github.io/rust-jack/jack/index.html)

Check out the `examples` directory for usage.

## Crates

### Stable
```toml
[dependencies]
jack = "0.6"
```

### Master
```toml
[dependencies]
jack = { git = "https://github.com/RustAudio/rust-jack.git" }
```


## Completeness

For details on which functions from the JACK headers have been implemented, see `ffi_completeness.md`.

More high-level, creating clients, creating/reading/writing/connecting ports, audio, and midi are supported.

Missing categories include, JACK threading, synchronous processing, transport and control functionality.

## Running

* `libjack` is required. Consult your package manager or the [official](http://www.jackaudio.org/downloads/) website.

* The general workflow for a JACK application is to start up a JACK daemon and connect the client to it. [qjackctl](http://qjackctl.sourceforge.net/) is a convinient way to configure and bring up a JACK server through a GUI.

* [JACK FAQ](http://www.jackaudio.org/faq/)


## Testing

Testing requires setting up a dummy server and running the tests using a single
thread.

```bash
$ # Set up a dummy server for tests.
$ ./dummy_jack_server.sh
$ # Run tests with limited concurrency.
$ RUST_TEST_THREADS=1 cargo test
```

**Note:** We use a single thread for tests since too multiple client
instantiations in short periods of time cause the JACK server to become flaky.

#### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Increase the value used by `sleep_on_test` in `client/common.rs`.

Another case is that libjack may be broken on your setup.  Try switching between
libjack and libjack2 (they have the same API and libjack2 isn't necessarily
newer than libjack), or using a different version.


## "C" & Rust API differences
* String lengths in the "C" API include the `NULL` character while these Rust
  bindings do not. generally `rust_size(x) = c_size(x) - 1`.
* "C" bindings require functions to be registered while Rust bindings register
  an object with a trait.
* `jack_on_shutdown` has been removed, uses only `jack_on_info_shutdown`.
* Rust enums vs C enums
* Rust bitflags vs C integers used as flags
* deprecated JACK functions are not used/implemented in Rust bindings


## C JACK API

[Main Page](http://jackaudio.org/files/docs/html/index.html)
