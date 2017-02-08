//! Rust bindings for JACK, a real-time audio and midi library.
//!
//! # Server
//!
//! JACK provides a high priority server to manipulate audio and midi across applications. The rust
//! jack crate does not provide this functionality, so a server has to be set up with the `jackd`
//! commandline tool, `qjackctl` the gui tool, or another method.
//!
//! # Client
//!
//! Typically, applications connect clients to the server. For the rust jack crate, a connection can
//! be made with `client::Client::new`, which returns a `client::Client`.
//!
//! The `Client` can query the server for information, register ports, and manage connections for
//! ports.
//!
//! To commence processing audio/midi and other information in real-time, rust jack provides the
//! `client::AsyncClient::new`, which consumes a `Client` and an object that implements
//! `JackHandler`, and returns a `AsyncClient` which is processing data in real-time with the
//! provided `JackHandler` object.
//!
//! # Port
//!
//! A `Client` may obtain port information through the `Client::port_by_id` and
//! `Client::port_by_name` methods. These ports can be used to manage connections or to obtain port
//! metadata, though their port data (audio buffers and midi buffers) cannot be accessed safely.
//!
//! Ports can be registered with the `Client::register_port` method. This requires a `PortSpec`. The
//! jack crate comes with common specs such as `AudioInSpec`, `AudioOutSpec`, `MidiInSpec`, and
//! `MidiOutSpec` under the `port` mod.
//!
//! To access the data of registered ports, use wrappers that are valid when a `ProcessScope` is
//! present. The ones provided by the rust jack crate are `AudioInPort`, `AudioOutPort`,
//! `MidiInPort`, and `MidiOutPort`, all of which are under the `port` mod. It is also possible to
//! access the data without wrapping the newly registered `Port<PortSpec>` by using the
//! `Port::buffer` method, but this returns a void pointer and is unsafe.
#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;


/// Create and manage client connections to a JACK server.
pub mod client;

/// Control error and info logging from JACK.
pub mod logging;

/// Enum types in jack.
pub mod jack_enums;

mod jack_utils;

/// Types for safely interacting with port data from JACK.
pub mod port;

/// Platform independent types.
pub mod primitive_types;

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> primitive_types::JackTime {
    unsafe { jack_sys::jack_get_time() }
}

/// Contains every trait defined in the jack crate.
pub mod traits {
    pub use client::JackHandler;
    pub use port::PortSpec;
}

/// Contains most functionality needed to interact with JACK.
pub mod prelude {
    pub use primitive_types::{JackFrames, JackTime, JackPortId};
    pub use jack_enums::{JackErr, JackControl, LatencyType};
    pub use client::{AsyncClient, Client, CycleTimes, JackHandler, ProcessHandler, ProcessScope};
    pub use client::CLIENT_NAME_SIZE;
    pub use client::{ClientOptions, ClientStatus, client_options, client_status};
    pub use port::{AudioInPort, AudioInSpec, AudioOutPort, AudioOutSpec, MidiInPort, MidiInSpec,
                   MidiIter, MidiOutPort, MidiOutSpec, Port, RawMidi, Unowned, UnownedPort};
    pub use port::{PORT_NAME_SIZE, PORT_TYPE_SIZE};
    pub use port::{PortFlags, port_flags};
    pub use port::PortSpec;
    pub use logging::{set_info_callback, get_info_callback, reset_info_callback,
                      set_error_callback, get_error_callback, reset_error_callback};
}

#[cfg(test)]
mod test;
