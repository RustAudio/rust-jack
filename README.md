# Rust JACK

[![Build Status](https://travis-ci.org/wmedrano/rust-jack.svg?branch=master)](https://travis-ci.org/wmedrano/rust-jack)

Nice Rust bindings for
[JACK Audio Connection Kit](http://www.jackaudio.org/)

[Documentation](https://wmedrano.github.io/rust-jack/jack/index.html)

## Running

* `libjack` is required. Consult your package manager or the [official](http://www.jackaudio.org/downloads/) website.

* The general workflow for a jack application is to start up a jack daemon and connect the client to it. [qjackctl](http://qjackctl.sourceforge.net/) is a convinient way to configure and bring up a jack server through a GUI.

## Running Tests

Testing is a little awkward to setup since it relies on a Jack server.

### Setting Up Jack Dummy Server
Testing expects there to be an available Jack server running at a sample rate of
44.1kHz and a buffer size of 1024 samples.

```bash
$ jackd -r -ddummy -r44100 -p1024 & # Start the dummy jack server
```

#### Possible Issues

If the tests are failing, a possible gotcha may be timing issues.

1. Rust runs tests in parallel, it may be possible that the jack server is not keeping up. Set the environment variable RUN_TEST_TASKS to 1.
2. Increase the value of `DEFAULT_SLEEP_TIME` in `test.rs`.

Another case can be that libjack is broke. Try switching between libjack and
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
* deprecated Jack functions are not used/implemented in Rust bindings


## Progress

Sections based on the
[main page](http://jackaudio.org/files/docs/html/index.html) sections on the
Jack API.

### TODO
* Top priority: MIDI!!!
* Managing and determining latency
* Transport and Timebase control
* The non-callback API (possibly skip)
* Reading and writing MIDI data
* Session API for clients.
* managing support for newer/older versions of JACK
* the API for starting and controlling a JACK server
* * Metadata API.

### Other TODOS
* make safer
* better error reporting
* better testing
