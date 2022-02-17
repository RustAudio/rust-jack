//! Rust bindings for JACK, a real-time audio and midi library. These bindings are compatible with
//! all implementations of JACK (Pipewire JACK, JACK1, and JACK2).
//!
//! # Linking, dynamic loading, and packaging
//!
//! libjack is shared among all clients on the system, so there must only be a single
//! system-wide version of it. Applications typically should not ship their own copy of libjack.
//! This is an issue for distributing JACK compatible applications on Windows and macOS. On Linux
//! and BSDs, this is not an issue for system packages because the application and JACK server are
//! both distributed by the system package manager.
//!
//! To handle this, use the `dlopen` Cargo feature, which is enabled by default. This feature
//! dynamically loads libjack at runtime rather than linking libjack at build time. If the
//! user does not have JACK installed at runtime, [Client::new] will return [Error::LoadLibraryError].
//! In this case, have your application show an error message directing the user to install JACK from
//! <https://jackaudio.org/downloads/> and, if available, fall back to another audio API.
//!
//! With the `dlopen` feature, neither libjack nor the JACK pkgconfig file need to be present at build
//! time. This is convenient for automated Windows and macOS builds as well as cross compiling.
//!
//! If your application cannot be used without JACK, Linux and BSD packagers may prefer
//! to link libjack at build time. To do this, disable the `dlopen` feature by using
//! `default-features = false` in your application's Cargo.toml. For example:
//!
//! ```toml
//! [target.'cfg(any(windows, target_vendor = "apple"))'.dependencies]
//! # Load libjack at runtime.
//! jack = "0.9"
//!
//! [target.'cfg(not(any(windows, target_vendor = "apple")))'.dependencies]
//! # Link libjack at build time.
//! jack = { version = "0.9", default-features = false }
//! ```
//!
//! You can set the environment variable `RUST_JACK_DLOPEN` to `on` to enable the `dlopen` feature
//! without needing to edit your application's Cargo.toml. This can be useful for cross compiling
//! to Linux with a different CPU architecture.
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

#[cfg(feature = "dlopen")]
use lazy_static::lazy_static;

//only expose metadata if enabled
#[cfg(feature = "metadata")]
pub use crate::properties::*;

/// Create and manage client connections to a JACK server.
mod client;

/// Create and manage JACK ring buffers.
mod ringbuffer;

/// Enum types in jack.
mod jack_enums;

mod jack_utils;

/// Types for safely interacting with port data from JACK.
mod port;

/// Platform independent types.
mod primitive_types;

/// Transport.
mod transport;

/// Properties
mod properties;

#[cfg(feature = "dlopen")]
lazy_static! {
    pub(crate) static ref LIB: &'static jack_sys::JackLib = {
        let j = LIB_RESULT.as_ref().unwrap();
        j
    };
    static ref LIB_RESULT: Result<jack_sys::JackLib, dlib::DlError> =
        unsafe { jack_sys::JackLib::open(jack_sys::JACK_LIB) };
}

#[cfg(all(feature = "dlopen", feature = "metadata"))]
lazy_static! {
    pub(crate) static ref METADATA: jack_sys::JackMetadata =
        unsafe { jack_sys::JackMetadata::open(jack_sys::JACK_LIB).unwrap() };
}

#[cfg(all(feature = "dlopen", feature = "metadata"))]
lazy_static! {
    pub(crate) static ref UUID: jack_sys::JackUuid =
        unsafe { jack_sys::JackUuid::open(jack_sys::JACK_LIB).unwrap() };
}

/// Dynamically loads the JACK library. This is libjack.so on Linux and
/// libjack.dll on Windows.
#[cfg(feature = "dlopen")]
pub fn load_jack_library() -> Result<(), Error> {
    LIB_RESULT
        .as_ref()
        .map(|_| ())
        .map_err(|e| Error::LoadLibraryError(format!("{}", e)))
}

/// Return JACK's current system time in microseconds, using the JACK clock
/// source.
pub fn get_time() -> primitive_types::Time {
    #[cfg(feature = "dlopen")]
    let t = unsafe { (LIB.jack_get_time)() };
    #[cfg(not(feature = "dlopen"))]
    let t = unsafe { jack_sys::jack_get_time() };
    t
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread, time};

    #[test]
    fn time_can_get_time() {
        get_time();
    }

    #[test]
    fn time_is_monotonically_increasing() {
        let initial_t = get_time();
        thread::sleep(time::Duration::from_millis(100));
        let later_t = get_time();
        assert!(initial_t < later_t, "failed {} < {}", initial_t, later_t);
    }
}
