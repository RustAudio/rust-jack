#[macro_use]
extern crate bitflags;
extern crate jack_sys;
extern crate libc;

mod callbacks;
mod client;
mod enums;
mod flags;
mod info;
mod port;
mod utils;

pub use callbacks::{JackHandler, ProcessScope};
pub use client::{Client, ActiveClient, JackClient, client_name_size};
pub use enums::*;
pub use flags::*;
pub use port::{Port, Owned, Unowned, Input, Output, Audio, port_name_size, port_type_size};
pub use info::set_info_callbacks;

pub fn get_time() -> u64 {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
mod test;
