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

pub use callbacks::JackHandler;
pub use client::Client;
pub use enums::*;
pub use flags::*;
pub use port::Port;
pub use info::set_info_callbacks;

#[cfg(test)]
mod tests {
    #[test]
    fn it_builds() {}
}
