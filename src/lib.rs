//! JACK is a low-latency audio server that supports Linux, OS X, Solaris, FreeBSD, and Windows. It
//! can connect several client applications to an audio device, and allow them to share audio with
//! each other.
//!
//! # Capture Playback Example
//!
//! For a basic example, we will create an application that takes 2 channels of audio as input, and
//! outputs them as 2 channels. We will create a client, ports, handler, and then finally run the
//! program.
//!
//! ## Client
//!
//! The first step is to create our client. For this example, we will connect to an existing JACK
//! server (possibly started up with qjackctl), instead of possibly starting our own. To accomplish
//! this, we open a client with the `NO_START_SERVER_OPTION`;
//!
//! ```rust
//! extern crate jack;
//!
//! // Create client
//! let mut client = match Client::open("rust_jack_simple", client_options::NO_START_SERVER) {
//!     Ok((client, status)) => {
//!         println!("Opened JACK client \"{}\", with status {:?}",
//!                  client.name(),
//!                  status);
//!         client
//!     }
//!     Err(e) => panic!("Failed to open JACK client, error: {:?}", e),
//! };
//! # // Register ports. They will be used in a callback when new data is available.
//! # let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
//! # let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
//! # let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
//! # let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
//! # let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//! #     let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//! #     let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//! #     let in_a_p = jack::AudioInPort::new(&in_a, ps);
//! #     let in_b_p = jack::AudioInPort::new(&in_b, ps);
//! #     We can copy the data over as if they were slices.
//! #     out_a_p.clone_from_slice(&in_a_p);
//! #     out_b_p.clone_from_slice(&in_b_p);
//! #     // Continue to run the application
//! #     jack::JackControl::Continue
//! # };
//! # // Activate the cilent, which starts the processing.
//! # // Wait for user input to quit
//! # use std::io;
//! # println!("Press enter/return to quit...");
//! # let mut user_input = String::new();
//! # io::stdin().read_line(&mut user_input).ok();
//! #
//! # active_client.deactivate().unwrap();
//! ```
//!
//! ## Ports
//!
//! Now that we most likely have a client, we can interact with the JACK server. We can now register
//! ports. We will register two input ports, to collect audio from other JACK (even hardware)
//! ports. We will also register two output ports, to put the data we obtain from the inputs.
//!
//! ```rust
//! # extern crate jack;
//!
//! # // Create client
//! # let mut client = match Client::open("rust_jack_simple", client_options::NO_START_SERVER) {
//! #     Ok((client, status)) => {
//! #         println!("Opened JACK client \"{}\", with status {:?}",
//! #                  client.name(),
//! #                  status);
//! #         client
//! #     }
//! #     Err(e) => panic!("Failed to open JACK client, error: {:?}", e),
//! # };
//! // Register ports. They will be used in a callback when new data is available.
//! let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
//! let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
//! let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
//! let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
//! # let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//! #     let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//! #     let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//! #     let in_a_p = jack::AudioInPort::new(&in_a, ps);
//! #     let in_b_p = jack::AudioInPort::new(&in_b, ps);
//! #     We can copy the data over as if they were slices.
//! #     out_a_p.clone_from_slice(&in_a_p);
//! #     out_b_p.clone_from_slice(&in_b_p);
//! #     // Continue to run the application
//! #     jack::JackControl::Continue
//! # };
//! # // Activate the cilent, which starts the processing.
//! # // Wait for user input to quit
//! # use std::io;
//! # println!("Press enter/return to quit...");
//! # let mut user_input = String::new();
//! # io::stdin().read_line(&mut user_input).ok();
//! #
//! # active_client.deactivate().unwrap();
//! ```
//!
//! ## Logic Handler
//!
//! Rust JACK exposes an asynchronous interface for processing in JACK. To get it to process our
//! data, we have to provide an object that implements the `JackHandler` trait. For convenience, we
//! will use a closure which automatically implements the trait. The implementation simply maps the
//! `process_callback` to call itself.
//!
//! ```rust
//! # extern crate jack;
//!
//! # // Create client
//! # let mut client = match Client::open("rust_jack_simple", client_options::NO_START_SERVER) {
//! #     Ok((client, status)) => {
//! #         println!("Opened JACK client \"{}\", with status {:?}",
//! #                  client.name(),
//! #                  status);
//! #         client
//! #     }
//! #     Err(e) => panic!("Failed to open JACK client, error: {:?}", e),
//! # };
//! # // Register ports. They will be used in a callback when new data is available.
//! # let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
//! # let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
//! # let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
//! # let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
//! let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//! #     let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//! #     let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//! #     let in_a_p = jack::AudioInPort::new(&in_a, ps);
//! #     let in_b_p = jack::AudioInPort::new(&in_b, ps);
//! #     We can copy the data over as if they were slices.
//! #     out_a_p.clone_from_slice(&in_a_p);
//! #     out_b_p.clone_from_slice(&in_b_p);
//! #     // Continue to run the application
//! #     jack::JackControl::Continue
//! # };
//! # // Activate the cilent, which starts the processing.
//! # // Wait for user input to quit
//! # use std::io;
//! # println!("Press enter/return to quit...");
//! # let mut user_input = String::new();
//! # io::stdin().read_line(&mut user_input).ok();
//! #
//! # active_client.deactivate().unwrap();
//! ```
//!
//! ## Processing Logic
//!
//! Inside the `process_callback`, we can use data that the ports will hold. Ports can provide an
//! unsafe view into the data through their `buffer` method, but we will use safe wrappers around
//! them instead. `jack::AudioOutPort` and `jack::AudioInPort` provide safe views into their
//! respective ports when in a process scope. JACK also comes with the midi versions
//! `jack::MidiInPort` and `jack::MidiOutPort`.
//!
//! ```rust
//! # extern crate jack;
//!
//! # // Create client
//! # let mut client = match Client::open("rust_jack_simple", client_options::NO_START_SERVER) {
//! #     Ok((client, status)) => {
//! #         println!("Opened JACK client \"{}\", with status {:?}",
//! #                  client.name(),
//! #                  status);
//! #         client
//! #     }
//! #     Err(e) => panic!("Failed to open JACK client, error: {:?}", e),
//! # };
//! # // Register ports. They will be used in a callback when new data is available.
//! # let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
//! # let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
//! # let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
//! # let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
//! let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//!     let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//!     let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//!     let in_a_p = jack::AudioInPort::new(&in_a, ps);
//!     let in_b_p = jack::AudioInPort::new(&in_b, ps);
//!     We can copy the data over as if they were slices.
//!     out_a_p.clone_from_slice(&in_a_p);
//!     out_b_p.clone_from_slice(&in_b_p);
//!     // Continue to run the application
//!     jack::JackControl::Continue
//! };
//! # // Activate the cilent, which starts the processing.
//! # // Wait for user input to quit
//! # use std::io;
//! # println!("Press enter/return to quit...");
//! # let mut user_input = String::new();
//! # io::stdin().read_line(&mut user_input).ok();
//! #
//! # active_client.deactivate().unwrap();
//! ```
//!
//! ## Running
//!
//! Now that everything is set up, we can run the application. For the example, we will run for as
//! long as the user doesn't input anything to standard in.
//!
//! ```rust
//! # extern crate jack;
//!
//! # // Create client
//! # let mut client = match Client::open("rust_jack_simple", client_options::NO_START_SERVER) {
//! #     Ok((client, status)) => {
//! #         println!("Opened JACK client \"{}\", with status {:?}",
//! #                  client.name(),
//! #                  status);
//! #         client
//! #     }
//! #     Err(e) => panic!("Failed to open JACK client, error: {:?}", e),
//! # };
//! # // Register ports. They will be used in a callback when new data is available.
//! # let in_a = client.register_port("rust_in_l", jack::AudioInSpec::default()).unwrap();
//! # let in_b = client.register_port("rust_in_r", jack::AudioInSpec::default()).unwrap();
//! # let mut out_a = client.register_port("rust_out_l", jack::AudioOutSpec::default()).unwrap();
//! # let mut out_b = client.register_port("rust_out_r", jack::AudioOutSpec::default()).unwrap();
//! # let process_callback = move |ps: &jack::ProcessScope| -> jack::JackControl {
//! #     let mut out_a_p = jack::AudioOutPort::new(&mut out_a, ps);
//! #     let mut out_b_p = jack::AudioOutPort::new(&mut out_b, ps);
//! #     let in_a_p = jack::AudioInPort::new(&in_a, ps);
//! #     let in_b_p = jack::AudioInPort::new(&in_b, ps);
//! #     We can copy the data over as if they were slices.
//! #     out_a_p.clone_from_slice(&in_a_p);
//! #     out_b_p.clone_from_slice(&in_b_p);
//! #     // Continue to run the application
//! #     jack::JackControl::Continue
//! # };
//! // Activate the cilent, which starts the processing.
//! // Wait for user input to quit
//! use std::io;
//! println!("Press enter/return to quit...");
//! let mut user_input = String::new();
//! io::stdin().read_line(&mut user_input).ok();
//!
//! active_client.deactivate().unwrap();
//! ```
//!
//! ## Executing The Program
//!
//! Before running the program, it is necessary to have a JACK client up. The easiest way is to use
//! qjackctl.  After executing the program, open up qjackctl and route the ports however you
//! wish. The functionality is located under the "Connect" button under a running client.


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
mod primitive_types;

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
pub use primitive_types::{JackFrames, JackPortId, JackTime};

/// Return JACK's current system time in microseconds, using the JACK
/// clock source.
pub fn get_time() -> JackTime {
    unsafe { jack_sys::jack_get_time() }
}

#[cfg(test)]
pub use jack_utils::{default_sleep, default_longer_sleep};
#[cfg(test)]
pub struct DummyHandler;
#[cfg(test)]
impl JackHandler for DummyHandler {}
#[cfg(test)]
mod test;
