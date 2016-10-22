#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

mod callbacks;
mod client;
mod enums;
mod flags;
mod info;
mod port;
mod utils;

pub use callbacks::{JackHandler, ProcessScope};
pub use client::{Client, ActiveClient, JackClient, CLIENT_NAME_SIZE};
pub use enums::*;
pub use flags::*;
pub use port::{Port, Owned, Unowned, Input, Output, Audio, PORT_NAME_SIZE, PORT_TYPE_SIZE};
pub use info::set_info_callbacks;

pub fn get_time() -> u64 {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
mod test;
