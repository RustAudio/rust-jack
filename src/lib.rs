//! Rust bindings for JACK, a real-time audio and midi library.
//!
//! # Server
//!
//! JACK provides a high priority server to manipulate audio and midi across applications. The rust
//! jack crate does not provide server creation functionality, so a server has to be set up with the
//! `jackd` commandline tool, `qjackctl` the gui tool, or another method.
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
//! `Client::activate_async`, which consumes the `Client`, an object that implements
//! `NotificationHandler` and an object that implements `ProcessHandler` and returns a
//! `AsyncClient`. `AsyncClient` processes the data in real-time with the provided handlers.
//!
//! # Port
//!
//! A `Client` may obtain port information through the `Client::port_by_id` and
//! `Client::port_by_name` methods. These ports can be used to manage connections or to obtain port
//! metadata, though their port data (audio buffers and midi buffers) cannot be accessed safely.
//!
//! Ports can be registered with the `Client::register_port` method. This requires a `PortSpec`. The
//! jack crate comes with common specs such as `AudioIn`, `AudioOut`, `MidiIn`, and
//! `MidiOut`.
//!
//! To access the data of registered ports, use their specialized methods within a `ProcessHandler`
//! callback. For example, `Port<AudioIn>::as_mut_slice` returns a audio buffer that can be written
//! to.

pub use crate::client::{
    AsyncClient, Client, ClientOptions, ClientStatus, ClosureProcessHandler, CycleTimes,
    InternalClientID, NotificationHandler, ProcessHandler, ProcessScope, CLIENT_NAME_SIZE,
};
pub use crate::jack_enums::{Control, Error, LatencyType};
pub use crate::logging::{set_logger, LoggerType};
pub use crate::port::{
    AudioIn, AudioOut, MidiIn, MidiIter, MidiOut, MidiWriter, Port, PortFlags, PortSpec, RawMidi,
    Unowned, PORT_NAME_SIZE, PORT_TYPE_SIZE,
};
pub use crate::primitive_types::{Frames, PortId, Time};
pub use crate::ringbuffer::{RingBuffer, RingBufferReader, RingBufferWriter};
pub use crate::transport::{
    Transport, TransportBBT, TransportBBTValidationError, TransportPosition, TransportState,
    TransportStatePosition,
};

/// The underlying system bindings for JACK. Can be useful for using possibly experimental stuff
/// through `jack_sys::library()`.
pub use jack_sys;

//only expose metadata if enabled
#[cfg(feature = "metadata")]
pub use crate::properties::*;

mod client;
mod jack_enums;
mod jack_utils;
mod logging;
mod port;
mod primitive_types;
mod properties;
mod ringbuffer;
mod transport;

pub mod contrib {
    mod closure;

    pub use closure::ClosureProcessHandler;
}

static TIME_CLIENT: std::sync::LazyLock<Client> = std::sync::LazyLock::new(|| {
    Client::new("deprecated_get_time", ClientOptions::NO_START_SERVER)
        .unwrap()
        .0
});

/// Return JACK's current system time in microseconds, using the JACK clock
/// source.
#[deprecated = "Prefer using Client::time. get_time will be eventually be removed and it requires an extra client initialization."]
pub fn get_time() -> primitive_types::Time {
    TIME_CLIENT.time()
}
