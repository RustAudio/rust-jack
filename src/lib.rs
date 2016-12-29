//! ```rust
//! extern crate jack;
//! use std::io;
//!
//! fn main() {
//!     // Create client
//!     let (mut client, _status) =
//!         jack::Client::open("rust_jack_simple",
//!                            jack::client_options::NO_START_SERVER)
//!             .unwrap();
//!
//!     // Register ports, that will be used in a callback when new data is available.
//!     let in_a = client.register_port("rust_in_l", jack::AudioInSpec).unwrap();
//!     let in_b = client.register_port("rust_in_r", jack::AudioInSpec).unwrap();
//!     let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec).unwrap();
//!     let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec).unwrap();
//!     let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//!         let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//!         let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//!         let in_a_p = jack::AudioInPort::new(&in_a, ps);
//!         let in_b_p = jack::AudioInPort::new(&in_b, ps);
//!         out_a_p.clone_from_slice(&in_a_p);
//!         out_b_p.clone_from_slice(&in_b_p);
//!         jack::JackControl::Continue
//!     };
//!
//!     // Activate the client, which starts the processing.
//!     let active_client = client.activate(process_callback).unwrap();
//!
//!     // Wait for user input
//!     println!("Press enter/return to quit...");
//!     let mut user_input = String::new();
//!     io::stdin().read_line(&mut user_input).ok();
//!
//!     active_client.deactivate().unwrap();
//! }
//! ```
#[macro_use]
extern crate bitflags;
extern crate jack_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;

mod callbacks;
mod client;
mod info;
mod jack_enums;
mod jack_flags;
mod jack_utils;
mod port;
mod port_impls;

pub use callbacks::{ProcessScope, JackHandler};
pub use client::CLIENT_NAME_SIZE;
pub use client::{JackClient, Client, ActiveClient, CycleTimes};
pub use info::{set_info_callback, set_error_callback};
pub use jack_enums::{JackControl, JackErr};
pub use jack_flags::{ClientOptions, client_options};
pub use jack_flags::{ClientStatus, client_status};
pub use jack_flags::{PortFlags, port_flags};
pub use port::{PORT_NAME_SIZE, PORT_TYPE_SIZE};
pub use port::{Port, PortSpec, Unowned, UnownedPort};
pub use port_impls::{AudioInSpec, AudioInPort, AudioOutSpec, AudioOutPort};
pub use port_impls::{MidiInSpec, MidiInPort, MidiIter, MidiOutSpec, MidiOutPort, RawMidi};

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> u64 {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
mod test;
