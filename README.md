# Rust JACK

[![Build Status](https://travis-ci.org/wmedrano/rust-jack.svg?branch=master)](https://travis-ci.org/wmedrano/rust-jack)

Nice Rust bindings for
[JACK Audio Connection Kit](http://www.jackaudio.org/)

[Documentation](https://wmedrano.github.io/rust-jack/jack/index.html)

## Running

* `libjack` is required. Consult your package manager or the [official](http://www.jackaudio.org/downloads/) website.

* The general workflow for a jack application is to start up a jack daemon and connect the client to it. [qjackctl](http://qjackctl.sourceforge.net/) is a nice way to convinient way to configure and bring up a jack server through a GUI.


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
