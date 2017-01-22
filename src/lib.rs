#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

/// Defines callback traits and `ProcessScope`.
pub mod callbacks;

/// Create a connection to a JACK server.
pub mod client;

/// Control error and info logging from JACK.
pub mod logging;

mod jack_enums;
mod jack_utils;

/// Types for interacting with port data from JACK.
pub mod port;

mod primitive_types;
pub use jack_enums::{JackControl, JackErr};
pub use primitive_types::{JackFrames, JackPortId, JackTime};

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> JackTime {
    unsafe { jack_sys::jack_get_time() }
}

/// Contains every trait defined in the jack crate.
pub mod traits {
    pub use client::JackClient;
    pub use callbacks::JackHandler;
    pub use port::PortSpec;
}

/// Contains most functionality to interact with JACK.
pub mod prelude {
    pub use callbacks::{JackHandler, ProcessHandler, ProcessScope};
    pub use client::*;
    pub use logging::*;
    pub use jack_enums::*;
    pub use port::*;
    pub use primitive_types::*;
}

#[cfg(test)]
pub use jack_utils::{default_longer_sleep, default_sleep};
#[cfg(test)]
mod test;
