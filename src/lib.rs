//! JACK is a low-latency audio server that supports Linux, OS X, Solaris, FreeBSD, and Windows. It
//! can connect several client applications to an audio device, and allow them to share audio with
//! each other.


#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

mod callbacks;
mod client;
mod info;
mod jack_enums;
mod jack_flags;
mod jack_utils;
mod port;
mod port_impls;
mod primitive_types;

pub use callbacks::{ProcessScope, JackHandler};
pub use client::CLIENT_NAME_SIZE;
pub use client::{JackClient, Client, ActiveClient, CycleTimes};
pub use info::{set_info_callback, set_error_callback};
pub use jack_enums::{JackControl, JackErr};
pub use jack_flags::{ClientOptions, client_options};
pub use jack_flags::{ClientStatus, client_status};
pub use jack_flags::{PortFlags, port_flags};
pub use port::{PORT_NAME_SIZE, PORT_TYPE_SIZE};
pub use port::{Port, PortSpec, Unowned, UnownedPort};
pub use port_impls::{AudioInSpec, AudioInPort, AudioOutSpec, AudioOutPort};
pub use port_impls::{MidiInSpec, MidiInPort, MidiIter, MidiOutSpec, MidiOutPort, RawMidi};
pub use primitive_types::{JackFrames, JackPortId, JackTime};

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> JackTime {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
pub use jack_utils::{default_sleep, default_longer_sleep};
#[cfg(test)]
pub struct DummyHandler;
#[cfg(test)]
impl JackHandler for DummyHandler {}
#[cfg(test)]
mod test;
