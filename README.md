# Rust JACK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://goo.gl/j4uiqg)

[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://goo.gl/ZCHble)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://goo.gl/spnykb)

[![Build Status](https://travis-ci.org/wmedrano/rust-jack.svg?branch=master)](https://goo.gl/KCqB7F)
[![Coverage Status](https://coveralls.io/repos/github/wmedrano/rust-jack/badge.svg?branch=master)](https://goo.gl/KCqB7F)


Nice Rust bindings for
[JACK Audio Connection Kit](https://goo.gl/jYMGO8)

[Documentation for Master](https://goo.gl/xvXezZ)

Check out the `examples` directory for usage.

## Crates

### Stable
```toml
[dependencies]
jack = "0.5.0"
```

### Master
```toml
[dependencies]
jack = { git = "https://github.com/wmedrano/rust-jack.git" }
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

Testing is a little awkward to setup since it relies on a JACK server.

### Setting Up JACK Dummy Server

```bash
$ ./dummy_jack_server.sh
```

which runs the command

```bash
$ jackd -r -ddummy -r44100 -p1024 & # Start the dummy JACK server
```

Testing expects there to be an available JACK server running at a sample rate of
44.1kHz and a buffer size of 1024 samples.

### Running the tests

```bash
$ cargo test
```

If you want test coverage as well, try `cargo kcov`.

```bash
$ cargo install cargo-kcov
$ cargo kcov
```

#### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Rust runs tests in parallel, it may be possible that the JACK server is not keeping up. Set the environment variable `RUST_TEST_THREADS` to 1.
2. Increase the value used by `sleep_on_test` in `client/common.rs`.

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
