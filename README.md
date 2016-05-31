# rust jack

Nice rust bindings for the
[JACK Audio Connection Kit](http://www.jackaudio.org/)


## "C" & rust API differences
* String lengths in the "C" API include the `NULL` character while these rust
  bindings do not. generally `rust_size(x) = c_size(x) - 1`.
* "C" bindings require functions to be registered while rust bindings register
  an object with the trait `JackHandler`.
* `jack_on_shutdown` has been removed, uses only `jack_on_info_shutdown`.
* rust enums vs C enums
* rust bitflags vs C integers used as flags
* deprecated Jack functions are not used/implemented in rust bindings


## Progress

Sections based on the
[main page](http://jackaudio.org/files/docs/html/index.html) sections on the
Jack API.

* Creating & manipulating clients - completed, in `client.rs`
* Setting Client Callbacks - completed, in `callbacks.rs`
* Creating and managing client threads - none
* Controlling & querying JACK server operation - none
* Creating & manipulating ports - about half, registering/unregistering and getting/setting data, add to `port.rs`, ports created in `client.rs`.
* Looking up ports - none
* Managing and determining latency - none
* Handling time - none
* Transport and Timebase control - none
* Controlling error/information output - none
* The non-callback API - none
* Reading and writing MIDI data - none
* Session API for clients. - none
* managing support for newer/older versions of JACK - none
* the API for starting and controlling a JACK server - none
* * Metadata API. - none
