#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

pub mod callbacks;
pub mod client;
mod info;
mod jack_enums;
mod jack_utils;
pub mod port;
mod primitive_types;

pub use info::{set_info_callback, set_error_callback};
pub use jack_enums::{JackControl, JackErr};
pub use primitive_types::{JackFrames, JackPortId, JackTime};

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> JackTime {
    unsafe { jack_sys::jack_get_time() }
}

pub mod traits {
    pub use client::JackClient;
    pub use callbacks::JackHandler;
    pub use port::PortSpec;
}

pub mod prelude {
    pub use callbacks::*;
    pub use client::*;
    pub use port::*;
    pub use jack_enums::*;
    pub use primitive_types::*;
    pub use info::*;
}

#[cfg(test)]
pub use jack_utils::{default_sleep, default_longer_sleep};
#[cfg(test)]
mod test;
