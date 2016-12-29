# Rust JACK

[![Build Status](https://travis-ci.org/wmedrano/rust-jack.svg?branch=master)](https://travis-ci.org/wmedrano/rust-jack)
[![crates.io](https://img.shields.io/crates/v/jack.svg)](https://crates.io/crates/jack)
[![docs.rs](https://docs.rs/jack/badge.svg)](https://docs.rs/jack/)

Nice Rust bindings for
[JACK Audio Connection Kit](http://www.jackaudio.org/)

[Documentation for Master](https://wmedrano.github.io/rust-jack/jack/index.html)

Check out the `examples` directory.

## Running

* `libjack` is required. Consult your package manager or the [official](http://www.jackaudio.org/downloads/) website.

* The general workflow for a JACK application is to start up a JACK daemon and connect the client to it. [qjackctl](http://qjackctl.sourceforge.net/) is a convinient way to configure and bring up a JACK server through a GUI.

## Testing

Testing is a little awkward to setup since it relies on a JACK server.

### Setting Up JACK Dummy Server

```bash
$ ./dummy_jack_server.sh
```

which runs

```bash
$ jackd -r -ddummy -r44100 -p1024 & # Start the dummy JACK server
```

Testing expects there to be an available JACK server running at a sample rate of
44.1kHz and a buffer size of 1024 samples.

#### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Rust runs tests in parallel, it may be possible that the JACK server is not keeping up. Set the environment variable `RUST_TEST_THREADS` to 1.
2. Increase the value of `DEFAULT_SLEEP_TIME` in `test.rs`.

Another case is that libjack may be broken.
Try switching between libjack and
libjack2 (they have the same API and libjack2 isn't necessarily newer than
libjack), or using a different version.

### Running the tests

```bash
$ cargo test
```

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
