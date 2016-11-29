#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

pub mod callbacks;
pub mod client;
pub mod jack_enums;
pub mod flags;
pub mod info;
pub mod jack_port;
pub mod jack_port_impls;
pub mod utils;

pub use callbacks::{JackHandler, ProcessScope};
pub use client::{Client, JackClient};
pub use flags::NO_START_SERVER;
pub use jack_enums::{JackControl, JackErr};
pub use jack_port::Port;
pub use jack_port_impls::{Audio, Input, Output, AudioInputSpec, AudioOutputSpec, AudioInData, AudioOutData};


pub fn get_time() -> u64 {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
mod test;
