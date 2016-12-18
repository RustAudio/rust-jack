#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rimd;

mod callbacks;
mod client;
mod info;
mod jack_enums;
mod jack_flags;
mod jack_utils;
mod midi;
mod port;
mod port_impls;

pub use callbacks::{ProcessScope, JackHandler};
pub use client::CLIENT_NAME_SIZE;
pub use client::{JackClient, Client, ActiveClient, CycleTimes};
pub use info::set_info_callbacks;
pub use jack_enums::{JackControl, JackErr};
pub use midi::{MidiStream, MidiStreamReader, MidiIter};
pub use jack_flags::{ClientOptions, client_options};
pub use jack_flags::{ClientStatus, client_status};
pub use jack_flags::{PortFlags, port_flags};
pub use port::{PORT_NAME_SIZE, PORT_TYPE_SIZE};
pub use port::{Port, PortData, Unowned, UnownedPort};
pub use port_impls::{AudioIn, AudioOut};

/// An endpoint to read JACK 32bit floating point audio.
pub type AudioInPort<'a> = Port<AudioIn<'a>>;

/// An endpoint to output JACK 32bit floating point audio.
pub type AudioOutPort<'a> = Port<AudioOut<'a>>;
pub type MidiInPort = Port<MidiStreamReader>;

pub fn get_time() -> u64 {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
mod test;
