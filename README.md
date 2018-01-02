# Rust JACK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack)

[![Build Status](https://travis-ci.org/RustAudio/rust-jack.svg?branch=master)](https://travis-ci.org/RustAudio/rust-jack)
[![Coverage Status](https://coveralls.io/repos/github/RustAudio/rust-jack/badge.svg?branch=master&cache_less=3)](https://coveralls.io/github/RustAudio/rust-jack)


Nice Rust bindings for
[JACK Audio Connection Kit](https://www.jackaudio.org)

[Documentation for Master](https://RustAudio.github.io/rust-jack/jack/index.html)

Check out the `examples` directory for usage.

## Crates

### Stable
```toml
[dependencies]
jack = "0.5.7"
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

Testing requires setting up a test JACK server.

### Setting Up JACK Dummy Server

```bash
$ # Start a test jack server instance with buffer size 1024 and a sample rate of 44.1kHz.
$ # Make sure to set RUST_TEST_THREADS=1 when running cargo test.
$ ./dummy_jack_server.sh
```

### Running the tests

```bash
$ export RUST_TEST_THREADS=1
$ cargo test
```

If you want test coverage as well, try `cargo kcov`.

```bash
$ export RUST_TEST_THREADS=1
$ cargo install cargo-kcov
$ cargo kcov
```

#### Possible Issues

1. Rust runs tests in parallel, but the tests affect the global jack state so set the environment variable `RUST_TEST_THREADS` to 1.
2. Instability caused by interacting with JACK too quickly. Increase the value used by `sleep_on_test` in `client/common.rs`.
3. libjack may be broken on your setup.  Try switching between libjack and libjack2 (they have the same API and libjack2 isn't necessarily newer than libjack), or using a different version.


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
